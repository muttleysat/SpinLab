<?php
// Pannello di gestione delle licenze. Accesso con la password di config.php ('admin_hash').
require __DIR__ . '/../inc/lib.php';
session_set_cookie_params(['httponly' => true, 'samesite' => 'Strict', 'secure' => !empty($_SERVER['HTTPS'])]);
session_start();
header('X-Frame-Options: DENY'); header('Cache-Control: no-store');
$h = fn($s) => htmlspecialchars((string)$s, ENT_QUOTES, 'UTF-8');
$msg = '';

// accesso
if (isset($_POST['password'])) {
    $ok = false;
    try { rate_check(); $ok = password_verify($_POST['password'], cfg()['admin_hash']); if (!$ok) rate_fail(); } catch (Throwable $e) { $msg = 'Errore del server'; }
    if ($ok) { session_regenerate_id(true); $_SESSION['admin'] = 1; $_SESSION['csrf'] = bin2hex(random_bytes(16)); header('Location: ./'); exit; }
    $msg = $msg ?: 'Password errata';
}
if (isset($_GET['esci'])) { session_destroy(); header('Location: ./'); exit; }

if (empty($_SESSION['admin'])) {
    echo '<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>SpinLab licenze</title>'
       . '<body style="font:15px system-ui;background:#0b1018;color:#dbe6f5;display:grid;place-items:center;height:100vh;margin:0"><form method="post" style="display:grid;gap:10px;width:260px">'
       . '<b>SpinLab · licenze</b><input type="password" name="password" placeholder="Password" autofocus style="padding:9px"><button style="padding:9px">Entra</button>'
       . ($msg ? '<span style="color:#ff8a96">' . $h($msg) . '</span>' : '') . '</form>';
    exit;
}

// azioni (POST con token CSRF)
if ($_SERVER['REQUEST_METHOD'] === 'POST' && isset($_POST['act'])) {
    if (!hash_equals($_SESSION['csrf'], (string)($_POST['csrf'] ?? ''))) { http_response_code(403); exit('token'); }
    $db = db(); $id = (int)($_POST['id'] ?? 0);
    switch ($_POST['act']) {
        case 'crea':
            $key = new_license_key();
            $db->prepare('INSERT INTO licenses(license_key, email, name, max_activations, source, notes, created_at) VALUES(?,?,?,?,?,?,?)')
               ->execute([$key, mb_substr(trim($_POST['email'] ?? ''), 0, 190), mb_substr(trim($_POST['name'] ?? ''), 0, 190),
                          max(1, min(20, (int)($_POST['max'] ?? 2))), 'manuale', mb_substr(trim($_POST['notes'] ?? ''), 0, 500), now_sql()]);
            $msg = "Creata: $key";
            if (!empty($_POST['invia']) && send_key_mail(trim($_POST['email'] ?? ''), trim($_POST['name'] ?? ''), $key)) $msg .= ' (email inviata)';
            break;
        case 'revoca':   $db->prepare("UPDATE licenses SET status='revoked' WHERE id=?")->execute([$id]); $msg = 'Licenza revocata'; break;
        case 'riattiva': $db->prepare("UPDATE licenses SET status='active' WHERE id=?")->execute([$id]); $msg = 'Licenza riattivata'; break;
        case 'azzera':   $db->prepare('DELETE FROM activations WHERE license_id=?')->execute([$id]); $msg = 'Computer azzerati'; break;
        case 'max':      $db->prepare('UPDATE licenses SET max_activations=? WHERE id=?')->execute([max(1, min(20, (int)$_POST['max'])), $id]); $msg = 'Aggiornato'; break;
        case 'invia':
            $st = $db->prepare('SELECT * FROM licenses WHERE id=?'); $st->execute([$id]); $l = $st->fetch();
            $msg = $l && send_key_mail($l['email'], $l['name'], $l['license_key']) ? 'Email inviata' : 'Email non inviata';
            break;
        case 'elimina':  $db->prepare('DELETE FROM licenses WHERE id=?')->execute([$id]); $msg = 'Licenza eliminata'; break;
    }
}

