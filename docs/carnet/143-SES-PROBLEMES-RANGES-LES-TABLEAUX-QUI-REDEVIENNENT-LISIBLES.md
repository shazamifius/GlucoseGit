# 143 — Ses problèmes rangés ; les tableaux qui redeviennent lisibles (24/11, le relais, compte b)

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
