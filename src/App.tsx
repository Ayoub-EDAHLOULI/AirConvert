import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Moon, Sun, UploadCloud, FileImage, X, Loader2 } from "lucide-react";
import { useTheme } from "./theme/useTheme";
import { useFileDrop } from "./lib/useFileDrop";
import {
  OUTPUT_FORMATS,
  extensionOf,
  fileNameOf,
  isSupportedImage,
  type OutputFormatValue,
} from "./lib/imageFormats";
import "./App.css";

type QueuedFile = {
  path: string;
};

type ConversionResult = {
  source_path: string;
  success: boolean;
  output_path: string | null;
  error: string | null;
};

function App() {
  const { theme, toggleTheme } = useTheme();
  const [files, setFiles] = useState<QueuedFile[]>([]);
  const [targetFormat, setTargetFormat] = useState<OutputFormatValue>("png");
  const [isConverting, setIsConverting] = useState(false);
  const [results, setResults] = useState<ConversionResult[] | null>(null);

  const addPaths = useCallback((paths: string[]) => {
    setResults(null);
    setFiles((current) => {
      const existing = new Set(current.map((f) => f.path));
      const additions = paths
        .filter(isSupportedImage)
        .filter((path) => !existing.has(path))
        .map((path) => ({ path }));
      return [...current, ...additions];
    });
  }, []);

  const { isHovering } = useFileDrop(addPaths);

  async function handleBrowse() {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: "Images",
          extensions: [
            "jpg",
            "jpeg",
            "png",
            "webp",
            "gif",
            "bmp",
            "tiff",
            "tif",
            "svg",
          ],
        },
      ],
    });
    if (!selected) return;
    addPaths(Array.isArray(selected) ? selected : [selected]);
  }

  function removeFile(path: string) {
    setFiles((current) => current.filter((f) => f.path !== path));
    setResults(null);
  }

  function clearAll() {
    setFiles([]);
    setResults(null);
  }

  async function handleConvert() {
    if (files.length === 0) return;
    setIsConverting(true);
    setResults(null);
    try {
      const output = await invoke<ConversionResult[]>("convert_images", {
        paths: files.map((f) => f.path),
        targetFormat,
      });
      setResults(output);
    } catch (err) {
      setResults(
        files.map((f) => ({
          source_path: f.path,
          success: false,
          output_path: null,
          error: String(err),
        })),
      );
    } finally {
      setIsConverting(false);
    }
  }

  const resultByPath = new Map((results ?? []).map((r) => [r.source_path, r]));

  return (
    <main className="h-screen w-screen bg-background flex flex-col overflow-hidden">
      <header className="h-16 flex items-center justify-between px-6 border-b border-border shrink-0">
        <div className="flex items-center">
          <FileImage className="text-primary w-6 h-6 mr-3" />
          <h1 className="text-lg font-bold text-text tracking-wide">
            AirConvert
          </h1>
        </div>
        <button
          onClick={toggleTheme}
          className="flex items-center px-3 py-2 rounded-lg text-sm font-medium text-subText hover:bg-inputBg hover:text-text transition-colors"
        >
          {theme === "dark" ? (
            <Sun size={18} className="mr-2" />
          ) : (
            <Moon size={18} className="mr-2" />
          )}
          {theme === "dark" ? "Light mode" : "Dark mode"}
        </button>
      </header>

      <div className="flex-1 overflow-y-auto p-6 flex flex-col gap-6">
        <button
          onClick={handleBrowse}
          className={`flex flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed p-10 text-center transition-colors ${
            isHovering
              ? "border-primary bg-inputBg"
              : "border-border bg-card hover:bg-inputBg"
          }`}
        >
          <UploadCloud
            size={36}
            className={isHovering ? "text-primary" : "text-subText"}
          />
          <p className="text-text font-medium">
            Drag and drop images here, or click to browse
          </p>
          <p className="text-subText text-sm">
            jpg, png, webp, gif, bmp, tiff, svg
          </p>
        </button>

        {files.length > 0 && (
          <div className="flex flex-col gap-3">
            <div className="flex items-center justify-between">
              <h2 className="text-text font-semibold">
                Queue ({files.length})
              </h2>
              <button
                onClick={clearAll}
                className="text-sm text-subText hover:text-danger transition-colors"
              >
                Clear all
              </button>
            </div>

            <div className="flex flex-col gap-2 rounded-lg border border-border bg-card overflow-hidden">
              {files.map((file) => {
                const result = resultByPath.get(file.path);
                return (
                  <div
                    key={file.path}
                    className="flex items-center justify-between px-4 py-3 border-b border-border last:border-b-0"
                  >
                    <div className="flex items-center gap-3 min-w-0">
                      <FileImage size={18} className="text-subText shrink-0" />
                      <div className="min-w-0">
                        <p className="text-text text-sm truncate">
                          {fileNameOf(file.path)}
                        </p>
                        <p className="text-subText text-xs uppercase">
                          {extensionOf(file.path)}
                        </p>
                      </div>
                    </div>

                    <div className="flex items-center gap-3 shrink-0">
                      {result &&
                        (result.success ? (
                          <span className="text-xs text-primary">
                            Saved as {fileNameOf(result.output_path ?? "")}
                          </span>
                        ) : (
                          <span className="text-xs text-danger">
                            {result.error}
                          </span>
                        ))}
                      <button
                        onClick={() => removeFile(file.path)}
                        className="text-subText hover:text-danger transition-colors"
                      >
                        <X size={16} />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>

      <footer className="border-t border-border p-4 flex items-center justify-end gap-3 shrink-0">
        <select
          value={targetFormat}
          onChange={(e) => setTargetFormat(e.target.value as OutputFormatValue)}
          className="rounded-lg border border-border bg-inputBg px-3 py-2 text-sm text-text outline-none"
        >
          {OUTPUT_FORMATS.map((format) => (
            <option key={format.value} value={format.value}>
              Convert to {format.label}
            </option>
          ))}
        </select>
        <button
          onClick={handleConvert}
          disabled={files.length === 0 || isConverting}
          className="flex items-center gap-2 rounded-lg bg-primary px-5 py-2 text-sm font-medium text-white disabled:opacity-50 disabled:cursor-not-allowed transition-opacity"
        >
          {isConverting && <Loader2 size={16} className="animate-spin" />}
          {isConverting ? "Converting..." : "Convert"}
        </button>
      </footer>
    </main>
  );
}

export default App;
