# 08 — Android par `GameActivity`, et rien de plus

> Glucose sur les téléphones de ses dix testeurs (fiche 50, fiche 54). Cette note dit ce que le
> téléphone ajoute au dépôt — **aucune caisse nouvelle** — et ce qu'il retire, derrière une porte,
> de ce qui n'y compile pas.
>
> **Date** : 2026-10-07 · **Portée** : `crates/glucose-android`, `android/`,
> `outils/android/construire.sh`, `glucose-desktop` (`dialogue`, `presse_papiers`, `demarrage`)
> · **Fiche** : [`54`](../54-LA-NUIT-DU-PAVE-ET-ANDROID.md).

---

## Ce qu'il faut

* **La fonctionnalité `android-game-activity` de `winit`** : la fenêtre, la boucle et les entrées
  passent par `android-activity` 0.6.1, que `winit` tire déjà. `GameActivity` plutôt que
  `NativeActivity` : le clavier virtuel n'y marche que par elle (fiche 50 § 3) — et Glucose est
  fait d'écriture.
* **`androidx.games:games-activity` 4.4.0**, côté Java, la version exacte que réclame
  `android-activity` 0.6.1 (son journal des changements). Avec `appcompat` et `core`, que
  `GameActivity` étend. Ce sont des bibliothèques de Google, pas des caisses : elles vivent dans
  l'APK, pas dans le programme de bureau.
* **`libc`** dans la caisse du téléphone, déjà présente pour Linux et Android : le tube qui
  porte la sortie de Glucose vers le journal d'Android (`liblog`, dans chaque Android).
* **Gradle 9.8 et le plugin Android 9.4**, pour emballer l'APK ; **`cargo-ndk`** pour
  construire la bibliothèque Rust avec le NDK 29. Des outils de construction, sur la machine
  qui construit — jamais dans ce que reçoit l'utilisateur.

## Ce qui se retire derrière une porte

| | pourquoi | ce que le téléphone fait à la place |
|---|---|---|
| `rfd` | aucune voie Android | `dialogue` répond « rien » ; les sélecteurs du système viendront par JNI |
| `arboard` | aucune voie Android | le presse-papiers de Glucose seul : copier-coller entre ses nœuds |

Le **cliquet 12** tient ces deux caisses derrière leur porte : un emploi ailleurs casserait la
construction du téléphone, que seule la CI ferait voir.

## Pourquoi pas autre chose

| | ce que ça donne | pourquoi non |
|---|---|---|
| `NativeActivity` | aucune ligne de Java | le clavier virtuel ne vient pas (issue 44 d'`android-activity`) |
| `xbuild`, `cargo-apk` | un APK sans Gradle | `cargo-apk` est abandonné ; `GameActivity` demande les bibliothèques AndroidX, que seul Gradle assemble proprement |
| une interface Java ou Kotlin | des contrôles natifs | deux interfaces à tenir ; Glucose dessine tout lui-même, partout |

## Ce qui reste vrai

* **Le plancher : Android 5.0 (API 21)**, celui de Rust. `wgpu` prend Vulkan où il existe,
  OpenGL ES ailleurs : aucun téléphone exclu au-dessus.
* **Le bureau ne change pas** : `glucose-android` est vide hors d'Android.

## Sources

* `android-activity`, [CHANGELOG](https://github.com/rust-mobile/android-activity/blob/main/android-activity/CHANGELOG.md)
  — 0.6.1, `GameActivity` 4.4.0.
* Android, [*GameActivity — get started*](https://developer.android.com/games/agdk/game-activity/get-started).
* Android, [*About the Android Gradle plugin*](https://developer.android.com/build/releases/about-agp)
  — le plugin 9.4 demande Gradle 9.6 ou plus.
