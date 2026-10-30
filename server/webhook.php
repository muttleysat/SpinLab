<?php
// Ricevuta di vendita dalla piattaforma di pagamento: crea la licenza e manda la chiave per email.
// Firma: intestazione X-Signature = HMAC-SHA256 esadecimale del corpo, con 'webhook_secret' (schema di Lemon Squeezy; Paddle e Stripe
// usano altri schemi di firma: quando scegli la piattaforma si adatta solo verify_signature()).
// Formati accettati:
//   Lemon Squeezy: meta.event_name = order_created | order_refunded, data.id, data.attributes.user_email / user_name
//   Generico:      {"event":"order_created"|"order_refunded","order_id":"...","email":"...","name":"..."}
require __DIR__ . '/inc/lib.php';

function verify_signature(string $raw): bool {
    $secret = cfg()['webhook_secret'];
    if ($secret === '') return false;
    $sig = $_SERVER['HTTP_X_SIGNATURE'] ?? '';
    return $sig !== '' && hash_equals(hash_hmac('sha256', $raw, $secret), strtolower($sig));
}

if (($_SERVER['REQUEST_METHOD'] ?? '') !== 'POST') out(405, ['error' => 'metodo']);
$raw = file_get_contents('php://input', false, null, 0, 65536);
if (!verify_signature($raw ?: '')) out(401, ['error' => 'firma']);
$j = json_decode($raw, true);
if (!is_array($j)) out(400, ['error' => 'richiesta']);

if (isset($j['meta']['event_name'])) {                       // Lemon Squeezy
    $event = $j['meta']['event_name'];
    $order = (string)($j['data']['id'] ?? '');
    $email = (string)($j['data']['attributes']['user_email'] ?? '');
    $name = (string)($j['data']['attributes']['user_name'] ?? '');
    $source = 'lemonsqueezy';
} else {
    $event = (string)($j['event'] ?? '');
    $order = (string)($j['order_id'] ?? '');
    $email = (string)($j['email'] ?? '');
    $name = (string)($j['name'] ?? '');
    $source = 'webhook';
}
if ($order === '') out(400, ['error' => 'ordine']);
$db = db();

if ($event === 'order_refunded') {
    $db->prepare("UPDATE licenses SET status='revoked' WHERE order_ref=?")->execute([$order]);
    out(200, ['ok' => true, 'revoked' => true]);
}
if ($event !== 'order_created') out(200, ['ok' => true, 'ignored' => $event]);

// stessa vendita ricevuta due volte: non si crea una seconda chiave
$st = $db->prepare('SELECT license_key FROM licenses WHERE order_ref=?');
$st->execute([$order]);
if ($st->fetchColumn()) out(200, ['ok' => true, 'duplicate' => true]);

for ($try = 0; $try < 5; $try++) {
    $key = new_license_key();
    try {
        $db->prepare('INSERT INTO licenses(license_key, email, name, max_activations, source, order_ref, created_at) VALUES(?,?,?,?,?,?,?)')
           ->execute([$key, mb_substr($email, 0, 190), mb_substr($name, 0, 190), (int)cfg()['default_max_activations'], $source, mb_substr($order, 0, 120), now_sql()]);
        break;
    } catch (PDOException $e) { if ($try === 4) out(500, ['error' => 'db']); }
}
$sent = send_key_mail($email, $name, $key);
out(200, ['ok' => true, 'mail' => $sent]);
