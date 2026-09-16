use std::path::Path;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;

use crate::cancellation::CancellationState;
use crate::output_path::unique_output_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    Mp4,
    Mov,
    Avi,
    WebM,
    Gif,
}

impl VideoFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "mp4" => Ok(Self::Mp4),
            "mov" => Ok(Self::Mov),
            "avi" => Ok(Self::Avi),
            "webm" => Ok(Self::WebM),
            "gif" => Ok(Self::Gif),
            other => Err(format!("Unsupported target format: {other}")),
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mov => "mov",
            Self::Avi => "avi",
            Self::WebM => "webm",
            Self::Gif => "gif",
        }
    }

    /// FFmpeg codec args for encoding to this format (video + audio codec,
    /// where applicable). GIF has no entry here — it has no audio track
    /// and needs a dedicated two-pass palette pipeline (see
    /// `convert_to_gif`), not a plain -c:v/-c:a pair.
    fn encode_args(self) -> Vec<&'static str> {
        match self {
            Self::Mp4 => vec!["-c:v", "libx264", "-c:a", "aac", "-b:a", "192k"],
            Self::Mov => vec!["-c:v", "libx264", "-c:a", "aac", "-b:a", "192k"],
            Self::Avi => vec!["-c:v", "mpeg4", "-c:a", "libmp3lame"],
            Self::WebM => vec!["-c:v", "libvpx-vp9", "-c:a", "libopus"],
            Self::Gif => unreachable!("GIF uses convert_to_gif, not encode_args"),
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

/// Runs the ffmpeg sidecar with the given args, returning a short
/// stderr-tail-based error message on failure.
async fn run_ffmpeg(app: &AppHandle, args: &[String]) -> Result<(), String> {
    let sidecar = app.shell().sidecar("ffmpeg").map_err(|e| {
        format!(
            "FFmpeg sidecar is not available ({e}). It must be bundled at \
             src-tauri/binaries/ffmpeg-<target-triple>(.exe) for video conversion to work."
        )
    })?;

    match sidecar.args(args).output().await {
        Ok(output) if output.status.success() => Ok(()),
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
            Err(if tail.is_empty() {
                "FFmpeg exited with an error".to_string()
            } else {
                tail
            })
        }
        Err(e) => Err(format!("Failed to run FFmpeg: {e}")),
    }
}

/// GIF has no native quality knob the way h264/vp9 do — a naive single-pass
/// encode reuses a fixed 256-color web palette and looks visibly banded.
/// The standard fix is two passes: generate an optimized palette for this
/// specific clip, then encode using that palette. Both stages share the
/// same fps/scale filter so the palette matches what's actually encoded.
async fn convert_to_gif(
    app: &AppHandle,
    source_path: &Path,
    output_path: &Path,
) -> Result<(), String> {
    let palette_path = output_path.with_extension("palette.png");
    let filter = "fps=15,scale=480:-1:flags=lanczos";

    run_ffmpeg(
        app,
        &[
            "-y".to_string(),
            "-i".to_string(),
            source_path.display().to_string(),
            "-vf".to_string(),
            format!("{filter},palettegen"),
            "-update".to_string(),
            "1".to_string(),
            palette_path.display().to_string(),
        ],
    )
    .await?;

    let result = run_ffmpeg(
        app,
        &[
            "-y".to_string(),
            "-i".to_string(),
            source_path.display().to_string(),
            "-i".to_string(),
            palette_path.display().to_string(),
            "-filter_complex".to_string(),
            format!("{filter}[x];[x][1:v]paletteuse"),
            output_path.display().to_string(),
        ],
    )
    .await;

    let _ = std::fs::remove_file(&palette_path);

    result
}

async fn convert_one(
    app: &AppHandle,
    source_path: &Path,
    target_format: VideoFormat,
    output_dir: Option<&Path>,
) -> ConversionResult {
    let output_path = unique_output_path(source_path, target_format.extension(), output_dir);

    let result = if target_format == VideoFormat::Gif {
        convert_to_gif(app, source_path, &output_path).await
    } else {
        let mut args: Vec<String> = vec![
            "-y".to_string(),
            "-i".to_string(),
            source_path.display().to_string(),
        ];
        args.extend(target_format.encode_args().into_iter().map(String::from));
        args.push(output_path.display().to_string());
        run_ffmpeg(app, &args).await
    };

    match result {
        Ok(()) => ConversionResult::ok(source_path, output_path),
        Err(message) => ConversionResult::err(source_path, message),
    }
}

#[tauri::command]
pub async fn convert_video_files(
    app: AppHandle,
    cancellation: tauri::State<'_, CancellationState>,
    paths: Vec<String>,
    target_format: String,
    output_dir: Option<String>,
) -> Result<Vec<ConversionResult>, String> {
    let target_format = VideoFormat::parse(&target_format)?;
    let output_dir = output_dir.as_deref().map(Path::new);
    let total = paths.len();

    cancellation.reset();

    let mut results = Vec::with_capacity(total);
    for (index, path) in paths.iter().enumerate() {
        if cancellation.is_cancelled() {
            break;
        }

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
