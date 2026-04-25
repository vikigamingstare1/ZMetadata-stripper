use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use bytes::Bytes;
use img_parts::{jpeg::Jpeg, png::Png};
use std::io::Cursor;

use crate::types::{MetadataCategory, MetadataField, RiskLevel, StripOptions};


const APP1: u8 = 0xE1;
const APP2: u8 = 0xE2;
const APP13: u8 = 0xED;
const COM: u8 = 0xFE;

pub fn inspect_jpeg(data: &[u8]) -> Result<Vec<MetadataField>> {
    let mut fields = Vec::new();

    if let Ok(exif) = exif::Reader::new().read_from_container(&mut Cursor::new(data)) {
        for f in exif.fields() {
            let name = f.tag.to_string();
            let value = f.display_value().with_unit(&exif).to_string();
            let (cat, risk) = classify_field(&name, &value);
            fields.push(MetadataField { name, value, category: cat, risk });
        }
    }


    if let Ok(jpeg) = Jpeg::from_bytes(Bytes::from(data.to_vec())) {
        for seg in jpeg.segments() {
            if seg.marker() == APP1 {
                let c = seg.contents();
                if c.starts_with(b"http://ns.adobe.com/xap") {
                    let xmp = String::from_utf8_lossy(c);
                    for key in ["dc:creator", "xmp:CreatorTool", "xmp:CreateDate",
                                "photoshop:City", "photoshop:Country", "Iptc4xmpCore"] {
                        if xmp.contains(key) {
                            fields.push(MetadataField {
                                name: format!("XMP:{}", key),
                                value: extract_xmp_value(&xmp, key).unwrap_or_else(|| "present".into()),
                                category: xmp_field_category(key),
                                risk: xmp_field_risk(key),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(fields)
}

pub fn strip_jpeg(data: Vec<u8>, opts: &StripOptions) -> Result<Vec<u8>> {
    let b = Bytes::from(data);
    let mut jpeg = Jpeg::from_bytes(b).context("Failed to parse JPEG")?;

    jpeg.segments_mut().retain(|seg| match seg.marker() {
        APP1  => false,
        APP2  => !opts.strip_icc,
        APP13 => !opts.strip_iptc,
        COM   => !opts.strip_author,
        _     => true,
    });

    if opts.inject_neutral {
        inject_neutral_jpeg(&mut jpeg)?;
    }

    let mut out = Vec::new();
    jpeg.encoder().write_to(&mut out).context("Failed to encode JPEG")?;
    Ok(out)
}

fn inject_neutral_jpeg(jpeg: &mut Jpeg) -> Result<()> {
    let exif_header = b"Exif\x00\x00";
    let tiff_header = b"II\x2A\x00\x08\x00\x00\x00";
    let software_str = b"ZScrub 1.0\x00";
    let ifd_count: u16 = 3;
    let sw_offset: u32 = 8 + 2 + 3 * 12 + 4;

    let mut exif_data = exif_header.to_vec();
    exif_data.extend_from_slice(tiff_header);
    exif_data.extend_from_slice(&ifd_count.to_le_bytes());
    exif_data.extend_from_slice(&ifd_entry(0x0112, 3, 1, 1u32.to_le_bytes()));
    exif_data.extend_from_slice(&ifd_entry(0x0131, 2, software_str.len() as u32, sw_offset.to_le_bytes()));
    exif_data.extend_from_slice(&ifd_entry(0xA001, 3, 1, 1u32.to_le_bytes()));
    exif_data.extend_from_slice(&[0u8; 4]);
    exif_data.extend_from_slice(software_str);

    use img_parts::jpeg::JpegSegment;
    jpeg.segments_mut().insert(
        1,
        JpegSegment::new_with_contents(APP1, Bytes::from(exif_data)),
    );
    Ok(())
}

fn ifd_entry(tag: u16, typ: u16, count: u32, val: [u8; 4]) -> [u8; 12] {
    let mut e = [0u8; 12];
    e[..2].copy_from_slice(&tag.to_le_bytes());
    e[2..4].copy_from_slice(&typ.to_le_bytes());
    e[4..8].copy_from_slice(&count.to_le_bytes());
    e[8..].copy_from_slice(&val);
    e
}


pub fn inspect_png(data: &[u8]) -> Result<Vec<MetadataField>> {
    let mut fields = Vec::new();
    let png = Png::from_bytes(Bytes::from(data.to_vec())).context("Failed to parse PNG")?;

    for chunk in png.chunks() {
        match &chunk.kind() {
            b"tEXt" | b"zTXt" => {
                let raw = chunk.contents();
                if let Some(null) = raw.iter().position(|&b| b == 0) {
                    let key = String::from_utf8_lossy(&raw[..null]).to_string();
                    let val = String::from_utf8_lossy(&raw[null + 1..]).to_string();
                    let (cat, risk) = classify_field(&key, &val);
                    fields.push(MetadataField { name: key, value: val, category: cat, risk });
                }
            }
            b"iTXt" => {
                let raw = chunk.contents();
                if let Some(null) = raw.iter().position(|&b| b == 0) {
                    let key = String::from_utf8_lossy(&raw[..null]).to_string();
                    let val = String::from_utf8_lossy(&raw[null + 1..]).to_string();
                    let (cat, risk) = classify_field(&key, &val);
                    fields.push(MetadataField { name: key, value: val, category: cat, risk });
                }
            }
            b"eXIf" => {
                fields.push(MetadataField {
                    name: "EXIF block".into(),
                    value: format!("{} bytes", chunk.contents().len()),
                    category: MetadataCategory::Camera,
                    risk: RiskLevel::High,
                });
            }
            b"iCCP" => {
                fields.push(MetadataField {
                    name: "ICC Profile".into(),
                    value: "embedded".into(),
                    category: MetadataCategory::Software,
                    risk: RiskLevel::Low,
                });
            }
            _ => {}
        }
    }
    Ok(fields)
}

pub fn strip_png(data: Vec<u8>, opts: &StripOptions) -> Result<Vec<u8>> {
    let b = Bytes::from(data);
    let mut png = Png::from_bytes(b).context("Failed to parse PNG")?;

    png.chunks_mut().retain(|c| match &c.kind() {
        b"tEXt" | b"iTXt" | b"zTXt" => !(opts.strip_author || opts.strip_xmp),
        b"eXIf" => false,
        b"iCCP" => !opts.strip_icc,
        _ => true,
    });

    let mut out = Vec::new();
    png.encoder().write_to(&mut out).context("Failed to encode PNG")?;
    Ok(out)
}


pub fn inspect_webp(data: &[u8]) -> Result<Vec<MetadataField>> {
    let mut fields = Vec::new();
    if data.len() < 12 { return Ok(fields); }

    let mut pos = 12usize;
    while pos + 8 <= data.len() {
        let kind = &data[pos..pos + 4];
        let size = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let padded = size + (size & 1);

        match kind {
            b"EXIF" => {
                let exif_data = &data[pos + 8..(pos + 8 + size).min(data.len())];
                if let Ok(exif) = exif::Reader::new().read_raw(exif_data.to_vec()) {
                    for f in exif.fields() {
                        let name = f.tag.to_string();
                        let value = f.display_value().with_unit(&exif).to_string();
                        let (cat, risk) = classify_field(&name, &value);
                        fields.push(MetadataField { name, value, category: cat, risk });
                    }
                } else {
                    fields.push(MetadataField {
                        name: "EXIF".into(),
                        value: format!("{} bytes", size),
                        category: MetadataCategory::Camera,
                        risk: RiskLevel::High,
                    });
                }
            }
            b"XMP " => {
                fields.push(MetadataField {
                    name: "XMP metadata".into(),
                    value: format!("{} bytes", size),
                    category: MetadataCategory::Software,
                    risk: RiskLevel::Medium,
                });
            }
            b"ICCP" => {
                fields.push(MetadataField {
                    name: "ICC Profile".into(),
                    value: format!("{} bytes", size),
                    category: MetadataCategory::Software,
                    risk: RiskLevel::Low,
                });
            }
            _ => {}
        }

        pos += 8 + padded;
    }
    Ok(fields)
}

pub fn strip_webp(data: Vec<u8>, opts: &StripOptions) -> Result<Vec<u8>> {
    if data.len() < 12 { return Ok(data); }

    let mut output = data[..12].to_vec();
    let mut pos = 12usize;

    while pos + 8 <= data.len() {
        let kind = &data[pos..pos + 4];
        let size = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let padded = size + (size & 1);
        let end = (pos + 8 + padded).min(data.len());

        let skip = match kind {
            b"EXIF" => true,
            b"XMP " => opts.strip_xmp,
            b"ICCP" => opts.strip_icc,
            _ => false,
        };

        if !skip {
            output.extend_from_slice(&data[pos..end]);
        }
        pos = end;
    }

    let new_size = (output.len() - 8) as u32;
    output[4..8].copy_from_slice(&new_size.to_le_bytes());
    Ok(output)
}


pub fn inspect_tiff(data: &[u8]) -> Result<Vec<MetadataField>> {

    let mut fields = Vec::new();
    if let Ok(exif) = exif::Reader::new().read_from_container(&mut Cursor::new(data)) {
        for f in exif.fields() {
            let name = f.tag.to_string();
            let value = f.display_value().with_unit(&exif).to_string();
            let (cat, risk) = classify_field(&name, &value);
            fields.push(MetadataField { name, value, category: cat, risk });
        }
    }
    Ok(fields)
}

pub fn strip_tiff(data: Vec<u8>) -> Result<Vec<u8>> {
    let img = image::load_from_memory_with_format(&data, image::ImageFormat::Tiff)
        .context("Failed to decode TIFF")?;
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Tiff)
        .context("Failed to encode TIFF")?;
    Ok(out.into_inner())
}

pub fn strip_bmp(data: Vec<u8>) -> Result<Vec<u8>> {
    let img = image::load_from_memory_with_format(&data, image::ImageFormat::Bmp)
        .context("Failed to decode BMP")?;
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Bmp)
        .context("Failed to encode BMP")?;
    Ok(out.into_inner())
}

pub fn strip_gif(data: Vec<u8>) -> Result<Vec<u8>> {
    let img = image::load_from_memory_with_format(&data, image::ImageFormat::Gif)
        .context("Failed to decode GIF")?;
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Gif)
        .context("Failed to encode GIF")?;
    Ok(out.into_inner())
}

pub fn inspect_gif(data: &[u8]) -> Result<Vec<MetadataField>> {
    let mut fields = Vec::new();

    let mut pos = 6usize;
    if data.len() < 6 { return Ok(fields); }
    while pos + 2 < data.len() {
        if data[pos] == 0x21 && data[pos + 1] == 0xFE {

            pos += 2;
            let mut comment = Vec::new();
            while pos < data.len() {
                let block_size = data[pos] as usize;
                if block_size == 0 { pos += 1; break; }
                pos += 1;
                let end = (pos + block_size).min(data.len());
                comment.extend_from_slice(&data[pos..end]);
                pos = end;
            }
            fields.push(MetadataField {
                name: "GIF Comment".into(),
                value: String::from_utf8_lossy(&comment).into(),
                category: MetadataCategory::Other,
                risk: RiskLevel::Medium,
            });
        } else {
            pos += 1;
        }
    }
    Ok(fields)
}


pub fn thumbnail_b64(data: &[u8]) -> Option<String> {

    Some(STANDARD.encode(&data[..data.len().min(65536)]))
}


pub fn classify_field(name: &str, value: &str) -> (MetadataCategory, RiskLevel) {
    let n = name.to_lowercase();
    let v = value.to_lowercase();

    if n.contains("gps") || n.contains("latitude") || n.contains("longitude")
        || n.contains("altitude") || n.contains("location") || n.contains("geolocation")
    {
        return (MetadataCategory::Gps, RiskLevel::Critical);
    }
    if n.contains("author") || n.contains("artist") || n.contains("creator")
        || n.contains("copyright") || n.contains("byline") || n.contains("owner")
        || n.contains("writer") || n.contains("contact")
        || n == "dc:creator" || n == "xmp:creatortool"
    {
        return (MetadataCategory::Author, RiskLevel::High);
    }
    if n.contains("serial") || n.contains("unique") || n.contains("device")
        || n.contains("imei") || n.contains("uuid") || (n.contains("id") && v.len() > 10)
    {
        return (MetadataCategory::Software, RiskLevel::High);
    }
    if n.contains("software") || n.contains("processing") || n.contains("maker")
        || n.contains("tool") || n.contains("encoder") || n.contains("generator")
    {
        return (MetadataCategory::Software, RiskLevel::Medium);
    }
    if n.contains("model") || n.contains("make") || n.contains("lens")
        || n.contains("camera") || n.contains("flash") || n.contains("iso")
        || n.contains("exposure") || n.contains("aperture") || n.contains("focal")
        || n.contains("shutter") || n.contains("f-number") || n.contains("fnumber")
    {
        return (MetadataCategory::Camera, RiskLevel::Medium);
    }
    if n.contains("iptc") || n.contains("byline") || n.contains("city")
        || n.contains("country") || n.contains("province") || n.contains("headline")
    {
        return (MetadataCategory::Iptc, RiskLevel::High);
    }
    if n.contains("xmp") || n.starts_with("dc:") || n.starts_with("xap:")
        || n.starts_with("photoshop:") || n.starts_with("lr:")
    {
        return (MetadataCategory::Xmp, RiskLevel::Medium);
    }
    if n.contains("date") || n.contains("time") || n.contains("modified")
        || n.contains("created") || n.contains("digitized")
    {
        return (MetadataCategory::Timestamps, RiskLevel::Low);
    }
    (MetadataCategory::Other, RiskLevel::Low)
}


fn extract_xmp_value(xmp: &str, key: &str) -> Option<String> {
    let start = xmp.find(&format!("<{}", key))?;
    let gt = xmp[start..].find('>')? + start;
    let end = xmp[gt + 1..].find(&format!("</{}", key)).map(|i| gt + 1 + i)?;
    Some(xmp[gt + 1..end].trim().to_string())
}

fn xmp_field_category(key: &str) -> MetadataCategory {
    if key.contains("creator") || key.contains("Creator") { MetadataCategory::Author }
    else if key.contains("Date") || key.contains("date") { MetadataCategory::Timestamps }
    else { MetadataCategory::Xmp }
}

fn xmp_field_risk(key: &str) -> RiskLevel {
    if key.contains("creator") || key.contains("Creator") || key.contains("City") || key.contains("Country") {
        RiskLevel::High
    } else {
        RiskLevel::Medium
    }
}
