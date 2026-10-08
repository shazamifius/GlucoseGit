# 57 — Le clavier du téléphone, l'appui long, les documents qu'on renomme, et le bord à bord

> Session du 08/10/2026, au soir. Dans l'ordre de la fiche 56 § 7 : **le clavier** d'abord —
> sans lui, rien ne s'écrit au téléphone —, puis **l'appui long** (le clic droit du doigt), puis
> **renommer, dupliquer, supprimer** un document. Rien de ce qui suit n'a encore tourné sur un
> téléphone : le § 7 dit quoi essayer.

---

## 1. Ce que la lecture a trouvé avant d'écrire

* **Monter de version ne donne pas le clavier.** `winit` 0.31.0-beta.3 (04/09/2026), lue dans sa
  source (`winit-android`), jette encore l'évènement de texte de `GameActivity` (« Unknown
  android_activity input event ») : elle sait demander le clavier, pas lire ce qu'il écrit.
  `android-activity` 0.6.1 est la dernière (24/03/2026). Une demande d'ajout existe chez eux
  (PR 214, janvier 2026, les « editor actions ») ; rien n'est publié.
* **Le clavier tient le texte, pas Glucose.** `GameTextInput` garde un `Editable` Java — le
  texte entier, la sélection, le mot en composition — que Gboard réécrit : une lettre, une
  suggestion qui remplace un mot, une correction trois mots plus tôt, la dictée. Ce qu'on peut
  lire, ce sont des **états**, pas des frappes (`AndroidApp::text_input_state`).
* **Les positions sont en UTF-16.** Lu dans l'AAR livré (`games-activity` 4.4.0, désassemblé :
  `InputConnection.getSelection` appelle `Selection.getSelectionStart` d'un `Editable`) et dans
  `gametextinput.cpp`, qui les recopie telles quelles. Glucose compte en octets UTF-8 : sans
  conversion, un « é » décale le curseur d'un cran par accent. `android-activity` les borne à la
  longueur en octets — sans dommage, mais sans conversion non plus.
* **Écrire au clavier n'est pas fait quand l'appel rend la main.** `GameActivity_setTextInputState`
  pose l'état et une commande dans la file du fil de l'interface (`write_work`, lu dans
  `GameActivity.cpp`) ; relire juste après rend l'ancien.
* **Sans le drapeau multiligne, Entrée n'écrit rien** : `InputConnection.processKeyEvent` change
  Entrée en « action » et un filtre retire les sauts de ligne (lu dans le même AAR).
