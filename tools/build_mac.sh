#!/bin/zsh
# Crea l'installer per macOS (Apple Silicon): SpinLab.app con modelli, FFmpeg e licenze, impacchettata in un .dmg.
#
# Uso (dalla cartella del progetto):   zsh tools/build_mac.sh
# Risultato: src-tauri/target/release/bundle/dmg/SpinLab_<versione>_aarch64.dmg (da allegare alla Release su GitHub).
set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION=$(python3 -c "import json;print(json.load(open('$ROOT/src-tauri/tauri.conf.json'))['version'])")
echo "SpinLab $VERSION per macOS"

python3 "$ROOT/tools/fetch_model.py"                       # modelli dentro l'installer: niente download dopo l'installazione
cd "$ROOT/src-tauri"
cargo tauri build --bundles app --config tauri.release.conf.json

# il .dmg si crea con lo strumento di sistema (quello di Tauri a volte fallisce con «risorsa occupata»)
APP="target/release/bundle/macos/SpinLab.app"
DMG="target/release/bundle/dmg/SpinLab_${VERSION}_aarch64.dmg"
TMP=$(mktemp -d)
cp -R "$APP" "$TMP/" && ln -s /Applications "$TMP/Applications"
mkdir -p target/release/bundle/dmg
hdiutil create -volname "SpinLab $VERSION" -srcfolder "$TMP" -ov -format UDZO "$DMG" >/dev/null
rm -rf "$TMP"
echo "installer: $DMG"
shasum -a 256 "$DMG"
