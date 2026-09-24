# 33 — Le filet, et l'histoire

> **Rôle de ce document.** L'utilisateur a apporté deux images qui montrent que `Ctrl+B`
> fonctionne mal, et la sortie de sa session de test de la mémoire par étages
> (`sortie-etages.txt`, 970 s, `fuser.glucose`). Cette fiche dit ce que ses images ont
> appris, ce que sa sortie dit — et ce qu'elle ne permettait pas de dire —, et la conception
> de l'enregistrement « à la git » qu'il a demandée, **à lui présenter avant tout code**.
>
> **Date** : 2026-09-24 · commits `86a2a19` à `39f9748`, et celui de cette fiche.
> **État vérifié** : `cargo test --workspace` exit 0, **1 556 tests verts** — et non 1 549
> au départ : la suite annoncée verte ne l'était pas (§ 3). Clippy strict à zéro,
> `cargo fmt --check` à zéro, aucun plafond relevé.
>
> **Rien de ceci n'a encore été vu à l'écran.**

---

## 1. Ce que l'utilisateur a apporté

* Une image à marge blanche, et ce qu'en fait `Ctrl+B` : *« tous les bords haut, gauche et
  bas ont été parfaitement rognés, juste la droite reste — je ne sais pas du tout d'où vient
  ce bug »*.
* Sa sortie : *« ce qui va vraiment être super intéressant, c'est ça particulièrement à
  lire »*. Chronique copiée sous `sortie-chronique-2026-09-24-etages.txt` (ignorée par git).

---

## 2. BORDURES-5 — le filet (`86a2a19`)

### 2.1 La cause, au pixel

Son image est dans `fuser.glucose` : 849 × 1200, `glucose_depose\…c6e1ad63….jpg`. Rejouée par
le détecteur, elle perdait 96 pixels à gauche, 104 en haut, 94 en bas — et **1** à droite.

| colonne | contenu |
|---|---|
| 848, la dernière | **gris 88** uniforme, sur toute la hauteur |
| 847 → 741 | blanc 255 : la marge |
| 740, 739 | le fondu, puis la photo |

Le bord droit prenait le gris de sa première colonne pour la couleur de sa bande, en retirait
un pixel, et s'arrêtait sur le blanc. Les trois autres bords ne croisaient ce filet que sur
un pixel de leur ligne : dans la tolérance du centième. Ce n'était **pas** un critère des
fiches 26 à 28 qui était faux, c'était une situation qu'aucune n'avait vue.

### 2.2 La mesure sur ses 275 images, avant de choisir

Un portage Python du détecteur, vérifié contre le Rust (274 images identiques au pixel, la
275ᵉ à un pixel près — les deux décodeurs JPEG arrondissent différemment), a permis
d'essayer quatre règles sur toutes les images de ses documents lisibles :

| règle | ce qu'elle change | verdict |
|---|---|---|
| **emboîtement libre** : derrière toute bande, une ligne unie qui succède franchement à la précédente, et plate, ouvre une bande | les filets, **et** le fond gris-bleu uni d'une photo posée entre deux bandes noires : 123 lignes de photo en haut, 82 en bas | refusée |
| **plusieurs filets à la suite** | le haut d'une photo de dune — un voile clair qui s'assombrit ligne après ligne — devient une pile de filets, et **74 lignes de ciel** partent | refusée |
| un seul filet, sans condition sur ce qui le suit | les filets, et une ligne de dégradé de trop sur la dune | écartée |
| **un seul filet, devant une bande épaisse** | **exactement quatre images**, les quatre qui portent un filet devant une bande — la sienne comprise | **retenue** |

Une bande noire de film et le fond gris d'un studio sont les mêmes pixels ; seul le filet,
qui n'a qu'une ligne, ne se confond avec rien. Le passe-partout (marge, filet, marge) reste
donc un cas non traité, dit dans le module.

### 2.3 Ce que la règle dit, et la propriété qui la prouve

Un **filet** est une seule ligne unie, derrière laquelle commence une bande épaisse — au moins
deux lignes — d'une autre couleur. Il part, et la bande se cherche derrière lui comme au bord.
Aucun nombre nouveau : « unie » et « de sa couleur » sont les critères existants.

La propriété vérifiée sur **chaque** image des épreuves, sur chaque bord seul puis les quatre
ensemble, en gris et en blanc : **un filet au bord ajoute une ligne à ce que `Ctrl+B` trouve,
exactement.**

