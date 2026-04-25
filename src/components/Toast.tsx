import { AnimatePresence, motion } from "framer-motion";
import { useToastStore } from "../store/useToastStore";
import type { ToastType } from "../types";

const ICONS: Record<ToastType, string> = {
  success: "✓",
  error: "✕",
  warning: "⚠",
  info: "ℹ",
};

const COLORS: Record<ToastType, string> = {
  success: "border-emerald-500/50 bg-emerald-500/10 text-emerald-300",
  error: "border-red-500/50 bg-red-500/10 text-red-300",
  warning: "border-yellow-500/50 bg-yellow-500/10 text-yellow-300",
  info: "border-blue-500/50 bg-blue-500/10 text-blue-300",
};

export function ToastStack() {
  const { toasts, dismiss } = useToastStore();

  return (
    <div className="fixed bottom-6 right-6 flex flex-col gap-2 z-50 pointer-events-none">
      <AnimatePresence>
        {toasts.map((t) => (
          <motion.div
            key={t.id}
            initial={{ opacity: 0, x: 60, scale: 0.9 }}
            animate={{ opacity: 1, x: 0, scale: 1 }}
            exit={{ opacity: 0, x: 60, scale: 0.9 }}
            transition={{ type: "spring", stiffness: 300, damping: 28 }}
            onClick={() => dismiss(t.id)}
            className={`pointer-events-auto flex items-center gap-3 px-4 py-3 rounded-xl border text-sm font-medium shadow-xl backdrop-blur-md cursor-pointer ${COLORS[t.type]}`}
          >
            <span className="text-base leading-none">{ICONS[t.type]}</span>
            <span className="text-slate-100">{t.message}</span>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
