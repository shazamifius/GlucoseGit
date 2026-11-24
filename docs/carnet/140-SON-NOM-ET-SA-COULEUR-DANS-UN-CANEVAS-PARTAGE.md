# 140 — Son nom et sa couleur dans un canevas partagé (Q19)

> Le 24/11/2026, de 7 h 35 à 8 h, session du relais (compte b), après la fiche 139. Sa réponse à
> Q19, lue le 23/11 : *« carrément, je suis plutôt d'accord : on demande le pseudo ainsi que la
> couleur d'affichage »*. Notre proposition (`relais-rep-q19`) : demander les deux la première fois
> qu'il partage ou rejoint, une seule fois, et les changer ensuite dans Collaborer ; dire quand deux
> personnes ont la même couleur, sans choisir à sa place.

## 1. Ce qu'il y avait

Un nom tiré au sort la première fois (« Aigle Émeraude », comme Glucose Tauri,
`src/multiplayer/localUser.ts`), gardé dans `identite.txt`, et une couleur parmi les dix de Tauri —
**qu'on ne changeait que dans ce fichier**. Son autre appareil s'appelle « Aigle Émeraude » depuis
le 23/11.

## 2. Ce qui est fait — IDENTITE-2 et PASTILLES-1

* **L'identité retient qu'il l'a choisie** : une troisième ligne, `choisie`, que les versions
  d'avant ignorent (elles lisent les deux premières, vérifié par l'épreuve). Un fichier d'avant se
  relit « tirée ».
* **La première fois** qu'il clique « Partager ce canevas » ou « Rejoindre… » avec une identité
  tirée, Glucose demande d'abord « Ton nom et ta couleur » — le champ, son nom tout choisi (ce qu'il
  tape le remplace), et **dix pastilles** —, puis fait ce qu'il demandait. « Annuler » revient à
  Collaborer sans rien partager. **« Ton nom et ta couleur… »** est dans Collaborer, seul comme
  partagé.
* **Les pastilles** (PASTILLES-1, `ui/question/pastilles.rs`) : une rangée sous le champ, une case
  par couleur — la case entière est la cible d'un doigt, le carré au centre se voit ; la choisie est
  cernée du filet de la sélection. Au clavier, `Alt` + gauche ou droite (les flèches seules
  déplacent le curseur du champ, et doivent le garder) ; sans champ, les flèches seules.
* **Dans un canevas partagé**, « Garder » change le nom et la couleur du partage : chaque lettre les
  porte, et chez les autres chacune les relit (`un_pair`) — le changement part avec le geste ou le
  mouvement de souris suivant. Le facteur des photos reçoit les siens (`Course::Signer`) : ses
  lettres ne changent pas le nom affiché chez les autres, mais elles ne doivent pas mentir.
* **La couleur déjà prise** par quelqu'un qui est là : la question l'écrit (« « Loup Vert » a déjà
  cette couleur »), à chaque pastille touchée ; elle ne choisit pas à sa place.
* **Un nom vide** se redemande, la couleur choisie gardée.

**La couleur reste l'une des dix** — nous avions écrit « ta couleur sur la roue ». Deux raisons :
elles ont été choisies pour rester lisibles en pastille de quelques pixels sur le fond sombre, et la
lettre porte le **rang** de la couleur, qu'un Glucose déjà installé (la beta.5, ses autres
appareils) comprend ; une couleur libre demanderait un champ neuf dans la lettre, que les versions
publiées ne liraient pas. À contester s'il veut la roue : elle viendrait avec une publication.

## 3. Prouvé

* `identite_2_*` (5 épreuves, par les vrais chemins : le menu, la souris sur une pastille, `Alt` +
  flèche, le bouton « Garder ») et `test_pastilles_1_sous_le_champ_au_dessus_des_reponses` ; les
  trois épreuves de REJOINDRE-1 passent par la question du nom la première fois, comme lui.
* `identite-140.json` : **8 sabotages sur 8**.
* **Au pilote** (`~/.cache/glucose-bancs/q19-140/`) : Collaborer montre « Ton nom et ta couleur… » ;
  « Partager ce canevas » dans une maison neuve pose d'abord la question ; la pastille rose touchée
  se cerne, « Shaza » tapé remplace le nom tiré ; « Garder » écrit `Shaza / 4 / choisie`. **À la
  taille de son téléphone** (1080 × 2400 à 275 %) : les dix pastilles tiennent sur une ligne.

## 4. À contester

* Le cadre de la pastille choisie est le filet de la sélection, d'un pixel : discret sur un écran
  très dense.
* Deux personnes de la même couleur restent possibles : Glucose le dit et laisse faire, comme
  promis.
