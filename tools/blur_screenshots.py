#!/usr/bin/env python3
"""Sfoca leggermente le parti personali o protette degli screenshot (video, cartelle, titoli della libreria) prima di pubblicarli.

Le aree sono in pixel per le catture di tools/screenshot.ps1 (finestra 2250 px di larghezza, interfaccia al 150%).
Uso:  python tools/blur_screenshots.py      (lavora su docs/screenshots, gli originali restano in docs/screenshots/originali)
"""
import shutil
from pathlib import Path

from PIL import Image, ImageFilter

DIR = Path(__file__).resolve().parent.parent / "docs" / "screenshots"
RADIUS = 10  # sfocatura leggera: si capisce che c'e' testo o immagine, ma non si legge
MAX_W = 1600          # larghezza delle immagini pubblicate
MAX_BYTES = 1_000_000  # limite per immagine (GitHub)

TREE = (15, 845, 318, 1078)          # albero delle cartelle (Esplora), vista mixer
LIB_ROWS = lambda y0, y1: (385, y0, 912, y1)  # copertina, titolo e artista delle righe della libreria
STEM_STATUS = (12, 1288, 322, 1318)  # riga di stato in basso a sinistra (titolo del brano in separazione)

AREAS = {
    "mixer.png": [TREE, LIB_ROWS(900, 1289)],
    "libreria.png": [
        (15, 392, 318, 928),                       # albero delle cartelle
        LIB_ROWS(448, 1289),
        (1738, 992, 2240, 1092),                   # copertina, titolo e artista del brano selezionato
        (1736, 1240, 2240, 1289),                  # percorso del file
    ],
    "video.png": [(25, 172, 852, 417), (1398, 172, 2226, 417), TREE, LIB_ROWS(905, 1289)],
    "impostazioni.png": [(25, 172, 286, 417), (1964, 172, 2226, 417), (15, 845, 286, 1078), LIB_ROWS(1222, 1289)],
    "skin-industrial.png": [TREE, LIB_ROWS(905, 1318), STEM_STATUS],
    "skin-retro.png": [TREE, LIB_ROWS(905, 1318), STEM_STATUS],
}


def main():
    orig = DIR / "originali"
    orig.mkdir(exist_ok=True)
    for name, areas in AREAS.items():
        src = DIR / name
        keep = orig / name
        if not keep.exists() and not src.exists():
            print("manca", name)
            continue
        if not keep.exists():
            shutil.copy2(src, keep)  # l'originale si conserva una volta sola: rilanciare lo script riparte da li'
        im = Image.open(keep).convert("RGB")
        for a in areas:
            box = (0, 0, im.width, im.height) if a == "all" else (a[0], a[1], min(a[2], im.width), min(a[3], im.height))
            im.paste(im.crop(box).filter(ImageFilter.GaussianBlur(RADIUS)), box[:2])
        # pubblicata: JPEG largo MAX_W, qualita' scelta per stare sotto MAX_BYTES (il PNG di un'interfaccia piena di sfumature pesa troppo)
        if im.width > MAX_W:
            im = im.resize((MAX_W, round(im.height * MAX_W / im.width)), Image.LANCZOS)
        out = src.with_suffix(".jpg")
        for q in (88, 84, 80, 75, 70, 65, 60):
            im.save(out, "JPEG", quality=q, optimize=True, progressive=True, subsampling="4:2:0")
            if out.stat().st_size <= MAX_BYTES:
                break
        if src != out and src.exists():
            src.unlink()  # il PNG originale e' gia' in originali/
        print(f"{out.name}: {im.width}x{im.height}, qualita' {q}, {out.stat().st_size // 1024} KB")


if __name__ == "__main__":
    main()
