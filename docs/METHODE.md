# Comment on travaille sur Glucose

> **Au 06/10/2026.** Les règles qui font foi, la façon de prouver, et les leçons que ce dépôt a
> payées — écrites pour qu'une session neuve ne les repaie pas. La **charte** elle-même (100 images
> par seconde, aucune machine exclue, deux voies, élégance mathématique) vit dans les mémoires de
> Claude, qui font autorité : les lire **en entier** avant tout.

---

## 1. Avec lui

* **Il ne code pas.** Il porte la vision et le ressenti ; la technique se décide et s'exécute
  seule, et s'explique dans le rapport. Il parle français, et veut être **contredit** quand on a
  un argument, lui compris.
* **Il ne voit pas la console.** Ce qu'il lance s'écrit dans un fichier à la racine
  (`cargo run --release > sortie-x.txt 2>&1`, ignoré par git), qu'on lit soi-même.
* **Jamais de fenêtre sur son écran** pour mesurer : les bancs tournent hors écran. Ce que seul
  l'écran peut dire, c'est **lui** qui le lance, avec une liste courte de quoi regarder.
* **Ses données avant tout.** Une perte le fait paniquer. On diagnostique sur une **copie** de ses
  documents (`examples/verifier_images`, `examples/lire_histoire` lisent sans écrire), et
  `%LOCALAPPDATA%\Glucose` (ses brouillons, sa boîte noire) ne se touche jamais.
* **Ses mots** : « nœud », jamais « carte » pour un élément du tableau ; « carte graphique » en
  entier. « Glucose Tauri » et « Glucose Rust », tels quels.
* **Les questions se posent en prose**, jamais en menu à choix. Les sessions sont longues : on
  enchaîne les chantiers, on ne s'arrête que pour un essai à l'écran ou un jugement de ressenti.

## 2. Les règles de code qui font foi

* **R1 — Rien à moitié.** Une fonctionnalité est finie quand elle est **branchée, visible,
  annulable et enregistrée**. Un bouton qui ne fait rien encore le dit ; il ne fait jamais semblant.
* **R2 — Glucose Tauri dit *quoi*, jamais *comment*.** Les fiches 06 à 10 du carnet en sont la
  spécification ; son code n'est pas un modèle. On recrée, on ne traduit pas.
* **Les cliquets** (`crates/glucose-desktop/tests/cliquets_suite.rs`) l'emportent sur les seuils
  écrits dans la fiche 05 : **80 lignes par fonction, 600 par fichier**, le couplage au modèle, les
  toasts, les modules sans appelant, les marques de mesure. Un cliquet qui proteste demande une
  extraction, jamais un plafond relevé.
* La fiche 05 reste la référence pour le reste (§ 1.7 : un gestionnaire d'événements ne contient
  aucune logique ; § 3 : toute modification passe par le journal ; § 4 : le rendu lit, il ne
  calcule pas, et ne touche jamais le disque). Sa règle « deux dépendances au plus » est
  dépassée : la charte dit « minimales mais assumées », et chaque dépendance a sa note.
* **Une constante arbitraire qui peut disparaître disparaît.** Une qui ne le peut pas se dit comme
  un « nombre de ressenti », avec le journal de ses valeurs essayées (exemple : `elan.rs`).
* **Les invariants portent un nom** (MEMB-1, PICK-2…) et un commentaire qui dit **pourquoi**.

## 3. Prouver

1. **Une épreuve par invariant**, et elle doit **tomber** quand on casse ce qu'elle garde :
   `outils/saboter.py` remplace un passage, lance les épreuves, exige qu'elles tombent, restaure.
   Le 29/09, quatre épreuves aveugles trouvées ainsi.
2. **Une épreuve porte sa preuve à l'envers** : elle rejoue l'ancienne loi sur les mêmes données et
   vérifie qu'elle concluait l'inverse. Plus fort qu'un `git stash`, qui ne prouve qu'une fois.
3. **Les deux voies au bit près**, ou à un écart mesuré, borné, et expliqué.
4. **Regarder l'image**, pas seulement les nombres : trois régressions de rendu n'ont été vues qu'en
   posant deux captures côte à côte.
5. **Vérifier sur les machines qu'on ne possède pas** : la CI tourne sur six systèmes. Grouper les
   envois (un envoi plus récent annule le précédent) et suivre par
   `outils/suivre_ci.sh <empreinte complète>`.
