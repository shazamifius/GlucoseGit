# 35 — La frange et le vol

> **Rôle de ce document.** L'utilisateur a testé la fiche 34 : la marge de droite de sa
> gravure part, *« mais le problème, c'est qu'elle possède encore des contours »* — pareil sur
> une seconde gravure, dont il apprécie que les inscriptions au crayon du bas soient gardées.
> Il a aussi signalé l'animation de focus (`F`, les signets) : *« des parcours chelous… un full
> zoom qui traverse toute la map, qui fait mal aux yeux »*, avec deux propositions. Sortie :
> `sortie-tache.txt` (174 s).
>
> **Date** : 2026-09-24 · commits `6a5a08e` et `5cabff7`, et celui de cette fiche.
> **État vérifié** : `cargo test --workspace` exit 0, **1 574 tests verts**, clippy strict à
> zéro, `cargo fmt --check` à zéro — voir § 4 pour l'épreuve intermittente de la carte.
>
> **Rien de ceci n'a encore été vu à l'écran.**

---

## 1. BORDURES-7 — la frange d'un bord de plaque (`6a5a08e`)

Le « contour » était le coin de papier annoncé à la fiche 34 § 1.4 : il se voit, il gêne. Les
deux gravures montrent la même signature : en quelques lignes, la part de papier dégringole à
chaque ligne, puis se stabilise — 95, 90, 81, 63, puis 0,5 % ; 85, 73, 24, 4, puis 2,7 %. Une
plaque tirée à la main penche, ondule, l'encre bave. Sur la seconde, recadrée de travers avant
de lui parvenir, le haut et la droite n'avaient même plus de bande.

La règle : ces lignes forment une **frange** si le papier y recule de plus d'un centième à
chaque ligne, sur au plus un centième de la longueur d'une ligne, jusqu'à un contenu en
majorité sans papier. Un bord sans bande juge son papier sur la marge d'en face. Le sommet d'un
disque (il s'élargit sur bien plus d'un centième) et le haut d'un objet qui ne remplit pas la
ligne (ce qui suit est encore du papier) restent. Aucun nombre nouveau.

Ce qui a été essayé d'abord : « le papier forme un seul morceau collé à un bout de la ligne »
— la géométrie d'un bord droit. Les mesures l'ont démenti : un bord réel est fragmenté.

Sur ses **308 images distinctes**, 31 changent, de 1 à 9 pixels ; chaque bord changé a été
rendu agrandi et regardé. Rust et portage Python d'accord sur 307 (la dernière à un pixel,
décodeurs JPEG). Quatre épreuves ; le sabotage du partage de couleur ne faisait rien tomber,
d'où l'épreuve du bord sans marge. Le rendu par le vrai code : les premières rangées montrées
valent 60 à 120 de luminosité (la plaque), contre 240 à 250 (le papier) avant.

Le fichier dépassait les 600 lignes : la **lisière** — fondu, tache, frange — est sortie dans
`bordures/lisiere.rs`.

---

## 2. VOL-2 — le chemin de van Wijk et Nuij (`5cabff7`)

### 2.1 La cause

Le vol amortissait séparément le centre (en unités du monde) et le zoom (en octaves). Une fois
zoomé, la même distance du monde vaut des milliers de pixels : sur un saut lointain entre deux
vues zoomées, **cinq écrans en une seule image** à 240 Hz.

### 2.2 La réponse, et ses deux propositions dedans

