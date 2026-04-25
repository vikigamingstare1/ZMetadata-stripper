import { motion, AnimatePresence } from "framer-motion";
import { useSettingsStore } from "../store/useSettingsStore";
import type { OutputMode } from "../types";

interface Props {
  open: boolean;
  onClose: () => void;
}

export function Settings({ open, onClose }: Props) {
  const settings = useSettingsStore();

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 z-50 flex items-center justify-center"
          style={{ background: "rgba(8,8,15,0.75)", backdropFilter: "blur(8px)" }}
          onClick={onClose}
        >
          <motion.div
            initial={{ opacity: 0, scale: 0.93, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.93, y: 20 }}
            transition={{ type: "spring", stiffness: 320, damping: 28 }}
            onClick={(e) => e.stopPropagation()}
            className="glass-card w-full max-w-lg p-6 shadow-2xl"
          >
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-base font-bold gradient-text">Settings</h2>
              <button onClick={onClose} className="text-slate-500 hover:text-slate-200 transition-colors">✕</button>
            </div>

            <div className="flex flex-col gap-5">
              {/* Theme */}
              <Section title="Appearance">
                <Row label="Theme">
                  <Select
                    value={settings.theme}
                    options={[["dark", "Dark"], ["light", "Light"], ["system", "System"]]}
                    onChange={(v) => settings.setTheme(v as "dark" | "light" | "system")}
                  />
                </Row>
              </Section>

              {/* Output */}
              <Section title="Output">
                <Row label="Output mode">
                  <Select
                    value={settings.outputMode}
                    options={[
                      ["Overwrite", "Overwrite original"],
                      ["SaveCopy", "Save clean copy"],
                      ["CustomDir", "Custom directory"],
                    ]}
                    onChange={(v) => settings.setOutputMode(v as OutputMode)}
                  />
                </Row>
                <Row label="Auto-backup before overwrite">
                  <Toggle value={settings.autoBackup} onChange={settings.setAutoBackup} />
                </Row>
              </Section>

              {/* Processing */}
              <Section title="Processing">
                <Row label={`Threads (${settings.concurrency})`}>
                  <input
                    type="range"
                    min={1}
                    max={16}
                    value={settings.concurrency}
                    onChange={(e) => settings.setConcurrency(Number(e.target.value))}
                    className="w-32 accent-violet-500"
                  />
                </Row>
              </Section>

              {/* System */}
              <Section title="System">
                <Row label="Minimize to tray on close">
                  <Toggle value={settings.minimizeToTray} onChange={settings.setMinimizeToTray} />
                </Row>
              </Section>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <p className="text-[10px] font-bold uppercase tracking-widest text-slate-500 mb-2">{title}</p>
      <div className="glass-card p-3 flex flex-col gap-2">{children}</div>
    </div>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-sm text-slate-300">{label}</span>
      {children}
    </div>
  );
}

function Select({
  value,
  options,
  onChange,
}: {
  value: string;
  options: [string, string][];
  onChange: (v: string) => void;
}) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="bg-void-800 border border-void-700 text-slate-200 text-xs rounded-lg px-3 py-1.5 outline-none focus:border-violet-500 transition-colors"
    >
      {options.map(([val, label]) => (
        <option key={val} value={val}>{label}</option>
      ))}
    </select>
  );
}

function Toggle({ value, onChange }: { value: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!value)}
      className={`w-10 h-5 rounded-full transition-all duration-200 relative ${
        value ? "bg-violet-600" : "bg-slate-700"
      }`}
    >
      <span
        className={`absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform duration-200 ${
          value ? "translate-x-5" : "translate-x-0"
        }`}
      />
    </button>
  );
}
