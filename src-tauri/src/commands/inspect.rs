use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::formats::{detect_format, FileFormat};
use crate::formats::{audio, document, image, video};
use crate::types::{InspectResult, MetadataCategory, RiskLevel};

#[tauri::command]
pub async fn inspect_file(path: String) -> Result<InspectResult, String> {
    let p = Path::new(&path);
    let data = fs::read(p).map_err(|e| e.to_string())?;
    let fmt = detect_format(p, &data);

    let fields = match &fmt {
        FileFormat::Jpeg => image::inspect_jpeg(&data).map_err(|e| e.to_string())?,
        FileFormat::Png  => image::inspect_png(&data).map_err(|e| e.to_string())?,
        FileFormat::Webp => image::inspect_webp(&data).map_err(|e| e.to_string())?,
        FileFormat::Tiff => image::inspect_tiff(&data).map_err(|e| e.to_string())?,
        FileFormat::Gif  => image::inspect_gif(&data).map_err(|e| e.to_string())?,
        FileFormat::Bmp  => vec![],
        FileFormat::Pdf  => document::inspect_pdf(&data).map_err(|e| e.to_string())?,
        FileFormat::Docx | FileFormat::Xlsx | FileFormat::Pptx
        | FileFormat::Odt | FileFormat::Ods | FileFormat::Odp
                         => document::inspect_office(&data).map_err(|e| e.to_string())?,
        FileFormat::Rtf  => document::inspect_rtf(&data).map_err(|e| e.to_string())?,
        FileFormat::Svg  => document::inspect_svg(&data).map_err(|e| e.to_string())?,
        FileFormat::Mp3  => audio::inspect_mp3(p).map_err(|e| e.to_string())?,
        FileFormat::Flac => audio::inspect_flac(p).map_err(|e| e.to_string())?,
        FileFormat::Ogg  => audio::inspect_ogg(&data).map_err(|e| e.to_string())?,
        FileFormat::Wav  => video::inspect_wav(&data).map_err(|e| e.to_string())?,
        FileFormat::M4a  => video::inspect_mp4(p).map_err(|e| e.to_string())?,
        FileFormat::Aiff => audio::inspect_aiff(&data).map_err(|e| e.to_string())?,
        FileFormat::Mp4 | FileFormat::Mov
                         => video::inspect_mp4(p).map_err(|e| e.to_string())?,
        FileFormat::Mkv | FileFormat::Webm
                         => video::inspect_mkv(&data).map_err(|e| e.to_string())?,
        _ => vec![],
    };

    let sensitivity_score = compute_score(&fields);
    let thumbnail_b64 = match &fmt {
        FileFormat::Jpeg => image::thumbnail_b64(&data),
        _ => None,
    };

    Ok(InspectResult {
        fields,
        sensitivity_score,
        thumbnail_b64,
        format: fmt.label().to_string(),
    })
}

fn compute_score(fields: &[crate::types::MetadataField]) -> u32 {
    let mut score: u32 = 0;
    for f in fields {
        let pts = match f.risk {
            RiskLevel::Critical => 40,
            RiskLevel::High     => 20,
            RiskLevel::Medium   => 10,
            RiskLevel::Low      => 2,
        };
        score = (score + pts).min(100);
    }
    if fields.iter().any(|f| f.category == MetadataCategory::Gps) {
        score = score.max(80);
    }
    score
}
