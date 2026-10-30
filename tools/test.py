"""Esegue i test del motore su Windows:  python tools/test.py [filtro] [-- opzioni del test]

Da quando il programma e' anche una libreria (porting Android), l'eseguibile dei test non riceve il manifest «Common Controls 6»
che Tauri incorpora solo nell'app, e Windows lo rifiuta (STATUS_ENTRYPOINT_NOT_FOUND). Qui il manifest viene messo accanto
all'eseguibile dei test, che poi viene lanciato normalmente.
"""
import json, os, shutil, subprocess, sys

ROOT = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "src-tauri")
MANIFEST = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0"><dependency><dependentAssembly>
<assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
</dependentAssembly></dependency></assembly>
"""

env = dict(os.environ, PATH=os.environ["PATH"] + os.pathsep + os.path.join(os.path.expanduser("~"), ".cargo", "bin"))
cargo = shutil.which("cargo", path=env["PATH"]) or "cargo"
r = subprocess.run([cargo, "test", "--release", "--lib", "--no-run", "--message-format=json"], cwd=ROOT, env=env, stdout=subprocess.PIPE, text=True)
exe = None
for line in r.stdout.splitlines():
    try:
        m = json.loads(line)
    except ValueError:
        continue
    if m.get("reason") == "compiler-artifact" and m.get("executable") and m.get("profile", {}).get("test"):
        exe = m["executable"]
if r.returncode or not exe:
    sys.exit("compilazione dei test fallita")
if os.name == "nt":
    with open(exe + ".manifest", "w") as f:
        f.write(MANIFEST)
args = [a for a in sys.argv[1:]]
sys.exit(subprocess.run([exe] + args, cwd=ROOT).returncode)
