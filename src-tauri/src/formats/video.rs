use anyhow::{Context, Result};
use std::path::Path;

use crate::types::{MetadataCategory, MetadataField, RiskLevel};

// ── MP4 / MOV / M4A ───────────────────────────────────────────────────────────

pub fn inspect_mp4(path: &Path) -> Result<Vec<MetadataField>> {
    let tag = mp4ameta::Tag::read_from_path(path).context("Failed to read MP4 tag")?;
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

    push!("Title",       tag.title(),        MetadataCategory::Other,      RiskLevel::Low);
    push!("Artist",      tag.artist(),       MetadataCategory::Author,     RiskLevel::High);
    push!("Composer",    tag.composer(),     MetadataCategory::Author,     RiskLevel::High);
    push!("Album",       tag.album(),        MetadataCategory::Other,      RiskLevel::Low);
    push!("Year",        tag.year(),         MetadataCategory::Timestamps, RiskLevel::Low);
    push!("Comment",     tag.comment(),      MetadataCategory::Other,      RiskLevel::Medium);
    push!("Copyright",   tag.copyright(),    MetadataCategory::Author,     RiskLevel::High);
    push!("Encoder",     tag.encoder(),      MetadataCategory::Software,   RiskLevel::Medium);
    push!("Album Artist",tag.album_artist(), MetadataCategory::Author,     RiskLevel::High);

    Ok(fields)
}

/// Strip MP4/MOV metadata by removing the `udta` atom from `moov` and
/// zeroing creation/modification timestamps in `mvhd` / `mdhd`.
pub fn strip_mp4(data: Vec<u8>) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(data.len());
    strip_atoms(&data, &mut out, false)?;
    Ok(out)
}

fn strip_atoms(src: &[u8], dst: &mut Vec<u8>, inside_moov: bool) -> Result<()> {
    let mut pos = 0;

    while pos < src.len() {
        if pos + 8 > src.len() {
            break;
        }

        let (hdr_len, atom_size) = read_atom_size(src, pos);
        if atom_size == 0 || pos + atom_size > src.len() {
            // Last atom or corrupt — copy remainder verbatim
            dst.extend_from_slice(&src[pos..]);
            break;
        }

        let kind = &src[pos + 4..pos + 8];
        let payload = &src[pos + hdr_len..pos + atom_size];

        match kind {
            // Remove the entire user-data subtree (iTunes/QuickTime metadata)
            b"udta" => {}

            // Recurse into container atoms
            b"moov" | b"trak" | b"mdia" | b"minf" | b"stbl" | b"edts" => {
                let mut inner = Vec::new();
                strip_atoms(payload, &mut inner, kind == b"moov" || inside_moov)?;
                push_atom(dst, kind, hdr_len, &inner);
            }

            // Zero creation & modification timestamps (MP4 epoch = Jan 1 1904)
            b"mvhd" | b"mdhd" => {
                let mut patched = payload.to_vec();
                patch_timestamps(&mut patched);
                push_atom(dst, kind, hdr_len, &patched);
            }

            // Copy everything else verbatim
            _ => {
                dst.extend_from_slice(&src[pos..pos + atom_size]);
            }
        }

        pos += atom_size;
    }
    Ok(())
}

/// Read atom size; returns (header_length, total_atom_size).
fn read_atom_size(src: &[u8], pos: usize) -> (usize, usize) {
    let raw = u32::from_be_bytes(src[pos..pos + 4].try_into().unwrap()) as usize;
    if raw == 1 {
        // Extended 64-bit size
        if pos + 16 > src.len() {
            return (8, src.len() - pos);
        }
        let ext = u64::from_be_bytes(src[pos + 8..pos + 16].try_into().unwrap()) as usize;
        (16, ext)
    } else if raw == 0 {
        (8, src.len() - pos)
    } else {
        (8, raw)
    }
}

fn push_atom(dst: &mut Vec<u8>, kind: &[u8], hdr_len: usize, payload: &[u8]) {
    let total = hdr_len + payload.len();
    if hdr_len == 16 {
        dst.extend_from_slice(&(1u32).to_be_bytes());
        dst.extend_from_slice(kind);
        dst.extend_from_slice(&(total as u64).to_be_bytes());
    } else {
        dst.extend_from_slice(&(total as u32).to_be_bytes());
        dst.extend_from_slice(kind);
    }
    dst.extend_from_slice(payload);
}

fn patch_timestamps(payload: &mut [u8]) {
    if payload.is_empty() { return; }
    let version = payload[0];
    if version == 0 && payload.len() >= 9 {
        // version 0: creation(4) + modification(4) at offset 1
        payload[1..5].fill(0);
        payload[5..9].fill(0);
    } else if version == 1 && payload.len() >= 17 {
        // version 1: creation(8) + modification(8) at offset 1
        payload[1..9].fill(0);
        payload[9..17].fill(0);
    }
}

