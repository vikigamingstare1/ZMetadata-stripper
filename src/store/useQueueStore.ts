import { create } from "zustand";
import type { QueueFile } from "../types";

interface QueueStore {
  files: QueueFile[];
  activeFileId: string | null;
  isProcessing: boolean;
  addFiles: (files: QueueFile[]) => void;
  removeFile: (id: string) => void;
  updateFile: (id: string, updates: Partial<QueueFile>) => void;
  setActiveFile: (id: string | null) => void;
  clearQueue: () => void;
  reorderFiles: (fromIndex: number, toIndex: number) => void;
  setProcessing: (v: boolean) => void;
}

export const useQueueStore = create<QueueStore>((set) => ({
  files: [],
  activeFileId: null,
  isProcessing: false,

  addFiles: (incoming) =>
    set((s) => ({
      files: [
        ...s.files,
        ...incoming.filter((f) => !s.files.find((e) => e.path === f.path)),
      ],
    })),

  removeFile: (id) =>
    set((s) => ({
      files: s.files.filter((f) => f.id !== id),
      activeFileId: s.activeFileId === id ? null : s.activeFileId,
    })),

  updateFile: (id, updates) =>
    set((s) => ({
      files: s.files.map((f) => (f.id === id ? { ...f, ...updates } : f)),
    })),

  setActiveFile: (id) => set({ activeFileId: id }),

  clearQueue: () => set({ files: [], activeFileId: null }),

  reorderFiles: (fromIndex, toIndex) =>
    set((s) => {
      const files = [...s.files];
      const [moved] = files.splice(fromIndex, 1);
      files.splice(toIndex, 0, moved);
      return { files };
    }),

  setProcessing: (v) => set({ isProcessing: v }),
}));
