import { motion, AnimatePresence } from "framer-motion";
import {
  Clock, Loader2, Sparkles, CheckCircle2, XCircle, SkipForward, X,
} from "lucide-react";
import { useQueueStore } from "../store/useQueueStore";
import { scoreColor, scoreBg, scoreLabel } from "../lib/sensitivityScore";
import type { ContextMenuState } from "./ContextMenu";
import type { QueueFile } from "../types";

const STATUS_STYLE: Record<QueueFile["status"], { pill: string; icon: React.ReactNode; spin?: boolean }> = {
  pending:    { pill: "bg-slate-500/15 border-slate-500/30 text-slate-400",   icon: <Clock size={9} /> },
  inspecting: { pill: "bg-blue-500/15 border-blue-500/30 text-blue-400",      icon: <Loader2 size={9} />, spin: true },
  ready:      { pill: "bg-violet-500/15 border-violet-500/30 text-violet-400", icon: <Sparkles size={9} /> },
  processing: { pill: "bg-yellow-500/15 border-yellow-500/30 text-yellow-400", icon: <Loader2 size={9} />, spin: true },
  clean:      { pill: "bg-emerald-500/15 border-emerald-500/30 text-emerald-400", icon: <CheckCircle2 size={9} /> },
  failed:     { pill: "bg-red-500/15 border-red-500/30 text-red-400",          icon: <XCircle size={9} /> },
  skipped:    { pill: "bg-slate-600/15 border-slate-600/30 text-slate-500",    icon: <SkipForward size={9} /> },
};

export function QueueCard({
  file,
  onContextMenu,
}: {
  file: QueueFile;
  onContextMenu: (state: ContextMenuState) => void;
}) {
  const { activeFileId, setActiveFile, removeFile } = useQueueStore();
  const isActive = activeFileId === file.id;
  const { pill, icon, spin } = STATUS_STYLE[file.status];

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    onContextMenu({ x: e.clientX, y: e.clientY, fileId: file.id });
  };

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 10, scale: 0.97 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, scale: 0.92 }}
      transition={{ type: "spring", stiffness: 280, damping: 26 }}
      onClick={() => setActiveFile(file.id)}
      onContextMenu={handleContextMenu}
      className={`relative glass-card p-3 cursor-pointer overflow-hidden select-none ${
        isActive ? "glass-card-active" : ""
      }`}
    >
      {}
      <AnimatePresence>
        {file.status === "processing" && (
          <motion.div
            key="wipe"
            initial={{ x: "-100%" }}
            animate={{ x: "200%" }}
            transition={{ duration: 1.1, ease: "easeInOut", repeat: Infinity, repeatDelay: 0.4 }}
            className="wipe-overlay"
          />
        )}
      </AnimatePresence>

      <div className="flex items-center gap-3">
        {}
        <div
          className="w-9 h-9 rounded-lg shrink-0 flex items-center justify-center overflow-hidden"
          style={{ background: "rgba(28,28,48,0.9)" }}
        >
          {file.thumbnail ? (
            <img src={`data:image/jpeg;base64,${file.thumbnail}`} alt=""
              className="w-full h-full object-cover rounded-lg" />
          ) : (
            <span className="text-[9px] font-bold text-slate-500 tracking-tight">{file.format}</span>
          )}
        </div>

        {}
        <div className="flex-1 min-w-0">
          <p className="text-xs font-semibold text-slate-200 truncate leading-tight mb-1">
            {file.name}
          </p>
          <div className="flex items-center gap-1.5 flex-wrap">
            {}
            <span className={`pill text-[9px] gap-1 ${pill}`}>
              <span className={spin ? "animate-spin" : ""}>{icon}</span>
              {file.status}
            </span>

            {}
            {file.sensitivityScore > 0 && (
              <span className={`pill text-[9px] ${scoreBg(file.sensitivityScore)} ${scoreColor(file.sensitivityScore)}`}>
                {scoreLabel(file.sensitivityScore)} {file.sensitivityScore}
              </span>
            )}

            {}
            {file.bytesSaved !== undefined && file.bytesSaved > 0 && (
              <span className="text-[9px] text-emerald-500/70">
                −{fmtBytes(file.bytesSaved)}
              </span>
            )}
          </div>
        </div>

        {}
        <button
          onClick={(e) => { e.stopPropagation(); removeFile(file.id); }}
          className="text-slate-600 hover:text-red-400 transition-colors rounded p-0.5 shrink-0"
        >
          <X size={13} />
        </button>
      </div>

      {}
      <AnimatePresence>
        {file.status === "clean" && (
          <motion.div
            initial={{ opacity: 0.5 }}
            animate={{ opacity: 0 }}
            transition={{ duration: 0.9 }}
            className="absolute inset-0 rounded-2xl bg-emerald-500/15 pointer-events-none"
          />
        )}
      </AnimatePresence>

      {}
      {file.status === "processing" && (
        <motion.div
          className="absolute bottom-0 left-0 h-0.5 gradient-accent"
          initial={{ width: "0%" }}
          animate={{ width: `${file.progress}%` }}
        />
      )}
    </motion.div>
  );
}

function fmtBytes(b: number) {
  if (b < 1024) return `${b}B`;
  if (b < 1_048_576) return `${(b / 1024).toFixed(1)}KB`;
  return `${(b / 1_048_576).toFixed(2)}MB`;
}
