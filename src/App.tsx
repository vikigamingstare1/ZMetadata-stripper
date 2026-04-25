import { useState, useCallback, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AnimatePresence, motion } from "framer-motion";
import { Titlebar } from "./components/Titlebar";
import { Sidebar } from "./components/Sidebar";
import { DropZone } from "./components/DropZone";
import { QueueCard } from "./components/QueueCard";
import { MetadataPanel } from "./components/MetadataPanel";
import { StatusBar } from "./components/StatusBar";
import { CommandPalette } from "./components/CommandPalette";
import { Settings } from "./components/Settings";
import { About } from "./components/About";
import { ToastStack } from "./components/Toast";
import { ContextMenu } from "./components/ContextMenu";
import type { ContextMenuState } from "./components/ContextMenu";
import { useQueueStore } from "./store/useQueueStore";
import { useSettingsStore } from "./store/useSettingsStore";
import { useToastStore } from "./store/useToastStore";
import { stripFile, getStripOptions } from "./lib/tauri";
import type { MetadataCategory, StripOptions } from "./types";

function applyExclusions(opts: StripOptions, excluded: MetadataCategory[]): StripOptions {
  const o = { ...opts };
  for (const cat of excluded) {
    if (cat === "Gps")        o.strip_gps = false;
    if (cat === "Author")     o.strip_author = false;
    if (cat === "Camera")     o.strip_camera = false;
    if (cat === "Timestamps") o.strip_timestamps = false;
    if (cat === "Software")   o.strip_icc = false;
    if (cat === "Iptc")       o.strip_iptc = false;
    if (cat === "Xmp")        o.strip_xmp = false;
    if (cat === "Other")      { o.strip_makernotes = false; o.strip_thumbnail = false; }
  }
  return o;
}

type ResizeDir = "top"|"bottom"|"left"|"right"|"topLeft"|"topRight"|"bottomLeft"|"bottomRight";

const RESIZE_EDGES: { dir: ResizeDir; className: string }[] = [
  { dir: "top",         className: "absolute top-0 inset-x-0 h-1 cursor-n-resize z-50" },
  { dir: "bottom",      className: "absolute bottom-0 inset-x-0 h-1 cursor-s-resize z-50" },
  { dir: "left",        className: "absolute left-0 inset-y-0 w-1 cursor-w-resize z-50" },
  { dir: "right",       className: "absolute right-0 inset-y-0 w-1 cursor-e-resize z-50" },
  { dir: "topLeft",     className: "absolute top-0 left-0 w-3 h-3 cursor-nw-resize z-50" },
  { dir: "topRight",    className: "absolute top-0 right-0 w-3 h-3 cursor-ne-resize z-50" },
  { dir: "bottomLeft",  className: "absolute bottom-0 left-0 w-3 h-3 cursor-sw-resize z-50" },
  { dir: "bottomRight", className: "absolute bottom-0 right-0 w-3 h-3 cursor-se-resize z-50" },
];

