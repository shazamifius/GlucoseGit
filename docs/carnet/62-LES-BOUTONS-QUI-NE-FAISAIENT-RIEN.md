# 62 — Les boutons qui ne faisaient rien

> Session du 12/10/2026. Il a dit : *« il y a encore des dizaines d'options et de boutons qui ne
> fonctionnent pas sur Glucose »*, et *« 100 % des boutons existants, ainsi que toutes les
> fonctionnalités à venir, sont quasi obligatoires — je n'en délaisserai pas un seul »*. Cette
> fiche fait les premiers : les tris par la couleur, l'export en image, les Presets et leurs
> zones, « Par slot preset », les champs de nombre du dock. **§ 8 : ce qui reste en façade.
> § 9 : ce qui attend sa parole.**

## 1. Ses réponses, mot pour mot ou presque

* **L'envoi** : « envoyer tout sur GitHub, mais pour l'instant pas avec la CI » — fait le 12/10
  (`[skip ci]`), **vérifié : aucune exécution**.
* **Sa clé de signature** : « ma clé est sauvegardée ».
* **`%TEMP%\glucose_pasted`** : « tu peux effacer les 2,9 Go » — le dossier était déjà vide.
* **GitHub** : « merci énormément de les avoir vus ; vérifie avant » — les quatre brouillons de
  release de juillet et la branche `feat/mcp-glucose-bridge` (entièrement contenue dans `main`)
  sont retirés ; relu le 12/10 au soir : il ne reste que `main` et `tauri-v1.0.1`.
* **Les boutons** : tous obligatoires — **ne jamais proposer d'en retirer un**.
* **Collaborer** : « absolument OBLIGATOIRE, Glucose et la collaboration vont de pair » — pas
  tout de suite. **Trans-domaines** : pas pour maintenant.
* **Le téléphone** : une des dernières étapes, mais **toute l'UX à revoir** — ultra rapide,
  fluide, pratique au niveau des boutons. Les dossiers s'y suppriment parfaitement.
* **Les images vers Discord** : marchent avec `cargo run --release` (pas avec la version en
  ligne, qui ne porte pas COPIER-1 : attendu).
* Il a fait **supprimer la mémoire « interface minimale »** : « trompeuse, pas vraie du tout ».

## 2. ORDONNER-2 et MOYENNE-1 — les trois tris par la couleur (`64b5922`)

« Couleur », « Sombre → Clair », « Clair → Sombre » étaient grisés depuis toujours. Ils rangent par
la **couleur moyenne** de chaque image, lue dans **Oklab** (`glucose_core::couleur`) :

* par teinte, le cercle se coupe **au plus grand vide** entre deux couleurs — aucun angle de départ
  choisi —, puis les gris, du sombre au clair ; une image est grise quand sa couleur ne se
  distingue pas d'un gris (la différence à peine visible de CSS Color 4, 0,02) ;
* par clarté, la clarté **perçue** d'Oklab — la luminosité HSL de Tauri disait égaux un jaune et
  un bleu ;
* une image illisible va à la fin, et le message le dit (Tauri lui donnait 0,5 en silence).

La moyenne est **le sommet de la pyramide** de l'image ; la réduction par deux tronquait (−0,4 par
étage) et donnait au bord d'un côté impair le poids d'un bloc entier. Elle pondère désormais par la
surface native et arrondit au plus proche — et va plus vite (49 ms contre 70 pour 12 Mpx). Un
rangement par la couleur **attend** les couleurs en chemin sans figer l'écran (`Attentes`). Mesuré
sur une copie de son document : `examples/essai_ordonner`.

## 3. EXPORT-PNG-1 — l'export en image (`9fca9ca`, `decisions/12`)

