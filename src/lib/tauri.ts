import { invoke } from "@tauri-apps/api/core";
import type { MetadataField, StripOptions } from "../types";

export interface InspectResult {
  fields: MetadataField[];
  sensitivity_score: number;
  thumbnail_b64: string | null;
  format: string;
}

export interface StripResult {
  success: boolean;
  output_path: string;
  fields_stripped: MetadataField[];
  fields_kept: MetadataField[];
  fields_injected: MetadataField[];
  bytes_saved: number;
  error: string | null;
}

export const inspectFile = (path: string): Promise<InspectResult> =>
  invoke("inspect_file", { path });

export const stripFile = (path: string, options: StripOptions): Promise<StripResult> =>
  invoke("strip_file", { path, options });

export const getPresets = (): Promise<{ id: string; label: string; description: string }[]> =>
  invoke("get_presets");

export const getStripOptions = (presetId: string): Promise<StripOptions> =>
  invoke("get_strip_options", { presetId });