6. **« Réglé » seulement vérifié** : sur une copie de ses documents, sur GitHub, ou à son écran.
   Jamais parce que ça compile.

## 4. Les leçons qui ont coûté des sessions

| la faute | ce qu'il faut faire |
|---|---|
| **Une marque de mesure absorbe ce qui la précède** — cinq fois (`occlusion`, `recolte`, `blit`, `minimap`, `bande`), et chaque fois le mauvais coupable pendant des sessions | poser la marque là où le travail change de nature ; les postes de l'entracte se somment au bit près |
| **Un compteur déclaré et jamais lu vaut zéro, et un zéro se lit comme une mesure** | cliquet 9 : aucun champ de la chronique ne reste vide |
| **Trier par médiane cache ce qui tient le p99**, et sommer laisse une image aberrante décider du typique | lire médiane, p99 et pire ; ne jamais additionner des centiles |
| **Un accusé de réception n'est pas une livraison** : `present` répondait « réussi » pendant que l'écran restait figé | mesurer ce que l'œil reçoit, pas ce que l'API répond |
| **Les cercles vicieux** — six : un mécanisme qui s'adapte à un coût qu'il ne commande pas (la finesse asservie à sa propre sortie, la résolution qui lisait les gels du pilote…) | demander « lequel **commande** ce symptôme », pas « lequel le produit » |
| **Un test qui simule ce que le code suppose ne teste que l'arithmétique** | simuler ce que la machine fait vraiment |
| **Comparer deux exécutions, c'est comparer du bruit** (2,3, 6,2 puis 5,7 ms pour la même mesure) | les deux régimes dans la **même** exécution, entrelacés |
| **Deux sessions de terrain ne jouent pas le même geste** | ne pas conclure d'une comparaison qui n'en autorise aucune |
| **L'œil a raison contre le banc** (le lissage à 5 ms « mesuré bon » était lagy) | un nombre de ressenti se juge à l'écran, par lui |
| **La première hypothèse est souvent fausse** (l'antivirus de GitHub, le rapporteur éteint, mes instances restées vivantes…) | diagnostiquer avant de corriger ; dire « probablement, à confirmer » |
| **Lire un titre de la charte et sous-lire la phrase d'après** : une session perdue à gagner des millisecondes au processeur pendant que la carte graphique faisait cinq fois mieux sans abîmer l'image | relire les mémoires en entier ; demander « quelle voie » avant d'optimiser celle en place |
| **Un réglage hérité d'un autre outil** : tout le lissage de la navigation a été réglé au pavé tactile, et la souris en a hérité | une voie par source, jamais un réglage commun |

## 5. Les pièges pratiques

* **Git Bash mange les barres obliques inverses** dans un heredoc : écrire un script ou un message
  de commit avec l'outil d'écriture, dans le scratchpad, puis `git commit -F fichier`.
* **PowerShell ne distingue pas les majuscules** dans les noms de variables (`$tauri` est
  `$Tauri`) : un script d'épreuve s'y est trompé.
* **Les épreuves tournent en parallèle** : deux qui écrivent le même fichier temporaire tombent
  au hasard (nommer par l'empreinte du contenu).
* **Une épreuve graphique se saute sans carte** ; sur GitHub, `GLUCOSE_EXIGER_UNE_CARTE` la fait
  tomber au lieu de mentir en vert.
* **Une épreuve de temps** qui tombe sur une machine de GitHub dit que la manière doit changer,
  jamais que la borne doit monter.
* **`cargo test --workspace` compile beaucoup** : `target/` a atteint 94 Go avant le rangement du
  06/10. `cargo clean` de temps en temps ne coûte qu'une recompilation.

## 6. À la fin d'une session

1. Mettre à jour [ÉTAT](ETAT.md) (ce qui est vrai maintenant) et cocher [SUITE](SUITE.md).
2. Si le travail est important, écrire une fiche datée dans le carnet (numéro suivant), et sa ligne
   dans [`carnet/00-INDEX.md`](carnet/00-INDEX.md). Le code cite les fiches par leur numéro : on
   ne renumérote jamais.
3. Lui donner **une seule** liste d'essais à l'écran, courte, avec la commande qui écrit la sortie
   dans un fichier.
4. Ne jamais écrire un jeton dans le dépôt.
