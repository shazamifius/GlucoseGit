# 118 — Sa capture d'écran : les boutons de Windows, les tiroirs, les bouts de flèche, le miroir des images

> Session du 17/11/2026, de 14 h 28 à la nuit. Il a envoyé **une capture d'écran annotée** de Glucose
> (douze défauts, ses mots ci-dessous) et demandé deux choses : *« corriger TOUS les bugs présents dans
> la capture d'écran »*, puis *« pouvoir répondre à toutes les questions »* qui attendent sa parole —
> *« une fiche avec le contexte de la question, quelle est la question, quelles sont tes
> propositions »*. Et de juger la session d'avant, arrêtée en plein changement (les photos du
> co-working, B1). Son PC s'éteint à 1 h (sa demande, `shutdown -P 01:00` et un minuteur de secours).

## 1. Ce qu'il a écrit sur sa capture (mot pour mot, orthographe gardée)

* *« pourquoi le story board es aussi long a sa creation ? »*
* *« aucun systeme permettant de suprimer des point orange d'une fleche »* ; *« aucun systeme
  permettant de bouger indepandament les point A et B dune fleche »*
* *« dailleur le tiroire de domaine lui a litteralement UX SUR le bouton pour refermer le tiroire »*
* *« UX de dossier lui es carrement couper en 2 ne permetant meme pas de bien naviguer »*
* *« pourquoi es ce que UX a des bulle partout non cohérent avec ecran qui en plus de sa
  sentrechoque avec les autre bule souvent trop grosse comme le styboard time machine et domaine et
  vraiment aucune structure particulière qui permeterai identifier pourquoi ces tiroire souvre comme
  sa »*
* *« sur enormment ecran integraliter des bouton UX que ce soit des nom des mots des logo et bien ces
  très très pixeliser alors que je posede souvent des ecran bien plus performant »*
* *« sur linux dailleur glucose ne ce mes pas en pleinne ecran et tout le systeme de REF avec ctrl +
  maj + A ne fonctionne tout simplement pas pour integraliter des bureau linux… ce qui serai bien ce
  serai avoir : un mode fennetre classique […], un mode qui force le pleine ecran, et un mode qui force
  le fait être au dessus de toute les application »*
* *« lorsque on essai sur windows de cliquer sur les bouton de la barre du haut on ce retrouve a faire
  une selection qui nous amenne vers le haut […] plus le fait avoir retirer de manière pas super propre
  la barre blanche windows […] le combo des 2 fait que AUCUN bouton n'es accessible sur windows »*
* *« crée une option de flip pour les image »* ; *« j'ai imprétion aussi que les clique box des bouton
  son decaler ??? »*

Et dans la fiche de questions, après vérification sur un vrai Windows : *« oui après vérification c'est
exactement ça ! les boutons fonctionnent, ils sont juste décalés ! merci de les corriger »*.

## 2. Chaque défaut, sa cause, sa loi

Tout a été **reproduit dans le vrai Glucose, au pilote**, avant d'être corrigé, et regardé après.

