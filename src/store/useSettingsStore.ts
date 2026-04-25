import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { AppSettings, OutputMode } from "../types";

interface SettingsStore extends AppSettings {
  setTheme: (t: AppSettings["theme"]) => void;
  setActivePreset: (id: string) => void;
  setOutputMode: (m: OutputMode) => void;
  setOutputDir: (dir: string | null) => void;
  setConcurrency: (n: number) => void;
  setAutoBackup: (v: boolean) => void;
  setMinimizeToTray: (v: boolean) => void;
}

export const useSettingsStore = create<SettingsStore>()(
  persist(
    (set) => ({
      theme: "dark",
      activePresetId: "max_privacy",
      outputMode: "Overwrite",
      outputDir: null,
      concurrency: 4,
      autoBackup: false,
      renamOnOutput: false,
      minimizeToTray: false,

      setTheme: (theme) => set({ theme }),
      setActivePreset: (activePresetId) => set({ activePresetId }),
      setOutputMode: (outputMode) => set({ outputMode }),
      setOutputDir: (outputDir) => set({ outputDir }),
      setConcurrency: (concurrency) => set({ concurrency }),
      setAutoBackup: (autoBackup) => set({ autoBackup }),
      setMinimizeToTray: (minimizeToTray) => set({ minimizeToTray }),
    }),
    { name: "zms-settings" }
  )
);
