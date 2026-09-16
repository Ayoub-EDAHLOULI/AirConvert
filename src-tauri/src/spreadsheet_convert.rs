use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::cancellation::CancellationState;
use crate::output_path::unique_output_path;

/// Data-only spreadsheet conversion: no formulas, no macros, no styling —
/// just the cell values of the first worksheet. ODS is readable (calamine
/// supports it natively) but not a write target: there's no mature
/// pure-Rust ODS writer, so the output picker only ever offers csv/xlsx.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpreadsheetFormat {
    Csv,
    Xlsx,
}

impl SpreadsheetFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "csv" => Ok(Self::Csv),
            "xlsx" => Ok(Self::Xlsx),
            other => Err(format!("Unsupported target format: {other}")),
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Xlsx => "xlsx",
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

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERROR: {e:?}"),
    }
}

/// Reads a spreadsheet's first worksheet into a plain grid of strings.
/// First worksheet only — there's no per-sheet selection UI, matching the
/// "data only" scope.
///
/// CSV is handled separately: calamine's `open_workbook_auto` has no CSV
/// support at all (its format-detection enum only covers xls/xlsx/xlsb/ods),
/// so a `.csv` source is read directly with the `csv` crate instead.
fn read_first_sheet(source_path: &Path) -> Result<Vec<Vec<String>>, String> {
    let is_csv = source_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("csv"))
        .unwrap_or(false);

    if is_csv {
        return read_csv(source_path);
    }

    let mut workbook = open_workbook_auto(source_path)
        .map_err(|e| format!("Failed to open spreadsheet: {e}"))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| "Spreadsheet has no worksheets".to_string())?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("Failed to read worksheet '{sheet_name}': {e}"))?;

    Ok(range
        .rows()
        .map(|row| row.iter().map(cell_to_string).collect())
        .collect())
}

fn read_csv(source_path: &Path) -> Result<Vec<Vec<String>>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(source_path)
        .map_err(|e| format!("Failed to open CSV file: {e}"))?;

    reader
        .records()
        .map(|record| {
            record
                .map(|r| r.iter().map(String::from).collect())
                .map_err(|e| format!("Failed to read CSV row: {e}"))
        })
        .collect()
}

fn write_csv(rows: &[Vec<String>], output_path: &Path) -> Result<(), String> {
    let mut writer =
        csv::Writer::from_path(output_path).map_err(|e| format!("Failed to create CSV file: {e}"))?;
    for row in rows {
        writer
            .write_record(row)
            .map_err(|e| format!("Failed to write CSV row: {e}"))?;
    }
    writer
        .flush()
        .map_err(|e| format!("Failed to finish writing CSV file: {e}"))
}

fn write_xlsx(rows: &[Vec<String>], output_path: &Path) -> Result<(), String> {
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();

    for (row_idx, row) in rows.iter().enumerate() {
        for (col_idx, value) in row.iter().enumerate() {
            worksheet
                .write(row_idx as u32, col_idx as u16, value.as_str())
                .map_err(|e| format!("Failed to write cell: {e}"))?;
        }
    }

    workbook
        .save(output_path)
        .map_err(|e| format!("Failed to save XLSX file: {e}"))
}

fn convert_one(
    source_path: &Path,
    target_format: SpreadsheetFormat,
    output_dir: Option<&Path>,
) -> ConversionResult {
    let rows = match read_first_sheet(source_path) {
        Ok(rows) => rows,
        Err(message) => return ConversionResult::err(source_path, message),
    };

    let output_path = unique_output_path(source_path, target_format.extension(), output_dir);

    let write_result = match target_format {
        SpreadsheetFormat::Csv => write_csv(&rows, &output_path),
        SpreadsheetFormat::Xlsx => write_xlsx(&rows, &output_path),
    };

    match write_result {
        Ok(()) => ConversionResult::ok(source_path, output_path),
        Err(message) => ConversionResult::err(source_path, message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_to_xlsx_and_back() {
        let dir = std::env::temp_dir().join("airconvert_spreadsheet_test");
        std::fs::create_dir_all(&dir).unwrap();
        let csv_path = dir.join("sample.csv");
        std::fs::write(
            &csv_path,
            "Name,Age,City\nAlice,30,New York\nBob,25,\"Los Angeles, CA\"\nCharlie,35,Chicago\n",
        )
        .unwrap();

        let result = convert_one(&csv_path, SpreadsheetFormat::Xlsx, None);
        assert!(result.success, "csv->xlsx failed: {:?}", result.error);
        let xlsx_path = std::path::PathBuf::from(result.output_path.unwrap());
        assert!(xlsx_path.exists());

        let result2 = convert_one(&xlsx_path, SpreadsheetFormat::Csv, None);
        assert!(result2.success, "xlsx->csv failed: {:?}", result2.error);
        let roundtrip_csv = std::path::PathBuf::from(result2.output_path.unwrap());
        let content = std::fs::read_to_string(&roundtrip_csv).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("Los Angeles, CA"));
        assert!(content.contains("Charlie"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn xlsx_to_csv_preserves_numbers_and_empty_cells() {
        let dir = std::env::temp_dir().join("airconvert_spreadsheet_test_2");
        std::fs::create_dir_all(&dir).unwrap();
        let xlsx_path = dir.join("numbers.xlsx");

        let mut workbook = rust_xlsxwriter::Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet.write(0, 0, "Label").unwrap();
        worksheet.write(0, 1, "Value").unwrap();
        worksheet.write(1, 0, "Row A").unwrap();
        worksheet.write(1, 1, 42.5).unwrap();
        worksheet.write(2, 0, "Row B").unwrap();
        // Cell (2, 1) intentionally left empty.
        workbook.save(&xlsx_path).unwrap();

        let result = convert_one(&xlsx_path, SpreadsheetFormat::Csv, None);
        assert!(result.success, "xlsx->csv failed: {:?}", result.error);
        let csv_path = std::path::PathBuf::from(result.output_path.unwrap());
        let content = std::fs::read_to_string(&csv_path).unwrap();
        assert!(content.contains("Label,Value"));
        assert!(content.contains("Row A,42.5"));
        assert!(content.contains("Row B"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[tauri::command]
pub fn convert_spreadsheets(
    app: AppHandle,
    cancellation: tauri::State<'_, CancellationState>,
    paths: Vec<String>,
    target_format: String,
    output_dir: Option<String>,
) -> Result<Vec<ConversionResult>, String> {
    let target_format = SpreadsheetFormat::parse(&target_format)?;
    let output_dir = output_dir.as_deref().map(Path::new);
    let total = paths.len();

    cancellation.reset();

    let mut results = Vec::with_capacity(total);
    for (index, path) in paths.iter().enumerate() {
        if cancellation.is_cancelled() {
            break;
        }

        let result = convert_one(Path::new(path), target_format, output_dir);

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
