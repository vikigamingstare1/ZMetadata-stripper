import { motion } from "framer-motion";
import { BarChart2, CheckCircle2, MapPin, Zap, ListChecks } from "lucide-react";
import { useQueueStore } from "../store/useQueueStore";
import { usePresetStore } from "../store/usePresetStore";
import { useSettingsStore } from "../store/useSettingsStore";

export function Sidebar() {
  const files = useQueueStore((s) => s.files);
  const presets = usePresetStore((s) => s.presets);
  const { activePresetId, setActivePreset } = useSettingsStore();

  const total = files.length;
  const clean = files.filter((f) => f.status === "clean").length;
  const gpsCount = files.filter((f) =>
    f.metadataBefore.some((m) => m.category === "Gps")
  ).length;
  const savedBytes = files.reduce((a, f) => a + (f.bytesSaved ?? 0), 0);

  return (
    <aside className="w-60 shrink-0 relative border-r border-white/5">
    <div className="absolute inset-0 overflow-y-auto flex flex-col gap-2 p-3">

      {}
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        className="glass-card p-3"
      >
        <div className="flex items-center gap-1.5 mb-2.5">
          <BarChart2 size={11} className="text-slate-500" />
          <h2 className="text-[10px] font-bold uppercase tracking-widest text-slate-500">
            Session Stats
          </h2>
        </div>
        <Stat icon={<ListChecks size={10} />} label="Processed" value={total} />
        <Stat icon={<CheckCircle2 size={10} className="text-emerald-500" />} label="Cleaned" value={clean} color="text-emerald-400" />
        <Stat icon={<MapPin size={10} className="text-red-500" />} label="GPS removed" value={gpsCount} color="text-red-400" />
        <Stat icon={<Zap size={10} className="text-cyan-500" />} label="Data stripped" value={fmtBytes(savedBytes)} color="text-cyan-400" />
      </motion.div>

      {}
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.04 }}
        className="glass-card p-3"
      >
        <h2 className="text-[10px] font-bold uppercase tracking-widest text-slate-500 mb-2.5">
          Presets
        </h2>
        <div className="flex flex-col gap-1">
          {presets.map((p) => (
            <button
              key={p.id}
              onClick={() => setActivePreset(p.id)}
              className={`text-left px-2.5 py-2 rounded-lg text-xs transition-all duration-150 ${
                activePresetId === p.id
                  ? "bg-violet-600/25 border border-violet-500/40 text-violet-200"
                  : "hover:bg-white/[0.04] text-slate-400 hover:text-slate-200 border border-transparent"
              }`}
              title={p.description}
            >
              <span className="font-semibold block text-[11px]">{p.label}</span>
              <span className="text-slate-500 text-[9px] leading-tight line-clamp-1">
                {p.description}
              </span>
            </button>
          ))}
        </div>
      </motion.div>

      {}
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.08 }}
        className="glass-card p-3 flex-1 min-h-0 flex flex-col"
      >
        <h2 className="text-[10px] font-bold uppercase tracking-widest text-slate-500 mb-2.5 shrink-0">
          Activity
        </h2>
        <div className="flex flex-col gap-1 overflow-y-auto flex-1 min-h-0">
          {files.filter((f) => f.status === "clean").length === 0 ? (
            <p className="text-[10px] text-slate-600 italic">No completed operations yet.</p>
          ) : (
            files
              .filter((f) => f.status === "clean")
              .slice(-20)
              .reverse()
              .map((f) => (
                <div key={f.id} className="flex items-center gap-2 py-0.5">
                  <CheckCircle2 size={9} className="text-emerald-500 shrink-0" />
                  <span className="text-[10px] text-slate-400 truncate">{f.name}</span>
                </div>
              ))
          )}
        </div>
      </motion.div>
    </div>
    </aside>
  );
}

function Stat({
  icon, label, value, color = "text-slate-200",
}: {
  icon: React.ReactNode;
  label: string;
  value: string | number;
  color?: string;
}) {
  return (
    <div className="flex items-center justify-between py-1 gap-2">
      <div className="flex items-center gap-1.5">
        <span className="text-slate-600 shrink-0">{icon}</span>
        <span className="text-[10px] text-slate-500">{label}</span>
      </div>
      <span className={`text-[10px] font-bold mono ${color}`}>{value}</span>
    </div>
  );
}

function fmtBytes(b: number) {
  if (b < 1024) return `${b}B`;
  if (b < 1_048_576) return `${(b / 1024).toFixed(1)}KB`;
  return `${(b / 1_048_576).toFixed(2)}MB`;
}
