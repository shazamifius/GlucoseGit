# 136 — L'appui dans le texte qu'on écrit est au texte (EDIT-PRISE)

> Le 23/11/2026, de 22 h 58 à 23 h 15, session du relais : commencé par le compte b (le correctif de
> `pick.rs`, écrit dans ses dernières minutes, ni commité ni éprouvé), repris par le compte a (97 % de
> sa semaine, jusqu'au mur). Sa note du soir (`text-kpll-370`, membrane « Groupe » de `membrane-299`) :
> *« ok finalement c'est même pas à cause du fait qu'il y a trop de bazar, c'est que… tout simplement
> un texte, parfois on ne peut pas sélectionner à la souris une partie du texte car… ça quitte le bloc
> texte carrément »*. Réponse dans son Glucose : `relais-rep-370`.

## 1. Ce qui se passait

En écriture, un appui dans le texte ne va au texte que si **l'arbitre de clic** (PICK-1) le désigne en
premier. Trois choses le lui prenaient :

1. **la partie du texte qui a grandi pendant la saisie.** Le document n'apprend la hauteur du texte
   qu'à la validation (TEXT-FIT-1) ; l'arbitre ne connaît que la boîte du document. Un appui dans
   les lignes neuves tombait dans le vide : la saisie était validée, un cadre de sélection
   commençait — *« ça quitte le bloc texte »*. Il n'y a même pas besoin de taper : **ouvrir un texte
   mis en forme montre sa source** (`**`, `#`, les lignes vides), plus longue que le rendu, donc plus
   haute que sa boîte (`renderer/card.rs`, `boite_montree` : `TextMode::Source` en saisie) ;
2. **une poignée du bord** : sa prise (24 px, au plus 35 % du petit côté, POIGNEE-1) entre à moitié
   dans le texte ; un glisser parti de la première ou de la dernière ligne d'un petit texte
   redimensionnait ;
3. **une flèche dont le trait passe dans le texte** (rang 20 contre 40). Une flèche libre qui le
   traverse le **contourne** (le contournement des flèches) : seule celle dont un bout tombe dans le
   texte y passe — l'épreuve l'a appris en tombant à sa condition préalable.

**Reproduit au pilote** (`~/.cache/glucose-bancs/edit-prise-136/`, `demarrer.sh`) : un texte de trois
lignes, huit après la frappe ; l'appui dans la ligne 7 (sous l'ancien bas) puis un glisser vers la
ligne 8. **Avec l'ancien code** (`7c.png`) : rien n'est sélectionné, la saisie est quittée (le curseur
redevient une flèche). **Avec le correctif** (`6c.png`) : « sept, tapée. / Ligne huit, tap » sélectionné,
la saisie continue.

## 2. La loi (EDIT-PRISE, `interactions/pick.rs`)

**Dans la boîte que le texte qu'on écrit montre** — la boîte vivante de FLECHE-EDITION
(`renderer.boite_vivante`, posée avant chaque image) —, **rien ne passe devant lui** :
`pick_for_click` le rend d'office (`le_texte_ecrit_sous`), et les poignées de coude et de taille ne se
prennent pas dans cette boîte (`dans_le_texte_ecrit`). Ses poignées se prennent par leur moitié du
dehors. Aucune constante neuve : la boîte est celle que le dessin montre (la même leçon qu'ANCRE-1,
fiche 135 : ce qui se vise est ce qui se voit).

Le compte b cite tldraw, qui cache les poignées pendant l'écriture : **pas vérifié** par cette session
(sa semaine à 97 % ; aucune recherche faite). La loi tient sans cette référence.

## 3. Les preuves

* **Quatre épreuves** (`interactions/pick/edit_prise_tests.rs`), par le vrai chemin de la souris
  (`handle_cursor_moved`, `handle_mouse_down`) ; chacune vérifie d'abord que l'ancienne loi donnait
  l'appui à un autre (l'arbitre ne voit pas le texte ; la flèche passe devant ; la poignée prend le
  point) : la partie qui a grandi, la flèche dont le bout tombe dans le texte, la poignée dedans, et la
  poignée **du dehors** qui se prend toujours.
* **Trois sabotages sur trois** (`outils/sabotages/edit-prise-136.json`) : l'arbitre seul (deux
  épreuves tombent), les poignées encore prises dedans, toute la scène donnée au texte.
* **Regardé au pilote**, l'avant et l'après (§ 1).

## 4. Ce qui reste

* **Son écran** : son Glucose recompilé sur `7ee455e` ; sa relance le lui donne.
* **Hors saisie, la même question se posait** : `boite_montree` prend `max(hauteur du document,
  hauteur que le texte demande)` (`CardLayout::text_card`) ; si le document était plus bas, le bas du
  texte ne se prendrait pas d'un clic. **Relu dans le code** : chaque chemin qui écrit un texte mesure
  sa hauteur — l'ouverture d'un document (`fit_all_text_cards`, `persist/disque.rs`, `import.rs`,
  `ajout.rs`), l'outil texte (`tools.rs`), les IA par la porte (`agents/executer.rs`, trois outils), le
  redimensionnement (`resize.rs`). **Non relu** : un texte reçu d'un autre participant du co-working ;
  aucune épreuve ne garde l'égalité des deux boîtes hors saisie.
* La garde `ann_id == id` de `dans_le_texte_ecrit` (la boîte vivante d'un texte quitté juste avant
  qu'un autre s'ouvre, entre deux images) n'a pas d'épreuve.
* **Faute de cette session** : la passation a d'abord porté « 23 h 25 », une heure estimée au lieu
  d'être lue (il était 23 h 05) — la règle de MÉTHODE § 3 n'est pas une formalité.
