# 05 — Le réseau hors de Windows : `ureq` et `rustls`

> La fiche [`05`](../05-STANDARDS-DE-CODE.md) § 8.2 exige une note par dépendance. Celle-ci en
> ajoute une, **hors de Windows seulement** : Windows garde WinHTTP
> ([`decisions/02`](02-TELECHARGER-CE-QU-ON-DEPOSE.md)).
>
> **Date** : 2026-09-29 · **Portée** : `crates/glucose-desktop`, cible `cfg(not(windows))` ·
> **Fiche** : [`48`](../48-LA-MISE-A-JOUR-ET-LA-BASCULE.md).

---

## Pourquoi maintenant

Ses utilisateurs de Glucose Tauri sont sous Windows **et Linux**. Basculés vers Glucose Rust,
ceux de Linux doivent pouvoir recevoir la version suivante — sans quoi la bascule les laisserait
sur une version qui ne se met plus à jour. (Le dépôt d'une image depuis un navigateur, lui, ne
reçoit d'adresses que par le pont natif de Windows : hors de Windows, rien ne change pour lui.)

## Ce qu'on prend

| Ce qu'on prend | Pourquoi |
|---|---|
| `ureq` 3.4, `default-features = false`, `features = ["rustls"]` | un client HTTP bloquant, sans exécuteur asynchrone : le téléchargement vit déjà sur un fil à part |
| — dont `rustls` et son fournisseur `ring` | TLS écrit en Rust : aucune bibliothèque du système à trouver, dans aucune version |
| — dont `webpki-roots` | les racines de confiance de Mozilla, qui voyagent avec le programme ; **les mises à jour les renouvellent** |

**Laissé de côté** : la décompression (`gzip`) — un installeur et un `latest.json` ne se
compressent pas en route ; les cookies, le JSON, les proxys SOCKS.

## Pourquoi pas la pile du système

Sous Linux, il n'y en a pas une : chaque distribution livre sa bibliothèque TLS (OpenSSL,
GnuTLS…), dans sa version, et un AppImage ne peut compter sur aucune. `native-tls` lierait
OpenSSL — une version par distribution, et l'AppImage casserait sur la moitié d'entre elles.
`reqwest` tirerait un exécuteur asynchrone et une centaine de caisses.

## Ce qu'il tire transitivement

Seize caisses, lues dans `Cargo.lock` : `ureq`, `ureq-proto`, `http`, `httparse`, `base64`,
`utf8-zero`, `rustls`, `rustls-pki-types`, `rustls-webpki`, `webpki-roots`, `ring`,
`untrusted`, `getrandom`, `wasi`, `subtle`, `zeroize`. **Aucune sous Windows** : la
dépendance est déclarée pour `cfg(not(windows))`.

## Comment on s'en défait

Tout tient dans `plateforme/telechargement.rs`, derrière `plateforme::telecharger(url,
limite)` — la même porte que WinHTTP. Le jour où un système donne mieux (macOS et son
`NSURLSession`), il prend sa voie au même endroit, et aucun appelant ne change.

## Ce qui a été vérifié, et comment

Les épreuves de bout en bout de la mise à jour (`mise_a_jour::cycle::tests::de_bout_en_bout`)
tournent sur **chaque** système de la vérification GitHub : un vrai serveur HTTP local sert le
fichier des versions et un installeur signé ; le téléchargeur du système les rapporte, la
signature se vérifie, l'installeur se pose — ou, altéré, ne touche jamais le disque.

---

**Retour** : [`00-INDEX.md`](../00-INDEX.md)
