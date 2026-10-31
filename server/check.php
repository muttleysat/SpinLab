<?php
// Controllo dell'installazione. Si carica, si apre una volta, e POI SI CANCELLA dall'hosting.
// Accesso:  https://.../check.php?k=PRIMI12  dove PRIMI12 sono i primi 12 caratteri di 'webhook_secret' in config.php.
// Non mostra mai password, chiavi o hash: solo OK / PROBLEMA.
header('Content-Type: text/plain; charset=utf-8');
header('Cache-Control: no-store');
$f = __DIR__ . '/inc/config.php';
if (!is_file($f)) { http_response_code(500); exit("PROBLEMA: inc/config.php non trovato (va caricato nella cartella inc/)\n"); }
$c = require $f;
$k = (string)($_GET['k'] ?? '');
if (strlen($c['webhook_secret'] ?? '') < 32 || !hash_equals(substr($c['webhook_secret'], 0, 12), $k)) { http_response_code(403); exit("Accesso negato\n"); }

$ok = true;
function say(string $what, bool $good, string $detail = ''): void {
    global $ok; if (!$good) $ok = false;
    echo ($good ? 'OK       ' : 'PROBLEMA ') . $what . ($detail !== '' ? " - $detail" : '') . "\n";
}

say('PHP ' . PHP_VERSION, version_compare(PHP_VERSION, '7.2.0', '>='), 'serve 7.2 o piu\' recente');
say('libsodium (firma delle licenze)', function_exists('sodium_crypto_sign_detached'));
say('PDO MySQL', extension_loaded('pdo_mysql'));
say('HTTPS attivo su questo indirizzo', !empty($_SERVER['HTTPS']) || ($_SERVER['HTTP_X_FORWARDED_PROTO'] ?? '') === 'https', 'le licenze viaggiano su internet: serve il certificato');

// configurazione
foreach (['db_host', 'db_name', 'db_user', 'db_pass', 'sign_key_hex', 'admin_hash', 'webhook_secret', 'mail_from'] as $field) {
    say("config: $field compilato", isset($c[$field]) && $c[$field] !== '');
}
say("config: mail_from non e' quello di esempio", strpos($c['mail_from'] ?? '', 'example.com') === false);

// chiave di firma: firma e verifica di prova
$sk = hex2bin($c['sign_key_hex'] ?? '');
if ($sk !== false && strlen($sk) === 64 && function_exists('sodium_crypto_sign_detached')) {
    $pub = sodium_crypto_sign_publickey_from_secretkey($sk);
    $sig = sodium_crypto_sign_detached('prova', $sk);
    say('chiave di firma valida', sodium_crypto_sign_verify_detached($sig, 'prova', $pub));
    echo '         chiave pubblica (deve coincidere con src-tauri/license_pubkey.txt): ' . bin2hex($pub) . "\n";
} else {
    say('chiave di firma valida', false, 'sign_key_hex deve essere di 128 caratteri esadecimali');
}
say('hash della password admin (bcrypt)', (bool)preg_match('/^\$2y\$\d\d\$.{53}$/', $c['admin_hash'] ?? ''));

// database
try {
    $pdo = new PDO("mysql:host={$c['db_host']};dbname={$c['db_name']};charset=utf8mb4", $c['db_user'], $c['db_pass'], [PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION]);
    say('connessione al database', true);
    $have = $pdo->query('SHOW TABLES')->fetchAll(PDO::FETCH_COLUMN);
    foreach (['licenses', 'activations', 'attempts'] as $t) say("tabella $t", in_array($t, $have, true), 'importa inc/schema.sql da phpMyAdmin');
    // scrittura e lettura vere
    $pdo->beginTransaction();
    $pdo->prepare("INSERT INTO licenses(license_key, created_at) VALUES('SPIN-TESTT-TESTT-TESTT-TESTT', UTC_TIMESTAMP())")->execute();
    $n = (int)$pdo->query("SELECT COUNT(*) FROM licenses WHERE license_key='SPIN-TESTT-TESTT-TESTT-TESTT'")->fetchColumn();
    $pdo->rollBack();
    say('scrittura e lettura nel database (annullate subito)', $n === 1);
} catch (Throwable $e) {
    say('connessione al database', false, preg_replace('/(password|pass)[^ ]*/i', '***', $e->getMessage()));
}

// cartella inc protetta dal web
$base = (!empty($_SERVER['HTTPS']) ? 'https' : 'http') . '://' . ($_SERVER['HTTP_HOST'] ?? '') . dirname($_SERVER['SCRIPT_NAME']) . '/inc/schema.sql';
$ctx = stream_context_create(['http' => ['timeout' => 5, 'ignore_errors' => true], 'ssl' => ['verify_peer' => false]]);
$body = @file_get_contents($base, false, $ctx);
$code = 0; if (isset($http_response_header[0]) && preg_match('/ (\d{3})/', $http_response_header[0], $m)) $code = (int)$m[1];
say('cartella inc/ non leggibile dal web', $code === 403 || $code === 404 || ($body !== false && strpos($body, 'CREATE TABLE') === false), "risposta $code");

echo "\n" . ($ok ? "TUTTO A POSTO. Ora CANCELLA check.php dall'hosting.\n" : "Ci sono dei PROBLEMI da sistemare (poi ricarica questa pagina). Alla fine CANCELLA check.php dall'hosting.\n");
