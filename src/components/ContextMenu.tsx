import { useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Zap, Trash2 } from "lucide-react";

export interface ContextMenuState {
  x: number;
  y: number;
  fileId: string;
}

interface Props {
  menu: ContextMenuState | null;
  onClose: () => void;
  onStripSolo: (fileId: string) => void;
  onRemove: (fileId: string) => void;
}

export function ContextMenu({ menu, onClose, onStripSolo, onRemove }: Props) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!menu) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [menu, onClose]);


  const x = menu ? Math.min(menu.x, window.innerWidth - 160) : 0;
  const y = menu ? Math.min(menu.y, window.innerHeight - 100) : 0;

  return (
    <AnimatePresence>
      {menu && (
        <motion.div
          ref={ref}
          initial={{ opacity: 0, scale: 0.94, y: -4 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.94, y: -4 }}
          transition={{ duration: 0.1 }}
          style={{
            position: "fixed",
            left: x,
            top: y,
            zIndex: 999,
            background: "rgba(12,12,22,0.97)",
            border: "1px solid rgba(109,40,217,0.25)",
            borderRadius: 10,
            boxShadow: "0 8px 32px rgba(0,0,0,0.6), 0 0 0 1px rgba(255,255,255,0.03)",
            minWidth: 152,
            padding: "4px",
          }}
        >
          <MenuItem
            icon={<Zap size={12} />}
            label="Strip Solo"
            onClick={() => { onStripSolo(menu.fileId); onClose(); }}
            variant="accent"
          />
          <div style={{ height: 1, background: "rgba(255,255,255,0.05)", margin: "2px 0" }} />
          <MenuItem
            icon={<Trash2 size={12} />}
            label="Remove"
            onClick={() => { onRemove(menu.fileId); onClose(); }}
            variant="danger"
          />
        </motion.div>
      )}
    </AnimatePresence>
  );
}

function MenuItem({
  icon,
  label,
  onClick,
  variant,
}: {
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
  variant: "accent" | "danger";
}) {
  const color = variant === "danger" ? "hover:text-red-400 hover:bg-red-500/10" : "hover:text-violet-300 hover:bg-violet-500/10";
  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-[11px] font-semibold text-slate-400 transition-all ${color}`}
    >
      {icon}
      {label}
    </button>
  );
}
