-- Tabelle del server licenze (MySQL / MariaDB). Si importa una volta da phpMyAdmin.
CREATE TABLE IF NOT EXISTS licenses (
  id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  license_key VARCHAR(32) NOT NULL UNIQUE,
  email VARCHAR(190) NOT NULL DEFAULT '',
  name VARCHAR(190) NOT NULL DEFAULT '',
  max_activations TINYINT UNSIGNED NOT NULL DEFAULT 2,
  status ENUM('active','revoked') NOT NULL DEFAULT 'active',
  source VARCHAR(40) NOT NULL DEFAULT 'manuale',
  order_ref VARCHAR(120) NOT NULL DEFAULT '',
  notes VARCHAR(500) NOT NULL DEFAULT '',
  created_at DATETIME NOT NULL,
  INDEX (email),
  INDEX (order_ref)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS activations (
  id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  license_id INT UNSIGNED NOT NULL,
  machine VARCHAR(64) NOT NULL,
  label VARCHAR(120) NOT NULL DEFAULT '',
  app_version VARCHAR(20) NOT NULL DEFAULT '',
  first_seen DATETIME NOT NULL,
  last_seen DATETIME NOT NULL,
  ip VARCHAR(45) NOT NULL DEFAULT '',
  UNIQUE KEY lic_machine (license_id, machine),
  FOREIGN KEY (license_id) REFERENCES licenses(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- tentativi falliti per IP, per rallentare chi prova chiavi a caso
CREATE TABLE IF NOT EXISTS attempts (
  id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  ip VARCHAR(45) NOT NULL,
  at DATETIME NOT NULL,
  INDEX (ip, at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