Et une image où **tout** est bande n'a plus de bordure : elle finissait en un trait d'un
centième de large, l'invariant du recadrage gardant ce qu'il pouvait.

### 2.4 Une épreuve qui laissait passer un défaut depuis BORDURES-3

La « peinture » synthétique des épreuves de BORDURES-3 et 4, censée n'être « jamais unie », ne
variait que de ±20 niveaux — sous le seuil de 24. Chacune de ses colonnes était donc une bande
pour le détecteur : le bord gauche la dévorait entière, et **l'ancien code réduisait déjà ces
images à un trait**, pendant que les épreuves, qui ne lisaient qu'un bord, passaient.
Désormais chaque cas se vérifie sur ses quatre bords, la peinture parcourt quatre-vingt-trois
niveaux, et la peinture sombre est en bandes obliques.

### 2.5 La preuve à l'envers

| sabotage | ce qui tombe |
|---|---|
| sans la règle du filet | son image, et la propriété |
| sans la condition d'épaisseur | « un filet ne se reconnaît que devant une bande » |
| plusieurs filets admis | « un dégradé au bord n'est pas une pile de filets » |
| sans « tout est bande, rien ne l'est » | l'image unie |
| l'ancienne peinture | cinq épreuves |

Le rendu de son image par le vrai code (`apercu_recadrage`) : la photo va jusqu'au bord des
quatre côtés ; à droite, la colonne montrée vaut 25,7 de luminosité contre 25,9 pour sa
voisine.

---

## 3. La suite n'était pas verte (`48c56bb`)

`test_ce_qui_ne_tient_a_aucun_cran_deborde` (ETAGES-3, `a30753b`) décalait `1_000_000u64` de
64 bits au trente-deuxième cran : panique de débordement en profil de test, **à chaque
passage**, vérifié au commit de départ `5a3c89b`. Les « 1 549 tests verts » de la fiche 32
n'étaient donc pas ceux de `cargo test --workspace` — sans doute un passage en `--release`,
qui ne vérifie pas les débordements. La production calcule en flottant et n'a pas le défaut.

---

## 4. Ce que sa sortie dit

### 4.1 La mémoire : l'étage tient

| | fiche 32 § 1 (avant ETAGES) | le 24/09 |
|---|---:|---:|
| mémoire de travail à la fin | 1 543 Mo | **448 Mo** (644 au pire) |
| images tenues / offertes | — | 15 Mo / 1 117 Mo |
| niveaux offerts, repris, jetés | — | 9 736, 8 962, 0 |
| processeur au repos | — | 0,3 % d'un cœur |

C'est la première mesure en conditions réelles d'ETAGES-1, et elle est bonne. **À
confirmer par ce qu'il a vu au gestionnaire des tâches** : la mémoire *engagée* ne baisse pas
(fiche 32 § 4).

### 4.2 La cadence : le verdict n° 1, et une régression

| | fiche 32 § 1 | le 24/09 |
|---|---:|---:|
| images par seconde entre deux images | 72 | **43** |
| tempo | 3-4 balayages | **5 à 8** |
| image médiane (zoomer) | 5-7 ms | 8,2 ms, p90 13,8, p99 27,6 |

Le tempo monte d'un cran dès que plus d'une image sur cent rate son balayage, et ne redescend
qu'après deux cents images sans raté un cran plus bas : il suit la **queue** des coûts, pas
leur médiane. Au p99, la queue vient de `blit` (11,6 ms), `soumettre` (9,7), `textures`
(6,9), `docks` (4,9) — aucun n'est la pose des photos, que la pixelisation commande.

Deux lectures, que la chronique ne départageait pas :

1. **une vraie queue de coûts** — des postes qui explosent une image sur dix ;
2. **un cercle** — un tempo haut espace les images, la vue avance trois fois plus entre deux,
   chacune a plus à redessiner et coûte plus cher, et le tempo reste haut. Le module du tempo
   raconte déjà un cercle de la même famille.

**TEMPO-2** (`39f9748`) ajoute à la chronique, sans rien qui grandisse avec la session, ce
qu'une image coûte **selon le tempo où elle est dessinée** — plus chère à huit balayages qu'à
trois, c'est le cercle — et **où vont les images qui ratent leur balayage**, poste par poste.
Trois épreuves ; celle qui passe par le vrai `GlucoseApp` en mouvement est née d'un sabotage
du relevé qu'aucune autre n'attrapait. **Aucune correction du tempo n'est tentée avant ces
chiffres** : corriger à l'aveugle un asservissement est la façon dont les cercles naissent.

