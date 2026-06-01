# Contribuer à Glucose

Merci de l'intérêt que tu portes à Glucose ! Les contributions — code, idées,
rapports de bugs, retours — sont les bienvenues.

## 🐛 Signaler un bug / proposer une idée

- Un **bug** ? Ouvre une [issue](../../issues) avec : ce que tu faisais, ce qui
  était attendu, ce qui s'est passé, et ta plateforme (OS + version).
- Une **idée** ou une question ouverte ? Lance une
  [Discussion](../../discussions) — c'est l'endroit pour débattre avant de coder.

## 🛠️ Mettre en place l'environnement de dev

**Prérequis :**
- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) (toolchain stable)
- Les [dépendances système Tauri](https://tauri.app/start/prerequisites/) pour ta plateforme

**Lancer en local :**

```bash
npm install
npm run tauri dev      # lance l'app desktop (hot-reload du frontend)
```

## ✅ Avant d'ouvrir une Pull Request

La CI vérifie ces trois points — fais-les tourner en local d'abord :

```bash
npm run typecheck      # tsc --noEmit, zéro erreur
npm run lint           # biome
npm test               # vitest — la suite doit rester verte
```

**Règles simples :**
- Une PR = un sujet. Décris le *pourquoi*, pas seulement le *quoi*.
- Tout nouveau comportement non trivial doit être **couvert par un test** (c'est
  ce qui garde le projet stable et non-régressable).
- Garde le style du code environnant (le `domain core` — `store/`, `types/` — ne
  doit pas importer PixiJS/Tauri/React).

## 🏗️ Repères d'architecture

- **Frontend** : React 19 + PixiJS 8 (canvas) + Zustand (état) + Automerge (CRDT, source de vérité).
- **Backend** : Rust via Tauri 2 (accès disque, scan de dossiers, lancement de fichiers).
- Le store (`src/store/`) est la source de vérité ; les mutations passent par
  `mutate` (annulable) ou `mutateView` (état de vue, non annulable).

## 📜 Code de conduite

Ce projet suit un [Code de conduite](CODE_OF_CONDUCT.md). En participant, tu
t'engages à le respecter.
