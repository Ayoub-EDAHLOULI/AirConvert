export const SUPPORTED_INPUT_EXTENSIONS = [
  "mp3",
  "wav",
  "flac",
  "ogg",
  "m4a",
  "aac",
  "wma",
] as const;

export const OUTPUT_FORMATS = [
  { value: "mp3", label: "MP3" },
  { value: "wav", label: "WAV" },
  { value: "flac", label: "FLAC" },
  { value: "ogg", label: "OGG" },
  { value: "m4a", label: "M4A" },
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

export function isSupportedAudio(path: string): boolean {
  return (SUPPORTED_INPUT_EXTENSIONS as readonly string[]).includes(
    extensionOf(path),
  );
}
