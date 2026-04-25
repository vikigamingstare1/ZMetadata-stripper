import { create } from "zustand";
import type { Preset } from "../types";

const BUILT_IN: Preset[] = [
  { id: "max_privacy",    label: "Max Privacy",    description: "Strip everything — full forensic-safe clean." },
  { id: "keep_quality",   label: "Keep Quality",   description: "Strip GPS & author, keep ICC & exposure data." },
  { id: "social_media",   label: "Social Media",   description: "Strip GPS & author, inject neutral metadata option." },
  { id: "documents_only", label: "Documents Only", description: "Focused on PDF / Office metadata scrub." },
];

interface PresetStore {
  presets: Preset[];
  addCustomPreset: (p: Preset) => void;
  removeCustomPreset: (id: string) => void;
}

export const usePresetStore = create<PresetStore>((set) => ({
  presets: BUILT_IN,
  addCustomPreset: (p) => set((s) => ({ presets: [...s.presets, p] })),
  removeCustomPreset: (id) =>
    set((s) => ({ presets: s.presets.filter((p) => p.id !== id) })),
}));
