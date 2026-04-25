import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  MapPin, User, Camera, Clock, Cpu, FileText, Tag, MoreHorizontal,
  ChevronDown, ChevronRight, AlertTriangle, Zap, CheckCircle2, Shield, EyeOff,
} from "lucide-react";
import { useQueueStore } from "../store/useQueueStore";
import type { MetadataCategory, MetadataField, RiskLevel } from "../types";


const CAT_META: Record<MetadataCategory, { icon: React.ReactNode; label: string }> = {
  Gps:        { icon: <MapPin size={11} />,         label: "GPS Location" },
  Author:     { icon: <User size={11} />,           label: "Author / Identity" },
  Camera:     { icon: <Camera size={11} />,         label: "Camera" },
  Timestamps: { icon: <Clock size={11} />,          label: "Timestamps" },
  Software:   { icon: <Cpu size={11} />,            label: "Software" },
  Iptc:       { icon: <FileText size={11} />,       label: "IPTC" },
  Xmp:        { icon: <Tag size={11} />,            label: "XMP" },
  Other:      { icon: <MoreHorizontal size={11} />, label: "Other" },
};

const CAT_ORDER: MetadataCategory[] = [
  "Gps", "Author", "Camera", "Timestamps", "Software", "Iptc", "Xmp", "Other",
];

const RISK_DOT: Record<RiskLevel, string> = {
  Critical: "bg-red-500",
  High:     "bg-orange-400",
  Medium:   "bg-yellow-400",
  Low:      "bg-slate-600",
};

const RISK_LABEL: Record<RiskLevel, string> = {
  Critical: "C", High: "H", Medium: "M", Low: "L",
};

const RISK_TEXT: Record<RiskLevel, string> = {
  Critical: "text-red-400",
  High:     "text-orange-400",
  Medium:   "text-yellow-400",
  Low:      "text-slate-600",
};

function groupByCategory(fields: MetadataField[]) {
  const map: Partial<Record<MetadataCategory, MetadataField[]>> = {};
  for (const f of fields) (map[f.category] ??= []).push(f);
  return map;
}


