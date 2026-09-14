# AirConvert

A fully offline, local-first file converter for desktop — pick a file, pick a target format, convert, done. No uploads, no network calls, no cloud dependency.

Built because I work on air-gapped/internet-restricted VMs at my day job and needed a Convertio/CloudConvert-style tool that actually works with zero connectivity.

## Why offline matters here

Cloud converters (Convertio, CloudConvert, etc.) upload your file to a server, convert it there, and send it back. That's a non-starter on a locked-down or offline machine — and it's also a privacy concern for sensitive files. AirConvert does every conversion on-disk, on your machine, with no network activity at all. This is verifiable: you can block network access at the OS level and it will keep working exactly the same.

## Status

🚧 Early development. Currently scaffolding **Phase 1: Images**. See [Roadmap](#roadmap) below.

## Tech stack

- **Shell/runtime:** [Tauri](https://tauri.app/) (Rust)
- **Frontend:** React + TypeScript
- **Conversion engines:** pure-Rust crates where possible (see [Engine choices](#engine-choices-and-trade-offs)), external sidecar binaries only where necessary

## Roadmap

Phased by format category, one shipped and working before the next starts:

| Phase       | Category     | Formats                                                                     | Approach                                                                  |
| ----------- | ------------ | --------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| 1 (current) | Images       | jpg, png, webp, gif, bmp, tiff, svg, ico, tga, pnm, qoi, avif (output only) | Pure Rust (`image`, `resvg`) — no external binaries                       |
| 2           | Audio        | mp3, wav, flac, ogg, m4a                                                    | Bundled FFmpeg (LGPL build) sidecar                                       |
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
| FFmpeg (LGPL build)            | Audio, Video | Industry-standard, well understood bundling pattern (same approach used by apps like HandBrake)                       |
| Pandoc                         | Documents    | Single-binary sidecar, handles markup-style formats well; explicitly not a fidelity-preserving office-document engine |
| `calamine` / `rust_xlsxwriter` | Spreadsheets | Pure Rust, handles tabular data without pulling in a full spreadsheet engine                                          |

## Features

- Drag-and-drop file input with format auto-detection
- Target-format picker
- Batch conversion (multiple files at once)
- Zero network calls, verifiable by blocking network access at the OS level
- Portable, installable desktop app (Windows-first, cross-platform as feasible)

## Planned (not yet built)

- CLI mode for scripting conversions on headless/VM environments
- Basic image resize/compress options alongside conversion

## Installation

_(Coming once Phase 1 has a first release build.)_

## Development

```bash
git clone https://github.com/Ayoub-EDAHLOULI/airconvert-desktop.git
cd airconvert-desktop
npm install
npm run tauri dev
```

## Contributing

This is an early-stage personal project, but issues and suggestions are welcome. Contribution guidelines will be added once the Phase 1 MVP is stable.

## License

MIT (see [LICENSE](LICENSE))

## Author

**Ayoub Edahlouli** — [GitHub](https://github.com/Ayoub-EDAHLOULI)
