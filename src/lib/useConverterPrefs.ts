import { useEffect, useState } from "react";

/**
 * Remembers the last-used target format and output folder per converter
 * page, scoped by a storage key unique to that page (e.g. "images",
 * "audio"). Falls back silently to the given defaults if localStorage is
 * unavailable (private browsing, blocked storage, etc.) — this is a
 * convenience, not something that should ever break the page.
 */
export function useConverterPrefs<F extends string>(
  storageKey: string,
  defaultFormat: F,
) {
  const formatKey = `airconvert-${storageKey}-format`;
  const outputDirKey = `airconvert-${storageKey}-output-dir`;

  const [targetFormat, setTargetFormatState] = useState<F>(() => {
    try {
      return (localStorage.getItem(formatKey) as F | null) ?? defaultFormat;
    } catch {
      return defaultFormat;
    }
  });

  const [outputDir, setOutputDirState] = useState<string | null>(() => {
    try {
      return localStorage.getItem(outputDirKey);
    } catch {
      return null;
    }
  });

  useEffect(() => {
    try {
      localStorage.setItem(formatKey, targetFormat);
    } catch {
      // Ignore — remembering the format is a convenience, not essential.
    }
  }, [formatKey, targetFormat]);

  useEffect(() => {
    try {
      if (outputDir) {
        localStorage.setItem(outputDirKey, outputDir);
      } else {
        localStorage.removeItem(outputDirKey);
      }
    } catch {
      // Ignore — same as above.
    }
  }, [outputDirKey, outputDir]);

  return {
    targetFormat,
    setTargetFormat: setTargetFormatState,
    outputDir,
    setOutputDir: setOutputDirState,
  };
}