export default function App() {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [ctxMenu, setCtxMenu] = useState<ContextMenuState | null>(null);
  const win = getCurrentWindow();
  const { files, updateFile, setProcessing, isProcessing } = useQueueStore();
  const { activePresetId, outputMode, outputDir, concurrency, theme } = useSettingsStore();

  useEffect(() => {
    const root = document.documentElement;
    if (theme === "system") {
      const mq = window.matchMedia("(prefers-color-scheme: light)");
      root.setAttribute("data-theme", mq.matches ? "light" : "dark");
      const handler = (e: MediaQueryListEvent) =>
        root.setAttribute("data-theme", e.matches ? "light" : "dark");
      mq.addEventListener("change", handler);
      return () => mq.removeEventListener("change", handler);
    } else {
      root.setAttribute("data-theme", theme);
    }
  }, [theme]);
  const { push: pushToast } = useToastStore();

  const handleStripAll = useCallback(async () => {
    const pending = files.filter((f) => f.status === "ready" || f.status === "pending");
    if (pending.length === 0) {
      pushToast("warning", "No files ready to strip.");
      return;
    }

    setProcessing(true);
    const opts = await getStripOptions(activePresetId);
    opts.output_mode = outputMode;
    opts.output_dir = outputDir;

    const chunks: typeof pending[] = [];
    for (let i = 0; i < pending.length; i += concurrency) {
      chunks.push(pending.slice(i, i + concurrency));
    }

    for (const chunk of chunks) {
      await Promise.all(
        chunk.map(async (f) => {
          updateFile(f.id, { status: "processing", progress: 0 });
          try {
            const fileOpts = applyExclusions({ ...opts }, f.excludedCategories);
            const result = await stripFile(f.path, fileOpts);
            if (result.success) {
              updateFile(f.id, {
                status: "clean",
                progress: 100,
                metadataAfter: result.fields_kept,
                fieldsInjected: result.fields_injected,
                outputPath: result.output_path,
                bytesSaved: result.bytes_saved,
              });
            } else {
              updateFile(f.id, { status: "failed", error: result.error ?? "Unknown error" });
              pushToast("error", `Failed: ${f.name}`);
            }
          } catch (e) {
            const msg = String(e);
            updateFile(f.id, { status: "failed", error: msg });
            pushToast("error", `${f.name}: ${msg}`);
          }
        })
      );
    }

    pushToast("success", `${pending.length} file(s) stripped successfully.`);
    setProcessing(false);
  }, [files, activePresetId, outputMode, outputDir, concurrency, updateFile, setProcessing, pushToast]);

  const handleStripSolo = useCallback(async (fileId: string) => {
    const f = files.find((f) => f.id === fileId);
    if (!f || f.status === "processing") return;
    const opts = await getStripOptions(activePresetId);
    opts.output_mode = outputMode;
    opts.output_dir = outputDir;
    updateFile(f.id, { status: "processing", progress: 0 });
    try {
      const soloOpts = applyExclusions({ ...opts }, f.excludedCategories);
      const result = await stripFile(f.path, soloOpts);
      if (result.success) {
        updateFile(f.id, {
          status: "clean", progress: 100,
          metadataAfter: result.fields_kept,
          fieldsInjected: result.fields_injected,
          outputPath: result.output_path,
          bytesSaved: result.bytes_saved,
        });
      } else {
        updateFile(f.id, { status: "failed", error: result.error ?? "Unknown error" });
        pushToast("error", `Failed: ${f.name}`);
      }
    } catch (e) {
      updateFile(f.id, { status: "failed", error: String(e) });
      pushToast("error", `${f.name}: ${String(e)}`);
    }
  }, [files, activePresetId, outputMode, outputDir, updateFile, pushToast]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") handleStripAll();
      if ((e.ctrlKey || e.metaKey) && e.key === ",") setSettingsOpen(true);
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [handleStripAll]);

  return (
    <div className="relative flex flex-col h-screen overflow-hidden"
      style={{ background: "var(--color-void-950)" }}>

      {RESIZE_EDGES.map(({ dir, className }) => (
        <div key={dir} className={className}
          onMouseDown={() => win.startResizeDragging(dir as any)} />
      ))}

      <Titlebar onAbout={() => setAboutOpen(true)} onSettings={() => setSettingsOpen(true)} />

      <div className="flex flex-1 min-h-0 overflow-hidden">
        <Sidebar />

        <main className="flex-1 flex flex-col gap-3 p-3 min-w-0 overflow-hidden">
          <DropZone />

          <div className="flex-1 overflow-y-auto min-h-0">
            {files.length === 0 ? (
              <div className="flex items-center justify-center h-full">
                <p className="text-xs text-slate-700 italic">Drop files above to begin.</p>
              </div>
            ) : (
              <div className="flex flex-col gap-2">
                <AnimatePresence>
                  {files.map((f) => (
                    <QueueCard
                      key={f.id}
                      file={f}
                      onContextMenu={setCtxMenu}
                    />
                  ))}
                </AnimatePresence>
              </div>
            )}
          </div>

          {files.length > 0 && (
            <motion.button
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              onClick={handleStripAll}
              disabled={isProcessing}
              className={`w-full py-3 rounded-xl font-bold text-sm tracking-wide transition-all duration-200 shadow-lg ${
                isProcessing
                  ? "opacity-50 cursor-not-allowed bg-slate-800 text-slate-400"
                  : "gradient-accent text-white hover:brightness-110"
              }`}
            >
              {isProcessing ? (
                <span className="flex items-center justify-center gap-2">
                  <span className="animate-spin inline-block">⟳</span> Processing…
                </span>
              ) : (
                `Strip All  (${files.filter((f) => f.status === "ready" || f.status === "pending").length} ready)  ▶`
              )}
            </motion.button>
          )}
        </main>

        <MetadataPanel />
      </div>

      <StatusBar onSettings={() => setSettingsOpen(true)} />
      <CommandPalette onStripAll={handleStripAll} onOpenSettings={() => setSettingsOpen(true)} />
      <Settings open={settingsOpen} onClose={() => setSettingsOpen(false)} />
      <About open={aboutOpen} onClose={() => setAboutOpen(false)} />
      <ToastStack />
      <ContextMenu
        menu={ctxMenu}
        onClose={() => setCtxMenu(null)}
        onStripSolo={handleStripSolo}
        onRemove={(id) => { const { removeFile } = useQueueStore.getState(); removeFile(id); setCtxMenu(null); }}
      />
    </div>
  );
}
