# 03 — Lire les documents de Glucose Tauri : notre lecteur, et deux oracles

> La fiche [`05`](../05-STANDARDS-DE-CODE.md) § 8.2 exige une note par dépendance. Celle-ci
> n'ajoute **aucune caisse à l'application** : elle explique pourquoi la bibliothèque qui
> savait déjà lire ces fichiers n'y entre pas, et ce qu'elle fait à la place — servir
> d'oracle dans les épreuves.
>
> **Date** : 2026-09-24 · **Portée** : `glucose-core::persist::tauri`,
> `glucose-desktop::tauri`, et les `[dev-dependencies]` de `glucose-desktop`.

---

## Le besoin

Un utilisateur de Glucose Tauri qui passe à Glucose Rust doit retrouver ses documents. Ils
sont au format binaire d'**Automerge** depuis juillet 2026 (des morceaux, des colonnes
compressées, l'histoire entière des opérations), en JSON pour les plus anciens.

## Ce qui a été essayé, et mesuré sur ses fichiers

| | lit ses 4 documents Automerge | ce qu'il coûte à l'application |
|---|---|---|
| la caisse `automerge` 0.12 | oui, en 0,6 à 104 ms | **24 caisses** de plus (dont un second zlib, sha2, serde), pour lire un ancien fichier une fois |
| **notre lecteur** (`persist::tauri`) | oui, **identique arbre pour arbre** | **aucune caisse** — 2 506 lignes de noyau commentaires compris (DEFLATE, JSON et traduction en projet avec), 86 dans le bureau |

Ce qu'il faut pour tirer l'**état courant** d'un document tient dans la spécification publique
(<https://automerge.org/automerge-binary-format-spec/>) : des morceaux, des colonnes codées
par plages, et deux règles — une valeur est visible tant que rien ne l'a remplacée, et entre
deux valeurs visibles la plus grande horloge de Lamport l'emporte ; l'ordre d'une liste suit
l'arbre RGA. L'histoire elle-même n'est pas gardée.

## Les deux oracles, dans les épreuves seulement

| `[dev-dependencies]` | ce qu'elle prouve |
|---|---|
| `automerge = "0.12"` | notre lecteur rend la **même valeur** que la référence : sur les vrais fichiers (`examples/oracle_tauri.rs`), et sur des documents fabriqués qui exercent chaque règle — frères RGA, conflits, deltas ajoutés, deltas compressés, compteurs, fin tronquée (`tests/tauri_suite.rs`) |
| `miniz_oxide = "0.8"` | notre DEFLATE (`persist::tauri::inflate`) rend à l'octet près ce qu'elle compresse, aux onze niveaux. **Aucune caisse de plus** : `png` la tire déjà à cette version |

Chaque règle a été sabotée une à une : dix sabotages, dix épreuves qui tombent (fiche 37 § 2).

## Ce qu'un fichier forgé ne peut pas faire

Aucune profondeur maximale n'est choisie, parce qu'aucune n'est nécessaire : le JSON se lit
sans récursion, l'état Automerge se construit sans récursion, et une [`Valeur`] se détruit
sans récursion — cent mille niveaux d'imbrication se lisent et se libèrent. Les colonnes se
lisent paresseusement : une plage qui annonce 2⁶⁰ valeurs ne déplie rien. Le nombre
d'opérations annoncé se réserve par `try_reserve`, qui refuse au lieu d'avorter. Un nom
`asset:../../x` ne sort pas du magasin de Tauri.

## Combien de lignes pour le remplacer

Rien à remplacer : c'est le remplacement. Si un jour il fallait lire l'**histoire** d'un
document Tauri — et non son seul état —, la caisse `automerge` redeviendrait la bonne
réponse, et cette note serait à réécrire.

---

**Retour** : [`00-INDEX.md`](../00-INDEX.md)
