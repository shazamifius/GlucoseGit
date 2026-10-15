# 74 — Tes raccourcis : la couche de l'utilisateur (P4)

> Session du 15 au 16/10/2026, après la fiche 73 (P8 fini et fusionné, P8 bis, la CI verte).
> Sa parole de passation : *« Après P8 : P4, la couche de raccourcis de l'utilisateur, sur le
> disque et modifiable »* — la suite de la fondation des presets (fiche 72 § 7). Puis il a dit
> *« continue »*.

## 1. Ce qui manquait

La fiche 72 avait fait des raccourcis **une donnée** : une couche de Glucose en texte, une pile
qui mélange les couches, une table de commandes, le menu et la palette qui lisent le clavier.
Mais rien ne permettait à l'utilisateur de **changer** un raccourci : il aurait fallu écrire une
couche à la main, et rien ne la lisait. R1 : une fonction n'est finie que branchée, visible,
annulable et enregistrée.

## 2. La recherche

* **Blender** : les préférences — keymap compris — s'enregistrent **d'elles-mêmes à chaque
  changement** (*« user preferences are automatically saved when changed »*) ; « Load Factory
  Settings » rend tout.
* **VS Code** : un éditeur de raccourcis, la source de chaque liaison (`@source:user`,
  `@source:default`), les **conflits marqués** — un accord lié à plusieurs commandes ; le JSON de
  l'utilisateur posé sur celui par défaut.
* **PureRef 2** : les raccourcis se changent dans les Réglages, dans `PureRef.ini` (`%APPDATA%`
  sous Windows, `~/.config` ailleurs) ; l'import d'un fichier de réglages remplace ce qu'on avait.

Sources : docs.blender.org (*Defaults*, « Preferences are automatically saved ») ;
code.visualstudio.com/docs/configure/keybindings ; pureref.com/handbook/settings,
pureref.com/handbook/shortcuts.

## 3. La conception

* **Une commande « Changer un raccourci… »** (et « Rétablir tous les raccourcis de Glucose… ») :
  dans la table, donc dans la palette, au clavier comme au doigt. Elle ouvre la palette avec un
  **but** (`But::ChangerLeRaccourci`) : la commande choisie n'est pas lancée, elle reçoit la
  question *« Appuie sur le nouvel accord — aujourd'hui : Ctrl+D. Échap renonce. »*, et trois
  boutons : **Retirer ce raccourci**, **Rétablir celui de Glucose**, **Annuler**. Pour ce but, la
  palette se trouve elle-même : son raccourci se change aussi.
* **La touche frappée** devient l'accord par la loi même des raccourcis (ACCORD-1) : lue deux
  fois par la fenêtre — telle qu'elle écrit, telle que les raccourcis la lisent (les chiffres par
  leur touche physique) —, dans `prendre_avant_tout`, la porte que les épreuves empruntent. Sur un
  clavier français, la touche marquée 5 écrit « ( » : l'accord retenu est `Ctrl+5`. Un
  modificateur seul attend la touche ; Échap seul renonce.
* **Changer veut dire remplacer** : la commande n'a plus que cet accord, et l'accord ne mène plus
  qu'à elle. Ce qu'il faisait d'autre est dit dans le message (*« « Dupliquer » : Ctrl+B — à la
  place de « Retirer les bordures » »*), comme VS Code signale un conflit.
* **Le noyau** porte la loi, pure (`Clavier::lier_seul`, `delier`, `retablir`, `Couche::oublier`)
  : elle n'écrit que dans la **couche du dessus**, celle de l'utilisateur ; le preset ne bouge
  jamais. Retirer laisse un accord partagé mener à l'autre commande, et ne retombe pas sur le
  preset (`-`). Rétablir fait oublier à la couche du dessus chaque accord qui touche la commande —
  ce qu'il faisait d'autre en dessous revient avec lui.
* **Le fichier** : `raccourcis.txt`, dans le dossier de l'application (celui des brouillons,
  posé par `habiter`), au format même de la couche de Glucose, avec un en-tête qui dit comment le
  lire et l'écrire. Il s'écrit **à chaque changement**, d'un bloc (SAVE-1) ; il se relit au
  lancement.