// ── WAV ───────────────────────────────────────────────────────────────────────

pub fn inspect_wav(data: &[u8]) -> Result<Vec<MetadataField>> {
    let mut fields = Vec::new();
    if data.len() < 12 || &data[..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Ok(fields);
    }

    let mut pos = 12usize;
    while pos + 8 <= data.len() {
        let id = &data[pos..pos + 4];
        let size = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let padded = size + (size & 1);

        if id == b"LIST" && pos + 12 <= data.len() && &data[pos + 8..pos + 12] == b"INFO" {
            let mut info_pos = pos + 12;
            let list_end = pos + 8 + padded;
            while info_pos + 8 <= list_end.min(data.len()) {
                let tag = &data[info_pos..info_pos + 4];
                let tag_size = u32::from_le_bytes(data[info_pos + 4..info_pos + 8].try_into().unwrap()) as usize;
                let val_end = (info_pos + 8 + tag_size).min(data.len());
                let val = String::from_utf8_lossy(&data[info_pos + 8..val_end])
                    .trim_end_matches('\0')
                    .to_string();
                let name = String::from_utf8_lossy(tag).into_owned();
                let (cat, risk) = wav_field_classify(&name);
                if !val.is_empty() {
                    fields.push(MetadataField { name, value: val, category: cat, risk });
                }
                let padded_tag = tag_size + (tag_size & 1);
                info_pos += 8 + padded_tag;
            }
        }

        pos += 8 + padded;
    }
    Ok(fields)
}

pub fn strip_wav(data: Vec<u8>) -> Result<Vec<u8>> {
    if data.len() < 12 || &data[..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Ok(data);
    }

    let mut out = data[..12].to_vec();
    let mut pos = 12usize;

    while pos + 8 <= data.len() {
        let id = &data[pos..pos + 4];
        let size = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let padded = size + (size & 1);
        let end = (pos + 8 + padded).min(data.len());

        let skip = matches!(id, b"LIST" | b"id3 " | b"ID3 " | b"bext" | b"aXML" | b"iXML" | b"_PMX");

        if !skip {
            out.extend_from_slice(&data[pos..end]);
        }
        pos = end;
    }

    let new_size = (out.len() - 8) as u32;
    out[4..8].copy_from_slice(&new_size.to_le_bytes());
    Ok(out)
}

fn wav_field_classify(tag: &str) -> (MetadataCategory, RiskLevel) {
    match tag {
        "IART" | "IENG" | "ICOP" | "IMUS" | "ISTR" => (MetadataCategory::Author, RiskLevel::High),
        "ISFT" | "IARL" => (MetadataCategory::Software, RiskLevel::Medium),
        "ICRD" | "IDIT" => (MetadataCategory::Timestamps, RiskLevel::Low),
        _ => (MetadataCategory::Other, RiskLevel::Low),
    }
}

// ── MKV / WebM — best-effort ──────────────────────────────────────────────────

/// MKV/WebM: strip the Segment → Tags element by zeroing its EBML ID.
/// This is a best-effort approach without a full EBML parser.
pub fn strip_mkv(mut data: Vec<u8>) -> Result<Vec<u8>> {
    // EBML ID for Tags element: 0x1254C367 (big-endian)
    let tag_id: [u8; 4] = [0x12, 0x54, 0xC3, 0x67];
    // Replace it with a Void element ID 0xEC so the data is ignored by players
    let void_id: [u8; 4] = [0xEC, 0, 0, 0];

    let mut i = 0;
    while i + 4 <= data.len() {
        if &data[i..i + 4] == &tag_id {
            data[i..i + 4].copy_from_slice(&void_id);
        }
        i += 1;
    }
    Ok(data)
}

pub fn inspect_mkv(data: &[u8]) -> Result<Vec<MetadataField>> {
    let mut fields = Vec::new();
    // Scan for common UTF-8 strings in MKV metadata elements
    // Title EBML ID: 0x7BA9
    // DateUTC EBML ID: 0x4461
    let s = std::str::from_utf8(data).unwrap_or("");
    if let Some(idx) = data.windows(2).position(|w| w == [0x7B, 0xA9]) {
        if let Some(end) = data[idx + 3..].iter().position(|&b| b == 0) {
            let title = String::from_utf8_lossy(&data[idx + 3..idx + 3 + end]);
            if !title.is_empty() {
                fields.push(MetadataField {
                    name: "Title".into(),
                    value: title.into(),
                    category: MetadataCategory::Other,
                    risk: RiskLevel::Medium,
                });
            }
        }
    }
    // WritingApp / MuxingApp appear as plain text in the segment
    for label in ["MuxingApp", "WritingApp", "Encoder"] {
        if s.contains(label) {
            fields.push(MetadataField {
                name: label.into(),
                value: "present (use exiftool for full extraction)".into(),
                category: MetadataCategory::Software,
                risk: RiskLevel::Medium,
            });
        }
    }
    Ok(fields)
}
