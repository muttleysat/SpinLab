"""Preparazione del sistema licenze.

  python tools/license_setup.py keys    genera la coppia di chiavi di firma (una volta sola!)
  python tools/license_setup.py hash    genera l'hash della password del pannello admin

keys: la chiave PRIVATA va in ~/.spinlab/license-private.key (fuori dal progetto, non viene mai mostrata) e serve solo sul server
      (campo sign_key_hex di server/inc/config.php); la PUBBLICA va in src-tauri/license_pubkey.txt e viene compilata nel programma.
      Se la coppia esiste gia' non la sovrascrive: cambiarla invaliderebbe tutte le licenze gia' emesse.
Serve:  pip install cryptography bcrypt
"""
import getpass, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PRIV = os.path.join(os.path.expanduser("~"), ".spinlab", "license-private.key")
PUB = os.path.join(ROOT, "src-tauri", "license_pubkey.txt")


def keys():
    from cryptography.hazmat.primitives import serialization as s
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    if os.path.exists(PRIV) or os.path.exists(PUB):
        sys.exit(f"Esiste gia' una coppia di chiavi ({PRIV} / {PUB}): non la sovrascrivo. Cambiarla invaliderebbe le licenze emesse.")
    k = Ed25519PrivateKey.generate()
    seed = k.private_bytes(s.Encoding.Raw, s.PrivateFormat.Raw, s.NoEncryption())
    pub = k.public_key().public_bytes(s.Encoding.Raw, s.PublicFormat.Raw)
    os.makedirs(os.path.dirname(PRIV), exist_ok=True)
    with open(PRIV, "w") as f:
        f.write((seed + pub).hex() + "\n")          # formato di libsodium: seme + pubblica (64 byte)
    try:
        os.chmod(PRIV, 0o600)
    except OSError:
        pass
    with open(PUB, "w") as f:
        f.write(pub.hex() + "\n")
    print(f"Chiave privata: {PRIV}  (da copiare in server/inc/config.php come sign_key_hex; metti una copia di sicurezza fuori dal PC)")
    print(f"Chiave pubblica: {PUB}  (va nel repository)")


def hashpw():
    import bcrypt
    p = getpass.getpass("Password del pannello admin: ")
    if len(p) < 10:
        sys.exit("Troppo corta: almeno 10 caratteri.")
    if p != getpass.getpass("Ripeti: "):
        sys.exit("Non coincidono.")
    h = bcrypt.hashpw(p.encode(), bcrypt.gensalt(12)).decode().replace("$2b$", "$2y$", 1)
    print("\nMetti questo valore in config.php, campo admin_hash (tra apici singoli):\n" + h)


if __name__ == "__main__":
    {"keys": keys, "hash": hashpw}.get(sys.argv[1] if len(sys.argv) > 1 else "", lambda: sys.exit(__doc__))()