*Smooth and efficient zooming and panning* (van Wijk et Nuij, InfoVis 2003 —
[article](https://vanwijk.win.tue.nl/zoompan.pdf)), le vol de Google Earth, Mapbox et d3
([`interpolateZoom`](https://github.com/d3/d3-interpolate/blob/main/src/zoom.js)) : le plus
court chemin pour une mesure du mouvement **perçu**. Zoom et translation y avancent ensemble —
sa première proposition —, et quand c'est loin la caméra prend de la hauteur, d'autant plus que
c'est loin, puis redescend — sa seconde. `ρ = √2`, la valeur préférée de leur étude.

Deux soins par rapport à l'article : la forme de `u(s)` est simplifiée
(`sinh(ρs) / cosh(ρs + r0)`), et `r = −asinh(b)` — aucune des deux ne soustrait plus deux
nombres immenses quand on part de très près.

La limite connue de l'article — départ et arrivée secs
([Tokyo 2018](https://arxiv.org/abs/1801.09358)) — est levée par le **profil de secousse
minimale** (Flash et Hogan, 1985), le mouvement d'un bras humain : vitesse et accélération
nulles aux deux bouts.

### 2.3 Ce qu'on voit

| trajet (écran 2160 px) | durée | hauteur prise |
|---|---:|---:|
| photo zoomée → une autre à 500 unités | 0,39 s | ×1,4 |
| à 2 000 unités | 0,95 s | ×3,8 |
| à 10 000 unités | 1,70 s | ×18,5 |
| à 50 000 unités | 2,46 s | ×92,6 |
| `F` depuis un zoom ×20 vers tout le contenu | 1,56 s | — |

L'allure — trois unités de chemin par seconde, au plus vite quatre largeurs d'écran par
seconde à mi-vol — **se juge à l'écran**.

### 2.4 Ce qui a changé

* `glucose_core::chemin` : la forme du chemin, pure et éprouvée (arrivée exacte, dézoom puis
  rezoom, même chemin dans les deux sens, longueur de l'article) ;
* `Vol::voler_vers` pour `F` et les signets ; `Vol::viser` garde le **suivi** de la minimap,
  qui change de cible à chaque mouvement de souris ;
* l'`Animator` des dossiers suit le même chemin avec sa durée et sa courbe : il y avait deux
  géométries de vol, il n'y en a plus qu'une.

Épreuves : la carte ne défile jamais plus vite que l'allure du chemin, départ et arrivée en
douceur, indépendance à la cadence, changement de destination sans saut. Remettre l'ancien vol
en fait tomber trois.

---

## 3. Sa sortie

* Deux gels de 1,8 s (133 et 170 s) : ses `Ctrl+S`, comme à la fiche 34 § 2.1.
* La cadence : 51 images par seconde entre deux images, tempo à 6 balayages sur 39 % des
  images en mouvement ; 56 images ratées (1,2 %). Les pires images de zoom portent `docks` et
  `relever` jusqu'à 25 ms — à regarder avec le chantier de la cadence (fiche 34 § 2.2).

---

## 4. Une épreuve de la carte qui tombe au hasard

`test_une_photo_reduite_se_rend_pareil_sur_les_deux_voies` (`tests/voies_suite.rs`) tombe
**2 fois sur 10 au commit `984eec0`**, 1 fois sur 10 après cette session — seule, jamais :
*« l'écart dépasse 3 sur 11 535 canaux sur 1 920 000 (pire 102) : la carte ne lit pas le niveau
que le processeur lit »*. Préexistante, sans lien avec `Ctrl+B`. Hypothèse écartée en lisant le
code : ce n'est pas l'offre d'un niveau par la mémoire par étages (elle ne se décide qu'à la fin
d'un rendu, après que le niveau voulu a été marqué). Suspect suivant : les vignettes de la voie
processeur, construites avec un budget de temps, donc selon la charge. **À enquêter.**

---

## 5. Ce qui attend sa parole

1. **L'allure du vol** : trois unités de chemin par seconde — plus lent, plus vif ?
2. **L'enregistrement** : où vit l'histoire — dans le fichier (recommandé) ou dans un dossier
   à côté (fiche 34 § 3). Sans réponse à la fiche 34.

---

## 6. Ce qu'il faut tester à l'écran

```text
cargo run --release > sortie-vol.txt 2>&1
```

1. **`Ctrl+B` sur les deux gravures** : plus de contour clair ; les inscriptions du bas de la
   seconde restent.
2. **`F` depuis un zoom très proche**, et **les signets** (`Ctrl+1` pose, `1` y vole) entre
   deux endroits zoomés et éloignés : on doit voir la caméra prendre de la hauteur puis
   redescendre, sans que la carte défile ; dire si l'allure convient.
3. **Entrer dans un dossier et en sortir** : même famille de mouvement.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
