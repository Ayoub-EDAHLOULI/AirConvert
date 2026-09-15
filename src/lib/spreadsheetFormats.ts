export const SUPPORTED_INPUT_EXTENSIONS = [
  "csv",
  "xlsx",
  "xls",
  "ods",
] as const;

export const OUTPUT_FORMATS = [
  { value: "csv", label: "CSV" },
  { value: "xlsx", label: "XLSX" },
] as const;

export type OutputFormatValue = (typeof OUTPUT_FORMATS)[number]["value"];

export function extensionOf(path: string): string {
  const fileName = path.split(/[/\\]/).pop() ?? path;
  const dotIndex = fileName.lastIndexOf(".");
  if (dotIndex === -1 || dotIndex === fileName.length - 1) return "";
  return fileName.slice(dotIndex + 1).toLowerCase();
}

export function fileNameOf(path: string): string {
  return path.split(/[/\\]/).pop() ?? path;
}

export function isSupportedSpreadsheet(path: string): boolean {
  return (SUPPORTED_INPUT_EXTENSIONS as readonly string[]).includes(
    extensionOf(path),
  );
}
