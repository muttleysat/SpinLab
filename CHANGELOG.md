# SpinLab DJ changelog

**English** · [Italiano](CHANGELOG.it.md)

## 0.2.0 — 26 November 2026

Compared with 0.1.1 (31 October 2026, the test release for automatic updates). First version for **macOS** too (Apple Silicon).

### Free and open source
- SpinLab becomes **free software** under the **GNU GPL 3.0 or later** and **free of charge**, with **voluntary donations** via PayPal (button in the About window).
- Removed licence-key registration, the licence server and automatic updates: new versions are downloaded from the GitHub Releases.
- Credits to openDJ, from which the beatgrid fitting and the use of Beat This! are derived.

### Video
- **Videos on both decks**, in time with the music: they load, pause, seek and scratch like a track; beatgrid, BPM, key and stems work on their audio.
- **Output to a window or to an external screen / projector**, with hot plugging and unplugging of the screen.
- **13 transitions** (fade, cut, wipe, zoom, glitch, cube, vortex…) driven by the crossfader or by the channel faders.
- For tracks without video: **card with cover, title and artist** or **10 animated visualizers** in time; title and artist also over the videos.
- **Automatic VIDEO mode**: it turns on by itself when you load a video.
- **Quality adapted to the computer**: graphics, decoding and encoding are measured at startup, Low / Medium / High / Maximum profiles, adjustment during the set.
- **Set recording in MP4** (H.264 + AAC) with the master's picture and audio.
- **Deck with a video**: the preview screen replaces waveforms and platter, with a scrubbable overview and scratching by dragging on the screen.
- **FFmpeg included** in the program (LGPL build only), with licences and credits.

### Beatgrid and analysis
- More accurate beatgrid: fixed the 3:2 tempo error, limited tempo interpolation, new tracking logic (Pulse Engine) with accelerating and slowing tempo.
- **Beats from the Beat This! neural network** and per-track modes **AUTO / FIXED / FLEX** (the last one follows a real drummer bar by bar).
- **⚓ ANCHOR** button on the deck to correct the grid at a point; simplified GRID window (½, ×2, reset).
- BPM and key restored when a track comes back to the library.

### Playing
- **PLAY** starts immediately even with SYNC.
- **→0**: gradual return of the pitch to zero (0.5 points per second).
- **8 hot cues on one page**, unlit while empty and lit in their colour when stored.
- Loops: OUT with the loop active exits it instead of shortening it; new loop lengths.
- **Red flashing countdown in the last 20 seconds**.
- Headphones: the pre-listen delay is reset immediately; the preview reopens by itself if the stream stops.
- Stems on Windows: they start on the CPU straight away, the GPU is used only if it is faster.

### Look
- **New "Dark Club" look**: graphite, aluminium and purple lights; platter with a perforated rim and a light that follows the rotation; metal knobs with a value arc; Pioneer-style faders; CUE and PLAY with a chrome ring.
- **Industrial and Retro Hardware skins** with real materials (perforated and diamond plate, brushed aluminium, leather) and **motorised platters**.
- **Custom theme colour** with a Photoshop-style picker and a live preview of the interface.
- **Layout for ultrawide monitors** (32:9): decks and mixer on the left, full-height library on the right, stems, sampler and FX side by side.
- Stems with knobs instead of faders; bigger EQ knobs.
- LIBRARY view with the selected-track panel; redesigned About window.
- New logo, app icon and splash screen.
- Vertical waveforms (scratch view) and smoother waveforms.
- Performance diagnostics (Ctrl+Shift+D).

### AUTOMIX
- Queue saved between sessions, reordering and adding tracks by dragging; the Start button no longer loses clicks.

### Documentation
- Updated manual (video, beatgrid, skins, ultrawide), technical document for the video module, decisions and licence logs, terms of use and privacy notice for the free software, instructions for testing on Windows.

## 0.1.1 — 31 October 2026

Test release for automatic updates (Windows only).
