<div align="center">
  <img src="src-tauri/icons/icon.png" width="96" height="96" alt="ZMetadata Stripper" />
  <h1>ZMetadata Stripper</h1>
  <p><strong>Privacy-first metadata scrubber for images, documents, audio &amp; video.</strong></p>

  [![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-violet.svg)](LICENSE)
  [![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-blue.svg)]()
  [![Version](https://img.shields.io/badge/Version-0.1.0-cyan.svg)]()
  [![Built with Tauri](https://img.shields.io/badge/Built%20with-Tauri%20v2-24C8D8.svg)](https://tauri.app)
  [![Website](https://img.shields.io/badge/Website-zsync.eu%2Fzmetadata--stripper-8B5CF6.svg)](https://zsync.eu/zmetadata-stripper/)

  <br/>

  [**Download**](https://zsync.eu/zmetadata-stripper/) &nbsp;·&nbsp; [**Website**](https://zsync.eu/zmetadata-stripper/) &nbsp;·&nbsp; [**zsync.eu**](https://zsync.eu)
</div>

---

> [!IMPORTANT]
> ZMetadata Stripper performs a **full metadata wipe** — not just EXIF. It removes GPS coordinates, author identity, device fingerprints, software signatures, and embedded thumbnails from 25+ file formats. Once stripped, this data is unrecoverable.

> [!NOTE]
> Downloads are available exclusively through the [official landing page](https://zsync.eu/zmetadata-stripper/). No third-party mirrors.

---

## What it does

Every file you create carries hidden metadata — your name, GPS location, device model, software version, timestamps. ZMetadata Stripper scrubs all of it before you share.

| Metadata type | Examples removed |
|---|---|
| **GPS & Location** | Latitude, longitude, altitude, GPS timestamp, direction |
| **Identity** | Author, artist, copyright, creator, "Last Modified By" |
| **Device** | Camera model, lens info, serial number, firmware |
| **Software** | Editing software, version strings, processing history |
| **Timestamps** | DateTimeOriginal, ModifyDate, CreateDate, file dates |
| **Embedded data** | Thumbnail images, MakerNotes, XMP, IPTC, ICC (optional) |

---

## Supported formats

<table>
<tr>
<td><strong>Images</strong></td>
<td>JPEG, PNG, WEBP, HEIC/HEIF, TIFF, BMP, GIF, AVIF, RAW (CR2, NEF, ARW, DNG)</td>
</tr>
<tr>
<td><strong>Documents</strong></td>
<td>PDF, DOCX, XLSX, PPTX, ODT, ODS, ODP</td>
</tr>
<tr>
<td><strong>Audio</strong></td>
<td>MP3 (ID3v2), FLAC, OGG, M4A</td>
</tr>
<tr>
<td><strong>Video</strong></td>
<td>MP4, MOV, MKV</td>
</tr>
</table>

---

## Features

### Preset profiles

| Preset | What it does |
|---|---|
| **Max Privacy** | Full forensic-safe wipe — strips everything including ICC profiles and all timestamps |
| **Keep Quality** | Strips GPS & author identity, preserves ICC color profile and exposure data |
| **Social Media** | Strips GPS & author, preserves color profile for platform delivery |
| **Documents Only** | Focused PDF & Office metadata scrub |

> [!TIP]
> The **Max Privacy** preset triggers a full PDF rewrite via `lopdf` — no incremental update history, no rollback-recoverable data. Standard forensic tools cannot reconstruct the original metadata.

### Inject neutral metadata

The **Social Media** preset includes an optional toggle to write back a minimal non-identifying EXIF block after stripping:

```
Software:    ZScrub 1.0
ColorSpace:  sRGB
Orientation: 1 (Normal)
```

This prevents some platforms (Instagram, X/Twitter) from auto-flagging completely metadata-free images. **Off by default.** Every injected field is marked `[injected]` in the audit report.

### Metadata inspector

- Before / after diff viewer with red (removed) / green (kept) field highlighting
- Collapsible field categories: GPS, Camera, Author, Software, Timestamps
- Sensitive score indicator — shows how identifying the metadata is
- Click any field to toggle it in or out of the strip operation

### Batch processing

- Drag entire folders into the queue
- Configurable concurrency (1–16 threads via Rust rayon)
- Per-file status: Pending → Processing → Clean / Failed
- Right-click context menu per card: Preview, Inspect, Strip Solo, Remove

### Output options

- **Overwrite** — replace original in-place
- **Save copy** — writes `filename_clean.ext` alongside the original
- **Custom folder** — pick once, applies to entire batch

---

## Installation

> [!WARNING]
> Do **not** run untrusted binaries. Always download from the [official page](https://zsync.eu/zmetadata-stripper/) and verify the file matches the release hash published there.

| Platform | Package |
|---|---|
| Linux | `.AppImage` (universal), `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL) |
| Windows | `.exe` NSIS installer |

**[zsync.eu/zmetadata-stripper](https://zsync.eu/zmetadata-stripper/)**

---

## Build from source

**Requirements:** Rust 1.77+, Node.js 20+, pnpm 9+

```bash
git clone https://github.com/TheHolyOneZ/ZMetadata-stripper
cd ZMetadata-stripper
pnpm install
pnpm tauri build
```

> [!IMPORTANT]
> This project uses **pnpm exclusively**. Do not use `npm` or `yarn` — the lockfile will conflict and the build will fail.

Binaries are output to `src-tauri/target/release/`.

---

## Tech stack

| Layer | Technologies |
|---|---|
| **Frontend** | React 19, TypeScript, Tailwind CSS v4, Framer Motion, Zustand |
| **Backend** | Rust, Tauri v2 |
| **Metadata libs** | lopdf, img-parts, kamadak-exif, id3, metaflac, mp4ameta, quick-xml, zip |
| **Design** | Deep Void dark theme, glassmorphism bento cards, Inter + JetBrains Mono |

---

## License

Released under the [GNU General Public License v3.0](LICENSE).

---

<div align="center">
  Made by <a href="https://github.com/TheHolyOneZ"><strong>TheHolyOneZ</strong></a> &nbsp;·&nbsp;
  <a href="https://zsync.eu/zmetadata-stripper/">Project page</a> &nbsp;·&nbsp;
  <a href="https://zsync.eu">zsync.eu</a>
</div>
