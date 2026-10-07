-- La base de la boîte noire qui voyage (fiche 54) : une ligne par session reçue.
-- Aucune colonne pour une adresse IP, ni pour une heure : le jour suffit.
CREATE TABLE IF NOT EXISTS sessions (
  installation TEXT NOT NULL,
  session      TEXT NOT NULL,
  recue        TEXT NOT NULL,
  version      TEXT NOT NULL,
  systeme      TEXT NOT NULL,
  architecture TEXT NOT NULL,
  lignes       TEXT NOT NULL,
  PRIMARY KEY (installation, session)
);
CREATE INDEX IF NOT EXISTS sessions_par_jour ON sessions (recue);
