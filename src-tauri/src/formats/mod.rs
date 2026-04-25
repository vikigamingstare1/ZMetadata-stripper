pub mod audio;
pub mod document;
pub mod image;
pub mod video;

use std::io::{Read, Seek};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum FileFormat {

    Jpeg,
    Png,
    Webp,
    Tiff,
    Gif,
    Bmp,
    Avif,
    Heic,

    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Odt,
    Ods,
    Odp,
    Rtf,

    Mp3,
    Flac,
    Ogg,
    Wav,
    M4a,
    Aiff,

    Mp4,
    Mov,
    Mkv,
    Webm,
    Avi,

    Svg,
    Eps,

    Unknown,
}

impl FileFormat {
    pub fn label(&self) -> &'static str {
        match self {
            FileFormat::Jpeg => "JPEG",
            FileFormat::Png  => "PNG",
            FileFormat::Webp => "WebP",
            FileFormat::Tiff => "TIFF",
            FileFormat::Gif  => "GIF",
            FileFormat::Bmp  => "BMP",
            FileFormat::Avif => "AVIF",
            FileFormat::Heic => "HEIC",
            FileFormat::Pdf  => "PDF",
            FileFormat::Docx => "DOCX",
            FileFormat::Xlsx => "XLSX",
            FileFormat::Pptx => "PPTX",
            FileFormat::Odt  => "ODT",
            FileFormat::Ods  => "ODS",
            FileFormat::Odp  => "ODP",
            FileFormat::Rtf  => "RTF",
            FileFormat::Mp3  => "MP3",
            FileFormat::Flac => "FLAC",
            FileFormat::Ogg  => "OGG",
            FileFormat::Wav  => "WAV",
            FileFormat::M4a  => "M4A",
            FileFormat::Aiff => "AIFF",
            FileFormat::Mp4  => "MP4",
            FileFormat::Mov  => "MOV",
            FileFormat::Mkv  => "MKV",
            FileFormat::Webm => "WebM",
            FileFormat::Avi  => "AVI",
            FileFormat::Svg  => "SVG",
            FileFormat::Eps  => "EPS",
            FileFormat::Unknown => "Unknown",
        }
    }

    /// Whether we have a full stripping implementation for this format
    pub fn is_supported(&self) -> bool {
        !matches!(self, FileFormat::Avif | FileFormat::Heic | FileFormat::Mkv
            | FileFormat::Webm | FileFormat::Avi | FileFormat::Eps | FileFormat::Unknown)
    }
}


/// Detect format from magic bytes first, fall back to file extension.
pub fn detect_format(path: &Path, header: &[u8]) -> FileFormat {
    let by_magic = detect_by_magic(header);
    if by_magic != FileFormat::Unknown {
        return by_magic;
    }
    detect_by_extension(path)
}

pub fn detect_by_magic(h: &[u8]) -> FileFormat {
    let h32 = &h[..h.len().min(32)];


    if h32.starts_with(b"\xFF\xD8\xFF") { return FileFormat::Jpeg; }

    if h32.starts_with(b"\x89PNG\r\n\x1A\n") { return FileFormat::Png; }

    if h32.starts_with(b"II\x2A\x00") || h32.starts_with(b"MM\x00\x2A") { return FileFormat::Tiff; }

    if h32.starts_with(b"GIF87a") || h32.starts_with(b"GIF89a") { return FileFormat::Gif; }

    if h32.starts_with(b"BM") { return FileFormat::Bmp; }

    if h32.starts_with(b"%PDF") { return FileFormat::Pdf; }

    if h32.starts_with(b"fLaC") { return FileFormat::Flac; }

    if h32.starts_with(b"OggS") { return FileFormat::Ogg; }

    if h32.starts_with(b"ID3")
        || h32.starts_with(b"\xFF\xFB")
        || h32.starts_with(b"\xFF\xF3")
        || h32.starts_with(b"\xFF\xF2")
        || h32.starts_with(b"\xFF\xFA")
    {
        return FileFormat::Mp3;
    }

    if h32.starts_with(b"{\\rtf") { return FileFormat::Rtf; }

    if h32.starts_with(b"<?xml") || h32.starts_with(b"<svg") { return detect_xml_type(h); }


    if h32.starts_with(b"RIFF") {
        return match h32.get(8..12) {
            Some(b"WEBP") => FileFormat::Webp,
            Some(b"WAVE") => FileFormat::Wav,
            Some(b"AVI ") => FileFormat::Avi,
            _ => FileFormat::Unknown,
        };
    }

    if h32.starts_with(b"FORM") && (h32.get(8..12) == Some(b"AIFF") || h32.get(8..12) == Some(b"AIFC")) {
        return FileFormat::Aiff;
    }


    if h32.starts_with(b"\x1A\x45\xDF\xA3") { return detect_ebml_type(h); }


    if h32.starts_with(b"PK\x03\x04") { return detect_zip_type(h); }


    if h32.get(4..8) == Some(b"ftyp") { return detect_mp4_type(h); }

    FileFormat::Unknown
}

