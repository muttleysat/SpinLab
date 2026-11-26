# Cattura la finestra di SpinLab in un PNG (per README e documentazione), senza bordi del sistema.
# Uso:  powershell -NoProfile -File tools\screenshot.ps1 docs\screenshots\mixer.png
# -Title: finestra da catturare (predefinita la principale; "SpinLab — Uscita video" per l'uscita video)
param([Parameter(Mandatory = $true)][string]$Out, [string]$Title = "SpinLab DJ")

Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class SlWin {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, string title);
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
"@
[SlWin]::SetProcessDPIAware() | Out-Null

# per titolo: in modalita' video SpinLab ha due finestre e quella "principale" del processo puo' essere l'uscita video
$h = [SlWin]::FindWindow([NullString]::Value, $Title)
if ($h -eq [IntPtr]::Zero) { Write-Error "finestra '$Title' non trovata (SpinLab e' aperto?)"; exit 1 }
# la copia dallo schermo prende cio' che e' visibile: SpinLab va portato in primo piano (e ripristinato se ridotto a icona)
$prev = [SlWin]::GetForegroundWindow()
if ([SlWin]::IsIconic($h)) { [SlWin]::ShowWindow($h, 9) | Out-Null }
[SlWin]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Milliseconds 700

# area client (senza barra del titolo) in coordinate dello schermo, copiata dallo schermo: PrintWindow non cattura WebView2
$r = New-Object SlWin+RECT; [SlWin]::GetClientRect($h, [ref]$r) | Out-Null
$pt = New-Object SlWin+POINT; [SlWin]::ClientToScreen($h, [ref]$pt) | Out-Null
$w = $r.Right - $r.Left; $hh = $r.Bottom - $r.Top
# niente barra delle applicazioni: si taglia la finestra all'area utile dello schermo che la contiene
$wa = [System.Windows.Forms.Screen]::FromHandle($h).WorkingArea
$w = [Math]::Min($w, $wa.Right - $pt.X); $hh = [Math]::Min($hh, $wa.Bottom - $pt.Y)
$bmp = New-Object System.Drawing.Bitmap $w, $hh
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($pt.X, $pt.Y, 0, 0, (New-Object System.Drawing.Size $w, $hh))
$dir = Split-Path -Parent $Out
if ($dir) { New-Item -ItemType Directory -Force $dir | Out-Null }
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
# torna alla finestra di prima (l'editor)
[SlWin]::SetForegroundWindow($prev) | Out-Null
"salvato $Out ($w x $hh)"
