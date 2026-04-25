import type { MetadataField, RiskLevel } from "../types";

const RISK_POINTS: Record<RiskLevel, number> = {
  Critical: 40,
  High: 20,
  Medium: 10,
  Low: 2,
};

export function computeScore(fields: MetadataField[]): number {
  let score = fields.reduce((acc, f) => acc + RISK_POINTS[f.risk], 0);
  score = Math.min(score, 100);
  if (fields.some((f) => f.category === "Gps")) score = Math.max(score, 80);
  return score;
}

export function scoreLabel(score: number): string {
  if (score >= 80) return "Critical";
  if (score >= 60) return "High";
  if (score >= 30) return "Medium";
  return "Low";
}

export function scoreColor(score: number): string {
  if (score >= 80) return "text-red-400";
  if (score >= 60) return "text-orange-400";
  if (score >= 30) return "text-yellow-400";
  return "text-emerald-400";
}

export function scoreBg(score: number): string {
  if (score >= 80) return "bg-red-500/20 border-red-500/40";
  if (score >= 60) return "bg-orange-500/20 border-orange-500/40";
  if (score >= 30) return "bg-yellow-500/20 border-yellow-500/40";
  return "bg-emerald-500/20 border-emerald-500/40";
}
