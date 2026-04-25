use anyhow::{Context, Result};
use id3::TagLike;
use std::path::Path;

use crate::types::{MetadataCategory, MetadataField, RiskLevel, StripOptions};


pub fn inspect_mp3(path: &Path) -> Result<Vec<MetadataField>> {
    let tag = id3::Tag::read_from_path(path).context("Failed to read ID3 tag")?;
    let mut fields = Vec::new();

    macro_rules! push {
        ($label:expr, $val:expr, $cat:expr, $risk:expr) => {
            if let Some(v) = $val {
                fields.push(MetadataField {
                    name: $label.into(),
                    value: v.to_string(),
                    category: $cat,
                    risk: $risk,
                });
            }
        };
    }

    push!("Artist",       tag.artist(),       MetadataCategory::Author,     RiskLevel::High);
    push!("Title",        tag.title(),        MetadataCategory::Other,      RiskLevel::Low);
    push!("Album",        tag.album(),        MetadataCategory::Other,      RiskLevel::Low);
    push!("Album Artist", tag.album_artist(), MetadataCategory::Author,     RiskLevel::High);
    push!("Genre",        tag.genre(),        MetadataCategory::Other,      RiskLevel::Low);
    push!("Composer",     tag.text_for_frame_id("TCOM"), MetadataCategory::Author, RiskLevel::High);
    push!("Copyright",    tag.text_for_frame_id("TCOP"), MetadataCategory::Author, RiskLevel::High);
    push!("Encoded by",   tag.text_for_frame_id("TENC"), MetadataCategory::Software, RiskLevel::Medium);
    push!("Publisher",    tag.text_for_frame_id("TPUB"), MetadataCategory::Author, RiskLevel::High);

    if let Some(year) = tag.year() {
        fields.push(MetadataField {
            name: "Year".into(),
            value: year.to_string(),
            category: MetadataCategory::Timestamps,
            risk: RiskLevel::Low,
        });
    }

    for pic in tag.pictures() {
        fields.push(MetadataField {
            name: format!("Embedded image ({})", pic.mime_type),
            value: format!("{} bytes", pic.data.len()),
            category: MetadataCategory::Other,
            risk: RiskLevel::Low,
        });
    }

    Ok(fields)
}

pub fn strip_mp3(path: &Path, _opts: &StripOptions) -> Result<()> {
    let empty = id3::Tag::new();
    empty
        .write_to_path(path, id3::Version::Id3v24)
        .context("Failed to write stripped ID3 tag")?;
    Ok(())
}


pub fn inspect_flac(path: &Path) -> Result<Vec<MetadataField>> {
    let tag = metaflac::Tag::read_from_path(path).context("Failed to read FLAC tag")?;
    let mut fields = Vec::new();

    if let Some(vc) = tag.vorbis_comments() {
        for (key, values) in &vc.comments {
            for val in values {
                let (cat, risk) = classify_audio_field(key);
                fields.push(MetadataField {
                    name: key.clone(),
                    value: val.clone(),
                    category: cat,
                    risk,
                });
            }
        }
    }


    let pics: Vec<_> = tag.pictures().collect();
    if !pics.is_empty() {
        fields.push(MetadataField {
            name: format!("Embedded image(s)"),
            value: format!("{} picture(s)", pics.len()),
            category: MetadataCategory::Other,
            risk: RiskLevel::Low,
        });
    }

    Ok(fields)
}

pub fn strip_flac(path: &Path, _opts: &StripOptions) -> Result<()> {
    let mut tag = metaflac::Tag::read_from_path(path).context("Failed to read FLAC tag")?;
    tag.remove_blocks(metaflac::BlockType::VorbisComment);
    tag.remove_blocks(metaflac::BlockType::Picture);
    tag.save().context("Failed to save stripped FLAC tag")?;
    Ok(())
}


pub fn inspect_ogg(data: &[u8]) -> Result<Vec<MetadataField>> {
    let mut fields = Vec::new();

    if let Some(comments) = parse_vorbis_comments(data) {
        for (key, val) in comments {
            let (cat, risk) = classify_audio_field(&key);
            fields.push(MetadataField { name: key, value: val, category: cat, risk });
        }
    }
    Ok(fields)
}

pub fn strip_ogg(data: Vec<u8>) -> Result<Vec<u8>> {


    let comment_marker_v = b"\x03vorbis";
    let comment_marker_o = b"OpusTags";

    let mut out = data.clone();
    let mut found = false;


    for window_start in 0..out.len().saturating_sub(7) {
        if out[window_start..].starts_with(comment_marker_v) {
            overwrite_vorbis_comment(&mut out, window_start, 7);
            found = true;
            break;
        }
        if out[window_start..].starts_with(comment_marker_o) {
            overwrite_opus_tags(&mut out, window_start, 8);
            found = true;
            break;
        }
    }

    if !found {
        return Ok(data);
    }

    Ok(out)
}

