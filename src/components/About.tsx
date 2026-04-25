import { motion, AnimatePresence } from "framer-motion";
import { X, GitFork, Globe, Shield, Layers, Code2, ExternalLink } from "lucide-react";
import { open as shellOpen } from "@tauri-apps/plugin-shell";

interface Props {
  open: boolean;
  onClose: () => void;
}

const STACK = [
  { layer: "Frontend",  items: ["React 19", "TypeScript", "Tailwind CSS v4", "Framer Motion", "Lucide React", "Zustand"] },
  { layer: "Backend",   items: ["Rust", "Tauri v2", "lopdf", "img-parts", "kamadak-exif", "id3", "metaflac", "mp4ameta", "quick-xml", "zip"] },
  { layer: "Design",    items: ["Deep Void dark theme", "Glassmorphism bento cards", "Inter + JetBrains Mono", "Violet → Cyan accent gradient"] },
];

const LINKS = [
  {
    icon: <Globe size={13} />,
    label: "Landing Page",
    href: "https://zsync.eu/zmetadata-stripper/",
    sub: "zsync.eu/zmetadata-stripper",
  },
  {
    icon: <GitFork size={13} />,
    label: "GitHub — ZMetadata Stripper",
    href: "https://github.com/TheHolyOneZ/ZMetadata-stripper",
    sub: "github.com/TheHolyOneZ/ZMetadata-stripper",
  },
  {
    icon: <GitFork size={13} />,
    label: "More Projects",
    href: "https://github.com/TheHolyOneZ",
    sub: "github.com/TheHolyOneZ",
  },
  {
    icon: <Globe size={13} />,
    label: "All Projects",
    href: "https://zsync.eu",
    sub: "zsync.eu",
  },
];

