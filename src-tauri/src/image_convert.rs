use std::path::{Path, PathBuf};

use serde::Serialize;

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

#[derive(Debug, Serialize)]
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
        Err("AVIF input is not supported yet — AVIF can only be a conversion target, not a source".to_string())
    } else {
        image::open(source_path).map_err(|e| format!("Failed to read image: {e}"))
    };

    let img = match decoded {
        Ok(img) => img,
        Err(message) => return ConversionResult::err(source_path, message),
    };

    let output_path = unique_output_path(source_path, target_format.extension(), output_dir);

    match img.save_with_format(&output_path, target_format.image_format()) {
        Ok(()) => ConversionResult::ok(source_path, output_path),
        Err(e) => ConversionResult::err(source_path, format!("Failed to write output: {e}")),
    }
}

/// Builds an output path that never collides with an existing file. The
/// output folder is `output_dir` if given, otherwise the source file's own
/// folder. If the natural `stem.new_ext` name already exists there
/// (including the case where source and target format are the same, which
/// would otherwise overwrite the original), appends "-converted", then
/// "-converted-2", etc.
fn unique_output_path(source_path: &Path, new_ext: &str, output_dir: Option<&Path>) -> PathBuf {
    let stem = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let source_parent = source_path.parent().unwrap_or_else(|| Path::new(""));
    let parent = output_dir.unwrap_or(source_parent);

    let candidate = parent.join(format!("{stem}.{new_ext}"));
    if !candidate.exists() {
        return candidate;
    }

    let mut counter = 1;
    loop {
        let file_name = if counter == 1 {
            format!("{stem}-converted.{new_ext}")
        } else {
            format!("{stem}-converted-{counter}.{new_ext}")
        };
        let candidate = parent.join(file_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
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

    resvg::render(&tree, tiny_skia::Transform::identity(), &mut pixmap.as_mut());

    let rgba = image::RgbaImage::from_raw(width, height, pixmap.data().to_vec())
        .ok_or_else(|| "Failed to build image buffer from rendered SVG".to_string())?;

    Ok(image::DynamicImage::ImageRgba8(rgba))
}

#[tauri::command]
pub fn convert_images(
    paths: Vec<String>,
    target_format: String,
    output_dir: Option<String>,
) -> Result<Vec<ConversionResult>, String> {
    let target_format = OutputFormat::parse(&target_format)?;
    let output_dir = output_dir.as_deref().map(Path::new);

    Ok(paths
        .iter()
        .map(|path| convert_one(Path::new(path), target_format, output_dir))
        .collect())
}
