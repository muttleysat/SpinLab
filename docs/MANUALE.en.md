# SpinLab DJ — User manual

**English** · [Italiano](MANUALE.md)

![ASIO Compatible - ASIO is a registered trademark of Steinberg Media Technologies GmbH](../ui/assets/asio-logo-bw.png)

SpinLab DJ is DJ software with two decks, mixer, effects, stem separation, drum machine, sampler, video output for screens and projectors, master recording and support for any MIDI controller. This manual explains how to use it, how to connect a controller and how to solve the most common problems.

> The program's interface is in Italian. Buttons and commands are written here as they appear on screen; where the label is Italian, the English meaning follows in brackets the first time. Hovering the mouse over almost any button shows a tooltip.

## First launch

At startup the SpinLab screen appears with a progress bar: it disappears by itself when the audio engine and the library are ready.

1. **Choose the sound card.** Open **SETTINGS** (or the gear at the top right) → **AUDIO** tab. If you have a card with an ASIO driver (for example Focusrite) choose the **ASIO®** driver: latency is much lower and the drum pads respond immediately. Otherwise leave **Windows (WASAPI)**. Press **Applica** (Apply).
2. **Choose the headphones.** In the same panel, «Uscita cuffie» (headphone output). See [Audio settings](#audio-settings).
3. **Add your music.** In the **ESPLORA** (browse) column on the left choose a folder: the tracks appear in the table. They are analysed (BPM, key, beatgrid) in the background.
4. **Load a track.** Select a track and press **LOAD** in its row, or drag it onto a deck.

---

## The screen

- **Top bar.** Navigation (**LIBRARY**, **MIXER**, **RECORD**, **SETTINGS**), **REC** indicator while recording, processor load (**CPU**), clock, headphone controls (buttons 1 and 2, **CUE MIX**, **PHONES**) and **MASTER** volume. To the right of the volume there are the **i** (about SpinLab: licence, version, credits and the **Dona con PayPal** — Donate with PayPal — button) and the **?** (this guide).
- **Centre.** The two decks with their waveforms and, in between, the two-channel mixer with the crossfader.
- **Bottom left.** Folder browser and playlists.
- **Bottom centre.** The library: search, track table, **TAG & INFO** panel.
- **Bottom right.** The panel with **AUTOMIX**, **SAMPLER**, **DRUMS**, **STEMS** and, below, the **FX**.

The graphics adapt to the window: if you make it much smaller, everything shrinks together. On an **ultrawide monitor** (2.4:1 and wider, for example 32:9) the layout changes by itself: see [Look: skins, colours and ultrawide monitors](#look-skins-colours-and-ultrawide-monitors).

**Shortcuts**

| Key | What it does |
|---|---|
| Space bar | Play / pause of the deck under the mouse (otherwise the last one used) |
| Up / down arrow | Browse the library tracks |
| Enter | Load the selected track on the deck in use |
| Delete | Remove the selected tracks from the open playlist |
| Z X C V · A S D F · Q W E R · 1 2 3 4 | Drum pads (with the DRUMS tab open) |
| Shift + click on a drum pad | Softer hit |

---

## Library and playlists

- **Folder browser.** Choose a folder: SpinLab reads its tracks and analyses them. The `stems` subfolder (see [Stems](#stems)) is ignored.
- **Search.** The field at the top searches by title, artist, tag, key and BPM. Columns are sorted with a click on the header.
- **Headphone preview.** The triangle button next to a track plays it in the headphones without going through the master. Shift + click starts from the middle of the track. Click or drag on the row to move through it.
- **Playlists.** **+** next to PLAYLIST creates one. Drag tracks onto a playlist to add them. Right click on a playlist: rename, delete, **Crea stem di tutta la playlist** (create stems for the whole playlist). Adding a track to a playlist starts its full analysis.
- **TAG & INFO.** The button with the «i» next to the search opens the panel for title, artist, album, genre, tags, star rating and comment.
- **Double click on a track** loads it on the first free deck.
- **Column widths.** Drag the left edge of the track list to widen or narrow the folders and playlists column, and the right edge for the right panel (AUTOMIX, SAMPLER, DRUMS, STEMS). The list takes the space that is left, without going below a minimum width. The sizes are remembered; a double click on the edge goes back to the default ones.
- **LIBRARY view.** With **LIBRARY** in the top bar the library takes more space. In the right panel the sections (stems, sampler, drums, automix) keep their size and below them appears the **selected track panel**: cover, title, artist, BPM, key, duration, album, genre, tags, rating, whether it is analysed, format, comment and file path. With several tracks selected it shows how many.
- **Right click on a track.** Load on deck 1 or 2, add to a playlist (existing, or a new one with the selection), analyse (BPM, key, grid), create or delete the prepared stems, remove from the playlist (if one is open).

---

## The decks

Each deck shows two waveforms: the large one (zoomed, scrolling) and the small one (overview of the whole track), the platter, the buttons and the pitch fader.

- **The countdown** at the top right (the time left until the end) **flashes red in the last 20 seconds** of the track, to warn you that it is about to end. With «Reduce motion» turned on in the system it stays steady red.
- **The platter** lights up according to the deck state: dim without a track, still when paused, with the light running along with the record while it plays, brighter while scratching and amber when you hold CUE while the track plays. In the Dark Club skin the tone arm puts the needle on the record when you load a track and moves towards the centre as the track goes on.

### Transport

- **PLAY** and **CUE**. Holding CUE plays the track from the cue point until you release it.
- **LOAD** loads the track selected in the library; the button next to it empties the deck.
- **The pitch** is set with the fader on the right. **±8** (or the value shown) changes the range: 4, 8, 16, 50, 100%. Double click on the fader brings it back to zero at once.
- **→0** (below the pitch value) brings it back to zero **gradually**, without audible jumps: the pitch moves towards 0 at a constant speed of 0.5 percentage points per second (from +8% it takes 16 seconds, from +2% four). The button lights up while the return is in progress and a second click stops it where it is. Moving the fader by hand, loading a track or turning on SYNC interrupts it; if the deck was following SYNC, SYNC turns off, as when you move the pitch by hand. The button is unlit when the pitch is already at 0.
- **SYNC** aligns tempo and phase to the leading deck. It stays on even after loading another track.

### Scrub

Hold the mouse on the large waveform and drag: the track follows like a record (dragging to the left moves forward). The platter can also be turned with the mouse. On the small waveform a click moves the playhead to that point.

### Beatgrid and key

The beatgrid is computed by the analysis and, when possible, refined by a neural network (Beat This!) that recognises the beats even in live-played tracks.

- **⚓ (anchor)**, on the deck above **GRID**: moves the whole grid so that the nearest beat falls exactly on the playhead. Stop on a kick drum and press it when the lines sit a little before or after the beats.
- **GRID** opens the grid window, saved for that track:
  - **½** halves the BPM (one marker every two), **×2** doubles it, **↺** returns to the grid from the analysis (it undoes ⚓, ½ and ×2).
  - **AUTO** uses a single tempo until the beats drift away from it for a long time, then starts a new section: fine for almost everything. **FISSA** (fixed) uses a constant tempo and phase for the whole track (electronic music, tracks made on a computer). **FLESS.** (flexible) follows a real drummer bar by bar (rock, funk, 70s disco, live recordings).
  - **QUANTIZE** like the QUANT button.
- **QUANT** snaps cues, loops, rolls and hot cues to the beats of the grid.
- **KEY** opens the key panel: **KEY LOCK** keeps the key when you change the pitch, Camelot step ±1 and semitone corrections.

### The pads

The button above the pads changes their function:

| Mode | What the pads do |
|---|---|
| **HOT CUE 1–8** | Eight pads in two rows. A click stores the point or jumps to it. Shift + click, or right click, deletes it. An empty hot cue is unlit; a stored one lights up in its colour and flashes briefly when you recall it. |
| **BEAT JUMP** | Jumps by −2, −1, +1, +2 times the size chosen with ‹ ›. |
| **LOOP ROLL** | Repeats a section while you hold it. The pads play 1×, 2×, 4× and 8× the base length (from 1/16 to 1 beat, with ‹ ›). |
| **FRENO · SPIN · REV · SLIP** | **BRAKE**: the record stops while you hold it. **SPIN**: spinback. **REV**: reverse while held. **SLIP**: on/off. |

In the modes other than hot cues the first four pads are used.

**SLIP**: during roll, brake and reverse the track keeps going «silently» and on release you find yourself where it would have got to. **Q** turns quantize on.

### Loops

- **‹ ›** halves and doubles the loop, the number in the middle turns it on or off.
- **IN** and **OUT** set start and end; OUT activates it.

---

## The mixer, headphones and pre-listen

Each channel has a fader, 3-band EQ (**HI**, **MID**, **LOW**), **FILTER** and a post-fader level meter. Knobs are turned by dragging up and down (with Shift the adjustment is fine) or with the mouse wheel; double click returns them to their initial value. The coloured arc around the knob shows the value. The **crossfader** is at the bottom; double click centres it.

**AUTO GAIN** brings a newly loaded track to the same loudness as the one playing. **AUTO STEM** separates in advance the stems of every loaded track.

**Headphones.** Buttons **1** and **2** at the top turn on the pre-listen (PFL) of the channels. **CUE MIX** balances between the two sides, **PHONES** is the volume. PFL can also be turned on from the headphone icon on the platter.

---

## Effects

The **FX** panel works on the deck chosen with the **DECK** buttons (1 / 2) above the pads; it stays aligned with the channel strip.

- **Signal effects:** ECHO, PING-PONG, REVERB, FLANGER, PHASER, BITCRUSH, GATER, SCATTER, NOISE.
- **Track effects:** BEAT JUMP, BRAKE, SPINBACK, REVERSE. They act on the record, not on the signal.
- **TIME** and **FEEL** (straight, triplet, dotted) set the step of tempo-synced effects.
- **The three knobs** change name depending on the effect. For example in the echo they are **TONE**, **FEEDBACK** and **MIX**.
- The button next to the effect name turns it on. For brake and reverse, holding it for more than 0.35 seconds makes the effect last as long as you press.

---

## Stems

Stems split a track into **vocals**, **drums**, **bass** and **other**. In the **STEMS** panel each stem has a volume knob (the segmented ring lights up in proportion), a mute button (**M**) and a solo button (**S**). A click on the stem's icon or name excludes or re-enables it. The **ACAPELLA**, **INSTRUMENTAL** and **ALL** presets set everything with one click.

SpinLab works in two ways:

- **Real time.** Stems are separated while the track plays, using the processor. Nothing needs to be prepared.
- **Prepared.** Stems are computed beforehand and saved. Loading is practically instant and quality is maximum. They are the choice for slower PCs.

In **Impostazioni → STEM** (Settings → STEM) choose the mode: **Automatico** (automatic, based on the measured speed of the PC), **Sempre in tempo reale** (always real time) or **Solo stem preparati** (prepared stems only). The **Misura la velocità di questo PC** (measure this PC's speed) button repeats the measurement. Real time needs at least 2× playback speed, because two decks separate at the same time.

**Preparing stems.** Right click on a track (or several tracks) or on a playlist → **Crea stem** (create stems). They are saved in a `stems` folder next to the tracks, losslessly compressed (about 45% of the original size). **Elimina gli stem preparati** (delete prepared stems) deletes them.

There is a single separation model. In «Solo stem preparati», a track without stems shows a notice inviting you to prepare them.

---

## AUTOMIX

AUTOMIX plays the library tracks on its own, one after the other, with beat-matched transitions as a DJ would. It is in the **AUTOMIX** tab of the bottom-right panel.

**How to use it**
1. Choose which tracks to play: open a playlist or a folder, or run a search. AUTOMIX uses the tracks you see in the table, starting from the selected one.
2. Choose **Ordine** (order), **Transizione** (transition) and **Stile** (style) (see below).
3. Press **AVVIA AUTOMIX** (start). If a deck is already playing, that track continues and the following ones start at its end.
4. **FERMA AUTOMIX** (stop) stops it: the decks stay as they are. **Salta** (skip) moves immediately to the next track with a transition.

**The settings**
- **Ordine** (order). *Come in elenco* (list order, as in the table), *BPM crescente* or *BPM decrescente* (ascending or descending BPM; tracks without BPM go last), *Intelligente* (smart: after each track it picks the one with the closest key, Camelot code, and tempo, for more harmonic mixes) or *Casuale* (random).
- **The queue can be edited.** Below the settings there is the list of the next tracks. On each row: ⤒ puts it first, ▲ and ▼ move it, ✕ removes it. When stopped, *ripristina* (reset) undoes the changes; changing the order rebuilds the queue from scratch.
- **Transizione** (transition). How many bars the mix lasts: 4, 8, 16 or 32. At 120 BPM, 16 bars are about 32 seconds.
- **Stile** (style). *Scambio dei bassi* (bass swap): the new track comes in without bass and halfway through the transition the bass moves from one track to the other, so two basslines never add up. *Dissolvenza semplice* (simple fade): crossfader only. *Taglio rapido* (quick cut): two bars, without tempo matching.
- **Ripeti** (repeat). When the queue ends, it starts again.

**What the program does**
- About 75 seconds before the end of the playing track, it loads the next one on the other deck.
- It matches tempo and beats with **SYNC**. If the two tracks have tempos that are too different (more than 12%, after accounting for doubling and halving) it does not sync them and makes a quick transition.
- When the transition is over it empties the old deck, and the new track gently returns to its original tempo.

**Good to know**
- Tracks must have been analysed (tempo and key). Without analysis the transition is a simple fade without sync.
- If you move the crossfader or the bass by hand during a mix, AUTOMIX puts them back at the next step.
- From a MIDI controller you can map **AUTOMIX avvia / ferma** (start / stop) and **AUTOMIX: salta** (skip).

---

## Drums and sampler

### Drums (DRUMS)

16 pads with single samples, to play with the mouse, the keyboard (Z X C V · A S D F · Q W E R · 1 2 3 4) or a controller. The drum kit starts empty at every launch; only the pads' MIDI notes are remembered.

- **Load a sample:** click on an empty pad, or drag a file onto the pad. Maximum 30 seconds. You can drag **several samples at once** (for example a whole kit selected in Finder): the first goes to the pad where you drop them, the others to the empty pads that follow. A file dropped between the pads goes to the first empty pad; whatever you drop in the panel never ends up in the playlist.
- **The pad's sound.** Below the pads is the strip of the **chosen pad** (the last one you clicked or played from the keyboard, with a lit border). The knobs are described in the table below.
- **Right click on a pad:** **◀ Reverse** (plays the sample backwards; the pad shows ◀ before its name), MIDI learn, **gate** mode (plays while you hold it), **aftertouch**, **choke group** (for example closed and open hi-hat silencing each other), MIDI note, clear.
- **Aftertouch.** Pressing the pad after the hit makes the sound grow. The target is per pad: off, volume (up to +60%), pitch (up to +7 semitones) or both. It works with per-pad and channel aftertouch. A coloured bar on the pad shows the pressure.
- **Default MIDI notes:** the 16 pads respond to notes 36–51.

| Knob | Values | What it does |
|---|---|---|
| **ATT** (attack) | 1 ms to 2 s | how long the sound takes to rise |
| **DEC** (decay) | none to 4 s | how long it takes to fall to the sustain level |
| **SUS** (sustain) | 0–100% | the level after the decay |
| **REL** (release) | 4 ms to 4 s | how long it takes to fade when you let go of the pad; you hear it in **gate** mode (in normal mode the hit plays to the end of the sample) |
| **VOL** | 0–150% | pad volume |
| **TUNE** | ±12 semitones | tuning, in half steps |

Each knob shows its value below; a double click brings it back to the default. With the default values the hit sounds as it is.

### Sampler (SAMPLER)

Pads for longer sounds and loops. **Each deck has its own bank of 8 pads**: the **DECK 1 / 2** selector at the top of the panel chooses which bank you see and load. Pads that are already playing keep playing when you switch to the other bank; a lit dot on button 1 or 2 shows the bank that is playing.

- **Load:** drag a file (or a library track) onto a pad, or click on an empty pad. Several files dragged together fill the empty pads that follow.
- **Play:** click to start, another click to stop. Keys **1–8** on the keyboard (bank shown).
- **Right click:** loop on or off, stop, sample BPM (typed in, ÷2, ×2), replace.
- **SYNC.** The pads of a bank follow **their deck**: they start on the next beat, stretch or shrink to its tempo (without changing key) and stay locked to the **bar**: if you launch a loop on beat 3 of the track, it comes in on its own beat 3, so kick and snare land in the right place. They also follow a **flexible** grid and line up again straight after a jump in the track (hot cue, beat jump). If the bank's deck is stopped they follow the playing deck. The same loop loaded in deck 1's bank and in deck 2's bank stays in time with two tracks at different BPM.
- **Sample BPM.** It is read from the file name when there is one (for example `electro-drums_132bpm.wav`), otherwise it is computed; a sample lasting a whole number of beats is a loop and starts from its beginning. The loop follows the deck's grid: if the track's grid doesn't sit on the beats, fix it first (see [Beatgrid and key](#beatgrid-and-key)).

---

## Video

SpinLab can show a picture on an external screen or projector while you play: the **videos** you load on the decks or, for tracks without video, the **cover with title and artist** or an **animated visualizer** in time with the music. The two decks are mixed with the crossfader, with a transition of your choice.

Video does not change the way you play: the audio is always in charge. A video loads, pauses, seeks and scratches like a track, and the beatgrid, BPM, key and stems work on its audio as for any other file.

### Turning on VIDEO mode

The mode turns on **by itself** when you load a video on a deck and turns off when only audio tracks are left on the decks. With the mode off the program uses no resources for video and opens no window.

The **VIDEO** button, at the top near the clock, forces it by hand: each click moves to the next choice among **automatic**, **always on** (useful to see the visualizers with audio tracks only) and **always off**. When it is on the button lights up and next to it appear the transition and visualizer menus and the quality in use.

### The deck with a video

When a deck has a video, it changes appearance by itself: instead of the waveforms and the platter it shows the **video preview screen**, muted and in time with the music. The **track overview** is in the strip above the pads, next to the pitch button, and works as usual (click or drag to move). Dragging horizontally on the screen **scratches** (the full width is 4 seconds). A deck with an audio track stays classic; when you remove the video, the deck goes back to how it was.

### Supported formats

The library recognises **mp4, m4v, mov, mkv, webm, avi, ts, mpg, mpeg**. Videos downloaded from YouTube and converted to **mp4 (H.264)** are the simplest case and work out of the box.

- The **picture** is played by the program's web engine: formats that the system (Windows or macOS) cannot read are not shown. H.264 is always the safest; for other codecs (H.265, VP9, AV1…) it depends on the computer.
- The **audio** is decoded like a track's. Less common formats need FFmpeg, as for tracks: see **Troubleshooting**.
- The output is limited to 1080p: 4K videos, if the system can read them, are shown at a lower resolution, and HDR is not handled.

### Output to a screen or projector

The output window opens by itself with VIDEO mode. To send it to an external monitor, a projector or a TV (HDMI, DisplayPort, USB-C), **right click the VIDEO button** and choose the screen from the list; **Finestra normale** (normal window) keeps it in a window on the program's monitor. The choice is remembered.

- If the chosen screen is **unplugged** while you play, the output moves to a normal window and a notice appears: music and video do not stop. When you **plug it back in** it returns to full screen.
- In the output window, **H** shows or hides the measurements (dropped frames, offset from the audio, quality in use) and **Esc** closes it.

### Transitions

The drop-down menu next to the VIDEO button chooses the effect with which the mix goes from one deck to the other. By default the mix follows the **crossfader**; if you mix with the channel volume faders and leave the crossfader still, you can choose **i fader dei canali** (the channel faders) in **Impostazioni → VIDEO → Miscela** (Settings → VIDEO → Mix), or from the right click on the VIDEO button: in that case the crossfader is ignored and the video moves from one deck to the other according to the volume of the two channels (if one is at zero only the other is shown; with equal volumes, the mix is halfway; with both at zero the picture stays as it was). The effect follows the position of the mix: on the left you see deck 1, on the right deck 2, in the middle the transition. It can be changed during the set too.

The transitions are: **Dissolvenza** (fade), **Taglio** (cut), **Dissolvenza al nero** (fade to black), **Flash bianco** (white flash), **Wipe**, **Zoom a spinta** (push zoom), **Glitch**, **Pixel**, **Caleidoscopio** (kaleidoscope), **Ondulazione** (ripple), **Cubo 3D** (3D cube), **Vortice** (vortex) and **Stroboscopio sul ritmo** (beat strobe). The last one alternates the two decks in time with the **beatgrid**, so it works best with tracks whose grid is right.

If only one deck has something to show, that one is shown without effects. With the **Bassa** (low) quality (see below) only the first five, the lightest, are available; the others appear disabled.

### Tracks without video: card and visualizers

A deck with a track without video still takes part in the mix. The second menu next to the VIDEO button chooses what to show:

- **Scheda del brano** (track card): the cover (if the file has one) with title and artist, on a background that picks up its colours.
- **Ten visualizers**: Anelli (rings), Tunnel, Onde luminose (light waves), Griglia al neon (neon grid), Stelle (stars), Plasma, Barre (bars), Esagoni (hexagons), Fluido (fluid) and Raggi (rays). They pulse on the grid's **beat**, react to the deck's **level** and take their colour from the track's **key**.

The visualizers are abstract animations made for SpinLab. They can be used without any video, with VIDEO mode turned on by hand, to fill a screen while you play music only.

### Title and artist

Over the videos and visualizers, at the bottom left, appear the **title** and **artist** of the track on the dominant side of the crossfader, with a soft fade-in when the track changes. On visualizers a light dark shadow at the bottom keeps the text readable. The track card already contains title and artist.

### Scratching, jumps and the working copy

Common videos have a complete picture only every few seconds: to show an arbitrary point the program has to rebuild it, and this slows scratching and jumps. For this reason, when you load a video, SpinLab prepares in the background a light **working copy** (480p) in which every frame is complete:

- The video starts immediately from the original; when the copy is ready it replaces it without you doing anything.
- The copy is in a temporary folder and is **deleted by itself** when the video leaves the deck. At startup the program removes those left over from an abnormal shutdown. The original is never modified.
- FFmpeg is needed. A copy takes about 85 MB for a four-minute video and is prepared in a few seconds.

### Stems on videos

Stems of videos are created as for tracks: right click on the video in the library or in a playlist → **Crea stem**. They are computed on the audio and stay saved for future use. For the modes and preparation see **Stems**.

### Quality and performance

In **Impostazioni → VIDEO** (Settings → VIDEO) choose the output quality:

| Profile | Resolution and frames | Transitions |
|---|---|---|
| **Bassa** (low) | up to 480p, 24 fps | only the five lightest |
| **Media** (medium) | up to 720p, 30 fps | all |
| **Alta** (high) | up to 1080p, 30 fps | all |
| **Massima** (maximum) | up to 1080p, 60 fps | all |

- **Automatico** (automatic, recommended). A few seconds after startup, with the decks stopped, the program analyses the computer (processor, memory, graphics card, speed of graphics, video decoding and encoding) and chooses a starting profile. The measurement is repeated by itself only if the hardware changes or if the first one failed.
- **Ottimizza con un video in riproduzione** (optimise with a playing video). Start a video on a deck, press the button and for about 10 seconds the program tries the highest quality and steps down until the output is smooth; the result becomes the automatic profile.
- **During the set.** If the computer struggles, the output drops by one level by itself and a notice appears; if the load goes down for a while, it goes back up. The audio is never touched.
- **Manual choice.** A profile chosen from the menu stays in force until you go back to **Automatico**: while it is forced, the measurement does not change it.

To know which quality you are at, look at the label next to the VIDEO button (for example «1080p 60», orange if the output has been reduced because of the load) or at the **Uscita in questo momento** (output right now) line in the VIDEO tab. The tab also shows the data of the computer analysis.

The profile limits the resolution and frame rate of the **full-screen** output: in a small window, or with a 25 fps video, different profiles look the same.

### Recording the set in MP4

Besides the audio, SpinLab can record **the picture of the video output** (with transitions, visualizers, title and artist) and save it together with the master sound in a single **MP4** file (H.264 video, AAC audio).

1. **RECORD** → in the **Formato** (format) menu choose **MP4 con video (H.264 + AAC)**.
2. **Qualità del video** (video quality): *Come il profilo video in uso* (same as the video profile in use, recommended) or 480p 24 fps, 720p 30 fps, 1080p 30 fps, 1080p 60 fps. The recording quality is independent of the output quality: you can project at one resolution and record at another. The recorded video has the **same shape as the picture on the output** (16:9, 16:10…), without added black bars: if the original video is 4:3 you will still see the bands at the sides, which are part of the picture. 1080p at 60 fps needs a powerful computer.
3. Choose where to save the file and press **Avvia registrazione** (start recording). The video output turns on by itself for the whole duration (even if there are only audio tracks on the decks: you will see the chosen card or visualizer).
4. **Stop** with a click on the **REC** indicator or on **STOP REC**. The file has to be finalised (audio and video are joined): it can take a few seconds, and a notice appears with the path and size.

Good to know:

- **FFmpeg** is needed (as for the working copies); if it is missing, recording does not start and a message explains it.
- Recording captures what is shown on the output, so it needs **WebGL drawing** on (the simple mode without WebGL cannot be recorded).
- If the computer cannot keep up, some frames are lost (the video looks a bit choppy) but **the audio is not interrupted**; at the end of the recording a notice shows the number of lost frames.
- If something goes wrong after the recording (for example the join fails), the audio stays saved in a temporary file and the message says where.
- The intermediate files are deleted by themselves when the final file is ready.

### Not there yet

- Video on more than two decks and a video fader separate from the crossfader.

Music videos and films are protected by copyright: what you project, record or publish is your responsibility.

### If something goes wrong with video

**The screen stays black or an error appears.** The format is not read by the system. Press **H** in the output window: if you see an error on the deck, convert the video to mp4 (H.264).

**The video is choppy or keeps realigning.** Wait a few seconds after loading: the working copy has to finish (in **H** it shows «copia di lavoro: sì», working copy: yes). If that is not enough, lower the quality in **Impostazioni → VIDEO** or press **Ottimizza** (optimise).

**The video is not in time with the music.** A fixed difference between audio and picture is adjusted with the **Allineamento della grafica** (graphics alignment) slider (**Impostazioni → AUDIO**), the same as for the waveforms.

**I can't see the external screen in the list.** Connect it before opening the menu and check that the system recognises it. Two identical monitors (same name and same resolution) cannot be told apart: SpinLab picks the first one.

**Transitions are disabled or there is only one.** With the **Bassa** profile only the lightest remain. Choose another profile in the Settings.

**MP4 recording does not start or the video is choppy.** Check that FFmpeg is available and that the video output shows the picture (press **H** in the output window: «disegno: WebGL», drawing: WebGL). With a slow computer choose a lower quality in the recording dialog.

**The panel shows zero measurements or «non misurata» (not measured).** The first analysis did not succeed (usually FFmpeg is missing or the graphics card was not read). Restart the program: it repeats by itself; or choose a profile by hand.

---

## Look: skins, colours and ultrawide monitors

Everything is chosen in **Impostazioni → INTERFACCIA** (Settings → INTERFACE) and applied immediately; the choice is remembered.

### Skins

| Skin | Look |
|---|---|
| **Dark Club** (default) | Graphite and aluminium with purple lights, platter with a perforated rim and a tone arm. |
| **Industrial** | Dark metal, perforated and diamond plate, amber lights; motorised platter with an amber light ring. |
| **Retro Hardware** | Brushed aluminium, cream-coloured buttons, red accents; an active button stays pressed and its label lights up. |

In the Industrial and Retro skins the platter turns like a motorised platter: the outer ring, the screws and the hub turn at real speed, while the reflections on the metal stay still.

### Theme colour

Each skin has its own colour (purple, amber or red), used for lights, knobs, platters, waveforms and hot cues. To change it pick the hue on the **colour bar** and the shade in the **square** above (as in Photoshop), or type the hexadecimal code. Below you see a **live preview** of the interface changing as you choose. **↺ Ripristina colore predefinito** (reset to default colour) returns to the skin's colour. Green PLAY, amber CUE and the green-yellow-red meters stay fixed because they carry a meaning.

### Ultrawide monitors

When the window is at least 2.4 times wider than tall (for example a 32:9 monitor in full screen), SpinLab uses a dedicated layout:

- **on the left** decks and mixer, in the same proportions as on a normal monitor;
- **below the decks**, side by side: the **stems** (always visible), the panel chosen among **SAMPLER**, **DRUMS** and **AUTOMIX**, and the **FX**;
- **on the right** folders, playlists and track list at full height.

In this layout the library is always open, so the LIBRARY view is not needed. If you narrow the window the normal layout comes back. On traditional monitors nothing changes.

---

## Recording the master

**RECORD** at the top opens the recording window, where you choose format, video quality and folder and press **Avvia registrazione** (start recording): the window closes and the **REC** indicator appears with the time. It records everything coming out of the master: decks with effects, sampler and drums. **After the first time** you can also start and stop with the round **●** button at the top, next to the clock: it uses the same settings and an automatic name (`SpinLab YYYY-MM-DD HH.MM.SS`) in the same folder, without reopening the window. While recording the button becomes the **REC** indicator (a click stops it) and the RECORD button becomes **STOP REC**.

1. Choose the format: WAV 16 or 24 bit, MP3 at 128, 192, 256 or 320 kbps, or **MP4 with video** (see [Recording the set in MP4](#recording-the-set-in-mp4)).
2. Choose where to save the file.
3. **Avvia registrazione.** The **REC** indicator appears with the time. A click on the indicator, or on **STOP REC**, stops the recording.

Audio settings cannot be changed while recording.

---

## Audio settings

**SETTINGS** → **AUDIO** tab.

- **Driver audio** (audio driver). **Windows (WASAPI)** works with any card. **ASIO** uses the card's driver and gives much lower latency, but the card becomes dedicated to SpinLab.
- **Uscita principale (master)** (main output). With ASIO the list shows the installed ASIO drivers. To use the PC's built-in card choose the **Windows (WASAPI)** driver.
- **Frequenza e buffer** (sample rate and buffer). A small buffer gives less latency but more risk of dropouts: if you hear clicks, increase it. 128 frames is a good starting point with ASIO.
- **Uscita cuffie** (headphone output). This is the output for pre-listen and track preview. The list shows the Windows devices. Alternatively **Stesso dispositivo dell'uscita principale, canali 3-4** (same device as the main output, channels 3-4): master on channels 1-2 and headphones on channels 3-4 of the same card (needs an output with at least 4 channels).

### ASIO: where the driver comes from

**SpinLab does not install any ASIO driver.** ASIO is a low-latency driver system: the driver is written and installed by the **sound card manufacturer** (for example Focusrite, Behringer, RME, MOTU, Denon DJ). When you install the card, its driver registers itself in Windows, and SpinLab lists it in the «Uscita audio principale» (main audio output) menu. Each user therefore uses the ASIO driver of **their own** card, already installed on their PC.

- **The card has an ASIO driver** (usually yes, for external sound cards and DJ and music interfaces): choose the **ASIO** driver and the card from the list.
- **The PC's built-in card or other cards without an ASIO driver**: use **Windows (WASAPI)**. It always works, with higher latency. If you still want ASIO on any card there are third-party universal drivers (for example ASIO4ALL or FlexASIO): if you install them, they appear in the list like the others.
- **With ASIO the card is dedicated to SpinLab**: other programs cannot use it at the same time.

**After «Applica»** (Apply), the area below shows the output actually in use and the estimated latency. If the chosen output does not start, SpinLab goes back to Windows (WASAPI) and tells you why.

**Example setups**

| Situation | Master | Headphones |
|---|---|---|
| External card with ASIO and a controller | ASIO → the card | Windows device of the controller's headphones (it usually appears with a name like «Line 3/4 (controller name)») |
| Everything on the controller | The controller's ASIO or WASAPI | Same device, channels 3-4 |
| PC only | WASAPI | A second device, or none |

---

## MIDI controllers

SpinLab works with any MIDI controller. Functions are mapped in the **SETTINGS → CONTROLLER MIDI** panel with MIDI learn. The mapping is saved automatically and can be exported and imported as text.

### The panel

- **Collegati** (connected) lists the controllers found. **Ultimo messaggio** (last message) shows in real time what arrives: use it to understand what a control sends.
- **Uscita LED** (LED output) is the port used to light the controller's LEDs.
- **Jog per giro** (jog per turn): how many messages the jog sends for one full turn of the platter. **Calibra il jog** (calibrate the jog) measures it by itself: press, turn the platter one full turn, press again.
- **Analizza un controllo** (analyse a control): records all messages for 8 seconds. Turn a control one way and then the other; the summary shows which message it sends and with which values. Useful to understand an encoder or the jog.
- **Ignora un tasto…** (ignore a key): for touch sensors (for example an encoder that sends a message when you touch it). Press, touch the sensor: that message no longer controls anything and learn skips it.
- **Cerca una funzione** (search a function) filters the list. Functions are grouped by deck and by mixer, library and effects.
- **Esporta / Importa / Cancella tutto** (export / import / clear all) to copy or reset the mapping.

### Mapping a function

1. Press **Learn** on the function's row.
2. Press the button, turn the knob or move the fader on the controller.
3. Learn recognises the type of control by itself: button, 7-bit fader, relative encoder (two's complement, offset 64, sign bit) or rotating absolute value (typical of a jog). If it gets it wrong, change the **Modo** (mode) in the row's menu.
4. To **remove** a mapping use the row's ✕. Esc cancels a Learn in progress.

For continuous controls (faders, knobs, encoders) Learn waits for a real movement: noise or sensors that always send the same value are not mapped.

### Multiple mappings

- A function can have **several mappings**: press **+ Altro** (+ another) to add one more (for example two buttons doing the same thing).
- A **message** can control **several functions** (for example a single click triggering two things). When this happens, the row shows a warning «⚠ anche …» (also …) listing the other functions: it helps you find old forgotten mappings.

### SHIFT

First map the controller's **SHIFT** button. To make a combination:

1. Press **Learn** on a function.
2. **Hold SHIFT** on the controller.
3. Press the button to combine (for example a pad) and release.

The function becomes «SHIFT + pad». The same pad, without SHIFT, can control another function. If you have not mapped SHIFT yet, hold the button and press the other one: SpinLab understands by itself that the first one is SHIFT. You can also map two SHIFT buttons (one per deck).

### Pad layers

If the controller has mode buttons for the pads (for example HOT CUE, LOOP, ROLL, BANK), SpinLab uses them as **layers**:

1. In the deck group map **Livello dei pad: Hot cue / Loop / Roll / Bank** (pad layer) to the four mode buttons.
2. Press **Learn** on a function, press the mode button (for example LOOP), then the pad.
3. The mapping is valid only in that layer. The **Livello** (layer) menu on the row lets you correct it.

If a layer has no mappings for a pad, the «Ogni livello» (every layer) one applies. Layers are independent per deck, and at startup the pads are in Hot cue.

### Jog, encoders and special buttons

- **Jog.** With the platter touched it scratches; with the track playing and the platter not touched, it does a small pitch bend. Map the rotation to **Jog (scratch e pitch bend)** and the touch to **Jog: tocco del piatto** (platter touch).
- **Encoder as a button.** On a button function (for example **Loop × 2**) you can use an encoder's rotation: Learn recognises it and sets «Encoder: solo rotazione oraria» (clockwise rotation only). For the other direction use the Modo menu.
- **Loop IN / OUT with a single button:** first click = IN, second = OUT, third = exits the loop.
- **Roll:** longer / shorter length / encoder.
- **Library:** browse the tracks (one encoder per deck, even 10 tracks per step) and load the chosen track.

### Effects from the controller

The **Effetti del controller** (controller effects) group is meant for controllers with a single set of effect controls:

- **Assign the effect to deck 1 / 2.** Pressed together (within 0.4 seconds) they apply to both decks.
- **Effect on / off** on the assigned decks.
- **Parameter (encoder)** with the **encoder click** switching between the first and second parameter (for example TONE and FEEDBACK). The underlined name in the FX panel shows which one is active.
- **MIX dry / wet** (knob).
- **Next / previous effect** or an effect-selection encoder.

Encoder steps have acceleration and a small ramp so they don't sound stepped. Steps right before a click are discarded, because they are only the movement caused by pressing.

### LEDs

Choose the port in **Uscita LED**. Every button mapping has two values (on and off) that SpinLab sends to the controller when the function changes state. It works only if the controller accepts LED messages via MIDI.

---

## Troubleshooting

**The ASIO driver does not start and I stay on Windows (WASAPI).**
Read the message below «Applica». Common causes: another program keeps the driver open (another copy of SpinLab, the card's control panel, a DAW); a controller with a sound card that is connected may have its ASIO driver loaded; SpinLab opens only the driver you choose and does not touch the others. Close extra copies of SpinLab from the Task Manager.

**Clicks or dropouts in the audio.** Increase the buffer. With ASIO try 256.

**The headphones are silent.**
- The headphone output has to be chosen among the Windows devices. Pre-listen has its own PFL buttons (1 and 2 at the top, or the icon on the platter) and the **PHONES** volume.
- For «Stesso dispositivo, canali 3-4» the main output must have at least 4 channels.

**The jog goes in one direction only, or in jumps.** The mode is wrong. Redo Learn turning the platter for a second, or choose *Assoluto, usato come differenza* (absolute, used as a difference) in the Modo menu. If it is too fast, use **Calibra il jog**.

**Scratching sounds distorted.** Recalibrate «Jog per giro». A value that is too low makes the record run.

**Learn maps by itself.** The controller is sending messages without you touching anything (noise, sensors). Learn for continuous controls waits for a real movement. If it continues, use **Ignora un tasto…**.

**An encoder moves two things.** Look at the «⚠ anche …» warnings in the list: an old mapping on the same message moves other functions. Delete it with the ✕.

**The video is not shown, is choppy or not in time.** See [Video](#video), in particular «If something goes wrong with video».

**Stems are slow or don't start.** See **Impostazioni → STEM**: with «Solo stem preparati» prepare the tracks first (right click → Crea stem).

---

## Licence, donations and credits

SpinLab DJ is **free and open-source software**: you can use, study, modify and redistribute it under the **GNU GPL 3.0 or later**. No registration is required and the program does not connect to the internet. The source code and new versions are on GitHub (https://github.com/muttleysat/SpinLab).

If you find SpinLab useful you can support it with a **donation of any amount**: the **Dona con PayPal** button in the About window (the **i** at the top right), which opens PayPal in the browser.

The beatgrid fitting is derived from openDJ (GPL-2.0 or later). Third-party components (FFmpeg, ONNX Runtime, HT-Demucs, Beat This! and the engine's libraries) and their licences are listed in the program's «licenses» folder.

ASIO is a trademark of Steinberg Media Technologies GmbH, registered in Europe and other countries.

![ASIO Compatible](../ui/assets/asio-logo-bw.png)
