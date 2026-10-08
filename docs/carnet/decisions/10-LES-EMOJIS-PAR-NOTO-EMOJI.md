# 10 — Les emojis, par Noto Emoji monochrome

> Son accord du 08/10 au soir (fiche 57 § 6) : un emoji tapé au clavier du téléphone
> s'affichait comme une case vide — Inter n'en a pas le dessin, et Glucose n'avait aucune police
> de repli. Cette note dit la police ajoutée, **aucune caisse nouvelle**, et pourquoi celle-là.
>
> **Date** : 2026-10-08 · **Portée** : `crates/glucose-desktop/assets/NotoEmoji-Regular.ttf`,
> `typography.rs` · **Fiche** : [`58`](../58-LES-QUESTIONS-DESSINEES-LE-RAIL-ET-LES-EMOJIS.md).

---

## Ce qu'il faut

* **Noto Emoji**, la version monochrome de Google (SIL Open Font License 1.1, texte dans
  `assets/LICENSE-NotoEmoji.txt`), version 3.002 — prise à la source, le dépôt de Google Fonts
  (`ofl/notoemoji/NotoEmoji[wght].ttf`, 1 982 596 octets, empreinte SHA-256
  `de6c1883…d27551`).
* **Son instance fixe « Regular »** (graisse 400), tirée une fois par `fontTools` (`varLib.instancer
  … wght=400`) : **886 628 octets** au lieu de 1,98 Mo. `fontdue` ne lit que l'instance par défaut
  d'une police variable : les tables de variation seraient du poids mort.
* **Ce qu'elle couvre** (lu dans sa table `cmap`) : 1 489 caractères, jusqu'à Unicode 15
  (U+1FAE8, le visage qui tremble). Le sélecteur de variante (U+FE0F) et le joint (U+200D) y ont
  une avance nulle et aucun contour : ils ne dessinent rien, comme il se doit.

## Comment elle sert

**En repli, et seulement en repli** : un caractère qu'Inter (ou JetBrains Mono) dessine reste à
lui ; un caractère qu'il n'a pas, et que Noto Emoji a, se dessine en Noto Emoji. Aucune table
écrite à la main : c'est la `cmap` des deux polices qui décide, caractère par caractère.

## Pourquoi pas autre chose

| | ce que ça donne | pourquoi non |
|---|---|---|
| **Noto Color Emoji** | les emojis en couleur | 10 Mo de bitmaps (CBDT) ou des calques (COLRv1) que `fontdue` ne sait pas dessiner ; des bitmaps flous dès qu'on zoome — un canevas infini zoome |
| **La police du système** (Segoe UI Emoji, celle d'Android) | rien à embarquer | en couleur, donc illisible par `fontdue` ; et différente sur chaque machine : Glucose embarque ses polices pour dessiner pareil partout (FONT-1) |
| **OpenMoji, Twemoji** | des dessins libres | en couleur d'abord ; OpenMoji est sous CC BY-SA, une licence qui contamine ce qu'on en dérive |
| **La police variable entière** | les graisses de 300 à 700 | 1,1 Mo de plus pour des graisses que `fontdue` ne lit pas |

Le monochrome est aussi ce que `style.md` demande à l'interface — mais un emoji est du
**contenu**, et un jour la couleur lui reviendra peut-être : il faudra un rastériseur qui lise
COLRv1, une question à poser le jour où il la demandera.

## Ce qui reste vrai, et ce qui manque

* **Pas de substitution de glyphes** : Glucose dessine caractère par caractère, sans moteur de
  mise en forme (ni `rustybuzz`, ni `harfbuzz`). Une famille composée par des joints
  (👨‍👩‍👧) s'affiche en personnes côte à côte, un drapeau (🇫🇷) en deux lettres, F R, un
  ton de peau en petit carré après la main. La ligature existe dans la police (sa table `GSUB`) :
  il faudrait la lire — un chantier à part, si l'usage le demande.
* **Le binaire grossit de 887 Ko**, l'APK aussi : le poids d'une photo.

## Sources

* Google Fonts, [`ofl/notoemoji`](https://github.com/google/fonts/tree/main/ofl/notoemoji) —
  la police, sa licence, son `METADATA.pb` (version, axes, provenance).
* `fontTools` 4.63, [`varLib.instancer`](https://fonttools.readthedocs.io/en/latest/varLib/instancer.html).
* `fontdue` 0.9 — sa lecture des polices TrueType (l'instance par défaut seulement).
