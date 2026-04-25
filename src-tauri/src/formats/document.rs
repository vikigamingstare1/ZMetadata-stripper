use anyhow::{Context, Result};
use lopdf::{Document, Object};
use std::io::{Cursor, Read, Write};
use zip::{write::FileOptions, ZipArchive, ZipWriter};

use crate::types::{MetadataCategory, MetadataField, RiskLevel, StripOptions};

// ── PDF ───────────────────────────────────────────────────────────────────────

pub fn inspect_pdf(data: &[u8]) -> Result<Vec<MetadataField>> {
    let doc = Document::load_mem(data).context("Failed to parse PDF")?;
    let mut fields = Vec::new();

    if let Ok(info_ref) = doc.trailer.get(b"Info").and_then(|o| o.as_reference()) {
        if let Ok(Object::Dictionary(dict)) = doc.get_object(info_ref) {
            for (key, val) in dict.iter() {
                let name = String::from_utf8_lossy(key).to_string();
                let value = match val {
                    Object::String(bytes, _) => String::from_utf8_lossy(bytes).to_string(),
                    Object::Name(n)          => String::from_utf8_lossy(n).to_string(),
                    _                        => format!("{:?}", val),
                };
                let (cat, risk) = classify_pdf_field(&name);
                fields.push(MetadataField { name, value, category: cat, risk });
            }
        }
    }

    // Check for XMP stream in catalog
    if let Ok(root_ref) = doc.trailer.get(b"Root").and_then(|o| o.as_reference()) {
        if let Ok(Object::Dictionary(cat)) = doc.get_object(root_ref) {
            if cat.has(b"Metadata") {
                fields.push(MetadataField {
                    name: "XMP Metadata Stream".into(),
                    value: "present".into(),
                    category: MetadataCategory::Xmp,
                    risk: RiskLevel::High,
                });
            }
        }
    }

    Ok(fields)
}

pub fn strip_pdf(data: Vec<u8>, opts: &StripOptions) -> Result<Vec<u8>> {
    let mut doc = Document::load_mem(&data).context("Failed to parse PDF")?;

    if let Ok(info_ref) = doc.trailer.get(b"Info").and_then(|o| o.as_reference()) {
        doc.objects.remove(&info_ref);
    }
    doc.trailer.remove(b"Info");

    if let Ok(root_ref) = doc.trailer.get(b"Root").and_then(|o| o.as_reference()) {
        if let Ok(Object::Dictionary(cat_dict)) = doc.get_object_mut(root_ref) {
            cat_dict.remove(b"Metadata");
        }
    }

    // Remove document ID from trailer
    doc.trailer.remove(b"ID");

    let mut out = Vec::new();
    doc.save_to(&mut out).context("Failed to serialize PDF")?;

    if opts.pdf_full_rewrite {
        // Double-pass: reload and re-save to guarantee no incremental update history
        let mut doc2 = Document::load_mem(&out).context("Failed to reload PDF")?;
        let mut final_out = Vec::new();
        doc2.save_to(&mut final_out).context("Forensic-safe rewrite failed")?;
        return Ok(final_out);
    }

    Ok(out)
}

// ── DOCX / XLSX / PPTX — ZIP + XML replacement ───────────────────────────────

const EMPTY_CORE_XML: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties""#,
    r#" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/""#,
    r#" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"></cp:coreProperties>"#
);

const EMPTY_APP_XML_WORD: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"></Properties>"#
);

const EMPTY_META_XML: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8"?>"#,
    r#"<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0""#,
    r#" xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0">"#,
    r#"<office:meta></office:meta></office:document-meta>"#
);

pub fn inspect_office(data: &[u8]) -> Result<Vec<MetadataField>> {
    let mut fields = Vec::new();
    let mut archive = ZipArchive::new(Cursor::new(data)).context("Not a valid Office ZIP")?;

    for target in ["docProps/core.xml", "docProps/app.xml", "meta.xml"] {
        if let Ok(mut file) = archive.by_name(target) {
            let mut content = String::new();
            file.read_to_string(&mut content).ok();
            fields.extend(extract_xml_metadata(&content, target));
        }
    }

    Ok(fields)
}