Son idée du 09/10 : les images d'une sélection composées en une, dans la disposition de l'auteur.
La sélection, sinon tout le tableau ; **à 100 %** (une unité du canevas, un pixel), sans plafond ;
la marge est ce que le contenu peint hors de ses boîtes (rien pour des photos, la portée de la
lueur dès qu'il y a des annotations) ; ni grille ni cadre ; par **bandes** de la surface d'un
écran, écrites au fil de l'eau par `png` sur un fil à soi ; le fichier ne prend sa place
qu'entier. Son tableau principal (copie, 35 photos) : 4 528 × 2 944, 6,4 Mo, 0,57 s.

**Question ouverte** : 100 % (mon choix) ou la pleine résolution des originaux (§ 9).

## 4. PRESET-1 — les gabarits et leurs zones (`c442ea2` à `367fa73`)

1. **Les six gabarits de Tauri**, mot pour mot et couleur pour couleur (`glucose_core::gabarits`) ;
   une zone porte `gabarit/catégorie`, unique, sans changer le format.
2. **Les zones dessinées sur les deux voies**, sous les membranes comme dans Tauri — l'épreuve des
   deux voies a vu la carte graphique les poser au-dessus (123 niveaux d'écart) : une passe à
   elles, `present::zones_gpu`, juste après le fond.
3. **Le panneau** pose un gabarit **là où on le voit** : armé, un fantôme suit le curseur, un clic
   le pose, Échap renonce ; le gabarit suivi montre une pastille par catégorie, **pleine dès
   qu'une image est dans sa zone** — celle de Tauri lisait un `slotId` jamais écrit.
4. **Une zone se prend par son bandeau** — choisir et glisser d'un geste ; ses coins la
   redimensionnent (100 × 80 au moins), sa croix et Suppr la retirent ; **son intérieur reste le
   canevas**. Tauri prenait tout clic sur son vide (plus de rectangle de sélection sous un
   gabarit) ; c'est l'une des demandes les plus votées du forum de Figma pour ses sections.
5. **« Par slot preset »** (Ordonner) : chaque zone range **ses** images (celles dont le centre y
   est, la plus haute zone l'emporte) sous son bandeau, en **rangées justifiées** qui remplissent
   exactement sa largeur ; le tri choisi vaut dans chaque zone ; hors zone, rien ne bouge ; sans
   zone, rien ne bouge et le message le dit. **Tout tient dans la zone** : sinon les images
   débordaient sur la zone d'en dessous, et un second rangement les aurait fait changer de zone.
   La hauteur visée descend alors, images et écarts d'un même facteur, par dichotomie sur la
   vraie mise en page, depuis une hauteur qui tient — jamais nulle, même pour 20 000 images dans
   une zone minuscule. Ranger deux fois donne le même rangement.

« Créer un preset custom » reste grisé — Tauri n'en faisait rien non plus (§ 8).

## 5. CHAMP-1 — les champs de nombre du dock (`82911ea`)

Les champs d'Ordonner (taille, espacement) et du Storyboard (largeur, colonnes, espacement) étaient
dessinés et **ne répondaient à rien** : Ordonner rangeait toujours à 280 et 16.

* **Recherche** : Photoshop, Figma, Blender prêtent deux gestes au même champ — glisser
  horizontalement fait défiler la valeur, cliquer ouvre la frappe ; les critiques du glisser
  seul (ni découvrable ni accessible à tous) disent de garder toujours la frappe
  ([css-tricks](https://css-tricks.com/?p=319804), [VectorStyler](https://www.vectorstyler.com/forum/post/1607),
  [forum de Figma](https://forum.figma.com/report-a-problem-6/why-can-t-i-drag-on-the-width-indicator-label-to-change-the-width-of-an-element-41651)).
* **Le panneau entier est la règle** : glisser sur toute sa largeur parcourt toutes les valeurs —
  aucune sensibilité à choisir. Le jeu sous lequel un appui reste un clic est **le tremblement
  de la main** déjà défini (`tremblement()`), et le glisser se compte depuis le bord du jeu.
* Cliquer : la valeur s'ouvre **choisie entière** ; Entrée, Tab (champ suivant) ou un clic
  ailleurs valident ; Échap renonce ; ↑ ↓ d'un pas. Le curseur annonce ↔. Bornes et pas de Tauri.
* **Ordonner retrouve « Colonnes (0 = auto) »**, perdu au portage — pour la grille et la mosaïque
  seulement (ailleurs il ne fait rien : il n'existe pas). **La taille dit ce qu'elle mesure** :
  HAUTEUR d'une rangée (compact, même hauteur, par zone) ou LARGEUR d'une colonne ; le panneau
  écrivait « LARGEUR CIBLE » partout, y compris là où c'est une hauteur.
* Le dock perd l'aiguillage du clic (`dock/clic.rs`) et gagne un seul `cadre_du_panneau` (510
  lignes au lieu de 598).
* **Au téléphone** : glisser marche ; **taper ne marche pas encore** — le miroir du clavier
  d'Android (CLAVIER-1) ne sert que les questions et les nœuds, aucun champ du dock (ni le nom
  d'un jalon, ni celui d'un domaine). À faire avec l'UX du téléphone.

## 6. Un dossier d'épreuve qui ne reste jamais derrière (`e880931`)

1 837 dossiers d'épreuves traînaient dans le dossier temporaire : `crate::epreuve::Dossier`, retiré
quand il sort de portée, qu'elle réussisse ou tombe.

## 7. Les erreurs de la session — ce que j'ai cru et que la mesure a démenti

* **Trois épreuves aveugles** trouvées en sabotant (le cercle des teintes, le magasin qui ne
  demande pas, des rangées toutes pareilles) : chacune passait avec sa garde cassée.
* **Les zones sur la carte graphique** passaient au-dessus des membranes : je croyais la couche
  du dessous assez basse ; l'épreuve des deux voies a dit le contraire.
* **« Par slot preset » sans borne basse** : écrit, éprouvé, saboté — et pourtant faux : une zone
  pleine débordait. Trouvé en relisant, pas par une épreuve.
* **Le libellé « LARGEUR CIBLE »** et le champ Colonnes perdu : présents depuis le portage,
  jamais vus, parce que personne n'avait comparé le panneau à celui de Tauri champ par champ.
* **Git Bash mange les barres obliques inverses** dans un script (deux fois) : écrire par
  l'outil d'édition, ou un fichier.

## 8. Ce qui reste en façade — l'inventaire

| où | quoi | ce qu'il faut |
|---|---|---|
| **Storyboard** | tout : « Activer » refuse, rien ne se dessine | le chantier suivant (§ 10) |
| Presets | « Créer un preset custom », grisé | à concevoir : enregistrer les zones du tableau comme gabarit du document (`project.presets` existe et se lit déjà) |
| Plugins | « Télécharger » le modèle d'IA | l'IA locale : un chantier à part |
| Exporter | le SVG ne porte ni les images ni les membranes | à finir |
| la bande | Collaborer, Trans-domaines | **plus tard, à sa parole** (Collaborer est obligatoire) |
| le téléphone | les champs du dock ne se tapent pas | avec l'UX du téléphone |

Le panneau Ordonner de Tauri a été relu champ par champ (§ 5) ; **les autres panneaux et les menus
ne l'ont pas encore été** : c'est la méthode qui a trouvé les deux défauts d'Ordonner, à refaire
pour chacun (Pomodoro, Domaines, Time Machine, les menus du clic droit, la bande).

## 9. Ce qui attend sa parole

1. **La taille de l'export PNG** : à 100 % (fait), ou à la pleine résolution des originaux ?
2. **L'envoi** des neuf commits de cette fiche (`64b5922` à `82911ea`, plus la documentation) :
   avec la CI, ou sans (`[skip ci]`) ?
3. **Ses essais à l'écran** (ci-dessous).

**À essayer à l'écran** (`cargo run --release`) :

* Ordonner : les trois tris par la couleur ; les champs (glisser, cliquer et taper, Tab, Échap) ;
  « Colonnes » en grille ; « Même hauteur » à 100 ;
* Exporter en PNG : une sélection, puis tout le tableau ;
* Presets : poser un gabarit, glisser une zone par son bandeau, ses coins, sa croix, Suppr, Échap
  pendant que le fantôme suit ;
* « Par slot preset » : des images dans deux zones, puis trop d'images dans une petite.

## 10. La suite : le Storyboard

Lu dans Tauri (`StoryboardControls.tsx`, `StoryboardLayer.ts`, `GlucoseCanvas.tsx` l. 1418-1530) :
format, largeur, colonnes, espacement ; « Activer » **arme un fantôme** de grille, un clic pose
`colonnes × 2` cases (ou replace celles qui existent : « Mettre à jour ») ; chaque case est un
cadre sombre numéroté, avec sa description dessous (40 px) ; la liste du panneau édite les
descriptions, ajoute, retire, renumérote ; « ✕ » efface tout. Les cases n'étaient **ni
sélectionnables ni déplaçables** sur le canevas, et leur image (`imageId`) **jamais assignable** :
une case ne pouvait rien contenir. Le modèle (`StoryboardPanel`), le journal et le format les
portent déjà ici ; le panneau a désormais ses champs (§ 5).

À concevoir après une recherche (Milanote, Boords, Storyboarder de Wonder Unit, les sections de
Figma) : poser la grille par le fantôme comme les gabarits ; dessiner les cases sur les deux voies
(les rectangles des zones peuvent les porter) ; **une image posée dans une case s'y ajuste** —
l'appartenance au centre, comme les zones ; la description qui se tape ; les cases qui se prennent
par leur bandeau, comme les zones.
