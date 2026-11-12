# SpinLab FluidGrid Lab 0.1

Standalone, local-first browser prototype for Phase 1.

## Run
Open `index.html` in Chrome or Edge. No installation, server, account, network request, or upload is required.

## Included
- Load local audio and render waveform via Web Audio API.
- Drag and drop audio files onto the main window or waveform.
- Animated playhead using requestAnimationFrame while audio is playing.
- Waveform zoom (1x–32x), fit-to-track, and horizontal navigation.
- Import `.beats`, `.txt`, and `.csv` (first column seconds; optional second column beat-in-bar).
- Import JSON with `beats`, `times`, or `beat_activation` + `fps`.
- Select, drag, add, and delete beat markers.
- Forward-only Anchor: selected marker is aligned to the playhead and the same delta is applied to subsequent markers only.
- Forward propagation using the mean of up to four preceding beat intervals.
- Undo/redo and CSV/JSON export.

## Limitations
- Standalone editor; SpinLab Rust DSP is not yet wired in.
- Beat This activations must be converted to JSON; original NumPy `.npz` is not parsed.
- Propagation currently assumes a constant local period. Variable-tempo curve propagation is planned.
- No Engine DJ / Engine OS database writes.
