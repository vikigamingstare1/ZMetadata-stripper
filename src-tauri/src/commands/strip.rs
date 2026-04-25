use anyhow::Result;
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::formats::{audio, detect_format, document, image, video, FileFormat};
use crate::types::{MetadataField, OutputMode, StripOptions, StripResult};

#[tauri::command]
pub async fn strip_file(path: String, options: StripOptions) -> Result<StripResult, String> {
    let p = Path::new(&path);
    let original_data = fs::read(p).map_err(|e| e.to_string())?;
    let original_size = original_data.len() as i64;
    let fmt = detect_format(p, &original_data);

    let fields_before = inspect_before(&fmt, p, &original_data);

    let stripped: Vec<u8> = match &fmt {
        FileFormat::Jpeg => image::strip_jpeg(original_data, &options).map_err(|e| e.to_string())?,
        FileFormat::Png  => image::strip_png(original_data, &options).map_err(|e| e.to_string())?,
        FileFormat::Webp => image::strip_webp(original_data, &options).map_err(|e| e.to_string())?,
        FileFormat::Tiff => image::strip_tiff(original_data).map_err(|e| e.to_string())?,
        FileFormat::Gif  => image::strip_gif(original_data).map_err(|e| e.to_string())?,
        FileFormat::Bmp  => image::strip_bmp(original_data).map_err(|e| e.to_string())?,

        FileFormat::Pdf  => document::strip_pdf(original_data, &options).map_err(|e| e.to_string())?,
        FileFormat::Docx | FileFormat::Xlsx | FileFormat::Pptx
        | FileFormat::Odt | FileFormat::Ods | FileFormat::Odp
                         => document::strip_office(original_data).map_err(|e| e.to_string())?,
        FileFormat::Rtf  => document::strip_rtf(original_data).map_err(|e| e.to_string())?,
        FileFormat::Svg  => document::strip_svg(original_data).map_err(|e| e.to_string())?,

        FileFormat::Mp3  => {
            let tmp = write_temp(p, &original_data).map_err(|e| e.to_string())?;
            audio::strip_mp3(&tmp, &options).map_err(|e| e.to_string())?;
            let out = fs::read(&tmp).map_err(|e| e.to_string())?;
            let _ = fs::remove_file(&tmp);
            out
        }
        FileFormat::Flac => {
            let tmp = write_temp(p, &original_data).map_err(|e| e.to_string())?;
            audio::strip_flac(&tmp, &options).map_err(|e| e.to_string())?;
            let out = fs::read(&tmp).map_err(|e| e.to_string())?;
            let _ = fs::remove_file(&tmp);
            out
        }
        FileFormat::Ogg  => audio::strip_ogg(original_data).map_err(|e| e.to_string())?,
        FileFormat::Wav  => video::strip_wav(original_data).map_err(|e| e.to_string())?,
        FileFormat::Aiff => audio::strip_aiff(original_data).map_err(|e| e.to_string())?,
        FileFormat::M4a | FileFormat::Mp4 | FileFormat::Mov
                         => video::strip_mp4(original_data).map_err(|e| e.to_string())?,
        FileFormat::Mkv | FileFormat::Webm
                         => video::strip_mkv(original_data).map_err(|e| e.to_string())?,

        _ => {
            return Ok(StripResult {
                success: false,
                output_path: path.clone(),
                fields_stripped: vec![],
                fields_kept: vec![],
                fields_injected: vec![],
                bytes_saved: 0,
                error: Some(format!("Unsupported format: {}", fmt.label())),
            });
        }
    };

    let new_size = stripped.len() as i64;
    let bytes_saved = original_size - new_size;
    let output_path = resolve_output_path(p, &options).map_err(|e| e.to_string())?;

    fs::write(&output_path, &stripped).map_err(|e| e.to_string())?;

    let fields_injected = if options.inject_neutral {
        vec![
            mk_field("Software", "ZScrub 1.0"),
            mk_field("ColorSpace", "sRGB"),
            mk_field("Orientation", "1"),
        ]
    } else {
        vec![]
    };

    Ok(StripResult {
        success: true,
        output_path: output_path.to_string_lossy().to_string(),
        fields_stripped: fields_before,
        fields_kept: vec![],
        fields_injected,
        bytes_saved,
        error: None,
    })
}

fn inspect_before(fmt: &FileFormat, p: &Path, data: &[u8]) -> Vec<MetadataField> {
    match fmt {
        FileFormat::Jpeg => image::inspect_jpeg(data).unwrap_or_default(),
        FileFormat::Png  => image::inspect_png(data).unwrap_or_default(),
        FileFormat::Webp => image::inspect_webp(data).unwrap_or_default(),
        FileFormat::Tiff => image::inspect_tiff(data).unwrap_or_default(),
        FileFormat::Gif  => image::inspect_gif(data).unwrap_or_default(),
        FileFormat::Pdf  => document::inspect_pdf(data).unwrap_or_default(),
        FileFormat::Docx | FileFormat::Xlsx | FileFormat::Pptx
        | FileFormat::Odt | FileFormat::Ods | FileFormat::Odp
                         => document::inspect_office(data).unwrap_or_default(),
        FileFormat::Rtf  => document::inspect_rtf(data).unwrap_or_default(),
        FileFormat::Svg  => document::inspect_svg(data).unwrap_or_default(),
        FileFormat::Mp3  => audio::inspect_mp3(p).unwrap_or_default(),
        FileFormat::Flac => audio::inspect_flac(p).unwrap_or_default(),
        FileFormat::Ogg  => audio::inspect_ogg(data).unwrap_or_default(),
        FileFormat::Wav  => video::inspect_wav(data).unwrap_or_default(),
        FileFormat::Aiff => audio::inspect_aiff(data).unwrap_or_default(),
        FileFormat::M4a | FileFormat::Mp4 | FileFormat::Mov
                         => video::inspect_mp4(p).unwrap_or_default(),
        FileFormat::Mkv | FileFormat::Webm
                         => video::inspect_mkv(data).unwrap_or_default(),
        _ => vec![],
    }
}

fn resolve_output_path(src: &Path, opts: &StripOptions) -> Result<PathBuf> {
    match opts.output_mode {
        OutputMode::Overwrite => Ok(src.to_path_buf()),
        OutputMode::SaveCopy => {
            let stem = src.file_stem().unwrap_or_default().to_string_lossy();
            let ext = src.extension().unwrap_or_default().to_string_lossy();
            let dir = src.parent().unwrap_or(Path::new("."));
            Ok(dir.join(format!("{}_clean.{}", stem, ext)))
        }
        OutputMode::CustomDir => {
            let dir = opts.output_dir.as_deref().unwrap_or(".");
            let filename = src.file_name().unwrap_or_default();
            Ok(PathBuf::from(dir).join(filename))
        }
    }
}

fn write_temp(src: &Path, data: &[u8]) -> io::Result<PathBuf> {
    let tmp = std::env::temp_dir().join(format!(
        "zms_{}.{}",
        uuid::Uuid::new_v4(),
        src.extension().unwrap_or_default().to_string_lossy()
    ));
    fs::write(&tmp, data)?;
    Ok(tmp)
}

fn mk_field(name: &str, value: &str) -> MetadataField {
    MetadataField {
        name: name.into(),
        value: value.into(),
        category: crate::types::MetadataCategory::Software,
        risk: crate::types::RiskLevel::Low,
    }
}
