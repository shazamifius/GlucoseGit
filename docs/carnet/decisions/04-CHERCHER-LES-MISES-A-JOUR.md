# 04 — Chercher les mises à jour : une requête de fond, une seule

> La note [`02`](02-TELECHARGER-CE-QU-ON-DEPOSE.md) promettait « aucune requête de fond ». La
> mise à jour en fait une, et cette note le dit plutôt que de laisser la promesse mentir. Elle
> n'ajoute **aucune caisse**.
>
> **Date** : 2026-09-29 · **Portée** : `crates/glucose-desktop`, `mise_a_jour::cycle` ·
> **Fiche** : [`48`](../48-LA-MISE-A-JOUR-ET-LA-BASCULE.md).

---

## La décision, et elle est de l'utilisateur

Le plan qu'il a approuvé (fiche [`46`](../46-LA-SUITE-DANS-L-ORDRE.md) § 3 B) : la mise à jour,
puis la bascule de ses cinq utilisateurs de Glucose Tauri — *« par le popup habituel »*. Glucose
Tauri cherchait déjà sa mise à jour à chaque lancement (`UpdatePrompt.tsx`, `check()`), au même
fichier. Rien ne change donc pour eux : c'est la même question, posée au même endroit.

## Ce que Glucose demande, à qui, et quand

* **Une requête `GET`** du fichier des versions,
  `https://github.com/shazamifius/GlucoseGit/releases/latest/download/latest.json` — que GitHub
  redirige vers son stockage. Rien n'est envoyé d'autre que ce qu'une requête porte : l'adresse
  IP de la machine, et le nom sous lequel Glucose se présente (celui d'un navigateur, suivi de
  « Glucose », note 02).
* **Une fois par lancement**, sur un fil à part : le lancement ne l'attend pas.
* **Après une session qui a mal fini, avant tout le reste** — avant la fenêtre : une version qui
  tombe au démarrage doit pouvoir recevoir sa correction. **Le prix** : ce lancement-là attend la
  réponse. Hors ligne, elle est immédiate ; sur un réseau qui ne répond pas, elle est bornée par
  les délais de WinHTTP (quinze secondes par étape, note 02). C'est le seul cas où Glucose attend
  le réseau.
* **L'installeur, seulement si l'utilisateur dit oui** — et sa signature est vérifiée **avant**
  que le moindre octet touche le disque. Au plus 256 Mo, la borne des dépôts.

Aucune télémétrie, aucun identifiant, aucun autre serveur. La boîte noire qui voyage (fiche 46
§ 3 C) sera une autre décision, avec le consentement de chacun.

## Une construction d'épreuve ne se convainc pas à l'exécution

L'adresse et la clé se fixent **à la compilation** (`GLUCOSE_MISE_A_JOUR`,
`GLUCOSE_CLE_PUBLIQUE`) : la vérification de GitHub y met un serveur local et une clé d'essai.
Un programme installé ne lit rien de tel dans son environnement : personne ne peut lui faire
accepter une autre clé en changeant une variable.

## La voie réseau, par système

La fiche 46 § 3 B demandait **une seule bibliothèque réseau pour tout**. La porte est unique —
`plateforme::telecharger(url, limite)`, par où passent le dépôt d'une image et la mise à jour —,
la voie dépend du système :

* **Windows : WinHTTP**, déjà là (note 02) — les certificats, le proxy et les correctifs de
  Windows. Aucune raison de lui préférer une pile embarquée.
* **Ailleurs** : `ureq` avec `rustls` et les racines de Mozilla — tranché le même soir, note
  [`05`](05-LE-RESEAU-HORS-DE-WINDOWS.md) : sous Linux, il n'y a pas de pile du système comme
  sous Windows.

## Comment on s'en défait

Tout le réseau de la mise à jour tient dans `mise_a_jour::cycle` : `chercher`, `preparer`. Ne
plus chercher, c'est ne plus créer la `Veille` dans `main.rs` et retirer l'appel
`mettre_a_jour_avant_tout` : deux lignes.

---

**Retour** : [`00-INDEX.md`](../00-INDEX.md)
