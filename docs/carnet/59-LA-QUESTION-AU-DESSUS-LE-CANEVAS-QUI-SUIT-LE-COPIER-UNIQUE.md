# 59 — La question au-dessus des panneaux, le canevas qui suit, le copier unique

> Session du 09/10/2026, au soir. Ses retours après ses essais, un par un, dans l'ordre qu'il a
> conseillé : *« pour les questions d'ergonomie, ne t'y concentre pas d'abord ; corrige plutôt les
> problèmes — l'ergonomie, la praticité, ce sont des domaines beaucoup plus longs et complexes »*.
> Le plan lui a été montré avant tout travail, et accepté.

---

## 1. Ses retours, mot pour mot ou presque

**Au bureau** — « vraiment plutôt cool ». Ce qui manque beaucoup :

1. un système qui va chercher sur Internet l'image dans sa plus grande qualité, et surtout son
   **origine**, avec le nom de l'artiste et plusieurs liens pour la retrouver (la fiche 27) ;
2. l'ergonomie des boutons et des fenêtres, « pas très pratique » ;
3. glisser une image pour la poser loin, hors du cadre visible : le canevas ne suit pas, « comme
   dans les suites » où le canevas fait dérouler les choses — l'idée de la boucle de Blender, « mais
   le glisser vers un autre Glucose ne fonctionnerait plus : on ne peut pas quitter l'écran » ;
4. « copier l'image » et « copier » sont deux choses, et devraient n'en être qu'une ; `Ctrl+V` dans
   Discord doit envoyer l'image, plusieurs images les envoyer toutes, « partout, pas seulement
   Discord » ; à défaut, les regrouper en une seule dans la disposition de l'auteur ; et dans
   Glucose rien ne change : un bloc colle un bloc ;
5. le glisser pareil : on glisse une image vers un autre Glucose, pas vers Google Docs ou Discord
   — « une image reste une image ? ».

**Au téléphone** :

1. la question du journal technique s'affiche derrière Ordonner et Pomodoro, qu'on ne peut pas
   fermer — on ne voit pas la question ; et commencer Glucose avec deux panneaux ouverts, « c'est
   un peu chiant » : aucun menu au lancement ;
2. pas de bouton « Supprimer », ni pour une membrane ni pour un document ;
3. des glitchs presque perpétuels — des blocs blancs, des rayures multicolores ;
4. la barre d'outils (le rail) « super moche, prend toute la place ».

## 2. Ce que la lecture a établi avant tout plan

* **Ses captures ne viennent pas toutes du même Glucose.** L'horloge du téléphone et la télémétrie
  les datent : celles de 23 h 28 à 23 h 43 sont du **07/10 au soir** (session `3994f86`,
  23 h 35 → 23 h 43), avec l'APK de la fiche 56 (`a3115ee`, 23 h 25) — avant le bord à bord et
  l'appui long. D'où la barre d'état sur les onglets, et la barre « 1 sélectionné · Supprimer »
  **sous** la barre de navigation. Celles de 01 h 06 et 01 h 08 viennent de l'APK actuel (session
  `09dbd55`, la batterie lue à 76 %) : le bord à bord y marche, et **les rayures y sont**.
* **La question sous les panneaux était toujours dans le code actuel**, sur les deux voies, alors
  que le clic, lui, allait déjà à la question d'abord.
* Le glisser vers une autre fenêtre ne portait que le lot et du texte : aucune image.
* « Copier » et « Copier l'image » s'effaçaient l'un l'autre.
* Le journal technique ne dit ni la voie, ni le moteur, ni la carte du téléphone.

## 3. DECISION-1 — la question passe par-dessus les panneaux

