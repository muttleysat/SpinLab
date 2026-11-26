# FFmpeg in SpinLab — licence, inclusion and updates

**English** · [Italiano](FFMPEG.md)

For developers and for whoever looks after the legal side. It summarises how FFmpeg is included in the program, which licence applies and what must be done to comply with it. It is not legal advice: the points to be checked are marked (decision D-10 in [DECISIONI.md](DECISIONI.md), Italian).

## In short

- FFmpeg is **included in the installer** as a separate program (Tauri **sidecar**): `ffmpeg.exe` on Windows, `ffmpeg` on macOS, next to the SpinLab executable.
- **Only an LGPL build** is distributed, that is **without** x264, x265, fdk-aac and other GPL or non-free components. Mind the version: the **macOS** build (compiled by us) is **LGPL 2.1 or later**; the **Windows** build (BtbN) is compiled with `--enable-version3` and is therefore **LGPL 3**.
- No licence has to be requested from anyone: the LGPL allows it to be included, under conditions that are met (table below).
- **Codec patents** (H.264, H.265, AAC) are a different matter from the software licence and remain to be checked with legal advice (D-10).

## What it is used for

| Use | Where |
|-----|------|
| Working copies of videos (scratch and jumps) | `video_proxy` |
| Decoding and encoding measurements | `video_bench_clip`, `video_bench_decode` |
| Encoding of the MP4 recording and joining it with the audio | `video_rec_*` |
| Audio of rare formats (and of videos without native decoding) | `decode.rs` |

How the program finds it (`decode::ffmpeg_path`): first `ffmpeg.exe` (Windows) or `ffmpeg` (elsewhere) **next to the executable**, otherwise the `PATH`. In development, where there is no sidecar, the one in the `PATH` is used.

## Which build and why

| Platform | Origin | Notes |
|-------------|---------|------|
| Windows 64-bit | BtbN LGPL build (`ffmpeg-n9.0-latest-win64-lgpl-9.0.zip`, version `n9.0.2-47-…`), with SHA-256 checked against the published list | Single executable of about 126 MB; **LGPL 3** (`--enable-version3`); contains the `h264_nvenc`, `h264_qsv`, `h264_amf`, `h264_mf` encoders (to be tested on a PC) |
| macOS Apple Silicon | **Compiled from the official sources** of ffmpeg.org with `--disable-gpl --disable-nonfree --disable-autodetect` | Native `h264_videotoolbox`, `aac` and `mjpeg` encoders; no library picked up from the system by chance |
| macOS Intel | Same procedure (`macos-x64`), needs `nasm` | Not prepared |

Why LGPL: light obligations (below). The GPL build (x264) was discarded; the common «full» builds (for example Gyan) are GPL and must **not** be included. A build with `--enable-nonfree` (for example with fdk-aac) is **not redistributable**.

## LGPL obligations and where they are met

| Obligation | How we meet it |
|---------|---------------------|
| Include the licence text | `licenses/FFMPEG-LICENSE.txt` (the one of the build: LGPL 2.1 on macOS, LGPL 3 on Windows) and all the texts in `licenses/ffmpeg/`, copied from the official sources by the script (not written by hand) |
| Notice that FFmpeg is used | The program's **About** tab, `THIRD-PARTY-LICENSES.md`, terms of use §5 (`docs/legale/EULA.md`) |
| Provide the sources of the distributed version (or a written offer) | `licenses/FFMPEG-BUILD.txt` (version, origin, checksums, configuration); the sources are at ffmpeg.org/releases. **Address to request the sources on another medium: `[TO BE DEFINED]`** |
| Allow the library to be replaced | FFmpeg is a separate file next to the program: it can be replaced with a compatible version |
| (LGPL 3) Also include the GPL 3 text, which the LGPL 3 refers to | `licenses/ffmpeg/COPYING.GPLv3`, taken from the official sources |
| Do not forbid modification and reverse engineering for the LGPL part | SpinLab is free software (GPL-3.0): modification and study are allowed for the whole program |
| Do not use the name as an endorsement | The About tab does not present SpinLab as endorsed by the FFmpeg project |

## Preparing and updating FFmpeg for the installer

```
python3 tools/fetch_ffmpeg.py windows        # on any computer: downloads and verifies the LGPL build
python3 tools/fetch_ffmpeg.py macos-arm64    # on a Mac: compiles from the official sources (a few minutes)
```

The result is in `src-tauri/binaries/` (ignored by git, the files are large): `ffmpeg-<triple>[.exe]`, `licenses/FFMPEG-LICENSE.txt`, `licenses/FFMPEG-BUILD.txt`. Building the installer with `src-tauri/tauri.release.conf.json` includes them (`externalBin` and `resources`). **Development builds do not need the sidecar.**

To update the version: change `SRC_VERSION` (and `SRC_SHA256`, which the script prints the first time) or the Windows URL in `tools/fetch_ffmpeg.py`, run the script again, then **repeat the checks** below.

### Checks before every release

1. `ffmpeg -hide_banner -buildconf` **must not** contain `--enable-gpl`, `--enable-nonfree`, `libx264`, `libx265`, `fdk`. `--enable-version3` makes the build LGPL 3 (acceptable, but it changes the texts and conditions: see above). The macOS build script checks everything by itself; for Windows the script detects `--enable-version3` and writes it in `FFMPEG-BUILD.txt`.
2. `ffmpeg -hide_banner -encoders` must show at least one system H.264 encoder (`h264_videotoolbox` on Mac; `h264_nvenc`, `h264_qsv`, `h264_amf` or `h264_mf` on Windows), `aac` and `mjpeg`.
3. The test `cargo test --lib video::` (MP4 recording with real FFmpeg) must pass **with that sidecar** (put it first in the `PATH`).
4. The installer contains `licenses/FFMPEG-LICENSE.txt` and `licenses/FFMPEG-BUILD.txt`.

## Things not to do

- Do not include GPL builds (x264/x265) or `nonfree` ones.
- Do not copy the licence text by hand: take it from the official distribution.
- Do not take FFmpeg out of the separate file (linking it into the executable would change the obligations).
- Do not promise specific formats or codecs in communications before the opinion on patents.

## Open points

| Point | Status |
|-------|-------|
| H.264, H.265, AAC patents for a distributed product | Legal advice (D-10); system encoders reduce the risk, they do not remove it. **The LGPL 3 (Windows build) refers to the GPL 3 patent conditions**: ask the lawyer whether they interact with the consortia licences, or compile our own LGPL 2.1 build for Windows |
| Testing the Windows build (available encoders, start from `Program Files`, file signature) | To be done on a Windows PC (D-13) |
| Installer size (the Windows LGPL build is a large executable) | To be measured; it can be reduced with a more targeted build |
| If the user has their own FFmpeg in the `PATH` | The program uses the bundled one first; a user's FFmpeg (even GPL) remains their own choice and responsibility |
| Address for the written offer of the sources | `[TO BE DEFINED]` |
