import { useEffect, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";

/**
 * Tauri's webview drag-drop event gives real filesystem paths, unlike the
 * HTML5 drag/drop API which only exposes File objects with no path in a
 * webview context. `isHovering` drives the drop-zone's active/hover style.
 */
export function useFileDrop(onDrop: (paths: string[]) => void) {
  const [isHovering, setIsHovering] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "over") {
          setIsHovering(true);
        } else if (event.payload.type === "drop") {
          setIsHovering(false);
          onDrop(event.payload.paths);
        } else {
          setIsHovering(false);
        }
      })
      .then((fn) => {
        unlisten = fn;
      });

    return () => unlisten?.();
  }, [onDrop]);

  return { isHovering };
}
