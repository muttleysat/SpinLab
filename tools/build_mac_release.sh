#!/bin/zsh
# Crea la versione per macOS pronta da pubblicare: installer .dmg e file dell'aggiornamento automatico firmati, piu' latest.json.
#
# Uso (dalla cartella del progetto):   zsh tools/build_mac_release.sh "note della versione"
#
# Serve la chiave di firma degli aggiornamenti in ~/.tauri/spinlab.key (la stessa del PC Windows): la password viene chiesta qui e non
# viene salvata da nessuna parte. Risultato in src-tauri/target/release/bundle/ (dmg e macos) e in server/updates/ (da caricare via FTP).
set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KEY="$HOME/.tauri/spinlab.key"
[[ -f "$KEY" ]] || { echo "manca la chiave $KEY (copiarla dal PC Windows)"; exit 1; }
VERSION=$(python3 -c "import json;print(json.load(open('$ROOT/src-tauri/tauri.conf.json'))['version'])")
NOTES="${1:-SpinLab $VERSION}"
echo "SpinLab $VERSION per macOS"
read -rs "PASS?Password della chiave di firma: "; echo
export TAURI_SIGNING_PRIVATE_KEY="$KEY" TAURI_SIGNING_PRIVATE_KEY_PASSWORD="$PASS"

python3 "$ROOT/tools/fetch_model.py"                       # modelli dentro l'installer: niente download dopo l'installazione
cd "$ROOT/src-tauri"
cargo tauri build --bundles app --config tauri.release.conf.json   # crea SpinLab.app e SpinLab.app.tar.gz + .sig (aggiornamento)
unset TAURI_SIGNING_PRIVATE_KEY_PASSWORD PASS

# il .dmg si crea a parte con lo strumento di sistema (quello di Tauri a volte fallisce con «risorsa occupata»)
APP="target/release/bundle/macos/SpinLab.app"
DMG="target/release/bundle/dmg/SpinLab_${VERSION}_aarch64.dmg"
TMP=$(mktemp -d)
cp -R "$APP" "$TMP/" && ln -s /Applications "$TMP/Applications"
mkdir -p target/release/bundle/dmg
hdiutil create -volname "SpinLab $VERSION" -srcfolder "$TMP" -ov -format UDZO "$DMG" >/dev/null
rm -rf "$TMP"
echo "installer: $DMG"
shasum -a 256 "$DMG"

python3 "$ROOT/tools/make_update_manifest.py" "$VERSION" "$NOTES"
