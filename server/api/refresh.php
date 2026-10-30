<?php
// Rinnova il certificato di un computer gia' registrato (controllo periodico): {key, machine, version} -> {cert}
// Se la licenza e' stata revocata o il computer disattivato risponde con un errore e il programma si de-registra.
require __DIR__ . '/../inc/lib.php';
$j = read_json();
$cert = activate((string)($j['key'] ?? ''), (string)($j['machine'] ?? ''), (string)($j['label'] ?? ''), (string)($j['version'] ?? ''), false);
out(200, ['cert' => $cert]);
