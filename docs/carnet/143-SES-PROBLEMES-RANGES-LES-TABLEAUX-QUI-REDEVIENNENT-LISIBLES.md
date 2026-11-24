# 143 — Ses problèmes rangés ; les tableaux qui redeviennent lisibles ; sélectionner dans une membrane (24/11, le relais, compte b)

Ses notes du 24/11 : *« re-organiser de manière plus propre, mieux formuler, plus complet toute cette
membrane, car elle va dans tous les sens »* (`text-g420-2319`) ; *« archive les blocs texte une fois
qu'ils sont corrigés, testés et certains d'être finis »* (`text-f5fn-369`, `text-f5fn-373`).

## 1. La membrane 🐞 rangée (`membrane-299`)

Seize problèmes, **une bulle chacun** : son numéro et son état dans le titre (🔧 en cours, 📝 au plan,
⏳ son avis, 🧪 corrigé à essayer, ✅ fait), sa note, sa capture et notre réponse ensemble ; la liste
`relais-bugs-liste` en haut à gauche ; deux colonnes dans l'ordre des numéros. Ses « Groupe » (une
capture et sa note, faits de sa main) sont gardés et renommés. **Rien d'archivé** : aucun n'est à la
fois corrigé et essayé par lui ; il écrit « 9 ok » dans la liste, et la bulle part à l'archive. Deux
notes n'avaient pas de réponse : `text-wiov-330` (les tableaux, § 2) et `text-wiov-325` (écrire un long
texte : l'état de chaque point, `relais-rep-325`).

**Pièges payés** :
* **`glucose_poser_texte` exige `texte`** : pour ranger un texte par `dans`, on lui redonne son texte
  entier, relu juste avant — une première fois, un texte recopié d'une lecture coupée a tronqué
  `relais-rep-364` ; rendu mot pour mot depuis le document (ses versions y restent, dans l'histoire).
  Le geste sûr : relire, poser avec `dans`, relire, comparer (un script, 23 textes, 23 identiques).
* `glucose_deplacer` ne fait pas entrer un nœud dans une petite bulle qu'il recouvre en partie ; une
  membrane posée hors de sa mère en sort, `glucose_poser_membrane … dans=` l'y remet sans la bouger.
* **`glucose_disposer` lit l'ordre rangée par rangée** : relancé sur des colonnes, il mélange l'ordre.
  Un ordre voulu se pose à la main, colonne par colonne.

## 2. TABLE-5 — seul le mot trop long se coupe

**Sa capture** (`img-paste-wiov-329`, 22/11) : un tableau de SUITE dans un nœud de 305 ; « [~] » sur
trois lignes, « télémétri-e », « connecteur-s ». *« Maintenant que tu as corrigé ces tableaux… ils
deviennent absolument illisibles. »*

**La cause** : TABLE-3 (fiche 123 § 3), en deçà des largeurs minimales, les réduisait **toutes en
proportion**. Une adresse de fichier de 45 signes, sans espace, faisait la somme des minimales ; toutes
les colonnes passaient sous leur mot le plus long.

**La recherche** : Typst, `shrink_auto_columns`
([`crates/typst-layout/src/grid/layouter.rs`](https://github.com/typst/typst/blob/main/crates/typst-layout/src/grid/layouter.rs)) —
*« redistribute space to auto columns so that each gets a fair share »* : une colonne plus étroite que
sa part la garde, les autres reçoivent la part, jusqu'à ce que plus rien ne change. La CSS, elle,
déborde (fiche 123).

**La correction** (`renderer/richtext/table.rs`, `repartir`) : ce partage, sur les minimales, triées —
une passe suffit. La somme fait exactement la place ; continue avec TABLE-3 (à la somme des minimales,
la part dépasse la plus large) ; aucun nombre à choisir. **Épreuves** : `test_table_5_seul_le_mot_trop_long_se_coupe`
(son tableau à 257 : l'adresse se coupe, aucun autre mot) et la répartition en nombres, dans les deux
ordres. **Sabotages** `outils/sabotages/table-143.json` : 3 sur 3 tombent (la proportion revenue, la part
non partagée, le tri ôté). **Au pilote** : son tableau collé dans un Glucose d'essai, « [~] » et chaque
mot entiers, seule l'adresse coupée (`~/.cache/glucose-bancs/table5-pilote.png`).

**Reste** : son écran — ses tableaux de SUITE, dans ses nœuds étroits.

## 3. MEMB-SEL-1 — sélectionner dans une membrane

**Sa note** (`text-g420-2316`, 24/11) : *« au sein d'une membrane, si on veut faire un maintien du clic
pour effectuer une sélection, c'est impossible, car ça sélectionne la membrane »* ; sa proposition :
appuyer sans bouger *« suffisamment longtemps »* prend la membrane, bouger trace le rectangle.

**La recherche** : Figma a la même plainte sur ses sections
([*Dragging inside sections moves the section instead of selecting*](https://forum.figma.com/t/in-ui3-dragging-inside-sections-moves-the-section-instead-of-selecting/88839)) ;
sa règle, rapportée dans ce fil : une section **non choisie** se traverse par le rectangle, une section
**choisie** se glisse. Glucose l'avait déjà pour les zones de gabarit (`interactions/zone.rs`, PRESET-1 :
on les prend par leur bandeau).

**Le choix, contre sa proposition** : pas de durée. Une durée est une constante à régler (trop courte,
le rectangle part en membrane ; trop longue, on attend), et elle se sent comme un retard. **La sélection
départage** : dans le vide d'une membrane non choisie, appuyer trace le rectangle (`appuyer_dans_une_membrane`,
`interactions/selection.rs`) ; relâché sans bouger, c'est un clic, qui la choisit (`membrane_au_clic`,
lu au relâchement) ; choisie, son vide la glisse comme avant ; son titre et son bord la prennent toujours
d'un geste ; `Maj` (l'ajouter) et `Alt` (la pile) ne changent pas. Déplacer une membrane non choisie :
un clic puis glisser, ou son titre — autant de gestes que l'appui long, sans attendre. Dit dans
`relais-rep-selection-membrane` ; à lui de trancher s'il préfère sa durée.

**Le noyau** (`Board::pris_par`) : le rectangle ne prend plus **la membrane dans laquelle il est tracé
tout entier** (ni ses mères) — tracé dans son vide, il vise ce qu'elle porte ; il prend toujours celle
dont il touche le bord ou le titre (TITRE-1).

**Le doigt** garde son geste (le vide déplace le canevas) : sa note parle de la souris.

**Épreuves** (`interactions/selection/tests.rs`) : glisser dans le vide prend le texte et ne bouge pas
la membrane ; un clic la choisit, puis son vide la glisse. **Sabotages** `outils/sabotages/memb-sel-143.json` :
4 sur 4 tombent. **Au pilote** : le rectangle part du vide, la membrane reste, seul le texte est choisi
(`~/.cache/glucose-bancs/memb-sel-pilote.png`). `click_node_at` passait à 82 lignes : l'appui dans le vide
en est extrait (`appuyer_dans_le_vide`).