* **L'ordre des couches, écrit une fois** (`renderer/decision.rs`) : le canevas, la chrome
  permanente (la bande, le fil d'Ariane, la minimap, le rail), les panneaux, puis **ce qui attend
  une décision** — la barre d'action, les options de flèche, la fenêtre des ancres, le toast, le
  menu, la question. C'est l'ordre même dans lequel le clic descend
  (`interactions::mouse::handle_left_down`) : ce qu'on voit au-dessus est ce qu'on touche.
* **Voie processeur** : la scène et la chrome, les panneaux, la décision, le liseré du passé.
* **Voie graphique** : sans panneau ouvert, la décision reste dans la couche du dessus — rien ne
  coûte de plus. Avec un panneau, elle se peint dans un tampon à elle (`app/decision.rs`), que la
  carte pose **après** les panneaux ; seules ses lignes partent, et sa clé ne change que si ses
  pixels changent.
* **Aucun panneau ouvert au lancement** (`dock.rs`), au bureau comme au téléphone.
* `ui.rs` perd le groupe déplacé (`ui/decision.rs`) : quatre-vingt-dix lignes de moins.
* **Vu à l'œil** : l'aperçu du téléphone (`examples/apercu_telephone.rs`,
  `telephone-question-sur-les-panneaux.png`), rendu par le peintre de l'application, montre la
  question au-dessus d'Ordonner et de Pomodoro, voilés.
* Huit épreuves, huit sabotages, huit chutes. Le banc rend « l'interface comprise » : il pose aussi
  la décision, et ses empreintes sont revenues au bit près.

## 4. Supprimer une membrane, un document, au téléphone

**Rien à corriger dans le code actuel** : l'APK de ses captures n'avait ni appui long ni marges du
système. Une épreuve le joue désormais par de vrais évènements de doigt — un appui long au milieu
d'une membrane vide la choisit, et son menu offre « Supprimer » ; la barre d'action est au-dessus
de la navigation (vu sur l'aperçu) ; un document se supprime par un appui long dans la liste
(DOCUMENTS-2). **À confirmer sur son téléphone**, avec l'APK neuf. Que l'appui long ne se devine
pas relève de l'ergonomie.

## 5. DEFILE-1 — le canevas défile quand on tient un nœud près du bord

* **La recherche** : *Push-Edge* (Malacria, Aceituno, Casiez, Roussel, CHI 2015) — la vue avance
  de ce que la main **pousse** contre le bord de l'écran, jusqu'à 13 % plus rapide que le
  défilement classique au bord, à la souris. C'est ce que le plan proposait.
* **Ce qui l'a fait abandonner, en lisant son usage** : il glisse surtout **au pavé**, bouton
  tenu — la poussée ne mène qu'à la longueur d'un passage du doigt ; sous Windows, le bord du bas
  d'une fenêtre plein écran est la **barre des tâches**, où le curseur sort, et le glisser
  partirait vers une autre fenêtre ; et « dérouler », son mot, est un défilement qui **continue**
  tant qu'on tient le nœud là.
* **Ce qui est fait** : une **bande** au bord du canevas, comme Word, l'explorateur, Figma et
  tldraw (`edgeScrollDistance`, `edgeScrollSpeed`). Sa largeur est la cible d'un doigt
  (`theme::CIBLE_DU_DOIGT`, 48 points, désormais écrite une fois au lieu de trois). La vitesse
  est un **nombre de ressenti** : au plus profond, une étendue du canevas par seconde, au carré
  de l'enfoncement. La souris, le pavé et le doigt passent par le même chemin ; sortir de la
  fenêtre fait toujours partir le glisser vers une autre.
* **Ce que la main tient suit la vue**, d'où que vienne le mouvement (`les_gestes_suivent_la_camera`) :
  les nœuds glissés, le coin tiré, le premier coin du rectangle de sélection — qui reste où il
  était dans le monde.
* Une raison de réveil de plus, « le bord fait défiler ».
* Sept sabotages, sept chutes. **À juger à son écran** : la vitesse et la largeur de la bande.

## 6. COPIER-1 — un seul copier, et une image reste une image

* **Lu à la source de Chromium** — donc de Discord, de Google Docs, de tout navigateur
  (`third_party/blink/renderer/core/clipboard/data_object.cc`, `ui/base/clipboard/clipboard_win.cc`) :
  chaque fichier d'un `CF_HDROP` devient une pièce jointe, **et** un bitmap présent (`CF_DIB`,
  que Windows tire de `CF_DIBV5`) en ajoute une. D'où la forme :
  * **une image seule** : un PNG (le format « PNG », que Chromium lit d'abord) et ses pixels en
    `CF_DIBV5` — ce que pose « Copier l'image » d'un navigateur ;
  * **plusieurs images** : la liste de leurs fichiers, octets d'origine, comme l'explorateur — et
    **pas** de bitmap, qui ferait une pièce jointe de trop ;
  * **toujours** le lot de Glucose (un bloc recolle un bloc) et le texte des nœuds de texte ;
  * le glisser emporte les fichiers dès une image : Discord, Docs, l'explorateur, le bureau.
* **Un seul objet Windows** (`plateforme/selection_windows.rs`, `IDataObject`) part au
  presse-papiers (`OleSetClipboard`) et dans le glisser (`DoDragDrop`).
* **Rien ne s'écrit avant qu'on le demande** : écrire les originaux à chaque `Ctrl+C` — trente
  images, des dizaines de mégaoctets — pour recoller dans Glucose, qui ne les lit jamais, aurait
  été du travail perdu. Les fichiers s'écrivent au premier collage qui les veut, une fois, dans
  `%TEMP%\glucose-copies\<date du lot>\` ; à la fermeture, `OleFlushClipboard` rend tout. Le
  dossier d'une copie vit jusqu'à la suivante ; un glisser ne l'efface pas.
* **« Copier l'image » disparaît du menu** : « Copier » fait tout.
* **Son idée de repli** — les images composées en une, dans la disposition de l'auteur — ne va pas
  dans le presse-papiers à côté des fichiers : Discord la recevrait en plus. Elle deviendra
  l'**export PNG** de la sélection, qui manque de toute façon (fiche 03, 17.6).
* **Ce qui reste** : sous Linux et Mac, `arboard` 3.6 ne pose qu'une forme à la fois — le lot, comme
  avant ; sous Android, le presse-papiers du système reste à faire (la suite, § 2).
* Neuf sabotages, neuf chutes. **Pas prouvé** : ce que Discord et Docs font vraiment — l'essai
  est à son écran (le jouer ici aurait écrit dans son presse-papiers pendant qu'il travaille).

## 7. Les rayures du téléphone — le diagnostic, où il en est

* **Pas reproduites ici** : les quinze épreuves qui comparent les deux voies passent sous Vulkan,
  GL et DX12 sur ce PC (`WGPU_BACKEND`, lu désormais par `banc_gpu::carte`). La logique de Glucose
  est donc juste sur ces pilotes ; la cause est **probablement** propre au téléphone — son pilote
  Mali, sa mémoire, ses limites. À confirmer.
* La recherche : wgpu sur Mali en Vulkan, des textures qui clignotent mais justes en GL
  ([wgpu #2399](https://github.com/gfx-rs/wgpu/issues/2399)) ; des artefacts Vulkan sur Mali-G52
  ([Flycast #1356](https://github.com/flyinghead/flycast/issues/1356)) ; des corruptions liées au
  pas des rangées sur Mali ([Flutter #193551](https://github.com/flutter/flutter/issues/193551)).
* **PLAFOND-1, fait** : une photo ne demandait jamais si la carte acceptait sa texture — seul le
  tampon de la fenêtre était vérifié. Un téléphone ou une vieille tablette refuse souvent plus de
  4 096 ou 8 192 pixels de côté. La présentation dit désormais son plafond au rendu à chaque image,
  et le niveau d'une photo se réduit jusqu'à tenir — le cran du budget compte de même. C'est une
  cause possible des rayures, et un défaut réel quoi qu'il en soit (« aucune machine exclue »).
  Cinq sabotages, cinq chutes.
* **Le journal du téléphone dit enfin sa carte** au lancement : le nom, le moteur (Vulkan ou GL),
  le pilote, le plafond des textures — lisible par le câble (`adb logcat -s Glucose`).
* **Pas fait, et pourquoi** : la ligne « carte » du journal technique. Le serveur refuse toute
  session qui porte une ligne inconnue, et Glucose ne renvoie jamais une session refusée : l'écrire
  avant de redéployer le serveur perdrait les sessions de son téléphone. Redéployer est un geste
  public — à sa parole. Et la vérification des deux voies **sur l'appareil**, au lancement : elle
  ne se justifie que si son journal accuse le pilote.

## 8. Ce qui n'est pas prouvé

* Rien de cette fiche n'a tourné sur son écran ni sur son téléphone.
* Le collage dans Discord et Google Docs, le glisser vers eux, et vers le bureau.
* La vitesse du défilement au bord : un nombre de ressenti.
* La cause des rayures.
* Le code de Linux et Mac changé pour le copier ne compile pas sur ce PC (la caisse `ring` veut un
  compilateur C de Linux) : relu à la main, il ne change que l'origine de ses arguments ; la CI le
  compilera.

## 9. À essayer

**Au bureau** (`cargo run --release > sortie-essais-59.txt 2>&1`, ou la prochaine version) :

1. Glucose s'ouvre sans panneau.
2. Une image : `Ctrl+C`, puis `Ctrl+V` dans Discord — l'image part ; trois images, `Ctrl+C`,
   `Ctrl+V` dans Discord — les trois partent ; et `Ctrl+V` dans Glucose recolle le bloc.
3. Glisser une image vers Discord, vers Google Docs, vers le bureau.
4. Tenir une image près d'un bord : le canevas défile, et vite au tout bord.

**Au téléphone** (l'APK neuf) : la question du journal technique, au-dessus de tout ; un appui long
sur une membrane → « Supprimer » ; les rayures sont-elles encore là — et si oui, brancher le
câble une fois.

## 10. Ce qui attend sa parole

* Redéployer le serveur de télémétrie pour la ligne « carte ».
* Brancher le téléphone en USB une fois, pour lire son journal.
* La clé SauceNAO, quand on attaquera la provenance (§ 1, point 1).
* L'envoi et la CI.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
