# 34 — La tache sur la marge

> **Rôle de ce document.** L'utilisateur a testé la fiche 33 : *« ça fait toujours la même
> chose avec `Ctrl+B` »*, deux captures d'une gravure (*Veľká mizéria*) dont la marge de
> droite reste, et sa sortie (`sortie-etages-2026-09-24-filet.txt`, 227 s, deux sessions
> sans le vouloir — sa touche Windows ne répondait plus). Il est **d'accord avec la direction
> de l'enregistrement** (fiche 33 § 5.4). Cette fiche dit ce que sa gravure a appris, ce que
> TEMPO-2 a répondu, et ce qui attend sa parole.
>
> **Date** : 2026-09-24 · commit `3acff6a`, et celui de cette fiche.
> **État vérifié** : `cargo test --workspace` exit 0, **1 560 tests verts**, clippy strict à
> zéro, `cargo fmt --check` à zéro.
>
> **Rien de ceci n'a encore été vu à l'écran.**

---

## 1. BORDURES-6 — une tache sur la marge (`3acff6a`)

### 1.1 Ce n'était pas le filet

Sa gravure est dans ses dépôts : 736 × 828, `glucose_depose\…4c4c911c….jpg`. Le détecteur
retirait 49, 51 et 51 pixels — et **11** à droite, sur 56. Aucun trait : la marge de droite
est blanche au milieu de l'image, mais porte **trois petites taches** sur le papier, la plus
grande à quinze pixels du bord, invisibles à l'œil, vingt-cinq à quarante niveaux sous le
blanc. Chaque colonne qui les traverse en porte un peu plus d'un centième — 8 à 16 pixels
pour 7 tolérés —, et la tolérance, jugée ligne par ligne, y voyait le début du contenu.

La règle de la fiche 33 était juste et trop étroite : elle traitait **un** défaut, celui des
quatre images qu'il y avait alors. Les tirages sur papier en portent de toutes sortes :
taches, crayon, cuvette.

### 1.2 Trois règles essayées sur ses 286 images

| règle | images qui changent | verdict |
|---|---:|---|
| la tolérance du centième jugée **sur la bande entière**, la coupe toujours sur une ligne propre | **48** | refusée : le texte sous une affiche, le titre manuscrit d'un dessin, un pan sombre d'une peinture de marais, les étoiles autour d'un arbre fractal partaient. Une zone sombre parsemée de détails ressemble trait pour trait à une marge tachée |
| la même, **si la marge finit sur un bord net** — une ligne en majorité hors de la marge | 1 | la gravure, mais pas son bord droit : la plaque penche, et sa rampe épuisait la tolérance |
| la même, le bord net atteint par une **rampe qui ne fait que monter** depuis la dernière reprise de la marge | **2** | **retenue** : la gravure, et une tache rose au-dessus d'une plante, retirée juste au-dessus de la pointe de sa tige |

Ce qui sépare les vrais cas des faux n'est pas la tache, c'est **la façon dont la marge
finit** : un bord droit, même penché, monte ; le bord d'une peinture, d'une aquarelle, d'une
zone sombre parsemée monte et descend. Aucun nombre nouveau : le centième et la majorité sont
ceux du module. Le Rust et le portage Python sont d'accord sur 285 images, la dernière à un
pixel près (les deux décodeurs JPEG arrondissent autrement).

### 1.3 La preuve à l'envers

| sabotage | ce qui tombe |
|---|---|
| sans enjamber | sa gravure reconstituée |
| sans la rampe qui monte | le bord qui hésite, et la propriété du filet |
| sans le centième à la reprise | un objet qui s'élargit rang après rang au-dessus d'une image — **ajouté parce que** le titre au-dessus d'une image ne le voyait pas : une seconde vérification, qui ne fait qu'anticiper la première pour borner le balayage, le rejetait avant |

### 1.4 Ce qui reste : le coin d'une plaque penchée

Rendue par le vrai code, la gravure recadrée garde un **liseré clair de 2 à 4 pixels** en
haut, à gauche et à droite : la plaque penche, et un cadre droit ne peut pas épouser un
rectangle penché. La coupe s'arrête à la dernière ligne entièrement blanche ; il reste un
coin de papier qui s'amincit jusqu'à zéro. Couper dans la rampe l'enlèverait, mais une rampe
qui monte est aussi le sommet arrondi d'un sujet — une tête, un disque, une tache floue — et
on le couperait. **Question de ressenti, à lui poser.**

