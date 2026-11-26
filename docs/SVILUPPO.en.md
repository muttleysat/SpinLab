# SpinLab DJ - developer notes

**English** · [Italiano](SVILUPPO.md)

This file is NOT part of the user manual (it does not end up in the guide inside the program). See also `docs/ASIO-licenza.md` ([English](ASIO-licenza.en.md)) for the obligations related to ASIO.

## Building the program

You need: Git, Rust (rustup.rs) with the Visual Studio C++ tools, LLVM/clang and Steinberg's **ASIO SDK** (not included in the project because of its licence). The SDK must be extracted into `src-tauri/asio-sdk` (the folder is ignored by git and must never be uploaded to the repository) so that it directly contains `common`, `host`, `driver`…: the path in `src-tauri/.cargo/config.toml` is relative, the same on every PC. Use SDK 2.3.4 (or newer) from the Steinberg website. LLVM is found automatically in `C:/Program Files/LLVM`: if it is elsewhere, set the environment variables `LIBCLANG_PATH` and `CLANG_PATH` (not in that file, which also applies on macOS). FFmpeg in the PATH is also needed (less common audio formats).

```
cargo install tauri-cli --version "^2"
git clone https://github.com/muttleysat/SpinLab.git
cd SpinLab\src-tauri
cargo tauri dev        (to try it)
cargo tauri build      (to build the installer)
```

User data (settings, library, MIDI mapping, stem model) lives in `%APPDATA%\com.spinlab.dj` and is not in the repository.

**«Access denied (os error 5)» while building.** The program is still open and Windows does not let it be replaced. Close SpinLab (and any copies left in the Task Manager).

## Building the installers and publishing a version

SpinLab is free software (GPL-3.0-or-later) and free of charge: there is no registration, licence server or automatic updates. Every
version is published as a **GitHub Release** with the installers attached; whoever wants it downloads it and installs it over the previous one.

First time on a computer: `python tools/fetch_model.py` (puts the stem model and the two beat models in `src-tauri/models`, from
where the installer includes them: the installed program downloads nothing) and `python tools/fetch_ffmpeg.py` (LGPL FFmpeg, see `docs/FFMPEG.en.md`).
The MP3 encoder DLL is already in `src-tauri/libs` (it is rebuilt with `tools/build_lame_dll.py`).

For every version:

1. Change the version number in `src-tauri/tauri.conf.json` and in `src-tauri/Cargo.toml`, and add what's new to `CHANGELOG.md` (and `CHANGELOG.it.md`).
2. If the dependencies changed: `python tools/gen_third_party.py` (updates `THIRD-PARTY-LICENSES.md`, which ends up in the installer).
3. Build the installers:
   - **Windows**, from the `src-tauri` folder: `cargo tauri build --config tauri.release.conf.json` → `target/release/bundle/nsis` (and `msi`).
   - **macOS**: `zsh tools/build_mac.sh` → `src-tauri/target/release/bundle/dmg/SpinLab_<version>_aarch64.dmg` (with its SHA-256).
   With `tauri.release.conf.json` the installer includes models, FFmpeg, DLLs and licences. On Windows `tauri.windows.conf.json` adds the
   Visual C++ libraries (`src-tauri/libs/vcredist`) and the offline WebView2: the installer is enough on its own even on a freshly installed Windows.
4. On GitHub: **Releases → Draft a new release**, tag `v<version>`, description (templates in `docs/release/`), attach the installers, **Publish**.

## Icons

The app icons are generated from a single square 1024 x 1024 image (with transparency):

```
cd src-tauri
cargo tauri icon path\icon.png
```

## Updating the guide inside the program

The manual is written in `docs/MANUALE.md` (Italian; the English translation is `docs/MANUALE.en.md`). After every change:

```
python docs/build_guida.py
```

It generates `ui/guida.html`, which the program shows from the **?** button.

## Stems on Android

On the phone stems are separated on the CPU (4 threads). Measurements on a Galaxy S26 Ultra with the normal model: 2.6-3.0 s for a 7.8 s segment, that is 1.95-2.2 times real time. The app accepts real time from 2 times upwards (`REALTIME_MIN_SPEED`), so with the normal model the phone is right on the threshold.

For this reason on Android the app prefers `htdemucs_mobile.onnx`, if present: it is the same model made lighter. It opens in 0.65 s instead of 6.4; separation runs at the same speed. With this model the measurement at app startup gave 2.63 times real time, so real time stays on. It is created from the normal model:

```
pip install onnx onnxslim onnxruntime
python tools/make_mobile_model.py htdemucs.onnx htdemucs_mobile.onnx --verify
```

It does not have a download address yet (`MODEL_URL_MOBILE` in `src-tauri/src/stems.rs`): for now it is copied by hand to a phone with the debug build installed:

```
adb push htdemucs_mobile.onnx /data/local/tmp/
adb shell run-as com.spinlab.dj mkdir -p models
adb shell run-as com.spinlab.dj cp /data/local/tmp/htdemucs_mobile.onnx models/
```

The app's data folder is `/data/user/0/com.spinlab.dj`, the one `run-as` starts in. On restart the app measures the speed again because the model changed.

**Test in the app (S26 Ultra, one deck, 342 s track).** Separation starts on load (AUTO STEM) and stays ahead of the playhead: after 21 s of playback the stems were ready up to 49 s, after 75 s up to 142 s. Removing the vocals at 22 s took effect immediately. In steady state, with the phone warm (38 °C), a segment takes about 3.4 s: 1.7 times real time. Enough for one deck; two decks separating together from scratch cannot keep up (priority goes to the deck being listened to).

To drive the app from the Mac: `adb forward tcp:9333 localabstract:webview_devtools_remote_<pid>` and then Chrome DevTools (`chrome://inspect`) or the CDP protocol. The DevTools screen capture does not show canvases: for screenshots use `adb exec-out screencap -p`.

Tried and discarded on the phone: NNAPI (37 s per segment), fp16 (186 s), dynamic int8 (no gain). To measure a model directly on the phone there is `src-tauri/examples/stem_bench.rs` (instructions at the top of the file).

## Video

The video module (output window, transitions, visualizers, quality profiles) is described in [VIDEO.md](VIDEO.md) (Italian); the requirements and product choices are in [PRD-video.md](PRD-video.md) (Italian). To develop and test it:

- **FFmpeg** is needed (for the video working copies, the decoding and encoding measurements, MP4 recording and the audio of less common formats): in development the one in the `PATH` is enough; in the installer it is included as an **LGPL** sidecar (see [FFMPEG.en.md](FFMPEG.en.md) and `tools/fetch_ffmpeg.py`);
- the code is in `src-tauri/src/video.rs` and in `ui/video*.js`, `ui/video-out.html`;
- in the output window **H** shows live measurements.
