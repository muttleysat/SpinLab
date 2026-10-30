<?php
// Funzioni comuni del server licenze di SpinLab. PHP >= 7.2 (serve libsodium, incluso di serie).

function cfg(): array {
    static $c = null;
    if ($c === null) {
        $f = __DIR__ . '/config.php';
        if (!is_file($f)) { http_response_code(500); exit('config.php mancante'); }
        $c = require $f;
    }
    return $c;
}

function db(): PDO {
    static $pdo = null;
    if ($pdo === null) {
        $c = cfg();
        $pdo = new PDO("mysql:host={$c['db_host']};dbname={$c['db_name']};charset=utf8mb4", $c['db_user'], $c['db_pass'], [
            PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
            PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
        ]);
    }
    return $pdo;
}

function now_sql(): string { return gmdate('Y-m-d H:i:s'); }

function client_ip(): string { return substr($_SERVER['REMOTE_ADDR'] ?? '', 0, 45); }

function out(int $code, array $body): void {
    http_response_code($code);
    header('Content-Type: application/json; charset=utf-8');
    header('Cache-Control: no-store');
    echo json_encode($body, JSON_UNESCAPED_UNICODE);
    exit;
}

function read_json(): array {
    if (($_SERVER['REQUEST_METHOD'] ?? '') !== 'POST') out(405, ['error' => 'metodo']);
    $raw = file_get_contents('php://input', false, null, 0, 8192);
    $j = json_decode($raw ?: '', true);
    if (!is_array($j)) out(400, ['error' => 'richiesta']);
    return $j;
}

function b64url(string $s): string { return rtrim(strtr(base64_encode($s), '+/', '-_'), '='); }

/** Chiave di licenza: SPIN-XXXXX-XXXXX-XXXXX-XXXXX (alfabeto senza 0/O/1/I/L, 100 bit casuali). */
function new_license_key(): string {
    $alpha = '23456789ABCDEFGHJKMNPQRSTUVWXYZ';
    $g = [];
    for ($i = 0; $i < 4; $i++) {
        $s = '';
        for ($k = 0; $k < 5; $k++) $s .= $alpha[random_int(0, strlen($alpha) - 1)];
        $g[] = $s;
    }
    return 'SPIN-' . implode('-', $g);
}

function norm_key(string $k): string { return strtoupper(trim($k)); }
function valid_key_format(string $k): bool { return (bool)preg_match('/^SPIN-[A-Z0-9]{5}-[A-Z0-9]{5}-[A-Z0-9]{5}-[A-Z0-9]{5}$/', $k); }
function valid_machine(string $m): bool { return (bool)preg_match('/^[0-9a-f]{64}$/', $m); }

/** Certificato firmato Ed25519: base64url(payload).base64url(firma). Il programma lo verifica con la chiave pubblica che ha dentro. */
function make_cert(array $lic, string $machine): string {
    $c = cfg();
    $now = time();
    $payload = json_encode([
        'v' => 1, 'lic' => $lic['license_key'], 'name' => $lic['name'], 'mid' => $machine,
        'iat' => $now, 'exp' => $now + 86400 * (int)$c['cert_days'], 'grace' => 86400 * (int)$c['grace_days'],
        'max' => (int)$lic['max_activations'],
    ], JSON_UNESCAPED_UNICODE);
    $sk = hex2bin($c['sign_key_hex']);
    if ($sk === false || strlen($sk) !== SODIUM_CRYPTO_SIGN_SECRETKEYBYTES) out(500, ['error' => 'server']);
    return b64url($payload) . '.' . b64url(sodium_crypto_sign_detached($payload, $sk));
}

