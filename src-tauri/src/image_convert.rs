use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::Emitter;

use crate::output_path::unique_output_path;

/// Formats we accept as conversion output. SVG is intentionally excluded:
/// resvg/usvg only get us rasterization (SVG -> bitmap), not bitmap -> SVG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Png,
    Jpeg,
    WebP,
    Gif,
    Bmp,
    Tiff,
    Ico,
    Tga,
    Pnm,
    Qoi,
    Avif,
}

impl OutputFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "png" => Ok(Self::Png),
            "jpg" | "jpeg" => Ok(Self::Jpeg),
            "webp" => Ok(Self::WebP),
            "gif" => Ok(Self::Gif),
            "bmp" => Ok(Self::Bmp),
            "tiff" | "tif" => Ok(Self::Tiff),
            "ico" => Ok(Self::Ico),
            "tga" => Ok(Self::Tga),
            "pnm" => Ok(Self::Pnm),
            "qoi" => Ok(Self::Qoi),
            "avif" => Ok(Self::Avif),
            other => Err(format!("Unsupported target format: {other}")),
        }
    }

    fn image_format(self) -> image::ImageFormat {
        match self {
            Self::Png => image::ImageFormat::Png,
            Self::Jpeg => image::ImageFormat::Jpeg,
            Self::WebP => image::ImageFormat::WebP,
            Self::Gif => image::ImageFormat::Gif,
            Self::Bmp => image::ImageFormat::Bmp,
            Self::Tiff => image::ImageFormat::Tiff,
            Self::Ico => image::ImageFormat::Ico,
            Self::Tga => image::ImageFormat::Tga,
            Self::Pnm => image::ImageFormat::Pnm,
            Self::Qoi => image::ImageFormat::Qoi,
            Self::Avif => image::ImageFormat::Avif,
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Ico => "ico",
            Self::Tga => "tga",
            Self::Pnm => "pnm",
            Self::Qoi => "qoi",
            Self::Avif => "avif",
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionOptions {
    /// JPEG/WebP encoding quality, 1-100. Ignored by formats that don't
    /// support lossy quality (png, bmp, tiff, ico, tga, pnm, qoi).
    pub quality: Option<u8>,
    /// Scale the image down to fit within this many pixels on its longest
    /// side, preserving aspect ratio. Images already smaller are left
    /// untouched — this only ever downsizes, never upscales.
    pub max_dimension: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ConversionResult {
    pub source_path: String,
    pub success: bool,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

impl ConversionResult {
    fn ok(source_path: &Path, output_path: PathBuf) -> Self {
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

/// Converts a single file to the target format, writing the output either
/// next to the source file or into `output_dir` if given. Never panics:
/// every failure mode is captured in the returned `ConversionResult`.
fn convert_one(
    source_path: &Path,
    target_format: OutputFormat,
    output_dir: Option<&Path>,
    options: ConversionOptions,
) -> ConversionResult {
    let source_ext = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let decoded = if source_ext.eq_ignore_ascii_case("svg") {
        rasterize_svg(source_path)
    } else if source_ext.eq_ignore_ascii_case("avif") {
        // We only bundle the AVIF encoder (ravif), not the decoder
        // (dav1d), to keep the binary lean. Reject up front instead of
        // letting `image::open` fail with a confusing generic error.
        Err(
            "AVIF input is not supported yet — AVIF can only be a conversion target, not a source"
                .to_string(),
        )
    } else {
        image::open(source_path).map_err(|e| format!("Failed to read image: {e}"))
    };

    let img = match decoded {
        Ok(img) => img,
        Err(message) => return ConversionResult::err(source_path, message),
    };

    let img = match options.max_dimension {
        Some(max_dim) if img.width().max(img.height()) > max_dim => {
            img.resize(max_dim, max_dim, image::imageops::FilterType::Lanczos3)
        }
        _ => img,
    };

    let output_path = unique_output_path(source_path, target_format.extension(), output_dir);

    let save_result = match (target_format, options.quality) {
        (OutputFormat::Jpeg, Some(quality)) => save_jpeg_with_quality(&img, &output_path, quality),
        (OutputFormat::WebP, Some(_)) => {
            // image's WebP encoder is lossless-only; a quality knob isn't
            // available without pulling in libwebp bindings, so we ignore
            // the requested quality here rather than pretend to honor it.
            img.save_with_format(&output_path, target_format.image_format())
        }
        _ => img.save_with_format(&output_path, target_format.image_format()),
    };

    match save_result {
        Ok(()) => ConversionResult::ok(source_path, output_path),
        Err(e) => ConversionResult::err(source_path, format!("Failed to write output: {e}")),
    }
}

fn save_jpeg_with_quality(
    img: &image::DynamicImage,
    output_path: &Path,
    quality: u8,
) -> image::ImageResult<()> {
    let file = std::fs::File::create(output_path).map_err(image::ImageError::IoError)?;
    let mut writer = std::io::BufWriter::new(file);
    let mut encoder =
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality.clamp(1, 100));
    encoder.encode_image(img)
}

/// SVG is not a bitmap codec, so we rasterize it via resvg/usvg at its
/// intrinsic size rather than decoding it through the `image` crate.
fn rasterize_svg(source_path: &Path) -> Result<image::DynamicImage, String> {
    let svg_data =
        std::fs::read(source_path).map_err(|e| format!("Failed to read SVG file: {e}"))?;

    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(&svg_data, &options)
        .map_err(|e| format!("Failed to parse SVG: {e}"))?;

    let size = tree.size();
    let width = size.width().ceil().max(1.0) as u32;
    let height = size.height().ceil().max(1.0) as u32;

    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Invalid SVG dimensions".to_string())?;

    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );

    let rgba = image::RgbaImage::from_raw(width, height, pixmap.data().to_vec())
        .ok_or_else(|| "Failed to build image buffer from rendered SVG".to_string())?;

    Ok(image::DynamicImage::ImageRgba8(rgba))
}

#[derive(Debug, Serialize, Clone)]
struct ConversionProgress {
    completed: usize,
    total: usize,
    result: ConversionResult,
}

#[tauri::command]
pub fn convert_images(
    app: tauri::AppHandle,
    paths: Vec<String>,
    target_format: String,
    output_dir: Option<String>,
    options: Option<ConversionOptions>,
) -> Result<Vec<ConversionResult>, String> {
    let target_format = OutputFormat::parse(&target_format)?;
    let output_dir = output_dir.as_deref().map(Path::new);
    let options = options.unwrap_or(ConversionOptions {
        quality: None,
        max_dimension: None,
    });
    let total = paths.len();

    let mut results = Vec::with_capacity(total);
    for (index, path) in paths.iter().enumerate() {
        let result = convert_one(Path::new(path), target_format, output_dir, options);

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
