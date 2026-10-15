//! **Les raccourcis comme donnée** (P4, fiche 72) : l'interface devient une donnée.
//!
//! # Pourquoi
//!
//! Sa direction (VISION) : un preset est **l'interface entière** — la disposition, le
//! comportement, les couleurs, les raccourcis —, et les presets sont des **couches** qu'on
//! mélange : les raccourcis de Figma sur l'allure de Glucose. Rien de cela n'existe tant que
//! chaque raccourci est écrit dans le code du geste qu'il déclenche. Ce module dit les
//! raccourcis **en données**, et rien d'autre : il ne sait pas ce que fait une commande, il la
//! nomme. L'application tient la table des commandes ; le noyau, la loi des touches.
//!
//! # Comment les meilleurs font (la recherche du 15/10/2026)
//!
//! * **VS Code** : chaque commande a un identifiant ; un raccourci est `{ key, command, when }`,
//!   les raccourcis de l'utilisateur se posent sur ceux par défaut, et `-commande` en retire un ;
//!   les jeux de raccourcis (Sublime, Vim, IntelliJ) sont des paquets de raccourcis.
//! * **Zed** : `keymap.json`, une liste de liaisons par contexte ; un `base_keymap` (VS Code,
//!   JetBrains, Sublime, Emacs…) sous les liaisons de l'utilisateur, qui l'emportent ; `null`
//!   retire une liaison.
//! * **Blender** : une *keyconfig* est une donnée — des *keymaps*, chacune une liste
//!   `(opérateur, évènement, propriétés)` ; « Industry Compatible » est une autre keyconfig,
//!   générée par le même code.
//! * **Krita** : des schémas de raccourcis (`.shortcuts`, une action par ligne) — Photoshop,
//!   Paint Tool SAI —, à part des espaces de travail.
//! * **PureRef 2** : chaque commande dans une palette qui la cherche, des raccourcis entièrement
//!   personnalisables, les réglages qu'on exporte.
//!
//! # Ce qu'on en prend, et ce qu'on change
//!
//! * Une **couche** est un texte, une liaison par ligne — `Ctrl+Maj+Z  retablir` —, comme les
//!   `.shortcuts` de Krita : lisible et modifiable par un inconnu qui forke, sans dépendance.
//! * Un accord désigne une **liste ordonnée** de commandes : la première qui s'applique
//!   l'emporte. C'est l'arbitrage qu'avaient déjà les chiffres de Glucose — une flèche en main,
//!   `1` la qualifie ; les mains vides, il vole au premier signet —, dit ici une fois, au lieu
//!   d'un `when` à évaluer (VS Code) : chaque commande sait si elle s'applique.
//! * Une couche du dessus **remplace** la liste d'une couche du dessous pour cet accord, ou la
//!   **retire** (`-`), comme le `null` de Zed et le `-commande` de VS Code.

mod accord;
mod couche;

pub use accord::{Accord, Nommee, Touche};
pub use couche::{Clavier, Couche, ErreurDeLigne};

#[cfg(test)]
mod tests;