---

## 2. Ce que sa sortie a dit

### 2.1 Les trois gels de deux secondes sont ses `Ctrl+S`

1 813, 1 877 et 2 012 ms, tous dans « écouter la main », à 43, 215 et 223 s. Le
téléchargement des onze épingles tourne sur un fil à part (`plateforme/rapatrier.rs`), et
ses dépôts tombent entre la 80ᵉ et la 132ᵉ seconde. `fuser.glucose` a été écrit à
14 h 12 min 53 s, soit la 223ᵉ seconde à une seconde près : **l'enregistrement**, 180 Mo
réécrits pour quelques octets changés (fiche 33 § 5.1). C'est le chantier qu'il a approuvé
qui les supprime.

### 2.2 TEMPO-2 répond : pas un cercle, un coût d'image trop haut

| tempo | images | médiane | p90 | p99 |
|---:|---:|---:|---:|---:|
| 4 balayages | 835 | 6,89 ms | 9,74 | 23,17 |
| 5 | 1 505 | 8,19 | 9,74 | 19,48 |
| 8 | 218 | 9,74 | 23,17 | 27,55 |

Le coût monte à peine avec le tempo : **ce n'est pas un cercle**. Seules **43 images sur
3 288** (1,3 %) ont raté leur balayage — juste au-dessus du centième que le tempo tolère, d'où
les montées. Leur temps va d'abord à `soumettre` (médiane 1,45 ms, p90 6,9), `blit` (1,2 ;
9,7), `docks` (1,0 ; 4,1), `textures` (p90 9,7).

Le vrai obstacle au plancher de la charte est ailleurs : **une image ordinaire coûte 7 à
8 ms**, répartis sur une dizaine de postes de 0,5 à 1 ms (`blit`, `soumettre`, `effacer`,
`relever`, `docks`, `present`…). À 240 Hz, cent images par seconde demandent deux balayages,
8,33 ms : la médiane y tient à peine, et le p90 non. Ce n'est pas une queue à couper, c'est
le coût de base d'une image sur la voie graphique, à décomposer poste par poste. **C'est le
chantier de la cadence**, et il ne demande rien à l'utilisateur.

### 2.3 Le reste

* La mémoire tient toujours : 441 Mo à la fin, 1 150 Mo offerts, rien jeté par le système.
* Au repos, 9,7 images par seconde et 5 % d'un cœur : les messages à l'écran (32,6 % des
  images) — les annonces de ses onze dépôts.

---

## 3. L'enregistrement — ce qui attend sa parole

Il a approuvé la direction de la fiche 33 § 5.4. Deux décisions restent les siennes, une à
la fois :

1. **Où vit l'histoire** : dans le fichier `.glucose` lui-même — un seul fichier à copier ou à
   envoyer, qui grossit de son histoire —, ou dans un dossier à côté, à la manière de
   `.git`. Recommandation : le fichier unique ; le conteneur réserve déjà une section de
   journal (`container::KIND_JOURNAL`).
2. **Ce que l'histoire garde** : une image supprimée reste dans le passé, donc dans le
   fichier. Tout garder, ou oublier au-delà d'un horizon qu'il choisit.

Ce que le code offre déjà : le journal d'annulation enregistre chaque geste comme un avant et
un après, image, annotation, dossier, zone, déplacement (`store/journal/edit.rs`), au coût du
geste (JRN-1). L'écrire sur le disque ne réinvente rien.

---

## 4. Ce qu'il faut tester à l'écran

```text
cargo run --release > sortie-tache.txt 2>&1
```

1. **`Ctrl+B` sur la gravure** *Veľká mizéria* : la marge de droite doit partir comme les
   autres. Regarder de près les bords : dire si le **fin liseré clair** du coin penché se
   voit, et s'il gêne.
2. **`Ctrl+B` sur quelques images qui marchaient déjà** : rien ne doit changer.

---

## 5. Ce que cette session a démenti

1. **La règle de la fiche 33 comme correction de « `Ctrl+B` qui laisse la marge de
   droite »** : elle ne traitait qu'une cause sur deux.
2. **Ma première règle des taches** : 48 images changées, la plupart en perdant du contenu.
3. **Mon épreuve du centième** : le sabotage passait, une autre vérification rejetant le cas
   avant elle.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
