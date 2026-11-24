# 146 — Ce que sa main peut faire ; la beta.6 ; un document, une salle (24/11, le relais, compte b)

Ses notes du 24/11 (rangées par sa main dans 🐞 et 🤖 Relais) : *« il faut que IA et humain puissent
avoir les mêmes droits »*, *« le truc réponse existe pas »*, *« sur Windows le mode F11 ne fait
strictement rien à part légèrement réduire la fenêtre »*, *« je ne peux pas vraiment tester tout ça
avant l'arrivée d'une version BETA6 »*, et sur les codes : *« un lien ne périt jamais, sauf si
l'utilisateur le remet à jour »*.

## 1. GESTE-19 — ce que sa main peut déjà faire se voit

Il croyait réservés aux IA deux gestes qu'il a depuis la beta.5 : **écrire sur une flèche** (son
double-clic, `interactions/pick.rs`, `open_node`) et **la faire partir d'un passage** (« Éditer le
texte lié » dans la barre d'options, FLECHE-4). Le second se montrait ; le premier, jamais. Une
flèche choisie porte maintenant **« Écrire sur la flèche »** (`ui/options_de_fleche.rs`,
`Action::Ecrire` ; `interactions/chrome.rs`, `ecrire_sur_la_fleche`) : la même saisie que le
double-clic. Épreuve `test_geste_19_ecrire_sur_la_fleche_depuis_sa_barre` ; regardé au pilote sur une
copie de son document (le bouton à côté de « Éditer le texte lié », un clic, « essai » tapé dans la
pastille). **Reste** : la table de chaque outil `glucose_*` avec son geste en face (🔍 Revue du code
dans son Glucose).

**Vu au passage, au pilote** : un clic sur la pastille d'une flèche **très courte** (deux nœuds
presque collés, la flèche « réponse ») choisit la membrane qui la contient, pas la flèche — la
pastille n'est pas une prise. Et deux flèches du bloc « URGENT IMPORTANT » sortent de leur propre
texte par l'intérieur (le contournement ne s'écarte pas du nœud de départ). Les deux sont dans 🔍.

**La pastille, faite le 24/11 au soir** : `Noeuds::pastille` (le noyau la demande, le rendu la mesure
par `arrow_label::pastille_au_monde`, aux mesures du dessin, levée avec le prédicat) ; `arrow::at`
passe par `viser`, à distance nulle dans la pastille. 2 épreuves (`selection/tests.rs`), 4 sabotages
sur 4 (`outils/sabotages/pastille-146.json`). Le doute de la fiche 147 § 3 (« Écrire sur la flèche » avec
plusieurs flèches) était déjà tranché par la barre — le bouton ne vient qu'à une flèche seule ; une
épreuve le garde maintenant (`ancrage/tests.rs`, 5ᵉ sabotage).

## 2. PLEIN-1 — F11 sous Windows

`winit` 0.30.13 (`platform_impl/windows/window.rs`, `set_fullscreen`) garde `WS_MAXIMIZE` et ne
retire que `WS_OVERLAPPEDWINDOW` ; `apply_diff` rappelle même `ShowWindow(SW_MAXIMIZE)` quand
l'agrandie est posée. Notre `WM_NCCALCSIZE` (`plateforme/cadre/cadre_windows.rs`, BARRE-1) cédait
alors en haut la bordure d'une fenêtre agrandie (`SM_CYSIZEFRAME` + `SM_CXPADDEDBORDER`, une douzaine
de pixels) à une fenêtre qui, en plein écran, ne déborde pas : une bande morte, « F11 réduit de 15
pixels ». La décision est pure (`plateforme/cadre.rs`, `retrait_du_haut(agrandie, cadree, bordure)`) :
sans cadre du système (`WS_CAPTION` absent : le plein écran), rien à céder. **Probablement la cause,
à confirmer sous Windows** : aucun essai possible ici, la CI compile la voie Windows.

Sabotages : `outils/sabotages/gestes-146.json`, 3 sur 3.

## 3. La beta.6

Il dirige son Glucose de loin avec la beta.5 et ne peut rien essayer de ce qui est corrigé depuis.
Les notes (`outils/publication/notes.md`) disent ce qui change depuis la beta.5 ; le format des
documents n'a pas changé (aucune différence de version dans `glucose-core` depuis
`v2.0.4-beta.5`), la beta.5 et la beta.6 partagent donc un canevas. La télémétrie n'écrit rien de
neuf : `verifier_serveur.py` passe sans déployer. Vérification GitHub, puis `publier.yml` en
brouillon ; le clic « Publish » est le sien.

## 4. Un document, une salle — le diagnostic des codes en double

Sa note (`text-c3kr-2251`) : *« objectif es de ne JAMAIS avoir de doublon […] 1 pour juste lire,
1 pour lire et écrire […] un lien ne périsse jamais […] je perds la sync, j'ai bien le fichier mais
seul sans mise à jour »*.

**Ce que fait le code** (`app/collaborer.rs`, `partager_ce_canevas` ; `app/collaborer/reprise.rs`) :
chaque « Partager » tire **un code neuf**, donc une salle neuve. Le document retient son code
(`canevas-partages.json`, par fichier, sur cet appareil) seulement tant qu'il est dedans :
« Quitter » l'oublie (`oublier_le_canevas`), et un arrêt net aussi (`reprendre_le_canevas_partage`,
branche `(Some(_), None)`). Après l'un ou l'autre, le prochain « Partager » ouvre une seconde salle :
ses appareils de loin restent dans la première, figée — *« le fichier, mais seul, sans mise à
jour »*. La salle (`outils/porte/src/salle.js`) ne sait ni fermer ni oublier : chaque salle ouverte
vit pour toujours. Deux codes par salle, c'est déjà vrai ; c'est le nombre de salles qui grandit.

**Le plan, dans l'ordre :**
1. **Le code appartient au document** (sur cet appareil), jusqu'à « Changer les codes » : « Quitter »
   ne l'oublie plus, et « Partager » un document qui a sa salle **y revient** au lieu d'en ouvrir une.
2. **Revenir dans sa salle par une fusion à trois.** Le numéro exact n'est connu qu'après une
   fermeture propre ; après un arrêt net ou un « Quitter », le fichier d'ici et la salle ont pu
   changer chacun de leur côté. La salle garde tout depuis la base : on y rebâtit son document au
   dernier numéro retenu (l'ancêtre commun, au moins) et maintenant, puis on fusionne nœud par nœud,
   par identifiant — changé ici seulement : on l'envoie ; là-bas seulement : on le prend ; des deux
   côtés et différemment : la salle gagne, et la version d'ici est gardée à côté, dite. Un numéro
   retenu trop bas ne gêne pas : un même changement des deux côtés se fond sans conflit (c'est la
   fusion à trois de Git). Rien n'efface le travail fait de loin pendant l'absence de ce PC.
3. **« Changer les codes »** : une salle neuve, le document comme base ; l'ancienne **se ferme**
   (un message de l'écrivain, que `salle.js` doit apprendre — le serveur avant le client, donc la
   connexion Cloudflare). Les anciens codes disent alors « ce canevas a changé de code ».
4. La liste des accès (Collaborer en trois portes, sa proposition sur `relais-rep-321`).

Le 1 sans le 2 serait une rustine (revenir en adoptant la salle effacerait son travail local, y
revenir en imposant le sien effacerait celui de loin) : ils partent ensemble.