export function MetadataPanel() {
  const { files, activeFileId, toggleCategory } = useQueueStore();
  const file = files.find((f) => f.id === activeFileId);
  const [tab, setTab] = useState<"before" | "after">("before");
  const [collapsed, setCollapsed] = useState<Set<MetadataCategory>>(new Set());

  const toggle = (cat: MetadataCategory) =>
    setCollapsed((p) => { const n = new Set(p); n.has(cat) ? n.delete(cat) : n.add(cat); return n; });


  if (!file) {
    return (
      <aside className="w-[340px] shrink-0 relative border-l border-white/5"
        style={{ background: "#08080f" }}>
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="flex flex-col items-center gap-3 px-8 text-center">
            <Shield size={32} className="text-slate-800" />
            <p className="text-xs text-slate-600 leading-relaxed">
              Select a file from the queue to inspect its metadata
            </p>
          </div>
        </div>
      </aside>
    );
  }

  const fields = tab === "before" ? file.metadataBefore : file.metadataAfter;
  const groups = groupByCategory(fields);
  const gpsFields = file.metadataBefore.filter((f) => f.category === "Gps");


  return (
    <aside className="w-[340px] shrink-0 relative border-l border-white/5"
      style={{ background: "#09090f" }}>

      {}
      <div className="absolute inset-0 flex flex-col overflow-hidden">

        {}
        <div className="shrink-0 px-4 pt-4 pb-3 border-b border-white/[0.06]">
          <p className="text-[11px] font-semibold text-slate-200 truncate leading-tight">
            {file.name}
          </p>
          <p className="text-[9px] text-slate-600 font-mono truncate mt-0.5">
            {file.format}&nbsp;·&nbsp;{file.path}
          </p>

          {}
          <div className="flex gap-1 mt-3 p-1 rounded-xl bg-black/40">
            {(["before", "after"] as const).map((t) => (
              <button key={t} onClick={() => setTab(t)}
                className={`flex-1 py-1.5 rounded-lg text-[11px] font-semibold transition-all ${
                  tab === t ? "gradient-accent text-white" : "text-slate-500 hover:text-slate-300"
                }`}>
                {t === "before" ? "Before Strip" : "After Strip"}
              </button>
            ))}
          </div>
        </div>

        {}
        <div className="flex-1 overflow-y-auto min-h-0" style={{ overflowY: "auto" }}>
          <div className="flex flex-col gap-2 p-3">

            {}
            <AnimatePresence>
              {tab === "before" && gpsFields.length > 0 && (
                <motion.div key="gps"
                  initial={{ opacity: 0, y: -6 }} animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0 }}
                  className="rounded-xl p-3 border border-red-500/25 shrink-0"
                  style={{ background: "rgba(239,68,68,0.08)" }}>
                  <div className="flex items-center gap-1.5 mb-2">
                    <AlertTriangle size={11} className="text-red-400 shrink-0" />
                    <span className="text-[11px] font-bold text-red-400">GPS Location Detected</span>
                  </div>
                  {gpsFields.map((f) => (
                    <div key={f.name} className="flex items-baseline justify-between gap-3 py-0.5">
                      <span className="text-[9px] text-red-300/60 font-mono shrink-0">{f.name}</span>
                      <span className="text-[9px] text-red-300 font-mono font-semibold text-right break-all">{f.value}</span>
                    </div>
                  ))}
                  <p className="text-[9px] text-red-400/40 mt-2 italic">Will be removed when stripped.</p>
                </motion.div>
              )}
            </AnimatePresence>

            {}
            <AnimatePresence>
              {tab === "after" && file.fieldsInjected.length > 0 && (
                <motion.div key="injected"
                  initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
                  className="rounded-xl p-3 border border-violet-500/20 shrink-0"
                  style={{ background: "rgba(124,58,237,0.07)" }}>
                  <div className="flex items-center gap-1.5 mb-2">
                    <Zap size={11} className="text-violet-400 shrink-0" />
                    <span className="text-[11px] font-bold text-violet-400">Neutral metadata injected</span>
                  </div>
                  {file.fieldsInjected.map((f) => (
                    <div key={f.name} className="flex justify-between gap-3 py-0.5">
                      <span className="text-[9px] text-slate-400 font-mono truncate">{f.name}</span>
                      <span className="text-[9px] text-violet-300 font-mono shrink-0">{f.value}</span>
                    </div>
                  ))}
                </motion.div>
              )}
            </AnimatePresence>

            {}
            <AnimatePresence>
              {tab === "after" && file.status === "clean" && file.outputPath && (
                <motion.div key="clean"
                  initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
                  className="rounded-xl p-3 border border-emerald-500/20 shrink-0"
                  style={{ background: "rgba(16,185,129,0.06)" }}>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-1.5">
                      <CheckCircle2 size={11} className="text-emerald-400 shrink-0" />
                      <span className="text-[11px] font-bold text-emerald-400">Clean</span>
                    </div>
                    {!!file.bytesSaved && file.bytesSaved > 0 && (
                      <span className="text-[9px] text-emerald-400/60 font-mono">
                        −{fmtBytes(file.bytesSaved)} stripped
                      </span>
                    )}
                  </div>
                  <p className="text-[9px] text-slate-500 font-mono truncate mt-1">{file.outputPath}</p>
                </motion.div>
              )}
            </AnimatePresence>

            {}
            {fields.length === 0 && (
              <div className="rounded-xl p-6 border border-white/[0.04] text-center shrink-0">
                <p className="text-[11px] text-slate-600">
                  {tab === "after" && file.status !== "clean"
                    ? "Strip the file to see what remains."
                    : tab === "after"
                      ? "No metadata remaining — file is clean."
                      : "No metadata fields found."}
                </p>
              </div>
            )}

            {}
            {CAT_ORDER.filter((cat) => groups[cat]).map((cat) => {
              const catFields = groups[cat]!;
              const open = !collapsed.has(cat);
              const excluded = tab === "before" && file.excludedCategories.includes(cat);
              const { icon, label } = CAT_META[cat];

              return (
                <div key={cat}
                  className="rounded-xl border overflow-hidden shrink-0 transition-colors"
                  style={{
                    background: excluded ? "rgba(239,68,68,0.04)" : "rgba(14,14,22,0.9)",
                    borderColor: excluded ? "rgba(239,68,68,0.2)" : "rgba(255,255,255,0.06)",
                  }}>

                  {}
                  <div className="flex items-center">
                    <button onClick={() => toggle(cat)}
                      className="flex-1 flex items-center gap-2 px-3 py-2.5 hover:bg-white/[0.03] transition-colors">
                      <span className={`shrink-0 ${excluded ? "text-red-500/50" : "text-slate-500"}`}>{icon}</span>
                      <span className={`text-[11px] font-semibold flex-1 text-left ${excluded ? "line-through text-slate-600" : "text-slate-300"}`}>{label}</span>
                      <span className="text-[9px] font-mono text-slate-600 mr-1">{catFields.length}</span>
                      {open
                        ? <ChevronDown size={11} className="text-slate-600 shrink-0" />
                        : <ChevronRight size={11} className="text-slate-600 shrink-0" />}
                    </button>
                    {tab === "before" && (
                      <button
                        title={excluded ? "Include in strip" : "Exclude from strip"}
                        onClick={() => toggleCategory(file.id, cat)}
                        className={`px-2.5 py-2.5 transition-colors shrink-0 ${excluded ? "text-red-400 hover:text-red-300" : "text-slate-700 hover:text-slate-400"}`}
                      >
                        <EyeOff size={10} />
                      </button>
                    )}
                  </div>

                  {}
                  <AnimatePresence initial={false}>
                    {open && (
                      <motion.div
                        key="body"
                        initial={{ height: 0 }}
                        animate={{ height: "auto" }}
                        exit={{ height: 0 }}
                        transition={{ duration: 0.16, ease: "easeInOut" }}
                        style={{ overflow: "hidden" }}>
                        <div className="px-3 pb-3 flex flex-col gap-1.5 border-t border-white/[0.04]">
                          {catFields.map((f) => (
                            <FieldRow key={f.name + f.value} field={f} />
                          ))}
                        </div>
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
              );
            })}

            {}
            <div className="h-4 shrink-0" />
          </div>
        </div>
      </div>
    </aside>
  );
}


function FieldRow({ field }: { field: MetadataField }) {
  return (
    <div className="rounded-lg px-2.5 py-2 flex items-start gap-2.5 mt-1"
      style={{ background: "rgba(255,255,255,0.03)" }}>
      <span className={`w-1.5 h-1.5 rounded-full shrink-0 mt-[3px] ${RISK_DOT[field.risk]}`} />
      <div className="flex-1 min-w-0">
        <p className="text-[9px] text-slate-500 font-mono leading-tight mb-0.5">{field.name}</p>
        <p className="text-[10px] text-slate-200 font-mono leading-snug break-words">{field.value}</p>
      </div>
      <span className={`text-[8px] font-bold shrink-0 mt-0.5 ${RISK_TEXT[field.risk]}`}>
        {RISK_LABEL[field.risk]}
      </span>
    </div>
  );
}

function fmtBytes(b: number) {
  if (b < 1024) return `${b}B`;
  if (b < 1_048_576) return `${(b / 1024).toFixed(1)}KB`;
  return `${(b / 1_048_576).toFixed(2)}MB`;
}
