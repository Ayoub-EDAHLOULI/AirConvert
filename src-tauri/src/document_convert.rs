use std::path::Path;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;

use crate::output_path::unique_output_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    Markdown,
    Txt,
    Html,
    Rtf,
    Odt,
    Docx,
}

impl DocumentFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "md" | "markdown" => Ok(Self::Markdown),
            "txt" => Ok(Self::Txt),
            "html" | "htm" => Ok(Self::Html),
            "rtf" => Ok(Self::Rtf),
            "odt" => Ok(Self::Odt),
            "docx" => Ok(Self::Docx),
            other => Err(format!("Unsupported target format: {other}")),
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Txt => "txt",
            Self::Html => "html",
            Self::Rtf => "rtf",
            Self::Odt => "odt",
            Self::Docx => "docx",
        }
    }

    /// Pandoc's own format identifiers, used with -f/-t. These don't always
    /// match file extensions (e.g. "md" the extension vs "gfm"/"markdown"
    /// the Pandoc format name).
    fn pandoc_name(self) -> &'static str {
        match self {
            Self::Markdown => "gfm",
            Self::Txt => "plain",
            Self::Html => "html",
            Self::Rtf => "rtf",
            Self::Odt => "odt",
            Self::Docx => "docx",
        }
    }
}

/// Maps a source file's extension to the Pandoc format name Pandoc should
/// read it as. Separate from `DocumentFormat` because Pandoc's *reader* set
/// doesn't perfectly mirror its *writer* set (e.g. plain text has no
/// reasonable "read as plain text" beyond treating it as markdown).
fn source_pandoc_name(source_path: &Path) -> Result<&'static str, String> {
    let ext = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "md" | "markdown" | "txt" => Ok("gfm"),
        "html" | "htm" => Ok("html"),
        "rtf" => Ok("rtf"),
        "odt" => Ok("odt"),
        "docx" => Ok("docx"),
        other => Err(format!("Unsupported source format: .{other}")),
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ConversionResult {
    pub source_path: String,
    pub success: bool,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

impl ConversionResult {
    fn ok(source_path: &Path, output_path: std::path::PathBuf) -> Self {
        Self {
            source_path: source_path.display().to_string(),
            success: true,
            output_path: Some(output_path.display().to_string()),
            error: None,
        }
    }

    fn err(source_path: &Path, message: impl Into<String>) -> Self {
        Self {
            source_path: source_path.display().to_string(),
            success: false,
            output_path: None,
            error: Some(message.into()),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
struct ConversionProgress {
    completed: usize,
    total: usize,
    result: ConversionResult,
}

async fn convert_one(
    app: &AppHandle,
    source_path: &Path,
    target_format: DocumentFormat,
    output_dir: Option<&Path>,
) -> ConversionResult {
    let from_format = match source_pandoc_name(source_path) {
        Ok(name) => name,
        Err(message) => return ConversionResult::err(source_path, message),
    };

    let output_path = unique_output_path(source_path, target_format.extension(), output_dir);

    let sidecar = match app.shell().sidecar("pandoc") {
        Ok(cmd) => cmd,
        Err(e) => {
            return ConversionResult::err(
                source_path,
                format!(
                    "Pandoc sidecar is not available ({e}). It must be bundled at \
                     src-tauri/binaries/pandoc-<target-triple>(.exe) for document conversion to work."
                ),
            )
        }
    };

    let args = vec![
        source_path.display().to_string(),
        "-f".to_string(),
        from_format.to_string(),
        "-t".to_string(),
        target_format.pandoc_name().to_string(),
        "-o".to_string(),
        output_path.display().to_string(),
    ];

    match sidecar.args(&args).output().await {
        Ok(output) if output.status.success() => ConversionResult::ok(source_path, output_path),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let tail: String = stderr
                .lines()
                .rev()
                .take(3)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" ");
            ConversionResult::err(
                source_path,
                if tail.is_empty() {
                    "Pandoc exited with an error".to_string()
                } else {
                    tail
                },
            )
        }
        Err(e) => ConversionResult::err(source_path, format!("Failed to run Pandoc: {e}")),
    }
}

#[tauri::command]
pub async fn convert_documents(
    app: AppHandle,
    paths: Vec<String>,
    target_format: String,
    output_dir: Option<String>,
) -> Result<Vec<ConversionResult>, String> {
    let target_format = DocumentFormat::parse(&target_format)?;
    let output_dir = output_dir.as_deref().map(Path::new);
    let total = paths.len();

    let mut results = Vec::with_capacity(total);
    for (index, path) in paths.iter().enumerate() {
        let result = convert_one(&app, Path::new(path), target_format, output_dir).await;

        let _ = app.emit(
            "conversion-progress",
            ConversionProgress {
                completed: index + 1,
                total,
                result: result.clone(),
            },
        );

        results.push(result);
    }

    Ok(results)
}
