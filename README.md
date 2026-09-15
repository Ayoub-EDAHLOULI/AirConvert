# AirConvert

A fully offline, local-first file converter for desktop — pick a file, pick a target format, convert, done. No uploads, no network calls, no cloud dependency.

Built because I work on air-gapped/internet-restricted VMs at my day job and needed a Convertio/CloudConvert-style tool that actually works with zero connectivity.

## Why offline matters here

Cloud converters (Convertio, CloudConvert, etc.) upload your file to a server, convert it there, and send it back. That's a non-starter on a locked-down or offline machine — and it's also a privacy concern for sensitive files. AirConvert does every conversion on-disk, on your machine, with no network activity at all. This is verifiable: you can block network access at the OS level and it will keep working exactly the same.

## Status

🚧 Early development. **Phase 1: Images** shipped. Currently building **Phase 2: Audio**. See [Roadmap](#roadmap) below.

## Tech stack

- **Shell/runtime:** [Tauri](https://tauri.app/) (Rust)
- **Frontend:** React + TypeScript
- **Conversion engines:** pure-Rust crates where possible (see [Engine choices](#engine-choices-and-trade-offs)), external sidecar binaries only where necessary

## Roadmap

Phased by format category, one shipped and working before the next starts:

| Phase       | Category     | Formats                                                                     | Approach                                                                  |
| ----------- | ------------ | --------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| 1 (done)    | Images       | jpg, png, webp, gif, bmp, tiff, svg, ico, tga, pnm, qoi, avif (output only) | Pure Rust (`image`, `resvg`) — no external binaries                       |
| 2 (current) | Audio        | mp3, wav, flac, ogg, m4a, aac, opus, wma                                    | Bundled FFmpeg sidecar (see licensing note below)                         |
| 3           | Documents    | md, txt, html, rtf, odt, docx                                               | Bundled Pandoc sidecar — content conversion, not full-fidelity layout     |
| 4           | Spreadsheets | csv, xlsx, ods                                                              | Pure Rust (`calamine`, `rust_xlsxwriter`) — data only, no formulas/macros |
| 5 (stretch) | Video        | mp4, mov, avi, webm, gif                                                    | FFmpeg sidecar                                                            |

**Deliberately out of scope for now:**

- **HEIC** — excluded due to HEVC patent licensing ambiguity around redistribution, not a technical limitation.
- **AVIF as an input format** — AirConvert can convert _to_ AVIF, but not _from_ it. Decoding AVIF needs the `dav1d` decoder, a much heavier dependency than the `ravif` encoder alone; may be added later if there's real demand.
- **Full-fidelity Office documents** (complex docx/xlsx/pptx with embedded objects, macros, exact layout preservation) — this realistically requires a full LibreOffice headless install (700MB+), which conflicts with the goal of a small, portable, offline-first bundle. May be revisited later as an optional detected-if-installed backend, never bundled by default.
- **PDF editing** (merge/split/compress) — planned as a separate "offline PDF toolkit" project, kept out of this repo's scope to keep it focused on format _conversion_.

## Engine choices and trade-offs

| Engine                         | Used for     | Why                                                                                                                   |
| ------------------------------ | ------------ | --------------------------------------------------------------------------------------------------------------------- |
| Rust `image` + `resvg`         | Images       | Compiles directly into the binary — no subprocess, no license concerns, minimal size impact                           |
| FFmpeg                         | Audio, Video | Industry-standard, well understood bundling pattern (same approach used by apps like HandBrake); see licensing note below |
| Pandoc                         | Documents    | Single-binary sidecar, handles markup-style formats well; explicitly not a fidelity-preserving office-document engine |
| `calamine` / `rust_xlsxwriter` | Spreadsheets | Pure Rust, handles tabular data without pulling in a full spreadsheet engine                                          |

**FFmpeg licensing note:** during development, any static FFmpeg build works (see the [FFmpeg sidecar](#ffmpeg-sidecar-required-for-audio-and-later-video) setup below). For an actual shipped/distributed release, the specific build matters: the common "essentials"-style builds (e.g. gyan.dev's essentials build) are GPL-licensed because they bundle libx264/libx265, and bundling GPL code into a distributed binary carries GPL's copyleft obligations for that binary. AirConvert only needs FFmpeg's audio codecs (mp3/wav/flac/ogg/m4a), so a release build should use an **LGPL-only** FFmpeg build (no libx264/libx265/other GPL-only components) to keep the LGPL bundling story intact — this hasn't been done yet; the dev setup below is not release-safe as-is.

## Features

- Drag-and-drop file input with format auto-detection
- Target-format picker
- Batch conversion (multiple files at once), with live per-file progress
- Optional output folder (defaults to saving next to the source file)
- Image conversion: optional max-dimension resize and JPG/WebP quality control
- Audio conversion (mp3, wav, flac, ogg, m4a, aac, opus, wma) via a bundled FFmpeg sidecar
- Zero network calls, verifiable by blocking network access at the OS level
- Portable, installable desktop app (Windows-first, cross-platform as feasible)

## Planned (not yet built)

- CLI mode for scripting conversions on headless/VM environments

## Offline Verification

AirConvert's zero-network-calls claim is checked two ways: a static code audit (done on every change) and an OS-level runtime block (done before tagging a release). Both are described below so the claim is reproducible, not just asserted.

### 1. Static code audit

Run these from the repo root. Each should return **no matches**:

```bash
# Browser-side network primitives
grep -rn "fetch(\|XMLHttpRequest\|WebSocket\|EventSource\|sendBeacon" src/

# Rust-side network primitives
grep -rn "reqwest\|TcpStream\|UdpSocket" src-tauri/src/
```

Unlike some offline-first apps, AirConvert has no exception to carve out here — there's no HTTP plugin dependency at all (`tauri-plugin-http` is not in `Cargo.toml`), and no feature in the app has any reason to make a network request. Also confirm `src-tauri/tauri.conf.json` has no `updater`/`analytics` config block (Tauri's auto-updater is opt-in and must be explicitly configured — absence of the block means it's off).

Audio conversion shells out to a bundled FFmpeg binary (`tauri-plugin-shell`'s sidecar mechanism) rather than calling a Rust crate directly — this is a real trust boundary worth being explicit about. The `shell:allow-execute` capability in `src-tauri/capabilities/default.json` is scoped to that one named sidecar (`binaries/ffmpeg`) with no other command execution permitted, so confirm that scoping hasn't been loosened. Running a local subprocess is not the same as making a network call — FFmpeg itself makes no outbound connections when simply transcoding a local file — but it's worth re-running the OS-level firewall check below specifically with an audio conversion in the test mix, not just images.

### 2. OS-level runtime block (Windows Firewall)

Build the release binary, then block all outbound traffic for it and confirm every conversion still works:

```powershell
# Build the release binary first: npm run tauri build
$exe = "src-tauri\target\release\airconvert-desktop.exe"
New-NetFirewallRule -DisplayName "AirConvert-Block-Out" -Direction Outbound `
  -Program (Resolve-Path $exe) -Action Block
```

With the rule active, launch the app and convert a batch of images across a few formats (including an SVG input and an AVIF output), plus an audio file through the FFmpeg sidecar — everything should work identically to an unblocked run, since no part of the conversion pipeline touches the network.

Remove the rule when done:

```powershell
Remove-NetFirewallRule -DisplayName "AirConvert-Block-Out"
```

For a stronger guarantee, run the same build inside a network-isolated VM (no virtual NIC, or a host-only adapter with no NAT) instead of relying on a firewall rule.

**Status:** the static audit above has been run against the current codebase with no matches. The OS-level firewall/VM run is a manual step to perform on your own machine before tagging a release as offline-verified — it hasn't been run against a signed release build yet.

## Installation

_(Coming once Phase 1 has a first release build.)_

## Development

```bash
git clone https://github.com/Ayoub-EDAHLOULI/airconvert-desktop.git
cd airconvert-desktop
npm install
npm run tauri dev
```

### FFmpeg sidecar (required for Audio and, later, Video)

Phase 2 (Audio) and the Phase 5 stretch goal (Video) call a bundled FFmpeg binary rather than a pure-Rust crate — FFmpeg isn't checked into the repo (it's large, and licensing means it should be fetched per-machine rather than committed). To build or run those phases locally:

1. Download a static Windows FFmpeg build — e.g. the "release essentials" build from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/), or a release from [BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds/releases).
2. Take `ffmpeg.exe` from the archive and place it at:
   ```
   src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe
   ```
   (the target-triple suffix is Tauri's sidecar naming convention — run `rustc -vV` if you're on a different platform/architecture to get the right suffix).

Without this file in place, the Rust build itself will fail (Tauri validates declared `externalBin` resources exist at build time), not just the audio conversion feature at runtime.

## Contributing

This is an early-stage personal project, but issues and suggestions are welcome. Contribution guidelines will be added once the Phase 1 MVP is stable.

## License

MIT (see [LICENSE](LICENSE))

## Author

**Ayoub Edahlouli** — [GitHub](https://github.com/Ayoub-EDAHLOULI)