pub fn strip_office(data: Vec<u8>) -> Result<Vec<u8>> {
    let mut archive = ZipArchive::new(Cursor::new(data)).context("Not a valid Office ZIP")?;
    let mut buf = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            let options: zip::write::FileOptions<'_, ()> = FileOptions::default()
                .compression_method(file.compression());

            writer.start_file(name.as_str(), options)?;

            match name.as_str() {
                "docProps/core.xml" | "docProps/Core.xml" => {
                    writer.write_all(EMPTY_CORE_XML.as_bytes())?;
                }
                "docProps/app.xml" | "docProps/App.xml" => {
                    writer.write_all(EMPTY_APP_XML_WORD.as_bytes())?;
                }
                "docProps/custom.xml" => {
                    // Drop custom properties file content
                    writer.write_all(b"")?;
                }
                "meta.xml" => {
                    // ODF documents
                    writer.write_all(EMPTY_META_XML.as_bytes())?;
                }
                _ => {
                    let mut content = Vec::new();
                    file.read_to_end(&mut content)?;
                    writer.write_all(&content)?;
                }
            }
        }

        writer.finish()?;
    }

    Ok(buf)
}

// ── SVG ───────────────────────────────────────────────────────────────────────

pub fn inspect_svg(data: &[u8]) -> Result<Vec<MetadataField>> {
    let mut fields = Vec::new();
    let text = String::from_utf8_lossy(data);

    for tag in ["<title>", "<desc>", "inkscape:version", "sodipodi:docname",
                "dc:creator", "dc:date", "cc:Agent", "rdf:RDF"] {
        if text.contains(tag) {
            let name = tag.trim_start_matches('<').trim_end_matches('>').to_string();
            fields.push(MetadataField {
                name,
                value: "present".into(),
                category: if tag.contains("creator") || tag.contains("Agent") {
                    MetadataCategory::Author
                } else {
                    MetadataCategory::Xmp
                },
                risk: if tag.contains("creator") { RiskLevel::High } else { RiskLevel::Medium },
            });
        }
    }
    Ok(fields)
}

pub fn strip_svg(data: Vec<u8>) -> Result<Vec<u8>> {
    use quick_xml::{Reader, Writer, events::Event};

    let mut reader = Reader::from_reader(Cursor::new(&data));
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut skip_depth: usize = 0;
    let mut buf = Vec::new();

    // Elements to strip entirely
    let strip_elements = [
        b"metadata".as_ref(),
        b"rdf:RDF".as_ref(),
        b"title".as_ref(),
        b"desc".as_ref(),
        b"dc:title".as_ref(),
        b"dc:description".as_ref(),
    ];

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) => {
                if skip_depth > 0 {
                    skip_depth += 1;
                } else if strip_elements.contains(&e.name().as_ref()) {
                    skip_depth = 1;
                } else {
                    // Strip metadata-related attributes from svg root
                    if e.name().as_ref() == b"svg" {
                        let mut clean = e.to_owned();
                        let attrs: Vec<_> = e.attributes()
                            .flatten()
                            .filter(|a| {
                                let k = std::str::from_utf8(a.key.as_ref()).unwrap_or("");
                                !k.starts_with("inkscape:")
                                    && !k.starts_with("sodipodi:")
                                    && !k.starts_with("dc:")
                                    && !k.starts_with("cc:")
                                    && !k.starts_with("rdf:")
                            })
                            .collect();
                        clean.clear_attributes();
                        for a in attrs {
                            clean.push_attribute(a);
                        }
                        writer.write_event(Event::Start(clean.into()))?;
                    } else {
                        writer.write_event(Event::Start(e.clone()))?;
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if skip_depth > 0 {
                    skip_depth -= 1;
                } else {
                    writer.write_event(Event::End(e.clone()))?;
                }
            }
            Ok(Event::Empty(ref e)) => {
                if skip_depth == 0 {
                    writer.write_event(Event::Empty(e.clone()))?;
                }
            }
            Ok(ev) => {
                if skip_depth == 0 {
                    writer.write_event(ev)?;
                }
            }
            Err(_) => break,
        }
        buf.clear();
    }

    Ok(writer.into_inner().into_inner())
}

// ── RTF ───────────────────────────────────────────────────────────────────────

