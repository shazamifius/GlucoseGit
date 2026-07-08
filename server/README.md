# Glucose — serveur de télémétrie (auto-hébergé, NixOS)

Reçoit les statistiques **anonymes et opt-in** envoyées par l'app Glucose et les
affiche dans un tableau de bord. **Python 3 stdlib uniquement**, aucune dépendance.

**Un seul port (24100)**, tout passe par le chemin :
- `POST /ingest` — lots envoyés par l'app, protégés par une clé **écriture-seule**
  (`X-Glucose-Key`). La fuiter ne permet que d'*envoyer* des logs, jamais d'en *lire*.
- `GET /health` — sonde de disponibilité (« ok »).
- `GET /` — tableau de bord, protégé par **mot de passe** (HTTP Basic, `GLUCOSE_DASH_PASS`).

> Pourquoi un seul port ? Sur la box cible, 24001 est déjà pris (web3game) et seul un
> port libre + ouvrable au firewall est dispo. Ingest et dashboard cohabitent donc
> sur 24100, distingués par le chemin + une auth différente.

L'app n'envoie **que des métriques** (FPS, gels, GPU, OS/RAM/cœurs, erreurs, version)
— **jamais** le contenu d'un document. Stockage : un fichier SQLite.

---

## Déploiement effectif (service systemd --user, sans sudo)

C'est ainsi que le serveur tourne actuellement sur la box (`shaza@nixos`) :

1. `python3` est installé dans le profil user via `nix-env -f '<nixpkgs>' -iA python3`
   (chemin stable `~/.nix-profile/bin/python3`, gc-rooté, pas de sudo).
2. `telemetry_server.py` est dans `~/glucose-telemetry/`.
3. Un service **user** `~/.config/systemd/user/glucose-telemetry.service` le lance
   (le *linger* utilisateur est activé → il survit à la déconnexion).

```bash
systemctl --user status glucose-telemetry     # état
systemctl --user restart glucose-telemetry     # après un changement d'env/mot de passe
journalctl --user -u glucose-telemetry -f      # logs en direct
```

Le script de déploiement (`scratchpad/deploy.sh`) automatise tout ça.

## ⚠ La seule étape sudo : ouvrir 24100 au firewall

Le pare-feu NixOS n'ouvre que 24001 par défaut. Pour rendre l'ingestion joignable
depuis l'extérieur, il faut ajouter 24100 à `networking.firewall.allowedTCPPorts`
dans `/etc/nixos/configuration.nix` puis `sudo nixos-rebuild switch`. Le port doit
aussi être **forwardé sur le routeur** vers `192.168.1.200:24100`.

## Clé d'ingestion (doit correspondre à l'app)

L'app embarque `TELEMETRY_INGEST_KEY` (dans `src-tauri/src/lib.rs`). Le serveur doit
utiliser la **même** valeur via `GLUCOSE_INGEST_KEY` (c'est le défaut du script). Si
tu la changes, change-la **des deux côtés**.

## Variables d'environnement

| Variable | Rôle | Défaut |
|---|---|---|
| `GLUCOSE_PORT` | port d'écoute | `24100` |
| `GLUCOSE_BIND` | interface (`127.0.0.1` pour tester en local) | `0.0.0.0` |
| `GLUCOSE_DB` | fichier SQLite | `./glucose_telemetry.db` |
| `GLUCOSE_INGEST_KEY` | clé attendue des clients | clé par défaut de l'app |
| `GLUCOSE_DASH_USER` | identifiant du dashboard | `glucose` |
| `GLUCOSE_DASH_PASS` | mot de passe du dashboard (vide = dashboard désactivé) | *(vide)* |

## (Optionnel) HTTPS via reverse-proxy

L'ingestion est en **HTTP clair** (métriques anonymes). Avec un **domaine**, on peut
chiffrer via Caddy (TLS auto) en `reverse_proxy localhost:24100`, puis passer
`TELEMETRY_URL` de l'app en `https://…`. Sur une IP publique nue (sans domaine),
Let's Encrypt ne peut pas émettre de certificat → on reste en HTTP.

## Schéma stocké

Une ligne par événement dans `events` :
`received_at, cid (anonyme), session, os, arch, app_version, kind, ts, data (JSON)`.
`kind` ∈ `session_start` (matériel + GPU) · `perf` (FPS/gels) · `error` · `event`.
