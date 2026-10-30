<?php
// Libera un posto: {key, machine} -> {ok}. Serve per spostare la licenza su un altro computer.
require __DIR__ . '/../inc/lib.php';
$j = read_json();
rate_check();
$key = norm_key((string)($j['key'] ?? '')); $machine = (string)($j['machine'] ?? '');
if (!valid_key_format($key) || !valid_machine($machine)) { rate_fail(); out(400, ['error' => 'chiave_non_valida']); }
$lic = find_license($key);
if (!$lic) { rate_fail(); out(404, ['error' => 'chiave_sconosciuta']); }
db()->prepare('DELETE FROM activations WHERE license_id=? AND machine=?')->execute([$lic['id'], $machine]);
out(200, ['ok' => true]);
