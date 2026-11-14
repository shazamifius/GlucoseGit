# 16 — Le WebSocket de la porte publique : `rustls` et `webpki-roots` nommés, WinHTTP sous Windows

> La fiche [`05`](../05-STANDARDS-DE-CODE.md) § 8.2 exige une note par dépendance. Celle-ci n'en
> ajoute **aucune** au programme compilé : elle nomme deux caisses déjà là par `ureq`
> ([`decisions/05`](05-LE-RESEAU-HORS-DE-WINDOWS.md)), et la caisse `windows` nomme un recoin de
> plus (`Win32_Security_Cryptography`, pour le hasard du système).
>
> **Date** : 2026-11-14 · **Portée** : `crates/glucose-desktop`, `plateforme/websocket*`,
> `plateforme/hasard.rs` · **Fiche** : [`116`](../116-LA-PORTE-PUBLIQUE.md).

---

## Pourquoi

Une IA du nuage (Claude.ai, ChatGPT) ne parle qu'à une adresse `https://`. Glucose, derrière la
box de l'utilisateur, n'en a pas : il se relie donc **lui-même** à son relais, par un WebSocket
sortant (RFC 6455) que le relais garde en sommeil sans rien payer (fiche 116 § 2). `ureq` ne sait
pas passer une connexion en WebSocket ; il fallait écrire la connexion.

## Ce qu'on prend

| Ce qu'on prend | Pourquoi |
|---|---|
| `rustls` 0.23, `default-features = false`, `["std", "tls12", "ring"]` — **hors de Windows** | les fonctions mêmes qu'`ureq` y allume : rien de plus ne se compile (`Cargo.lock` ne gagne que deux liens, aucune caisse) |
| `webpki-roots` 1 — **hors de Windows** | les racines de Mozilla, déjà là par `ureq` |
| `windows`, fonction `Win32_Security_Cryptography` | `BCryptGenRandom` : le hasard des clés, sous Windows |

**Écrit ici, sans caisse** : les trames et la poignée de main du WebSocket
(`plateforme/websocket/trames.rs`, pur, éprouvé octet par octet), SHA-1 pour l'acceptation que la
norme exige (`glucose-core/src/hash.rs`), le base64 dans les deux sens (`glucose-core/src/encodage.rs`,
où le décodeur de Tauri a déménagé), le hasard du système (`plateforme/hasard.rs` : `/dev/urandom`,
`BCryptGenRandom`).

**Écarté** : `tungstenite` (huit caisses de plus : `http`, `sha1`, `rand`, `thiserror`, `utf-8`,
`data-encoding`, `bytes`, `log`) pour deux cents lignes qu'on éprouve mieux soi-même ; `rustls`
sous Windows, qui perdrait le proxy et les certificats du système — une école qui filtre sa
sortie (le PC de l'école est sous Windows).

## Sous Windows

WinHTTP sait le WebSocket depuis Windows 8 (`WinHttpWebSocketCompleteUpgrade`), avec le proxy et
les certificats du système : la voie de Windows ne compile ni `rustls` ni les trames. Un fil reçoit
sans fin, l'autre envoie (WinHTTP admet une réception et un envoi en cours en même temps) ; la
liaison partagée se ferme quand le dernier des deux la lâche (`plateforme/websocket/windows.rs`).

## Comment on s'en défait

Tout passe par `plateforme::websocket::ouvrir(url, en-têtes) -> Box<dyn Canal>` : deux gestes,
envoyer et recevoir avec un délai. Une autre voie se pose derrière, sans qu'aucun appelant change.

## Ce qui a été vérifié, et comment

`plateforme::websocket::tests::un_websocket_s_ouvre_parle_et_se_ferme_contre_un_relais_local` ouvre
**la voie du système** — WinHTTP sur la vérification GitHub de Windows, la nôtre ailleurs — contre un
relais écrit dans l'épreuve : l'autorisation présentée, un message court et un de 320 Ko, le ping
répondu seul, la fermeture du relais vue comme une erreur. Les trames sont éprouvées dans les deux
sens à chaque frontière de longueur (125, 126, 65 535, 65 536), poussées un octet à la fois ; les
violations du protocole (masque contraire, bits réservés, contrôle trop long, texte non UTF-8)
ferment la connexion.

---

**Retour** : [`00-INDEX.md`](../00-INDEX.md)
