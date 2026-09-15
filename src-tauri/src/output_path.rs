use std::path::{Path, PathBuf};

/// Builds an output path that never collides with an existing file. The
/// output folder is `output_dir` if given, otherwise the source file's own
/// folder. If the natural `stem.new_ext` name already exists there
/// (including the case where source and target format are the same, which
/// would otherwise overwrite the original), appends "-converted", then
/// "-converted-2", etc.
pub fn unique_output_path(source_path: &Path, new_ext: &str, output_dir: Option<&Path>) -> PathBuf {
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