* **`adjustResize`** (le manifeste de Glucose) rétrécit la fenêtre quand le clavier sort — sur
  Android 10, son Redmi 9. Sur Android 16 avec la cible API 36, le bord à bord ne se refuse plus,
  et la fenêtre ne rétrécit plus : il faudra lire la hauteur du clavier (*insets*) — le chantier
  des barres du système ([Android 16, changements de comportement](https://developer.android.com/about/versions/16/behavior-changes-16)).
* **Le retour d'Android** arrive à Glucose (`winit` le nomme `BrowserBack` et le dit traité) :
  il ne faisait rien.
* **Le délai de l'appui long appartient à l'utilisateur** : `ViewConfiguration.getLongPressTimeout`
  lit le réglage d'accessibilité « Délai de pression prolongée », 400 ms à défaut (AOSP,
  [`ViewConfiguration.java`](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/core/java/android/view/ViewConfiguration.java)).
* **Une question dessinée répondait à l'appui.** Un appui long sur un document de la liste
  l'aurait ouvert avant de pouvoir prendre ; et le doigt qui ouvre la liste depuis le menu (le
  menu agit à l'appui) aurait répondu en se levant.

## 2. CLAVIER-1 à 3 — le clavier, miroir de la saisie

* **La saisie et le clavier tiennent le même état** (`interactions/text_edit/miroir.rs`). À chaque
  tour de boucle, le miroir compare : ce que le clavier a réécrit devient **la plus petite
  réécriture** d'un texte à l'autre (`epissure` : le début et la fin communs restent, sur des
  caractères entiers), passée par `apply_text_command` — elle s'annule mot par mot et
  s'enregistre comme une frappe ; ce que Glucose a changé lui-même (un toucher, `Ctrl+Z`) repart
  au clavier. Comparer des états plutôt qu'écouter des moments : aucun des chemins qui ouvrent ou
  ferment une saisie (clic, outil Texte, reprise après plantage, document changé) ne peut
  l'oublier.
* **CLAVIER-2** : l'état d'avant un envoi peut encore revenir ; le miroir le retient et l'ignore
  tant qu'il revient. Sans cela, il défaisait ce que Glucose venait d'écrire — l'épreuve le
  montre en servant la file du faux clavier à la main.
* **CLAVIER-3** : la ligne qu'on écrit reste en vue quand la fenêtre rétrécit sous le clavier —
  le plus petit déplacement, zéro si elle se voit, le haut d'abord si elle est plus haute que la
  place.
* Multiligne, majuscule en début de phrase, correction, **jamais le clavier plein écran** du
  paysage (il cacherait le nœud). Toucher le texte ressort le clavier que le geste retour a
  rentré ; la touche retour valide la saisie, comme Échap.
* **Le clavier vit dans `Lancement`** : seul le vrai lancement le donne, et le cliquet des 80
  lignes refusait un champ de plus dans `GlucoseApp::new` — le ranger là valait mieux qu'extraire.
* Treize sabotages, treize chutes. FONT-1 a vu l'emoji d'une épreuve : la police n'en a pas le
  dessin — **un emoji tapé au téléphone s'affichera comme une case vide** (§ 6).

## 3. APPUI-1 — l'appui long, le clic droit du doigt

* Un doigt posé et immobile — à une main qui tremble près, 8 points — qui tient le délai du
  système devient un appui long (`interactions/toucher.rs`). La boucle se réveille à l'échéance
  (une raison de réveil de plus, « un appui long se decide ») ; rien ne tourne entre-temps.
* **Ce qu'il fait** : le geste du doigt se termine sur place (un toucher : le nœud est choisi),
  puis le menu du clic droit s'ouvre, à la taille du doigt — celui du nœud, ou celui du vide.
  Dans le texte qu'on écrit, le mot sous le doigt se sélectionne, comme partout sous Android.
  Le téléphone **vibre** (`performHapticFeedback(LONG_PRESS)`, selon le réglage de chacun).
* **Le doigt qui se lève ensuite ne relâche rien une seconde fois** : un second relâchement fait
  descendre le choix à l'image du dessous, et le menu aurait agi sur une autre image — l'épreuve
  des images superposées le garde.
* **Une question répond au relâchement, sur la réponse pressée**, comme un bouton d'Android ;
  un doigt qui glisse ailleurs ou qui tient jusqu'à l'appui long n'a rien répondu. Et un
  relâchement que la question n'a pas reçu termine le geste qu'il terminait (un canevas qui
  glissait sous le doigt quand elle a paru) — vu en écrivant l'épreuve, avant tout essai.
* Côté Java, un seul ajout (`sentirLAppui`) ; l'appel des méthodes de l'activité est désormais
  écrit une fois (`glucose-android`, `java::Activite`), et toute exception Java s'efface au lieu
  de rester en suspens.

## 4. DOCUMENTS-2 — renommer, dupliquer, supprimer

* **Un appui long sur un document de la liste** demande ce qu'on en fait : « Renommer… »,
  « Dupliquer », « Supprimer… », « Annuler » — comme Infinite Painter et Concepts ; la liste le
  dit en tête, parce qu'un appui long ne se devine pas. Après chaque geste, **la liste revient**, et
  dit en tête ce qui vient d'arriver (« Renommé « Mary ». ») ; un nom refusé se redemande en
  disant pourquoi au-dessus du champ. Aucun message posé par-dessus : la première écriture en
  avait cinq, et le cliquet des toasts l'a refusée — il avait raison
  (`persist/documents/gestion.rs`).
* **Renommer** ouvre une question avec **un champ** (`ui::question::Champ`), le nom tout
  sélectionné — la première lettre tapée le remplace. Le clavier y écrit par le même miroir que
  dans un nœud : le champ d'une question passe devant, puisqu'elle prend tout tant qu'elle est
  posée. La question se centre dans la fenêtre : quand le clavier la rétrécit, elle remonte
  au-dessus de lui sans rien de plus.
  * **Le nom** : ni `/` (aucun système), ni `\ : * ? " < > |` (Windows — un document part un jour
    vers un autre appareil), ni caractère invisible, ni point au bord, et au plus
    `(255 − 1 − 7) / 4 = 61` caractères : un nom de fichier tient en 255 octets, extension
    comprise, un caractère en prend jusqu'à quatre. La constante n'est pas choisie, elle se
    déduit. Une extension tapée ne se double pas.
  * Un nom refusé ou déjà pris **se redemande avec ce qu'on avait tapé** ; rien ne s'écrase
    jamais — `persist::atomic::renommer_sans_ecraser`, puisque sous Windows `rename` remplace
    en silence (le cliquet 11 veut tout renommage dans `atomic`).
  * **Le document ouvert** : son scribe y écrit. Il passe par « Enregistrer sous » — copier le
    fichier, histoire comprise, et continuer dans la copie —, puis l'ancien s'efface.
* **Dupliquer** range « X copie » (puis « X copie 2 »…) à côté ; le document ouvert attend
  d'abord que son scribe ait tout posé sur le disque (`synchroniser`).
* **Supprimer** demande une fois, et dit que c'est définitif, comme Procreate : « le document,
  son histoire et ses images disparaissent de ce téléphone ». Seul « Supprimer » efface —
  l'épreuve répond « Annuler » par les vrais boutons et vérifie que le fichier reste. Le
  document qu'on regarde ne s'efface pas sous les yeux : un document vierge le remplace d'abord.
* **La course contre le scribe** : dupliquer le document ouvert attend qu'il ait tout posé
  (`synchroniser`) ; le sabotage de cette attente passait, le scribe gagnant toujours la course.
  Les épreuves peuvent désormais le **retenir** (`Ordre::Retenir`, sous `cfg(test)`) : la
  course se perd à coup sûr, et l'attente se prouve. Vingt et un sabotages, vingt et une chutes.

## 4 bis. BORD-1 — le bord à bord, et le clavier qui recouvre

**Avancé devant la mise à jour automatique**, contre l'ordre de la suite : sur Android 15 et 16,
avec la cible 36, Glucose est dessiné jusqu'au bord — sa barre d'onglets passait sous la barre
d'état et l'encoche, où elle ne se touche plus, et le clavier recouvrait le texte au lieu de
rétrécir la fenêtre. La mise à jour, elle, ne sert qu'une fois une version publiée.

* **Le bord à bord partout** (`EdgeToEdge.enable`, icônes claires sur la feuille sombre) : un
  seul chemin, que son Redmi 9 (Android 10) essaie comme les téléphones récents de ses testeurs.
* **Les marges viennent de Java** (`MainActivity.onApplyWindowInsets`, qui surcharge l'écouteur
  de `GameActivity` et l'appelle d'abord) : la barre d'état, l'encoche, la navigation, et le
  clavier à part (`plateforme::marges`, une boîte aux lettres qui réveille la boucle). Une marge
  dit ce qui **recouvre** la fenêtre : là où elle rétrécit encore sous le clavier, la marge du
  clavier vaut zéro — rien ne se compte deux fois.
* **En haut, une seule mesure change** : la barre du haut englobe la barre d'état — son fond la
  couvre, ses boutons et son titre se posent dessous. Tout ce qui se mesurait sur elle (les
  onglets, le canevas, le rail) suit sans rien savoir des marges. Quand le rail remplace la
  barre, le canevas se voit sous la barre d'état.
* **En bas, une seule notion** : *l'écran visible* (`UiState::ecran_visible`), l'écran sans la
  navigation, ni le clavier quand il est sorti. Le message, la question, la barre d'action, les
  options de flèche et la minimap s'y posent ; la ligne qu'on écrit y reste (CLAVIER-3). Le
  cadre de la caméra dans la minimap se mesure toujours sur l'écran entier : la minimap reçoit
  un *retrait*, pas un écran plus petit.
* À gauche, en paysage, le rail s'écarte de l'encoche.
* Une épreuve rend l'écran entier hors écran avec 300 pixels de navigation : les rangées du bas
  sont les mêmes avec et sans le message et la barre d'action.

## 5. Ce que son téléphone a déjà dit

La base de la télémétrie n'est plus vide : huit sessions, dont **quatre d'Android en 2.0.2-dev**
— une seule installation, la sienne (l'APK de son Bureau est le seul à porter cette version).
Elles datent du 07/10 au soir à l'horloge du téléphone, enregistrées par les APK d'avant et
parties au lancement suivant, après son « oui » à la question dessinée — **QUESTION-1 et VUE-1
ont donc marché chez lui**. Ce qu'elles disent du Redmi 9 :

* une image coûte **10 à 12 ms** en médiane ; en déplaçant la vue, le pire monte à **35-44 ms**
  — 25 à 30 images par seconde par moments, sous le plancher (charte : 60 sur un téléphone
  courant, 45 sur un très vieux) ;
* chaque session finit « interrompue » : Android tue Glucose sans prévenir, son régime normal ;
* **la batterie n'est pas lue sous Android** (`batterie_pct` nul) : à écrire.

## 6. Ce qui reste, et ce qui n'est pas prouvé

* **Un emoji tapé s'affiche comme une case vide** : la police (Inter) n'a pas son dessin, et
  Glucose n'a pas de police de repli. Gboard propose des emojis à chaque phrase ; c'est à
  traiter avant que ses testeurs écrivent.
* Le bord à bord **n'a pas tourné sur un téléphone** : la valeur réelle des marges sur son
  Redmi 9 (et sous Android 10, avec `adjustResize`) se lira dans son journal. Les panneaux
  (Time Machine, Ordonner…) ne s'écartent pas encore des bords.
* Le mot en composition (souligné par Gboard) n'est pas souligné dans le nœud ; la sélection
  du texte au doigt n'a ni poignées ni loupe.
* Deux gardes de l'appui long ne se voyaient pas (§ 3, les sabotages) : une ligne inutile
  retirée ; le contrat « le doigt ne mène plus de geste » affirmé par l'épreuve.

## 7. À essayer sur son téléphone

1. Toucher deux fois un texte (ou en créer un avec l'outil Texte) : le clavier sort ; écrire,
   accepter une suggestion, effacer, aller à la ligne ; `Ctrl+Z` n'existe pas au doigt — le
   dire s'il manque. Le texte reste au-dessus du clavier.
2. Le geste retour rentre le clavier ; toucher le texte le ressort ; la touche retour valide.
3. Un appui long sur une image, sur le vide, sur un mot du texte qu'on écrit.
4. « Ouvrir un document… », puis un appui long sur un document : renommer, dupliquer, supprimer.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