$q = trim($_GET['q'] ?? '');
$st = db()->prepare("SELECT l.*, (SELECT COUNT(*) FROM activations a WHERE a.license_id=l.id) AS n FROM licenses l
                     WHERE ?='' OR l.license_key LIKE ? OR l.email LIKE ? OR l.name LIKE ? OR l.order_ref LIKE ? ORDER BY l.id DESC LIMIT 300");
$like = "%$q%"; $st->execute([$q, $like, $like, $like, $like]);
$rows = $st->fetchAll();
$acts = [];
foreach (db()->query('SELECT * FROM activations ORDER BY last_seen DESC') as $a) $acts[$a['license_id']][] = $a;
$tok = $h($_SESSION['csrf']);
$btn = fn($act, $id, $label, $extra = '', $conf = '') => "<form method='post' style='display:inline' " . ($conf ? "onsubmit=\"return confirm('$conf')\"" : '') . "><input type='hidden' name='csrf' value='$tok'><input type='hidden' name='act' value='$act'><input type='hidden' name='id' value='$id'>$extra<button>$label</button></form>";
?>
<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>SpinLab licenze</title>
<style>body{font:14px system-ui;background:#0b1018;color:#dbe6f5;margin:0;padding:18px}table{border-collapse:collapse;width:100%}td,th{border-bottom:1px solid #1d2a3d;padding:7px 8px;text-align:left;vertical-align:top}
input,button{padding:6px 9px;background:#101a2a;color:#dbe6f5;border:1px solid #29405f;border-radius:6px}button{cursor:pointer}code{color:#4fd3ff}.rev{opacity:.5}.msg{background:#0d2a1e;color:#7df0b6;padding:8px 12px;border-radius:6px;margin:10px 0}small{color:#8da2c0}</style>
<h2>SpinLab · licenze <small><a href="?esci=1" style="color:#8da2c0">esci</a></small></h2>
<?php if ($msg) echo "<div class='msg'>" . $h($msg) . "</div>"; ?>
<form method="post" style="display:flex;gap:6px;flex-wrap:wrap;margin-bottom:14px"><input type="hidden" name="csrf" value="<?= $tok ?>"><input type="hidden" name="act" value="crea">
  <input name="name" placeholder="Nome"><input name="email" placeholder="Email" type="email"><input name="max" type="number" min="1" max="20" value="<?= (int)cfg()['default_max_activations'] ?>" style="width:60px" title="Computer">
  <input name="notes" placeholder="Note"><label><input type="checkbox" name="invia" value="1" checked> invia la chiave per email</label><button>Crea licenza</button></form>
<form method="get" style="margin-bottom:10px"><input name="q" value="<?= $h($q) ?>" placeholder="Cerca chiave, email, nome, ordine"><button>Cerca</button></form>
<table><tr><th>Chiave</th><th>Cliente</th><th>Computer</th><th>Stato</th><th>Azioni</th></tr>
<?php foreach ($rows as $r): ?>
<tr class="<?= $r['status'] === 'revoked' ? 'rev' : '' ?>">
  <td><code><?= $h($r['license_key']) ?></code><br><small><?= $h($r['created_at']) ?> · <?= $h($r['source']) ?> <?= $h($r['order_ref']) ?></small></td>
  <td><?= $h($r['name']) ?><br><small><?= $h($r['email']) ?></small><?= $r['notes'] ? '<br><small>' . $h($r['notes']) . '</small>' : '' ?></td>
  <td><?= (int)$r['n'] ?> / <?= (int)$r['max_activations'] ?><?php foreach ($acts[$r['id']] ?? [] as $a) echo '<br><small>' . $h($a['label']) . ' · v' . $h($a['app_version']) . ' · ' . $h($a['last_seen']) . '</small>'; ?></td>
  <td><?= $r['status'] === 'active' ? 'attiva' : 'REVOCATA' ?></td>
  <td><?= $r['status'] === 'active' ? $btn('revoca', $r['id'], 'Revoca', '', 'Revocare la licenza?') : $btn('riattiva', $r['id'], 'Riattiva') ?>
      <?= $btn('azzera', $r['id'], 'Azzera computer', '', 'Liberare tutti i computer di questa licenza?') ?>
      <?= $btn('invia', $r['id'], 'Rimanda email') ?>
      <?= $btn('max', $r['id'], 'Imposta max', '<input name="max" type="number" min="1" max="20" value="' . (int)$r['max_activations'] . '" style="width:56px">') ?>
      <?= $btn('elimina', $r['id'], 'Elimina', '', 'Eliminare definitivamente la licenza?') ?></td></tr>
<?php endforeach; ?></table>