### Ses données avant tout

Un fichier qu'on aurait écrit à la main peut porter des lignes fausses. **Glucose ne le réécrit
jamais en silence** : s'il ne se lit pas, ou porte une ligne fausse, le changement vaut pour la
session, et le message le dit (*« — pour cette session : ligne 3 : « Zorglub » n'est pas une
touche »*). Seul « Rétablir tous les raccourcis de Glucose », qu'on confirme, le remplace. Ses
bonnes lignes agissent quand même ; une commande que Glucose ne connaît pas se dit, et se garde.

Les épreuves qui écrivent habitent **chacune leur dossier** : `GlucoseApp::new` habite un dossier
temporaire partagé par toutes les épreuves, et un `raccourcis.txt` qu'une d'elles y laisserait
changerait le clavier de celles qui tournent à côté.

## 4. Ce que la vérification a trouvé

* **Deux messages, le second effaçant le premier** : une ligne fausse et une commande inconnue
  dans le même fichier donnaient deux messages à la suite — seul le second se voyait. Un seul
  message dit tout désormais. L'épreuve qui l'a trouvé demandait les deux.
* **Ma « ligne fausse » ne l'était pas** : `Ctrl+  rien` se lit comme `Ctrl` et la touche `+`
  (« `Ctrl++` et `+` nomment la touche `+` », le lecteur l'écrit). L'épreuve passait sans
  exercer la protection ; elle porte désormais `Ctrl+Zorglub`, et vérifie qu'elle part bien d'un
  fichier protégé.
* **Le cliquet des messages** (cliquet 2) est tombé : 52 sites pour 48 permis — les quatre de
  cette couche. Un cliquet ne se relève pas : la couche dit désormais tout en **deux** sites (ce
  que le lancement trouve ; chaque changement, protégé, écrit ou refusé par le disque — la même
  phrase au même endroit, la règle que l'export avait posée). Et la même règle a trouvé ailleurs
  quatre messages dits en trop d'endroits : les domaines disaient « créé » et son refus,
  « supprimé » (avec ou sans nœuds) et son refus, « assigné » et son refus, chacun en deux ou
  trois sites. Fondus : **46**, et le plafond descend à 46 pour garder le gain.
* **Une garde que rien ne pouvait voir** : la prise de l'accord vivait dans `handle_key`, dont le
  `KeyEvent` ne se construit pas hors de la fenêtre. `handle_key` ne fait plus que lire la touche
  deux fois ; `prendre_avant_tout` décide, et les épreuves passent par elle.

## 5. Les preuves

* Le noyau : `test_changer_un_raccourci_remplace_et_dit_ce_qu_il_remplace`,
  `test_retirer_un_raccourci`, `test_retablir_rend_le_preset_a_cette_commande`.
* L'application, par sa palette et ses touches : de bout en bout (écrire « dupl », Entrée,
  `Ctrl` seul, puis `Ctrl+B` ; le fichier ; une application relancée), retirer puis rétablir,
  Échap, le fichier faux jamais écrasé (octet pour octet), tout rétablir, le chiffre AZERTY.
* `outils/sabotages/p4-utilisateur.json` : **treize sabotages, treize chutes** du premier coup
  (la loi du noyau, la protection du fichier, le message unique, la relecture au lancement,
  Échap, le modificateur seul, la touche lue telle qu'elle écrit au lieu de sa touche physique,
  la prise débranchée, l'écriture). Les vingt de la fiche 72 rejoués, la palette ayant changé.

## 6. Ce qui reste de P4

* **Ses yeux** : `Ctrl+Maj+P`, « raccourci », changer celui de « Dupliquer », relancer Glucose.
* Le **thème** et la **disposition des panneaux** comme données ; les presets PureRef, Figma et
  Miro comme couches (sa liste PureRef à venir) ; un choix de preset dans l'interface.
* Un **éditeur** de tous les raccourcis d'un coup d'œil (la liste, la source de chacun) : la palette
  montre déjà chaque commande et son raccourci ; une vue « mes changements » (le `@source:user` de
  VS Code) viendrait avec les presets.
