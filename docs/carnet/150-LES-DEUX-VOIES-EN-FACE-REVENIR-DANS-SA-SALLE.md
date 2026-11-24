# 150 — Les deux voies, en face ; « Revenir dans sa salle » (24/11 vers minuit, le relais, compte b)

## 1. PARITE-19 — chaque outil des IA, avec son geste en face

Sa note 19 (`relais-bugs-liste`) : *« chaque outil des IA avec ton geste en face »*. La table vit
dans le code, à côté de la liste des outils : `agents/outils/parite.rs`, `EN_FACE` — pour chaque
outil `glucose_*`, les commandes de Glucose (leurs identifiants de `commandes.rs` : la touche, la
palette) et le geste que la main y ajoute, ou **ce qui manque**. Deux épreuves la tiennent juste :
chaque outil annoncé a sa ligne, une seule, et aucune ligne pour un outil disparu ; chaque commande
citée existe (`Commande::de_l_id`) ; une ligne dit au moins une chose. Un outil neuf sans son geste
ne passe donc pas. **5 sabotages sur 5** (`outils/sabotages/parite-19.json`), 3 668 épreuves vertes.

**Pourquoi dans le code plutôt qu'une page** : une page se périme au premier outil ajouté, et un
identifiant de commande renommé ne s'y voit pas ; ici, les deux font tomber une épreuve.

**Ce que la table a montré** — ce que seule une IA fait encore :
* **poser une page web** (`glucose_poser_page`) : coller ou déposer une adresse ne pose pas sa page
  (`interactions/drop.rs` ne prend que des fichiers) ;
* **« quoi de neuf depuis mon dernier regard »** (`glucose_quoi_de_neuf`) : la Time Machine montre
  l'histoire, sans marquer où il l'avait laissée ;
* **répondre en un geste** (`glucose_repondre`) : à la main, deux gestes (le texte, puis la flèche) ;
* **une note sur un geste de l'histoire** (`glucose_noter`) : la main nomme un moment (un jalon),
  pas un geste.

Ces quatre manques sont dans son Glucose (`relais-parite-19`) : à lui de dire lesquels comptent.

## 2. « Revenir dans sa salle » (fiche 148 § 3, le 1)

Dans Collaborer, seul, un document **dehors** qui a gardé sa salle montre « Revenir dans sa salle »
(et son texte) au lieu de « Partager ce canevas » : le bouton dit ce qu'il fera, sous les mêmes
codes. `collaborer.rs` était à 594 lignes : la question « seul » est passée dans
`collaborer/revenir.rs` (`question_seul`). Gardé par l'épreuve du « Quitter »
(`revenir_quitter_garde_le_code_et_partager_y_revient`) et par celle du bouton Collaborer (un
document neuf dit toujours « Partager ce canevas ») ; 2 sabotages sur 2. `470ad40`.