/** Piu' di 15 tentativi falliti in un'ora dallo stesso IP: si aspetta. */
function rate_check(): void {
    $st = db()->prepare('SELECT COUNT(*) FROM attempts WHERE ip=? AND at > ?');
    $st->execute([client_ip(), gmdate('Y-m-d H:i:s', time() - 3600)]);
    if ((int)$st->fetchColumn() >= 15) out(429, ['error' => 'troppi_tentativi']);
}
function rate_fail(): void {
    db()->prepare('INSERT INTO attempts(ip, at) VALUES(?,?)')->execute([client_ip(), now_sql()]);
    if (random_int(1, 50) === 1) db()->prepare('DELETE FROM attempts WHERE at < ?')->execute([gmdate('Y-m-d H:i:s', time() - 86400)]);
}

function find_license(string $key): ?array {
    $st = db()->prepare('SELECT * FROM licenses WHERE license_key=?');
    $st->execute([$key]);
    return $st->fetch() ?: null;
}

/** Attiva (o rinnova) la licenza per un computer. Restituisce il certificato o esce con un errore JSON. */
function activate(string $key, string $machine, string $label, string $version, bool $allowNew): string {
    rate_check();
    $key = norm_key($key);
    if (!valid_key_format($key) || !valid_machine($machine)) { rate_fail(); out(400, ['error' => 'chiave_non_valida']); }
    $lic = find_license($key);
    if (!$lic) { rate_fail(); out(404, ['error' => 'chiave_sconosciuta']); }
    if ($lic['status'] !== 'active') out(403, ['error' => 'licenza_revocata']);
    $db = db();
    $st = $db->prepare('SELECT id FROM activations WHERE license_id=? AND machine=?');
    $st->execute([$lic['id'], $machine]);
    $act = $st->fetch();
    $label = mb_substr(trim($label), 0, 120);
    $version = substr(preg_replace('/[^0-9A-Za-z.\-]/', '', $version), 0, 20);
    if ($act) {
        $db->prepare('UPDATE activations SET last_seen=?, ip=?, label=?, app_version=? WHERE id=?')->execute([now_sql(), client_ip(), $label, $version, $act['id']]);
    } else {
        if (!$allowNew) out(403, ['error' => 'non_attivata']);
        $n = $db->prepare('SELECT COUNT(*) FROM activations WHERE license_id=?');
        $n->execute([$lic['id']]);
        if ((int)$n->fetchColumn() >= (int)$lic['max_activations']) out(409, ['error' => 'limite_computer', 'max' => (int)$lic['max_activations']]);
        $db->prepare('INSERT INTO activations(license_id, machine, label, app_version, first_seen, last_seen, ip) VALUES(?,?,?,?,?,?,?)')
           ->execute([$lic['id'], $machine, $label, $version, now_sql(), now_sql(), client_ip()]);
    }
    return make_cert($lic, $machine);
}

function send_key_mail(string $to, string $name, string $key): bool {
    $c = cfg();
    if (!filter_var($to, FILTER_VALIDATE_EMAIL)) return false;
    $subject = '=?UTF-8?B?' . base64_encode('La tua licenza di SpinLab') . '?=';
    $body = "Ciao" . ($name !== '' ? " $name" : '') . ",\r\n\r\ngrazie per aver acquistato SpinLab.\r\n\r\nLa tua chiave di licenza e':\r\n\r\n    $key\r\n\r\n"
          . "Per registrare il programma: apri SpinLab, scegli «Registra», incolla la chiave e conferma (serve una connessione a internet solo in quel momento).\r\n"
          . "La chiave funziona su " . (int)$c['default_max_activations'] . " computer. Conservala: se cambi computer puoi spostare la licenza dal programma (Informazioni > Disattiva).\r\n\r\nBuona musica!\r\n";
    $from = '=?UTF-8?B?' . base64_encode($c['mail_name']) . '?= <' . $c['mail_from'] . '>';
    $headers = "From: $from\r\nReply-To: {$c['mail_from']}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nContent-Transfer-Encoding: 8bit";
    return mail($to, $subject, $body, $headers, '-f' . $c['mail_from']);
}
