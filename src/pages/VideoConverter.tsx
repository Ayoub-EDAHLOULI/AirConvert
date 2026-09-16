import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { Link } from "react-router-dom";
import {
  Moon,
  Sun,
  UploadCloud,
  Video,
  X,
  Loader2,
  FolderOpen,
  CheckCircle2,
  ArrowLeft,
  Info,
  Square,
} from "lucide-react";
import { useTheme } from "../theme/useTheme";
import { useFileDrop } from "../lib/useFileDrop";
import { useConverterPrefs } from "../lib/useConverterPrefs";
import {
  OUTPUT_FORMATS,
  extensionOf,
  fileNameOf,
  isSupportedVideo,
  type OutputFormatValue,
} from "../lib/videoFormats";
import "../App.css";

type QueuedFile = {
  path: string;
};

type ConversionResult = {
  source_path: string;
  success: boolean;
  output_path: string | null;
  error: string | null;
};

type ConversionProgress = {
  completed: number;
  total: number;
  result: ConversionResult;
};

export default function VideoConverter() {
  const { theme, toggleTheme } = useTheme();
  const [files, setFiles] = useState<QueuedFile[]>([]);
  const { targetFormat, setTargetFormat, outputDir, setOutputDir } =
    useConverterPrefs<OutputFormatValue>("video", "mp4");
  const [isConverting, setIsConverting] = useState(false);
  const [results, setResults] = useState<ConversionResult[] | null>(null);
  const [progress, setProgress] = useState<{
    completed: number;
    total: number;
  } | null>(null);

  useEffect(() => {
    const unlisten = listen<ConversionProgress>(
      "conversion-progress",
      (event) => {
        const { completed, total, result } = event.payload;
        setProgress({ completed, total });
        setResults((current) => {
          const withoutThis = (current ?? []).filter(
            (r) => r.source_path !== result.source_path,
          );
          return [...withoutThis, result];
        });
      },
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const addPaths = useCallback((paths: string[]) => {
    setResults(null);
    setProgress(null);
    setFiles((current) => {
      const existing = new Set(current.map((f) => f.path));
      const additions = paths
        .filter(isSupportedVideo)
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
          name: "Video",
          extensions: ["mp4", "mov", "avi", "webm", "mkv", "flv", "wmv"],
        },
      ],
    });
    if (!selected) return;
    addPaths(Array.isArray(selected) ? selected : [selected]);
  }

  async function handleChooseOutputDir() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected) return;
    setOutputDir(selected as string);
    setResults(null);
  }

  function removeFile(path: string) {
    setFiles((current) => current.filter((f) => f.path !== path));
    setResults(null);
    setProgress(null);
  }

  function clearAll() {
    setFiles([]);
    setResults(null);
    setProgress(null);
  }

  async function handleConvert() {
    if (files.length === 0) return;
    setIsConverting(true);
    setResults(null);
    setProgress({ completed: 0, total: files.length });
    try {
      const output = await invoke<ConversionResult[]>(
        "convert_video_files",
        {
          paths: files.map((f) => f.path),
          targetFormat,
          outputDir,
        },
      );
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

  async function handleCancel() {
    await invoke("cancel_conversion");
  }

  const resultByPath = new Map((results ?? []).map((r) => [r.source_path, r]));

  return (
    <main className="h-screen w-screen bg-background flex flex-col overflow-hidden">
      <header className="h-16 flex items-center justify-between px-6 border-b border-border shrink-0">
        <div className="flex items-center gap-4">
          <Link
            to="/"
            className="flex items-center text-subText hover:text-text transition-colors"
          >
            <ArrowLeft size={18} />
          </Link>
          <div className="flex items-center">
            <Video className="text-primary w-6 h-6 mr-3" />
            <h1 className="text-lg font-bold text-text tracking-wide">
              Video
            </h1>
          </div>
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
        <div className="flex items-start gap-2 rounded-lg border border-border bg-card px-4 py-3 text-sm text-subText">
          <Info size={16} className="shrink-0 mt-0.5 text-primary" />
          <p>
            Format conversion with solid default quality settings — no
            resolution or bitrate controls yet. Converting to GIF drops audio
            and uses a two-pass palette encode for better color quality.
          </p>
        </div>

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
            Drag and drop video files here, or click to browse
          </p>
          <p className="text-subText text-sm">
            mp4, mov, avi, webm, mkv, flv, wmv
          </p>
        </button>

        {files.length > 0 && (
          <div className="flex flex-col gap-3">
            <div className="flex items-center justify-between">
              <h2 className="text-text font-semibold">
                Queue ({files.length})
              </h2>
              <div className="flex items-center gap-4">
                {progress && (
                  <span className="text-sm text-subText">
                    {progress.completed} / {progress.total} converted
                  </span>
                )}
                <button
                  onClick={clearAll}
                  className="text-sm text-subText hover:text-danger transition-colors"
                >
                  Clear all
                </button>
              </div>
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
                      <Video size={18} className="text-subText shrink-0" />
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
                      {result ? (
                        result.success ? (
                          <span className="flex items-center gap-1.5 text-xs text-primary">
                            <CheckCircle2 size={14} />
                            Saved as {fileNameOf(result.output_path ?? "")}
                          </span>
                        ) : (
                          <span className="text-xs text-danger">
                            {result.error}
                          </span>
                        )
                      ) : (
                        isConverting && (
                          <span className="flex items-center gap-1.5 text-xs text-subText">
                            <Loader2 size={14} className="animate-spin" />
                            Converting...
                          </span>
                        )
                      )}
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

      <footer className="border-t border-border shrink-0">
        <div className="flex items-center justify-between gap-3 p-4">
          <button
            onClick={handleChooseOutputDir}
            className="flex items-center gap-2 min-w-0 rounded-lg border border-border bg-inputBg px-3 py-2 text-sm text-subText hover:text-text transition-colors"
          >
            <FolderOpen size={16} className="shrink-0" />
            <span className="truncate max-w-[280px]">
              {outputDir ?? "Save next to original file"}
            </span>
            {outputDir && (
              <span
                role="button"
                onClick={(e) => {
                  e.stopPropagation();
                  setOutputDir(null);
                  setResults(null);
                }}
                className="ml-1 shrink-0 hover:text-danger"
              >
                <X size={14} />
              </span>
            )}
          </button>

          <div className="flex items-center gap-3 shrink-0">
            <select
              value={targetFormat}
              onChange={(e) =>
                setTargetFormat(e.target.value as OutputFormatValue)
              }
              className="rounded-lg border border-border bg-inputBg px-3 py-2 text-sm text-text outline-none"
            >
              {OUTPUT_FORMATS.map((format) => (
                <option key={format.value} value={format.value}>
                  Convert to {format.label}
                </option>
              ))}
            </select>
            {isConverting ? (
              <button
                onClick={handleCancel}
                className="flex items-center gap-2 rounded-lg border border-danger px-5 py-2 text-sm font-medium text-danger hover:bg-danger hover:text-white transition-colors"
              >
                <Square size={14} />
                Cancel
              </button>
            ) : (
              <button
                onClick={handleConvert}
                disabled={files.length === 0}
                className="flex items-center gap-2 rounded-lg bg-primary px-5 py-2 text-sm font-medium text-white disabled:opacity-50 disabled:cursor-not-allowed transition-opacity"
              >
                Convert
              </button>
            )}
          </div>
        </div>
      </footer>
    </main>
  );
}
