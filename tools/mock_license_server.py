"""Server di licenze finto per provare il programma senza hosting (stesso protocollo di server/api/*.php).

  python tools/mock_license_server.py [porta]

Firma con la chiave privata in ~/.spinlab/license-private.key. Accetta la chiave SPIN-AAAAA-BBBBB-CCCCC-DDDDD (max 2 computer),
SPIN-REVOK-REVOK-REVOK-REVOK risponde «revocata». Il programma va compilato con SPINLAB_LICENSE_URL=http://127.0.0.1:<porta>/api
"""
import base64, json, os, sys, time
from http.server import BaseHTTPRequestHandler, HTTPServer
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

seed = bytes.fromhex(open(os.path.join(os.path.expanduser("~"), ".spinlab", "license-private.key")).read().strip())[:32]
SK = Ed25519PrivateKey.from_private_bytes(seed)
KEYS = {"SPIN-AAAAA-BBBBB-CCCCC-DDDDD": {"name": "Mario Rossi", "max": 2}}
ACT = {}  # chiave -> set di computer
b64 = lambda b: base64.urlsafe_b64encode(b).rstrip(b"=").decode()


def cert(key, machine, days=30):
    now = int(time.time())
    p = json.dumps({"v": 1, "lic": key, "name": KEYS[key]["name"], "mid": machine, "iat": now, "exp": now + 86400 * days, "grace": 14 * 86400, "max": KEYS[key]["max"]}).encode()
    return b64(p) + "." + b64(SK.sign(p))


class H(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def reply(self, code, obj):
        b = json.dumps(obj).encode()
        self.send_response(code); self.send_header("Content-Type", "application/json"); self.send_header("Content-Length", str(len(b))); self.end_headers(); self.wfile.write(b)

    def do_POST(self):
        j = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))) or b"{}")
        ep = self.path.rsplit("/", 1)[-1].replace(".php", "")
        key, mid = str(j.get("key", "")).upper(), j.get("machine", "")
        print(ep, key, mid[:8])
        if key == "SPIN-REVOK-REVOK-REVOK-REVOK":
            return self.reply(403, {"error": "licenza_revocata"})
        if key not in KEYS:
            return self.reply(404, {"error": "chiave_sconosciuta"})
        s = ACT.setdefault(key, set())
        if ep == "activate":
            if mid not in s and len(s) >= KEYS[key]["max"]:
                return self.reply(409, {"error": "limite_computer"})
            s.add(mid); return self.reply(200, {"cert": cert(key, mid)})
        if ep == "refresh":
            if mid not in s:
                return self.reply(403, {"error": "non_attivata"})
            return self.reply(200, {"cert": cert(key, mid)})
        if ep == "deactivate":
            s.discard(mid); return self.reply(200, {"ok": True})
        self.reply(404, {"error": "?"})


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8765
    print(f"in ascolto su http://127.0.0.1:{port}/api")
    HTTPServer(("127.0.0.1", port), H).serve_forever()
