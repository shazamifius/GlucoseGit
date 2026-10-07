# 09 — Le partage vers Glucose, par `jni`

> Sa priorité du 08/10 (fiche 55 § 4) : qu'une image partagée depuis Pinterest, la galerie ou
> un navigateur arrive dans Glucose. Cette note dit la seule caisse que cela nomme — **aucune
> nouvelle dans l'arbre** — et pourquoi la frontière avec Java passe là où elle passe.
>
> **Date** : 2026-10-08 · **Portée** : `crates/glucose-android`, `android/` (le manifeste,
> `MainActivity.java`), `glucose-desktop::plateforme::partage` · **Fiche** : [`56`](../56-LE-PARTAGE-ET-LA-VIE-DE-L-APPLICATION.md).

---

## Ce qu'il faut

* **`jni` 0.22**, dans `glucose-android` seulement : la caisse que `android-activity` tire
  déjà, à la même version (`cargo tree -i jni` : `jni v0.22.4 ← android-activity ← winit`).
  Elle copie trois tableaux et une chaîne de Java vers Rust, une fois par partage, et protège
  la frontière d'une panique (`with_env` l'attrape : une panique qui traverse vers la machine
  Java tue le processus).

## Où passe la frontière

Java fait **ce que seul Java peut faire** : recevoir l'intention (`onCreate`, `onNewIntent`),
et ouvrir les adresses `content://` que l'application qui partage a prêtées. Il ouvre chaque
fichier comme `ContentResolver.openInputStream` l'ouvrirait — par `openAssetFileDescriptor`,
vérifié dans le code d'Android — et confie à Rust **le descripteur**, son début et sa longueur.

Rust lit, borne, et décide : ce qu'est une image (l'en-tête des octets), ce qu'est une adresse,
où le partage se pose. Le tout suit **le chemin d'un dépôt** — un geste, une entrée
d'annulation, un compte-rendu — et un lien part au rapatriement qui sait déjà trouver l'image
d'une épingle.

## Pourquoi pas autre chose

| | ce que ça donne | pourquoi non |
|---|---|---|
| Java lit les octets et passe un `byte[]` | aucune lecture en Rust | l'image traverse deux fois la mémoire, et le tas de Java d'un vieux téléphone est petit ; 256 Mo d'image le feraient tomber |
| Rust ouvre les adresses par JNI | aucune ligne de Java | une quarantaine d'appels réflexifs (`getContentResolver`, `Uri.parse`, `openAssetFileDescriptor`…) qu'aucun compilateur ne vérifie, au lieu de dix lignes de Java qu'il vérifie |
| Java écrit un fichier, Rust le relève au retour | aucune caisse | il faudrait surveiller un dossier, ou deviner le retour : « rien ne tourne au repos » l'interdit, et un partage qui lance Glucose arrive avant sa fenêtre |
| `openFileDescriptor` | un descripteur plus simple | refuse un fichier qui n'est qu'un morceau (`Not a whole file`, dans le code d'Android) : un partage que `openInputStream` lirait serait perdu |

## Ce qui reste vrai

* **Un partage arrivé avant la fenêtre l'attend** (la boîte aux lettres de `partage`), et un
  partage vers une activité fermée attend la suivante, pas la précédente (`Branchement`).
* **Une intention rejouée ne se repose pas** : recréée par le système, ou rouverte depuis les
  applications récentes (`FLAG_ACTIVITY_LAUNCHED_FROM_HISTORY`), l'activité reçoit l'intention
  d'origine une seconde fois — elle l'ignore.
* **Rien d'illisible ne disparaît en silence** : un fichier que le système n'a pas laissé
  ouvrir se compte dans le compte-rendu ; un texte sans lien le dit.

## Sources

* Android, [*Receive simple data from other apps*](https://developer.android.com/training/sharing/receive)
  — les filtres `SEND` et `SEND_MULTIPLE`, `EXTRA_STREAM`, `EXTRA_TEXT` ; ne pas croire le
  type annoncé ; lire hors du fil de l'interface.
* AOSP, `core/java/android/content/ContentResolver.java` — `openInputStream` passe par
  `openAssetFileDescriptor` ; `openFileDescriptor` refuse un morceau de fichier.
* `jni` 0.22.4, sa documentation (`EnvUnowned::with_env`, `ErrorPolicy`).
* Sur l'intention rejouée : le suivi d'Android ([132754763](https://issuetracker.google.com/issues/132754763))
  et la documentation de [l'écran des applications récentes](https://developer.android.com/guide/components/recents).