fn detect_xml_type(h: &[u8]) -> FileFormat {
    let snippet = std::str::from_utf8(&h[..h.len().min(512)]).unwrap_or("");
    if snippet.contains("<svg") { return FileFormat::Svg; }
    if snippet.contains("%!PS") || snippet.contains("%%Creator") { return FileFormat::Eps; }
    FileFormat::Unknown
}

fn detect_ebml_type(h: &[u8]) -> FileFormat {
    let s = std::str::from_utf8(h).unwrap_or("");
    if s.contains("webm") { FileFormat::Webm } else { FileFormat::Mkv }
}

fn detect_mp4_type(h: &[u8]) -> FileFormat {

    match h.get(8..12) {
        Some(b"M4A ") | Some(b"M4B ") | Some(b"M4P ") | Some(b"M4V ") => FileFormat::M4a,
        Some(b"qt  ") => FileFormat::Mov,
        Some(b"MSNV") | Some(b"NDAS") | Some(b"NDSC") => FileFormat::Mp4,
        _ => FileFormat::Mp4,
    }
}

fn detect_zip_type(h: &[u8]) -> FileFormat {

    use std::io::Cursor;
    if let Ok(mut a) = zip::ZipArchive::new(Cursor::new(h)) {
        for i in 0..a.len() {
            if let Ok(f) = a.by_index(i) {
                match f.name() {
                    "word/document.xml"        => return FileFormat::Docx,
                    "xl/workbook.xml"          => return FileFormat::Xlsx,
                    "ppt/presentation.xml"     => return FileFormat::Pptx,
                    "content.xml"              => return FileFormat::Odt,
                    "META-INF/manifest.xml"    => return FileFormat::Odt,
                    _ => {}
                }
            }
        }
    }
    FileFormat::Unknown
}

fn detect_by_extension(path: &Path) -> FileFormat {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    match ext.as_deref() {
        Some("jpg") | Some("jpeg") => FileFormat::Jpeg,
        Some("png")                => FileFormat::Png,
        Some("webp")               => FileFormat::Webp,
        Some("tiff") | Some("tif")=> FileFormat::Tiff,
        Some("gif")                => FileFormat::Gif,
        Some("bmp")                => FileFormat::Bmp,
        Some("avif")               => FileFormat::Avif,
        Some("heic") | Some("heif")=> FileFormat::Heic,
        Some("pdf")                => FileFormat::Pdf,
        Some("docx")               => FileFormat::Docx,
        Some("xlsx")               => FileFormat::Xlsx,
        Some("pptx")               => FileFormat::Pptx,
        Some("odt")                => FileFormat::Odt,
        Some("ods")                => FileFormat::Ods,
        Some("odp")                => FileFormat::Odp,
        Some("rtf")                => FileFormat::Rtf,
        Some("mp3")                => FileFormat::Mp3,
        Some("flac")               => FileFormat::Flac,
        Some("ogg") | Some("oga") | Some("opus") => FileFormat::Ogg,
        Some("wav") | Some("wave") => FileFormat::Wav,
        Some("m4a") | Some("aac") => FileFormat::M4a,
        Some("aiff") | Some("aif")=> FileFormat::Aiff,
        Some("mp4") | Some("m4v") => FileFormat::Mp4,
        Some("mov")                => FileFormat::Mov,
        Some("mkv")                => FileFormat::Mkv,
        Some("webm")               => FileFormat::Webm,
        Some("avi")                => FileFormat::Avi,
        Some("svg") | Some("svgz") => FileFormat::Svg,
        Some("eps")                => FileFormat::Eps,
        _                          => FileFormat::Unknown,
    }
}
