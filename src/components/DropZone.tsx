import { useCallback, useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { listen } from "@tauri-apps/api/event";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
import { useQueueStore } from "../store/useQueueStore";
import { inspectFile } from "../lib/tauri";
import type { QueueFile } from "../types";

// Normalize paths — Tauri v2 on Linux sometimes gives file:// URIs
function normalizePath(p: string): string {
  if (p.startsWith("file://")) {
    return decodeURIComponent(p.slice(7));
  }
  return p;
}

function makeQueueFile(rawPath: string): QueueFile {
  const path = normalizePath(rawPath);
  const name = path.split(/[/\\]/).pop() ?? path;
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  return {
    id: crypto.randomUUID(),
    name,
    path,
    size: 0,
    format: ext.toUpperCase(),
    status: "pending",
    progress: 0,
    sensitivityScore: 0,
    metadataBefore: [],
    metadataAfter: [],
    fieldsInjected: [],
  };
}

export function DropZone() {
  const [draggingOver, setDraggingOver] = useState(false);
  const { addFiles, updateFile, setActiveFile } = useQueueStore();

  const enqueuePaths = useCallback(
    async (rawPaths: string[]) => {
      const newFiles = rawPaths.map(makeQueueFile);
      addFiles(newFiles);
      if (newFiles[0]) setActiveFile(newFiles[0].id);

      for (const f of newFiles) {
        updateFile(f.id, { status: "inspecting" });
        try {
          const result = await inspectFile(f.path);
          updateFile(f.id, {
            status: "ready",
            format: result.format,
            sensitivityScore: result.sensitivity_score,
            metadataBefore: result.fields,
            thumbnail: result.thumbnail_b64 ?? undefined,
          });
        } catch (err) {
          console.error("inspect failed:", f.path, err);
          updateFile(f.id, { status: "ready" });
        }
      }
    },
    [addFiles, updateFile, setActiveFile]
  );

  // Tauri v2 drag-drop events (Linux/Windows/macOS)
  useEffect(() => {
    // Tauri v2 renamed: file-drop → drag-drop, file-drop-hover → drag-over, file-drop-cancelled → drag-leave
    const unlisten = listen<{ paths: string[] }>("tauri://drag-drop", (ev) => {
      enqueuePaths(ev.payload.paths);
      setDraggingOver(false);
    });
    const unlistenOver = listen("tauri://drag-over", () => setDraggingOver(true));
    const unlistenLeave = listen("tauri://drag-leave", () => setDraggingOver(false));

    return () => {
      unlisten.then((f) => f());
      unlistenOver.then((f) => f());
      unlistenLeave.then((f) => f());
    };
  }, [enqueuePaths]);

  // Click-to-browse: use Tauri dialog (gives real absolute paths, works on all platforms)
  const handleClick = useCallback(async () => {
    const selected = await dialogOpen({
      multiple: true,
      filters: [
        { name: "Images", extensions: ["jpg","jpeg","png","webp","tiff","tif","gif","bmp","avif","heic","heif","svg"] },
        { name: "Documents", extensions: ["pdf","docx","xlsx","pptx","odt","ods","odp","rtf"] },
        { name: "Audio", extensions: ["mp3","flac","ogg","oga","opus","wav","wave","m4a","aiff","aif"] },
        { name: "Video", extensions: ["mp4","mov","mkv","webm","m4v"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    enqueuePaths(paths);
  }, [enqueuePaths]);

  return (
    <div
      onClick={handleClick}
      onDragOver={(e) => e.preventDefault()}
      className={`relative flex flex-col items-center justify-center h-52 rounded-2xl border-2 border-dashed cursor-pointer transition-all duration-300 ${
        draggingOver ? "drop-zone-active" : "drop-zone-idle"
      }`}
      style={{
        background: draggingOver
          ? "rgba(124, 58, 237, 0.08)"
          : "rgba(15, 15, 26, 0.6)",
        borderColor: draggingOver
          ? "rgba(124, 58, 237, 0.9)"
          : "rgba(109, 40, 217, 0.3)",
      }}
    >
      <AnimatePresence mode="wait">
        {draggingOver ? (
          <motion.div
            key="active"
            initial={{ scale: 0.8, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.8, opacity: 0 }}
            className="flex flex-col items-center gap-3 pointer-events-none"
          >
            <motion.div
              animate={{ y: [-4, 4, -4] }}
              transition={{ duration: 0.8, repeat: Infinity }}
              className="text-4xl"
            >
              ↓
            </motion.div>
            <p className="text-violet-300 font-bold text-sm tracking-wide">Drop to add files</p>
          </motion.div>
        ) : (
          <motion.div
            key="idle"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="flex flex-col items-center gap-3 pointer-events-none"
          >
            <div className="text-5xl opacity-30">◈</div>
            <p className="text-slate-400 text-sm font-medium">
              Drop files here or{" "}
              <span className="text-violet-400 underline underline-offset-2">click to browse</span>
            </p>
            <p className="text-slate-600 text-xs">
              Images · Documents · Audio · Video — 25+ formats
            </p>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
