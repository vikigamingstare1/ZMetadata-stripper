import { getCurrentWindow } from "@tauri-apps/api/window";
import { useState, useEffect } from "react";
import { Minus, Square, Maximize2, X, Info } from "lucide-react";

interface Props {
  onAbout: () => void;
}

export function Titlebar({ onAbout }: Props) {
  const [maximized, setMaximized] = useState(false);
  const win = getCurrentWindow();

  useEffect(() => {
    win.isMaximized().then(setMaximized);
    const unlisten = win.onResized(async () => {
      setMaximized(await win.isMaximized());
    });
    return () => { unlisten.then((f) => f()); };
  }, []);

  return (
    <div
      data-tauri-drag-region
      className="flex items-center justify-between h-9 px-4 shrink-0 select-none"
      style={{ background: "rgba(8,8,15,0.98)", borderBottom: "1px solid rgba(255,255,255,0.05)" }}
    >
      {/* Logo */}
      <div className="flex items-center gap-2 pointer-events-none">
        <img src="/icon.png" alt="ZMetadata Stripper" className="w-5 h-5 rounded-sm" />
        <span className="text-[11px] font-bold tracking-widest text-slate-300 uppercase">
          ZMetadata <span className="gradient-text">Stripper</span>
        </span>
      </div>

      {/* Centre: About button (not in drag region so it's clickable) */}
      <button
        onClick={onAbout}
        className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-slate-600 hover:text-slate-300 hover:bg-white/5 transition-all text-[10px]"
        style={{ pointerEvents: "all" }}
        data-tauri-drag-region="false"
      >
        <Info size={11} />
        <span className="font-medium">About</span>
      </button>

      {/* Window controls */}
      <div className="flex items-center gap-0.5">
        <WinBtn icon={<Minus size={12} />} hover="hover:bg-white/10" onClick={() => win.minimize()} />
        <WinBtn
          icon={maximized ? <Square size={11} /> : <Maximize2 size={11} />}
          hover="hover:bg-white/10"
          onClick={() => maximized ? win.unmaximize() : win.maximize()}
        />
        <WinBtn icon={<X size={12} />} hover="hover:bg-red-600/80" onClick={() => win.close()} />
      </div>
    </div>
  );
}

function WinBtn({ icon, hover, onClick }: { icon: React.ReactNode; hover: string; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className={`w-8 h-7 flex items-center justify-center rounded text-slate-500 hover:text-slate-200 transition-colors ${hover}`}
    >
      {icon}
    </button>
  );
}