pub fn inspect_rtf(data: &[u8]) -> Result<Vec<MetadataField>> {
    let mut fields = Vec::new();
    let text = String::from_utf8_lossy(data);

    for (tag, label, cat, risk) in [
        (r"\author",   "Author",   MetadataCategory::Author,     RiskLevel::High),
        (r"\company",  "Company",  MetadataCategory::Author,     RiskLevel::High),
        (r"\operator", "Operator", MetadataCategory::Author,     RiskLevel::High),
        (r"\creatim",  "Created",  MetadataCategory::Timestamps, RiskLevel::Low),
        (r"\revtim",   "Revised",  MetadataCategory::Timestamps, RiskLevel::Low),
    ] {
        if let Some(idx) = text.find(tag) {
            let snippet = &text[idx + tag.len()..];
            let val: String = snippet.chars().take(80)
                .take_while(|&c| c != '\\' && c != '}')
                .collect();
            if !val.trim().is_empty() {
                fields.push(MetadataField {
                    name: label.into(),
                    value: val.trim().to_string(),
                    category: cat,
                    risk,
                });
            }
        }
    }
    Ok(fields)
}

pub fn strip_rtf(data: Vec<u8>) -> Result<Vec<u8>> {
    let text = String::from_utf8_lossy(&data).to_string();
    let mut out = text;

    // Zero out RTF header metadata fields
    for tag in [r"\author", r"\company", r"\operator", r"\manager",
                r"\doccomm", r"\hlinkbase", r"\keywords", r"\subject"] {
        while let Some(idx) = out.find(tag) {
            // Find end of this control word group
            let start = idx + tag.len();
            let end = out[start..].find(|c: char| c == '\\' || c == '}')
                .map(|i| start + i)
                .unwrap_or(start);
            out.replace_range(start..end, " ");
        }
    }

    Ok(out.into_bytes())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn extract_xml_metadata(xml: &str, source: &str) -> Vec<MetadataField> {
    let mut fields = Vec::new();
    let tags = [
        ("dc:creator",      MetadataCategory::Author,     RiskLevel::High),
        ("dc:title",        MetadataCategory::Other,      RiskLevel::Low),
        ("dc:subject",      MetadataCategory::Other,      RiskLevel::Low),
        ("dc:description",  MetadataCategory::Other,      RiskLevel::Low),
        ("cp:keywords",     MetadataCategory::Other,      RiskLevel::Low),
        ("cp:lastModifiedBy", MetadataCategory::Author,   RiskLevel::High),
        ("dcterms:created", MetadataCategory::Timestamps, RiskLevel::Low),
        ("dcterms:modified",MetadataCategory::Timestamps, RiskLevel::Low),
        ("Application",     MetadataCategory::Software,   RiskLevel::Medium),
        ("Company",         MetadataCategory::Author,     RiskLevel::High),
        ("Template",        MetadataCategory::Software,   RiskLevel::Medium),
        ("meta:initial-creator", MetadataCategory::Author, RiskLevel::High),
        ("meta:creation-date",   MetadataCategory::Timestamps, RiskLevel::Low),
        ("meta:author",          MetadataCategory::Author, RiskLevel::High),
    ];

    for (tag, cat, risk) in &tags {
        if let (Some(s), Some(e)) = (
            xml.find(&format!("<{}", tag)).and_then(|i| xml[i..].find('>').map(|j| i + j + 1)),
            xml.find(&format!("</{}", tag)),
        ) {
            if s < e {
                let val = xml[s..e].trim().to_string();
                if !val.is_empty() {
                    fields.push(MetadataField {
                        name: format!("{} ({})", tag, source.rsplit('/').next().unwrap_or(source)),
                        value: val,
                        category: cat.clone(),
                        risk: risk.clone(),
                    });
                }
            }
        }
    }

    fields
}

fn classify_pdf_field(name: &str) -> (MetadataCategory, RiskLevel) {
    match name {
        "Author" | "Creator" | "Producer" => (MetadataCategory::Author,     RiskLevel::High),
        "Title" | "Subject" | "Keywords"  => (MetadataCategory::Other,      RiskLevel::Medium),
        "CreationDate" | "ModDate"        => (MetadataCategory::Timestamps,  RiskLevel::Low),
        _                                 => (MetadataCategory::Other,       RiskLevel::Low),
    }
}
