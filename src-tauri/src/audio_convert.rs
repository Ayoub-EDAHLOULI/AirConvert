use std::path::Path;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;

use crate::output_path::unique_output_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Mp3,
    Wav,
    Flac,
    Ogg,
    M4a,
}

impl AudioFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "mp3" => Ok(Self::Mp3),
            "wav" => Ok(Self::Wav),
            "flac" => Ok(Self::Flac),
            "ogg" => Ok(Self::Ogg),
            "m4a" => Ok(Self::M4a),
            other => Err(format!("Unsupported target format: {other}")),
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Ogg => "ogg",
            Self::M4a => "m4a",
        }
    }

    /// FFmpeg codec args for encoding to this format. `-vn` (no video
    /// stream) is applied separately since it's the same for every target.
    fn encode_args(self) -> Vec<&'static str> {
        match self {
            Self::Mp3 => vec!["-c:a", "libmp3lame", "-q:a", "2"],
            Self::Wav => vec!["-c:a", "pcm_s16le"],
            Self::Flac => vec!["-c:a", "flac"],
            Self::Ogg => vec!["-c:a", "libvorbis", "-q:a", "5"],
            Self::M4a => vec!["-c:a", "aac", "-b:a", "192k"],
        }
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
    target_format: AudioFormat,
    output_dir: Option<&Path>,
) -> ConversionResult {
    let output_path = unique_output_path(source_path, target_format.extension(), output_dir);

    let sidecar = match app.shell().sidecar("ffmpeg") {
        Ok(cmd) => cmd,
        Err(e) => {
            return ConversionResult::err(
                source_path,
                format!(
                    "FFmpeg sidecar is not available ({e}). It must be bundled at \
                     src-tauri/binaries/ffmpeg-<target-triple>(.exe) for audio conversion to work."
                ),
            )
        }
    };

    let mut args: Vec<String> = vec![
        "-y".into(),
        "-i".into(),
        source_path.display().to_string(),
        "-vn".into(),
    ];
    args.extend(target_format.encode_args().into_iter().map(String::from));
    args.push(output_path.display().to_string());

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
                    "FFmpeg exited with an error".to_string()
                } else {
                    tail
                },
            )
        }
        Err(e) => ConversionResult::err(source_path, format!("Failed to run FFmpeg: {e}")),
    }
}

#[tauri::command]
pub async fn convert_audio_files(
    app: AppHandle,
    paths: Vec<String>,
    target_format: String,
    output_dir: Option<String>,
) -> Result<Vec<ConversionResult>, String> {
    let target_format = AudioFormat::parse(&target_format)?;
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
