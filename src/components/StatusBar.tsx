import { useQueueStore } from "../store/useQueueStore";

function fmt(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

export function StatusBar() {
  const files = useQueueStore((s) => s.files);

  const total = files.length;
  const clean = files.filter((f) => f.status === "clean").length;
  const processing = files.filter((f) => f.status === "processing").length;
  const totalSaved = files.reduce((acc, f) => acc + (f.bytesSaved ?? 0), 0);
  const gpsCount = files.filter((f) =>
    f.metadataBefore.some((m) => m.category === "Gps")
  ).length;

  return (
    <div
      className="flex items-center justify-between h-8 px-4 text-xs text-slate-500 shrink-0"
      style={{
        background: "rgba(8,8,15,0.95)",
        borderTop: "1px solid rgba(28,28,48,0.8)",
      }}
    >
      <div className="flex items-center gap-4">
        <span>
          <span className="text-slate-400">{total}</span> files
        </span>
        {clean > 0 && (
          <span>
            <span className="text-emerald-400">{clean}</span> cleaned
          </span>
        )}
        {processing > 0 && (
          <span>
            <span className="text-violet-400">{processing}</span> processing
          </span>
        )}
        {totalSaved > 0 && (
          <span>
            <span className="text-cyan-400">{fmt(totalSaved)}</span> stripped
          </span>
        )}
        {gpsCount > 0 && (
          <span>
            <span className="text-red-400">{gpsCount}</span> GPS removed
          </span>
        )}
      </div>

      <div className="flex items-center gap-3">
        <span className="text-slate-600">pnpm · Tauri v2 · Rust</span>
        <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" title="Ready" />
      </div>
    </div>
  );
}
