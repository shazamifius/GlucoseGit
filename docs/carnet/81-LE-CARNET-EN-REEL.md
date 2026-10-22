# 81 — Le carnet en réel, et la carte d'accueil qui désignait à sa place

> Session du 22/10/2026 (l'après-midi), menée depuis Protéo, suite de la fiche 80. La fiche 80
> avait prouvé le carnet de Protéo par les épreuves ; il restait à le voir en réel, et à regarder
> l'image. Sa parole du jour, rappelée : *« l'interaction avec l'humain dans Glucose doit se passer
> parfaitement »*, et *« Glucose reçoit de Protéo critiques et ajouts »*.

## 1. L'essai réel du carnet

Le vrai Protéo (le binaire optimisé de son commit `98567dd`, vérifié au démarrage sans
modification, celui qui remplace désormais son `proteo.exe`), le vrai `qwen3.5:4b`, une copie de
ses données, et un Glucose sans fenêtre construit depuis `d1ea169` — cargo n'avait rien à
recompiler : ses binaires étaient bien ceux de ce commit. La main d'épreuve joue l'humain.

* « Combien y a-t-il de nombres premiers inférieurs à 100 000 ? » → le calcul voté de Protéo
  décide **9592** (quatre programmes contre un, dont un écrit en réfléchissant ; 20 s), et le
  programme pose la leçon dans l'onglet **« Ce que Protéo a appris »**, bulle « Calculs vérifiés » :
  sa première phrase dit quand s'en servir, puis la preuve, un des programmes, la date, le modèle.
* J'ajoute une note à la leçon **depuis l'autre onglet** (`recrire` avec `tableau`) ; puis « Merci.
  Tu as vu ma note dans ton carnet ? » → la relecture rapporte la leçon récrite en entier, et
  Protéo répond l'avoir vue (6 s).

L'image du carnet est lisible : la bulle, la leçon, le programme en bloc de code, la note.

## 2. Ce que l'image a montré

* **La carte d'accueil naissait choisie.** Sur l'image du tableau, la carte « Bienvenue dans
  Glucose » portait ses poignées, la barre d'action disait « 1 sélectionné » — et la bulle Demande
  de Protéo écrivait *« (Dans Glucose, il désigne : « # Bienvenue dans Glucose ! ».) »*. Personne
  n'avait rien montré : un statut qui ment, chez l'utilisateur comme chez l'agent. La cause : la
  carte naît par la fabrique d'une carte posée d'un clic, et `add_annotation` choisit ce qu'il
  pose ; `document_d_accueil` effaçait déjà le journal (« la carte n'est pas une modification de
  l'utilisateur ») — **ACCUEIL-1** : il efface aussi la sélection, pour la même raison. Glucose
  Tauri n'a pas de carte d'accueil : rien n'y est choisi au lancement. SUITE le notait depuis le
  18/10 (« la carte d'accueil qui naît choisie »).
* **Côté Protéo**, l'en-tête de sa relecture disait « sur le tableau où Protéo pose son
  travail » pour un geste fait dans le carnet : il dit « sur ton travail ».
* **La bulle n'a pas grandi** quand la leçon s'est allongée : le bas du nœud dépasse. C'est le
  mode ordinaire des membranes, comme dans Glucose Tauri, où seule une membrane en mode **étiré**
  grandit avec son contenu (MEMB-4 de Tauri, `membraneStretch.ts`). Les membranes étirées et
  minimisées attendent sa parole (SUITE, la 2.0.5) ; rien n'est changé ici.

## 3. Ce que l'usage de Protéo dit de Glucose, compté

Relevé dans son vrai journal de Protéo (lecture seule) : **444 gestes posés, aucun refusé**, et
quatre fois, le 21/10, le même manque — *« Glucose 2.0.3-beta.5 n'a pas encore la porte des
agents »*. Sur la copie d'essai : 755 gestes, aucun refus. Le seul besoin de Glucose que son usage
réel ait exprimé est **une version publiée qui ait la porte**. Un relevé programmé des besoins
(conception de Protéo, § 4.20, étape 5) n'aurait rien d'autre à dire tant qu'elle n'existe pas :
il vient après elle.

## 4. Prouver

* `test_le_document_d_accueil_ne_choisit_rien` : tombée d'abord sur le code intact
  (`["welcome-card"]`), puis verte ; **un sabotage, une chute** (`outils/sabotages/accueil-1.json`).
* La suite entière : **2 680 épreuves vertes**, 7 ignorées ; `fmt` et `clippy -D warnings` propres.

## 5. Ce qui reste

1. **Une version publiée qui ait la porte** (son geste) : rien des fiches 75 à 81 ne se voit chez
   lui avant. La CI n'a tourné sur aucun commit depuis la beta 5 (`[skip ci]`) : la publication la
   passera, la porte comprise sur les deux Mac.
2. Les commentaires ancrés, la provenance par nœud, les membranes repliables, le statut et la
   preuve écrits par le programme (fiches 79 et 80).
