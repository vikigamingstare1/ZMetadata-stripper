export type MetadataCategory =
  | "Gps"
  | "Author"
  | "Camera"
  | "Timestamps"
  | "Software"
  | "Iptc"
  | "Xmp"
  | "Other";

export type RiskLevel = "Critical" | "High" | "Medium" | "Low";

export interface MetadataField {
  name: string;
  value: string;
  category: MetadataCategory;
  risk: RiskLevel;
}

export type FileStatus =
  | "pending"
  | "inspecting"
  | "ready"
  | "processing"
  | "clean"
  | "failed"
  | "skipped";

export interface QueueFile {
  id: string;
  name: string;
  path: string;
  size: number;
  format: string;
  status: FileStatus;
  progress: number;
  sensitivityScore: number;
  metadataBefore: MetadataField[];
  metadataAfter: MetadataField[];
  fieldsInjected: MetadataField[];
  excludedCategories: MetadataCategory[];
  thumbnail?: string;
  outputPath?: string;
  bytesSaved?: number;
  error?: string;
}

export type OutputMode = "Overwrite" | "SaveCopy" | "CustomDir";

export interface StripOptions {
  preset: string;
  strip_gps: boolean;
  strip_author: boolean;
  strip_camera: boolean;
  strip_timestamps: boolean;
  strip_icc: boolean;
  strip_iptc: boolean;
  strip_xmp: boolean;
  strip_makernotes: boolean;
  strip_thumbnail: boolean;
  inject_neutral: boolean;
  pdf_full_rewrite: boolean;
  output_mode: OutputMode;
  output_dir: string | null;
  rename_file: boolean;
}

export interface Preset {
  id: string;
  label: string;
  description: string;
}

export type ToastType = "success" | "error" | "warning" | "info";

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
}

export interface AppSettings {
  theme: "dark" | "light" | "system";
  activePresetId: string;
  outputMode: OutputMode;
  outputDir: string | null;
  concurrency: number;
  autoBackup: boolean;
  renamOnOutput: boolean;
  minimizeToTray: boolean;
}
