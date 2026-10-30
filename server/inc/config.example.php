<?php
// Copia questo file in config.php (stessa cartella) e compila i valori. config.php NON va mai messo nel repository.
return [
    // database MySQL creato dal pannello Aruba
    'db_host' => 'localhost',
    'db_name' => '',
    'db_user' => '',
    'db_pass' => '',

    // chiave privata di firma (Ed25519): 128 caratteri esadecimali, generata da tools/gen_license_keys.py.
    // Chi la possiede puo' fabbricare licenze valide: non va a nessun altro, ne' in git, ne' in email.
    'sign_key_hex' => '',

    // password del pannello admin: genera l'hash con  php -r "echo password_hash('la tua password', PASSWORD_DEFAULT);"
    // oppure dal pannello stesso (admin/hash.php, poi cancellalo)
    'admin_hash' => '',

    // segreto condiviso con la piattaforma di vendita (firma HMAC dei webhook)
    'webhook_secret' => '',

    // mittente delle email con la chiave (un indirizzo del tuo dominio)
    'mail_from' => 'licenze@example.com',
    'mail_name' => 'SpinLab',

    'default_max_activations' => 2,
    // validita' del certificato: dopo questi giorni il programma deve ricontrollare online (poi c'e' la tolleranza sotto)
    'cert_days' => 30,
    'grace_days' => 14,
];