export function About({ open, onClose }: Props) {
  const go = (url: string) => shellOpen(url).catch(() => {});

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 z-50 overflow-y-auto p-4"
          style={{ background: "rgba(4,4,10,0.82)", backdropFilter: "blur(10px)" }}
          onClick={onClose}
        >
          <div className="flex min-h-full items-center justify-center">
          <motion.div
            initial={{ opacity: 0, scale: 0.94, y: 24 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.94, y: 16 }}
            transition={{ type: "spring", stiffness: 340, damping: 30 }}
            onClick={(e) => e.stopPropagation()}
            className="w-full max-w-xl relative overflow-hidden my-4"
            style={{
              background: "rgba(10,10,18,0.97)",
              border: "1px solid rgba(109,40,217,0.25)",
              borderRadius: 20,
              boxShadow: "0 0 60px rgba(124,58,237,0.12), 0 24px 80px rgba(0,0,0,0.7)",
            }}
          >
            {}
            <div className="absolute top-0 inset-x-0 h-32 pointer-events-none"
              style={{ background: "radial-gradient(ellipse at 50% -10%, rgba(124,58,237,0.18) 0%, transparent 70%)" }} />

            {}
            <button
              onClick={onClose}
              className="absolute top-4 right-4 z-10 text-slate-600 hover:text-slate-300 transition-colors rounded-lg p-1 hover:bg-white/5"
            >
              <X size={15} />
            </button>

            <div className="relative p-7 flex flex-col gap-6">

              {}
              <div className="flex items-start gap-4">
                <div className="w-14 h-14 rounded-2xl flex items-center justify-center shrink-0 overflow-hidden"
                  style={{ background: "linear-gradient(135deg,#7c3aed,#06b6d4)", boxShadow: "0 0 28px rgba(124,58,237,0.4)" }}>
                  <img src="/icon.png" alt="ZMetadata Stripper" className="w-10 h-10 object-contain" />
                </div>
                <div>
                  <h1 className="text-xl font-black tracking-tight gradient-text leading-tight">
                    ZMetadata Stripper
                  </h1>
                  <p className="text-[11px] text-slate-400 mt-1 leading-relaxed max-w-xs">
                    A privacy-first desktop tool that scrubs every trace of identifying
                    metadata — GPS, author, device, software, timestamps — from 25+ file
                    formats before you share them.
                  </p>
                  <div className="flex items-center gap-2 mt-2.5">
                    <Shield size={10} className="text-violet-400" />
                    <span className="text-[10px] text-slate-500">v0.1.0</span>
                    <span className="text-slate-700">·</span>
                    <span className="text-[10px] text-slate-500">GPL-3.0</span>
                    <span className="text-slate-700">·</span>
                    <span className="text-[10px] text-slate-500">Linux / Windows</span>
                  </div>
                </div>
              </div>

              {}
              <div>
                <SectionLabel icon={<Code2 size={10} />} label="Made by" />
                <div className="mt-2 flex items-center gap-3 px-3 py-2.5 rounded-xl"
                  style={{ background: "rgba(124,58,237,0.07)", border: "1px solid rgba(124,58,237,0.18)" }}>
                  <div className="w-7 h-7 rounded-lg flex items-center justify-center shrink-0"
                    style={{ background: "linear-gradient(135deg,#7c3aed,#06b6d4)" }}>
                    <span className="text-[10px] font-black text-white">Z</span>
                  </div>
                  <div>
                    <p className="text-[12px] font-bold text-slate-200">TheHolyOneZ</p>
                    <p className="text-[9px] text-slate-500">github.com/TheHolyOneZ &nbsp;·&nbsp; zsync.eu</p>
                  </div>
                </div>
              </div>

              {}
              <div>
                <SectionLabel icon={<Layers size={10} />} label="Tech Stack" />
                <div className="mt-2 flex flex-col gap-1.5">
                  {STACK.map(({ layer, items }) => (
                    <div key={layer} className="flex items-start gap-3 px-3 py-2 rounded-xl"
                      style={{ background: "rgba(255,255,255,0.03)" }}>
                      <span className="text-[9px] font-bold text-slate-600 uppercase tracking-wider w-16 shrink-0 mt-0.5">
                        {layer}
                      </span>
                      <div className="flex flex-wrap gap-1">
                        {items.map((t) => (
                          <span key={t} className="text-[9px] font-mono text-slate-400 px-1.5 py-0.5 rounded-md"
                            style={{ background: "rgba(124,58,237,0.1)", border: "1px solid rgba(124,58,237,0.15)" }}>
                            {t}
                          </span>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {}
              <div>
                <SectionLabel icon={<Globe size={10} />} label="Project Map" />
                <div className="mt-2 flex flex-col gap-1.5">
                  {LINKS.map(({ icon, label, href, sub }) => (
                    <button
                      key={href}
                      onClick={() => go(href)}
                      className="flex items-center gap-3 px-3 py-2 rounded-xl text-left transition-all hover:bg-white/[0.05] group"
                      style={{ border: "1px solid rgba(255,255,255,0.05)" }}
                    >
                      <span className="text-slate-600 group-hover:text-violet-400 transition-colors shrink-0">{icon}</span>
                      <div className="flex-1 min-w-0">
                        <p className="text-[11px] font-semibold text-slate-300 group-hover:text-slate-100 transition-colors">{label}</p>
                        <p className="text-[9px] font-mono text-slate-600 truncate">{sub}</p>
                      </div>
                      <ExternalLink size={10} className="text-slate-700 group-hover:text-violet-400 transition-colors shrink-0" />
                    </button>
                  ))}
                </div>
              </div>

              {}
              <div className="pt-1 border-t border-white/[0.05] flex items-center justify-between">
                <p className="text-[9px] text-slate-700">
                  © 2024 TheHolyOneZ — Released under GPL-3.0
                </p>
                <p className="text-[9px] text-slate-700 font-mono">
                  Built with Tauri + Rust + React
                </p>
              </div>

            </div>
          </motion.div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

function SectionLabel({ icon, label }: { icon: React.ReactNode; label: string }) {
  return (
    <div className="flex items-center gap-1.5">
      <span className="text-slate-600">{icon}</span>
      <span className="text-[10px] font-bold uppercase tracking-widest text-slate-600">{label}</span>
    </div>
  );
}
