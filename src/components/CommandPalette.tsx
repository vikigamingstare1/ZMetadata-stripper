import { useState, useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useQueueStore } from "../store/useQueueStore";
import { useSettingsStore } from "../store/useSettingsStore";

interface Command {
  id: string;
  label: string;
  description: string;
  shortcut?: string;
  action: () => void;
}

interface Props {
  onStripAll: () => void;
  onOpenSettings: () => void;
}

export function CommandPalette({ onStripAll, onOpenSettings }: Props) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const { clearQueue } = useQueueStore();
  const { setActivePreset } = useSettingsStore();

  const commands: Command[] = [
    { id: "strip_all",   label: "Strip All Files",      description: "Process entire queue with active preset",           shortcut: "Ctrl+Enter", action: onStripAll },
    { id: "clear",       label: "Clear Queue",           description: "Remove all files from the queue",                  shortcut: "Ctrl+W",     action: clearQueue },
    { id: "settings",    label: "Open Settings",         description: "Configure presets, output, and preferences",       shortcut: "Ctrl+,",     action: onOpenSettings },
    { id: "p_max",       label: "Preset: Max Privacy",   description: "Switch to Max Privacy preset",                                             action: () => setActivePreset("max_privacy") },
    { id: "p_quality",   label: "Preset: Keep Quality",  description: "Switch to Keep Quality preset",                                            action: () => setActivePreset("keep_quality") },
    { id: "p_social",    label: "Preset: Social Media",  description: "Switch to Social Media preset",                                            action: () => setActivePreset("social_media") },
    { id: "p_docs",      label: "Preset: Docs Only",     description: "Switch to Documents Only preset",                                          action: () => setActivePreset("documents_only") },
  ];

  const filtered = query
    ? commands.filter(
        (c) =>
          c.label.toLowerCase().includes(query.toLowerCase()) ||
          c.description.toLowerCase().includes(query.toLowerCase())
      )
    : commands;

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        setOpen((o) => !o);
      }
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, []);

  useEffect(() => {
    if (open) {
      setQuery("");
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  const run = (cmd: Command) => {
    cmd.action();
    setOpen(false);
  };

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 z-50 flex items-start justify-center pt-32"
          style={{ background: "rgba(8,8,15,0.7)", backdropFilter: "blur(6px)" }}
          onClick={() => setOpen(false)}
        >
          <motion.div
            initial={{ opacity: 0, scale: 0.93, y: -16 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.93, y: -16 }}
            transition={{ type: "spring", stiffness: 340, damping: 28 }}
            onClick={(e) => e.stopPropagation()}
            className="glass-card w-full max-w-xl overflow-hidden shadow-2xl"
          >
            <div className="flex items-center gap-3 px-4 py-3 border-b border-void-700/60">
              <span className="text-slate-500 text-sm">⌕</span>
              <input
                ref={inputRef}
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search commands…"
                className="flex-1 bg-transparent text-sm text-slate-200 placeholder-slate-600 outline-none"
              />
              <kbd className="text-[10px] px-1.5 py-0.5 rounded border border-slate-700 text-slate-600">Esc</kbd>
            </div>

            <div className="max-h-72 overflow-y-auto py-2">
              {filtered.length === 0 ? (
                <p className="text-xs text-slate-600 px-4 py-3">No commands found.</p>
              ) : (
                filtered.map((cmd) => (
                  <button
                    key={cmd.id}
                    onClick={() => run(cmd)}
                    className="w-full text-left px-4 py-2.5 flex items-center justify-between hover:bg-violet-500/10 transition-colors"
                  >
                    <div>
                      <p className="text-sm font-medium text-slate-200">{cmd.label}</p>
                      <p className="text-xs text-slate-500">{cmd.description}</p>
                    </div>
                    {cmd.shortcut && (
                      <kbd className="text-[10px] px-1.5 py-0.5 rounded border border-slate-700 text-slate-500 shrink-0 ml-4">
                        {cmd.shortcut}
                      </kbd>
                    )}
                  </button>
                ))
              )}
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
