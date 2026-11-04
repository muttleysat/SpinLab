#!/bin/bash
# Abbina e collega il Galaxy Watch via debug Wi-Fi, trovando da solo indirizzo e porte.
# Uso: sull'orologio apri Opzioni sviluppatore > Debug tramite Wi-Fi > Abbina nuovo dispositivo (lascia lo schermo acceso),
#      poi lancia questo script e scrivi il codice a 6 cifre mostrato. Il codice scade in fretta e un tentativo sbagliato lo invalida.
#      Con --dry-run cerca soltanto l'orologio, senza abbinare.
source "$(dirname "$0")/../env-android.sh" 2>/dev/null

# nome dell'istanza annunciata in rete per un tipo di servizio
instance() { ( dns-sd -B "$1" local. > /tmp/wb.$$ 2>&1 & p=$!; sleep 3; kill $p 2>/dev/null ); awk '/ Add /{print $NF}' /tmp/wb.$$ | head -1; rm -f /tmp/wb.$$; }
# "indirizzo-ip porta" di un'istanza
endpoint() {
  ( dns-sd -L "$1" "$2" local. > /tmp/wl.$$ 2>&1 & p=$!; sleep 3; kill $p 2>/dev/null )
  local hp host port ip
  hp=$(grep -oE "at [^ ]+:[0-9]+" /tmp/wl.$$ | head -1 | sed 's/^at //'); rm -f /tmp/wl.$$
  host=${hp%:*}; port=${hp##*:}
  ( dns-sd -G v4 "$host" > /tmp/wg.$$ 2>&1 & p=$!; sleep 2; kill $p 2>/dev/null )
  ip=$(grep -oE "([0-9]{1,3}\.){3}[0-9]{1,3}" /tmp/wg.$$ | head -1); rm -f /tmp/wg.$$
  echo "$ip $port"
}

PAIR=$(instance _adb-tls-pairing._tcp)
if [ -z "$PAIR" ]; then echo "L'orologio non annuncia nessun abbinamento: apri 'Abbina nuovo dispositivo' e riprova."; exit 1; fi
read -r IP PORT <<< "$(endpoint "$PAIR" _adb-tls-pairing._tcp)"
echo "Orologio trovato: $PAIR  ->  $IP:$PORT"
[ "$1" = "--dry-run" ] && exit 0

read -r -p "Codice di abbinamento: " CODE
adb pair "$IP:$PORT" "$CODE" || { echo "Abbinamento fallito: codice errato o scaduto. Riapri 'Abbina nuovo dispositivo' e rilancia lo script."; exit 1; }

sleep 2
CONN=$(instance _adb-tls-connect._tcp)
read -r IP2 PORT2 <<< "$(endpoint "$CONN" _adb-tls-connect._tcp)"
echo "Mi collego a $IP2:$PORT2"
adb connect "$IP2:$PORT2"
adb devices -l