### 4.3 Le reste

* Les gels à 6,2 s (le pilote qui s'initialise) et à la fermeture (112 ms) sont connus.
* Les images les plus lentes à 28,9-29,6 s : un zoom très proche d'un texte, 8,5 millions de
  pixels de lettres à rastériser dans une image.
* Entre 280 et 285 s, `textures` à 22-28 ms : un collage (le nœud 244).

---

## 5. L'enregistrement — ce qui existe, et la conception « à la git »

### 5.1 Ce qui existe

* **`Ctrl+S` relit chaque image depuis son fichier, en refait l'empreinte, et réécrit tout le
  document**, sur le fil qui dessine : 2,46 s sur sa longue session. Dans les 179,7 Mo de
  `fuser.glucose`, **le document lui-même pèse 96 Ko** ; le reste, ce sont les 243 images.
* **Une image sur deux vit dans le dossier temporaire de Windows** (142 sur 275) : déposée
  depuis un navigateur (`glucose_depose`), collée (`glucose_pasted`), ou restaurée à
  l'ouverture (`glucose-assets`). Si Windows fait le ménage de ses fichiers temporaires
  pendant qu'un document est ouvert, l'enregistrement suivant écrit le document **sans les
  octets de ces images**, par-dessus l'ancien. Risque conditionnel, mais c'est une perte.
* Le journal d'annulation du noyau enregistre déjà chaque geste comme « avant / après », au
  coût du geste et jamais du document (JRN-1). C'est exactement la matière d'un historique.

### 5.2 Glucose Tauri

Deux étages : les **gestes** (chaque modification Automerge, parcourue à la réglette avec un
aperçu en direct et un liseré ambre, « Restaurer cet état » en fait un nouveau commit), et les
**versions durables** (`<fichier>.versions/`, manuelles ou automatiques selon l'ampleur des
changements, élaguées). Chaque version était **une copie complète** du fichier — sur `fuser`,
dix jalons feraient 1,8 Go —, et le fichier gonflait sans fin de son historique Automerge, d'où
une compaction délicate (`src/utils/versions.ts`, `autoVersion.ts`, `compaction.ts`,
`components/TimelinePanel.tsx`).

### 5.3 Ce que font les autres

| | ce qu'on en retient |
|---|---|
| git ([objets](https://github.blog/open-source/git/gits-database-internals-i-packed-object-store/), [modèle](https://www.kenmuse.com/blog/understanding-how-git-stores-data/)) | un contenu est nommé par son empreinte et stocké **une fois** ; un commit est un instantané léger qui pointe vers des contenus partagés |
| Figma ([historique](https://help.figma.com/hc/en-us/articles/360038006754-View-a-file-s-version-history), [versions nommées](https://www.figma.com/blog/now-you-can-name-and-annotate-your-figma-version-history/)) | un point automatique toutes les trente minutes d'activité, des versions nommées, et les points automatiques **repliés** entre deux versions nommées |
| Google Docs ([historique](https://support.google.com/docs/answer/190843)) | le document est une liste de changements ; l'affichage regroupe les modifications proches |
| Automerge ([stockage](https://automerge.org/docs/reference/under-the-hood/storage/)) | des morceaux **incrémentaux** écrits à chaque changement, **compactés** en un instantané quand ils pèsent plus que lui |
| Redis ([persistance](https://redis.io/tutorials/operate/redis-at-scale/persistence-and-durability/)) | un journal en **ajout seul** pour ne rien perdre, des instantanés pour repartir vite |
| Vim, Emacs ([undo-tree](https://www.dr-qubit.org/undo-tree.html)) | l'annulation est un **arbre** : défaire puis faire autre chose ouvre une branche au lieu d'effacer |

### 5.4 La conception proposée — une seule structure, celle de git

1. **Chaque image entre une fois, sous son empreinte.** À l'import (dépôt, collage), son
   empreinte se calcule une seule fois et ses octets rejoignent un magasin d'objets. Le
   document cite l'empreinte, plus jamais un chemin — et encore moins un chemin temporaire.
   Une image n'est plus jamais relue, ré-empreintée ni réécrite. (Fiche 09 § 4.2 le
   prévoyait.)
2. **Chaque geste s'écrit à la suite.** Le journal d'annulation, déjà là, s'écrit sur le
   disque au fil des gestes : quelques centaines d'octets en ajout seul. Enregistrer devient
   **instantané et automatique** ; un plantage ne perd plus rien. `Ctrl+S` n'a plus à
   sauver : il pose un **jalon nommé**.
3. **Le wayback, c'est ce journal relu.** Chaque entrée porte l'avant et l'après : on remonte
   geste par geste jusqu'à la création du document, et on redescend. Pour sauter loin sans
   tout rejouer, un **instantané du document** de temps en temps — 96 Ko pour `fuser`, pas
   180 Mo, puisque les images sont partagées. Les jalons nommés sont des étiquettes sur ce
   fil ; les points automatiques se replient entre deux jalons, comme chez Figma. La réglette
   et l'aperçu ambré de Glucose Tauri s'y posent tels quels.
4. **Rien ne se perd, pas même les branches.** Revenir à un état ancien est un nouveau geste
   qui ne détruit rien ; défaire puis faire autre chose ouvre une branche.

Le chiffre qui résume : enregistrer coûte ce qu'on a changé, et remonter le temps coûte ce
qu'on a parcouru — jamais la taille des images.

### 5.5 Les questions qui sont les siennes

* **Où vit l'histoire** : dans le fichier `.glucose` lui-même (un seul fichier à copier ou à
  envoyer, qui grossit de son histoire), ou dans un dossier à côté, à la manière de `.git`.
  Ma recommandation : le fichier unique — il se partage, et le conteneur réserve déjà une
  section de journal.
* **Ce que l'histoire garde** : une image supprimée reste dans le passé, donc dans le
  fichier. Tout garder pour toujours, ou oublier au-delà d'un horizon qu'il choisit — et
  alors lequel ?
* **La réglette** : celle de Tauri telle quelle, ou autre chose.

**Rien de ce chantier n'est codé.** C'est une décision de produit.

---

## 6. Ce qui reste, chiffré

| # | Piste | Ce qu'on sait | Ce qu'il faut |
|---|---|---|---|
| 1 | **La cadence** : 43 i/s, tempo 5-8 | queue au p99 hors photos, ou cercle | **sa prochaine chronique**, lue par TEMPO-2 |
| 2 | **L'enregistrement à la git** | § 5 | **sa décision** sur § 5.5 |
| 3 | Les images dans le dossier temporaire | 142 sur 275 ; perte possible au `Ctrl+S` après un ménage de Windows | réglé par § 5.4-1 ; sinon, un filet en attendant |
| 4 | Le passe-partout pour `Ctrl+B` | marge, filet, marge : non traité | un cas réel chez lui |
| 5 | Étape 7, textures compressées | fiche 32 § 11 | un banc sur ses images |
| 6 | Le reste de la fiche 32 § 11 | autres plateformes, dossier des aperçus, lettres en octets, plantage rare | — |
| 7 | La gravure, SauceNAO, Trans-domaines | — | **sa parole** |

---

## 7. Ce qu'il faut tester à l'écran

```text
cargo run --release > sortie-filet.txt 2>&1
```

1. **`Ctrl+B` sur son image au coquillage** : la marge de droite doit partir comme les trois
   autres, sans liseré. Puis sur n'importe quelle image à marge : rien ne doit changer pour
   celles qui marchaient.
2. **Une session de mouvement normale** — se promener, zoomer, dézoomer — pour que TEMPO-2
   dise enfin pourquoi le tempo monte.
3. **Ce qu'il a vu au gestionnaire des tâches** pendant la session du 24/09, et si les photos
   ont paru floues ou vides à un moment.

---

## 8. Ce que cette session a démenti

1. **« 1 549 tests verts »** (fiche 32) : une épreuve paniquait à chaque passage.
2. **Les épreuves de BORDURES-3 et 4** : elles ne regardaient qu'un bord, et leurs images
   finissaient en un trait.
3. **Ma première règle** d'emboîtement des bandes : elle mangeait le fond d'une photo.
4. **Ma première documentation** de la condition d'épaisseur : elle lui attribuait la
   protection des 74 lignes de ciel, qui vient en fait de la règle « un seul filet ».

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
