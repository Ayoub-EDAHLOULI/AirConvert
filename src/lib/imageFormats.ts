export const SUPPORTED_INPUT_EXTENSIONS = [
  "jpg",
  "jpeg",
  "png",
  "webp",
  "gif",
  "bmp",
  "tiff",
  "tif",
  "svg",
] as const;

export const OUTPUT_FORMATS = [
  { value: "png", label: "PNG" },
  { value: "jpeg", label: "JPG" },
  { value: "webp", label: "WebP" },
  { value: "gif", label: "GIF" },
  { value: "bmp", label: "BMP" },
  { value: "tiff", label: "TIFF" },
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

export function isSupportedImage(path: string): boolean {
  return (SUPPORTED_INPUT_EXTENSIONS as readonly string[]).includes(
    extensionOf(path),
  );
}