fn overwrite_vorbis_comment(data: &mut Vec<u8>, start: usize, prefix_len: usize) {
    let pos = start + prefix_len;
    if pos + 4 > data.len() { return; }

    let vendor = b"ZScrub";
    let vendor_len = vendor.len() as u32;
    if pos + 4 > data.len() { return; }
    data[pos..pos + 4].copy_from_slice(&vendor_len.to_le_bytes());
    if pos + 4 + vendor.len() + 4 > data.len() { return; }
    data[pos + 4..pos + 4 + vendor.len()].copy_from_slice(vendor);

    let count_pos = pos + 4 + vendor.len();
    if count_pos + 4 <= data.len() {
        data[count_pos..count_pos + 4].fill(0);
    }
}

fn overwrite_opus_tags(data: &mut Vec<u8>, start: usize, prefix_len: usize) {
    overwrite_vorbis_comment(data, start, prefix_len);
}


pub fn inspect_aiff(data: &[u8]) -> Result<Vec<MetadataField>> {
    let mut fields = Vec::new();
    if data.len() < 12 { return Ok(fields); }

    let mut pos = 12usize;
    while pos + 8 <= data.len() {
        let id = &data[pos..pos + 4];
        let size = u32::from_be_bytes(data[pos + 4..pos + 8].try_into().unwrap()) as usize;

        match id {
            b"NAME" | b"AUTH" | b"(c) " | b"ANNO" => {
                let val = String::from_utf8_lossy(&data[pos + 8..(pos + 8 + size).min(data.len())]).into_owned();
                let name = match id {
                    b"NAME" => "Name",
                    b"AUTH" => "Author",
                    b"(c) " => "Copyright",
                    b"ANNO" => "Annotation",
                    _ => "Unknown",
                };
                fields.push(MetadataField {
                    name: name.into(),
                    value: val,
                    category: if id == b"AUTH" || id == b"(c) " { MetadataCategory::Author } else { MetadataCategory::Other },
                    risk: if id == b"AUTH" || id == b"(c) " { RiskLevel::High } else { RiskLevel::Low },
                });
            }
            b"ID3 " => {
                fields.push(MetadataField {
                    name: "ID3 tag".into(),
                    value: format!("{} bytes", size),
                    category: MetadataCategory::Other,
                    risk: RiskLevel::Medium,
                });
            }
            _ => {}
        }

        let padded = size + (size & 1);
        pos += 8 + padded;
    }
    Ok(fields)
}

pub fn strip_aiff(data: Vec<u8>) -> Result<Vec<u8>> {
    if data.len() < 12 { return Ok(data); }

    let mut out = data[..12].to_vec();
    let mut pos = 12usize;

    while pos + 8 <= data.len() {
        let id = &data[pos..pos + 4];
        let size = u32::from_be_bytes(data[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let padded = size + (size & 1);
        let end = (pos + 8 + padded).min(data.len());

        let skip = matches!(id, b"NAME" | b"AUTH" | b"(c) " | b"ANNO" | b"ID3 " | b"ID3!");
        if !skip {
            out.extend_from_slice(&data[pos..end]);
        }
        pos = end;
    }


    let new_size = (out.len() - 8) as u32;
    out[4..8].copy_from_slice(&new_size.to_be_bytes());
    Ok(out)
}


fn parse_vorbis_comments(data: &[u8]) -> Option<Vec<(String, String)>> {
    let marker = b"\x03vorbis";
    let start = data.windows(7).position(|w| w == marker)? + 7;
    let pos = start;
    if pos + 4 > data.len() { return None; }

    let vendor_len = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?) as usize;
    let pos = pos + 4 + vendor_len;
    if pos + 4 > data.len() { return None; }

    let count = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?) as usize;
    let mut pos = pos + 4;
    let mut results = Vec::new();

    for _ in 0..count {
        if pos + 4 > data.len() { break; }
        let len = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?) as usize;
        pos += 4;
        if pos + len > data.len() { break; }
        let entry = String::from_utf8_lossy(&data[pos..pos + len]).to_string();
        pos += len;
        if let Some(eq) = entry.find('=') {
            results.push((entry[..eq].to_string(), entry[eq + 1..].to_string()));
        }
    }
    Some(results)
}

fn classify_audio_field(name: &str) -> (MetadataCategory, RiskLevel) {
    match name.to_uppercase().as_str() {
        "ARTIST" | "ALBUMARTIST" | "ALBUM_ARTIST" | "COMPOSER" | "LYRICIST"
        | "CONTACT" | "COPYRIGHT" | "LICENSE" => (MetadataCategory::Author, RiskLevel::High),
        "DATE" | "YEAR" | "ORIGINALDATE" => (MetadataCategory::Timestamps, RiskLevel::Low),
        "ENCODER" | "ENCODEDBY" | "TOOL" | "TRACKTOTAL" | "DISCNUMBER" => {
            (MetadataCategory::Software, RiskLevel::Medium)
        }
        _ => (MetadataCategory::Other, RiskLevel::Low),
    }
}
