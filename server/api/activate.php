<?php
// Registra il programma su questo computer: {key, machine, label, version} -> {cert}
require __DIR__ . '/../inc/lib.php';
$j = read_json();
$cert = activate((string)($j['key'] ?? ''), (string)($j['machine'] ?? ''), (string)($j['label'] ?? ''), (string)($j['version'] ?? ''), true);
out(200, ['cert' => $cert]);
