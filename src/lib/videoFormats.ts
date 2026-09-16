export const SUPPORTED_INPUT_EXTENSIONS = [
  "mp4",
  "mov",
  "avi",
  "webm",
  "mkv",
  "flv",
  "wmv",
] as const;

export const OUTPUT_FORMATS = [
  { value: "mp4", label: "MP4" },
  { value: "mov", label: "MOV" },
  { value: "avi", label: "AVI" },
  { value: "webm", label: "WebM" },
  { value: "gif", label: "GIF" },
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

export function isSupportedVideo(path: string): boolean {
  return (SUPPORTED_INPUT_EXTENSIONS as readonly string[]).includes(
    extensionOf(path),
  );
}