| défaut | cause | la loi |
|---|---|---|
| **Windows : aucun bouton accessible, clics décalés** | Le dessin, le survol et les zones données à Windows retiraient la place de réduire, agrandir, fermer (BARRE-1) ; le clic ne la retirait pas : chaque bouton de droite cherché 138 px plus loin (207 à 150 %). Un clic sur « Domaines » ouvrait « Collaborer ». | **BARRE-2** : la place se retire dans `poser_la_barre`, la seule fonction qui pose la barre. |
| **…et la sélection qui monte** | Un clic tombé dans le vide de la bande descendait au canevas : sélection élastique, que le défilement au bord emportait. | **VIDE-1** : la bande prend tout clic qui tombe sur elle (`click_chrome`). |
| **Le dossier coupé en deux** | Le fil d'Ariane donnait à `draw_text` une ligne de base (70 % de la bande) là où elle attend le haut du corps : les lettres 12 px trop bas, coupées par leur tampon (CHROME-1). | **ARIANE-1** : `Typography::haut_d_une_ligne_centree`, la boîte de la ligne (montante, descente) centrée. |
| **Le Storyboard « aussi long »**, la Time Machine | Leur hauteur valait `f32::MAX` : tout l'écran pour deux rangées. | **TIROIR-2** : chaque panneau mesure sa hauteur sur sa mise en page (`hauteur_naturelle`, `reste`, `bas`) — Storyboard, Time Machine, Domaines, Plugins, le Mood board (`hauteur_de`, sans la place d'un gabarit suivi absent) ; Ordonner et Pomodoro gardent une hauteur écrite, **éprouvée égale à leur contenu** (le Pomodoro : 184, et non 210). |
| **Domaines sur sa poignée** | Le contenu d'un panneau du haut était posé sur toute sa hauteur, poignée comprise ; « + Nouveau domaine » est ancré en bas. | **TIROIR-1** : `PanelLayoutBox::contenu`, le cadre sans la bande de la poignée. |
| **Les tiroirs qui s'entrechoquent** | Deux rangées partent du bord gauche (haut, bas) ; un panneau « toute la hauteur » couvrait l'autre rangée. | **TIROIR-3** : un panneau qui en croiserait un autre glisse à sa droite (`hors_des_autres`). |
| **Les bouts A et B d'une flèche** | Aucune poignée (Tauri n'en avait pas : ses deux disques étaient décoratifs). | **BOUT-1** : `HandleKind::Extremite`, glissée par la loi d'accroche de l'outil Flèche (SNAP-3), jamais vers le nœud de l'autre bout ; `store/bouts.rs` (rebrancher, l'autre bout). |
| **Supprimer les points orange** | Le double-clic les retirait, sans que rien ne le dise. | « **Redresser la flèche** » (menu, palette, seulement s'il y a des coudes) ; **COUDE-1** : le clic droit sur un coude propose « Retirer ce coude ». |
| **Retourner une image** | N'existait pas (ni dans Tauri). | **MIROIR-1** (§ 3). |
| Plugins (vu en regardant) | « Densité » posé sur la légende « RÉGLAGES ». | La rangée part sous la carte, à 52. |

Ce qui **attend sa parole** au lieu d'être tranché (la fiche de questions, § 5) : la structure des
tiroirs (une colonne, des onglets, des tiroirs libres ; des bulles ou des panneaux collés au bord), les
trois modes de fenêtre, la barre blanche de Windows, et le texte « pixelisé » — **non reproduit** ici
(1920 × 1080 et 2560 × 1440, à 100 et 150 %, sous Wayland : lissé) ; sa capture est réduite, il faut sa
machine, son écran, son échelle, et une capture non réduite.

## 3. MIROIR-1 — retourner une image

* **Un seul interrupteur**, `BoardImage::miroir` : retourner de haut en bas, c'est retourner de gauche à
  droite puis un demi-tour. Écrite `R(θ)·M^m` (le miroir d'abord), l'image retournée de gauche à
  droite est `R(−θ)·M^(m+1)`, de haut en bas `R(π − θ)·M^(m+1)` — éprouvé par la matrice de l'image à
  l'écran, pour cinq angles, avec et sans miroir (`miroir_suite.rs`).
* **Schéma du document 6.** Un v5 se relit (témoin `riche-v5.glucose`, écrit par `c44ca9a`, la dernière
  build v5) ; une version d'avant refuse un v6 par son message. Chaque geste du journal porte son
  schéma : un document qu'une build v6 a touché ne s'ouvre plus en 2.0.4-beta.1 — **sa mise à jour
  automatique sous Windows et Linux, l'APK à la main au téléphone.**
* **La carte graphique** lit la fenêtre de la photo de droite à gauche : une largeur négative
  (`Pose::retournee`), le nuanceur (`uv = fenetre.xy + c × fenetre.zw`) inchangé. **Le processeur**
  pose la photo par un miroir autour du centre puis la rotation (`pose_a_l_ecran`). Les deux voies
  d'accord, recadrage compris (`test_miroir_1_les_deux_voies_retournent_pareil`).
* L'export SVG et la page web le disent ; l'arène le garde (une table éparse : les huit drapeaux sont
  pris) ; les tuiles le comptent dans leur empreinte.
* **Recadrer une image retournée** tire le bord qu'on voit (`Recadrage::en_tirant_le_bord_vu`, dans le
  noyau) ; « retirer les bordures » recule sa boîte du bon côté.
* **Les gestes** : « Retourner horizontalement » `Alt+Maj+H`, « verticalement » `Alt+Maj+V` — les
  raccourcis de PureRef ([manuel](https://www.pureref.com/handbook/images/)) —, au menu d'une image et
  à la palette ; chaque image sur elle-même ; une image verrouillée ne se retourne pas ; un seul
  `Ctrl+Z`. Une IA ne lit encore ni la rotation ni le miroir d'une image (`glucose_lire`).

## 4. Remis en question, et tranché seul

* **La session d'avant (B1)** : le correctif des photos fait 52 lignes dans 4 fichiers, hors du dépôt, et
  s'applique encore. Rien n'est cassé ; la correction de Windows touche tous ses utilisateurs Windows dès
  le premier clic. **D'abord sa capture, puis ses réponses, puis B1.**
* **La structure des tiroirs** n'est pas tranchée : seules les trois lois qui valent pour toute
  structure (hauteur de ce qu'on montre, contenu hors de la poignée, jamais l'un sur l'autre).
* **Le format** : écrire le plus petit schéma qui suffit (v5 tant qu'aucune image n'est retournée)
  garderait la compatibilité, mais chaque geste du journal porte son schéma : deux écritures pour un
  champ. Écarté — la v6 pour tous, dite.
* **Un retournement par image, sur elle-même** (pas un miroir de la disposition de la sélection,
  comme Figma) : on retourne une référence, pas une mise en page.

## 5. Ce qui attend sa parole — la fiche de questions

**[« Glucose — ce qui attend ta parole »](https://claude.ai/artifact/KwCMGG6Keymuzr5V4DJ4gN)** (un doc
Claude, où il répond en prose) : 19 questions — les quatre de sa capture (§ 2), les onze de SUITE § 4 et
des fiches 98, 100, 102, 105 (la ligne d'arrivée, la couleur des régions, la frontière des membranes,
se repérer, le dossier, glisser vers une fenêtre, trois gestes, la sélection, les grandes idées, ses
clés, Protéo), les quatre choix du co-working (le conflit par propriété, le lecteur, les photos du
retardataire, son nom) et l'ordre après B1. Il y répondra demain ; **ses réponses vont dans VISION.**

## 6. Les épreuves, les sabotages, la vérification

* **Six anciennes épreuves** supposaient l'ancien comportement : trois cliquaient à y = 20 à 50, sous la
  bande (dans le vrai Glucose, ces clics touchent la bande) ; trois supposaient les hauteurs fixes.
  Réécrites selon les lois neuves, sans rien relâcher. Deux empreintes témoins regardées et mises à
  jour (les poignées des bouts, « Redresser la flèche », puis les deux retournements au menu).
* **La suite entière : 3 318 vertes** sur `1ccd2bee` ; **la vérification de GitHub verte, dix tâches**,
  Windows compris (le vrai Glucose sans sa barre, sous un vrai Windows).
* **Les sabotages** (`outils/sabotages/capture-118.json`) : 19 sur 19 pour la capture — dont **un
  aveugle trouvé** (oublier la descente de la police décalait les lettres d'un pixel et demi, qui
  restaient dans leur bande ; l'épreuve vérifie désormais la loi exacte) ; puis le miroir et COUDE-1.

## 7. La recherche (consultée le 17/11/2026)

* **« Toujours devant » sous Wayland** : aucune requête standard pour un client ;
  [xdg-toplevel-tag](https://wayland.app/protocols/xdg-toplevel-tag-v1) (wayland-protocols 1.43,
  [Phoronix](https://www.phoronix.com/news/Wayland-Protocols-1.43)) identifie une fenêtre pour que le
  compositeur applique ses règles ; **niri ne sait pas encore** garder une fenêtre devant
  ([FAQ](https://niri-wm.github.io/niri/FAQ.html), [issue 932](https://github.com/niri-wm/niri/issues/932)) —
  une fenêtre flottante y passe au-dessus des tuilées. Pistes, selon le bureau : KWin (script par
  D-Bus), Hyprland (`pin`), sway (`sticky`), niri (flottante par son IPC), X11/XWayland
  (`_NET_WM_STATE_ABOVE`) ; GNOME n'a rien.
* **PureRef** retourne par `Alt+Maj+H` et `Alt+Maj+V`
  ([manuel 2.0](https://pureref.com/handbook/2.0/images/)).

## 8. Les leçons

* **Une épreuve qui cherche le bouton par la fonction du clic ne voit jamais un clic décalé** : elle
  trouvait chaque bouton là où le clic le cherchait. L'épreuve compare désormais ce que **le dessin**
  pose à ce que le clic actionne — et le vrai bug était là depuis BARRE-1 (fiche 85), invisible sans
  Windows.
* **« Ça tient dans sa bande » n'éprouve pas « c'est centré »** : un sabotage d'un pixel et demi
  passait. Éprouver la loi, pas une conséquence large de la loi.
* **Une épreuve qui clique sous la bande clique là où l'utilisateur ne peut pas** : trois épreuves
  vivaient d'un défaut (VIDE-1). Une scène d'épreuve se pose sous l'interface.
* **Chercher un nom avant de le donner** : LIGNE-1, BANDE-1 et `Bout` existaient déjà.
* **Le disque s'est rempli** (233 Go, 930 Mo libres) pendant une suite lancée dans l'arbre des
  sabotages : sa cible avait gagné 45 Go, celle du dépôt en pesait 82. Supprimée (un cache), 45 Go
  rendus ; son Glucose n'était pas ouvert. La suite se lance désormais dans le dépôt (MÉTHODE § 5).
* **`git -C <arbre> checkout HEAD` désigne le HEAD de cet arbre**, pas celui du dépôt : un sabotage
  rejoué sur l'ancien commit a « passé » une seconde fois. Nommer l'empreinte.

## 9. Ce qui reste

1. **Ses réponses** à la fiche de questions — puis les mettre en œuvre (structure des tiroirs, modes de
   fenêtre, barre de Windows, échelle de l'interface et dessin des lettres).
2. **La 2.0.4-beta.2, en brouillon** (le flux « Publier » vert à 20 h 59, onze fichiers, sur `4427733` ;
   la CI verte sur dix tâches sur `4dfba2a`) : son clic « Publish release », puis l'essai à l'école sous
   Windows. Son Glucose installé est à `4dfba2a`.
3. **B1, les photos du co-working** (`~/.cache/glucose-bancs/photos-117/photos-b1.patch`, fiche 117 § 7 bis).
4. Le reste de SUITE § 2 (son lag, la porte avec son compte, les boutons manquants).
