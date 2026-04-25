use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataField {
    pub name: String,
    pub value: String,
    pub category: MetadataCategory,
    pub risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetadataCategory {
    Gps,
    Author,
    Camera,
    Timestamps,
    Software,
    Iptc,
    Xmp,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripOptions {
    pub preset: String,
    pub strip_gps: bool,
    pub strip_author: bool,
    pub strip_camera: bool,
    pub strip_timestamps: bool,
    pub strip_icc: bool,
    pub strip_iptc: bool,
    pub strip_xmp: bool,
    pub strip_makernotes: bool,
    pub strip_thumbnail: bool,
    pub inject_neutral: bool,
    pub pdf_full_rewrite: bool,
    pub output_mode: OutputMode,
    pub output_dir: Option<String>,
    pub rename_file: bool,
}

impl Default for StripOptions {
    fn default() -> Self {
        Self {
            preset: "max_privacy".into(),
            strip_gps: true,
            strip_author: true,
            strip_camera: true,
            strip_timestamps: true,
            strip_icc: true,
            strip_iptc: true,
            strip_xmp: true,
            strip_makernotes: true,
            strip_thumbnail: true,
            inject_neutral: false,
            pdf_full_rewrite: true,
            output_mode: OutputMode::Overwrite,
            output_dir: None,
            rename_file: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputMode {
    Overwrite,
    SaveCopy,
    CustomDir,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripResult {
    pub success: bool,
    pub output_path: String,
    pub fields_stripped: Vec<MetadataField>,
    pub fields_kept: Vec<MetadataField>,
    pub fields_injected: Vec<MetadataField>,
    pub bytes_saved: i64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectResult {
    pub fields: Vec<MetadataField>,
    pub sensitivity_score: u32,
    pub thumbnail_b64: Option<String>,
    pub format: String,
}
