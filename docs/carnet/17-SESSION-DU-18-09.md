# 17 — Ce que la journée du 18/09 a trouvé, corrigé, et ce qu'elle laisse

> **Rôle de ce document.** La fiche [`15`](15-PLAN-DE-PERFORMANCE.md) disait ce qui empêchait
> la cadence *au 17/09*. Ses vagues A et B sont faites, et le goulot a changé trois fois dans
> la journée. Celui-ci dit **où on en est vraiment**, ce que la mesure a démenti, et dans quel
> ordre prendre la suite.
>
> **Date** : 2026-09-18 · quinze commits, de `d60cfbf` à `321e757`.
> **État vérifié** : `cargo test --workspace` exit 0, **1 162 tests verts**, clippy strict à
> zéro, neuf cliquets mécaniques.

---

## 1. Les chiffres, avant et après

Mesurés sur les sessions réelles de l'utilisateur, pas sur des bancs.

| | au matin | au soir |
|---|---:|---:|
| Images par seconde | 10 à 14 | **56** |
| Médiane au repos | ~40 ms | **~10 ms** |
| Surcouverture (`ecrans`) | 149,3× | ~1,1× |
| Gel à l'arrêt d'un geste | 471 ms | **aucun** |
| Vingt-sept photos empilées, zoom 1 (banc) | ~340 ms | **~3,4 ms** |
| Zoom proche, une photo plein écran (banc) | ~36 ms | ~17,5 ms |
| Justesse du modèle de coût | — | **98 %** |
| Erreurs de validation graphique | à répétition | aucune |

Ce qui **n'a pas** bougé, et qui domine maintenant :

| | |
|---|---:|
| Pic de présentation (`present`) | **471 ms** |
| Pic de téléversement (`blit`) | **488 ms** |
| Part des images redessinées entièrement | **100 %** |
| Part des photos servies par une vignette | 0 à 8 % |

---

## 2. Ce que la mesure a trouvé, dans l'ordre où elle l'a trouvé

### 2.1 L'occlusion calculait tout et n'en gardait rien — REPORT-1

Le test de l'utilisateur — cent images empilées les unes sur les autres — donnait
`ecrans 149.3x` pour soixante-seize photos : **la surcouverture avait augmenté**.

La cause tenait en une ligne. L'occlusion produit, pour chaque photo, la liste des morceaux
qui atteignent l'œil ; le rendu n'en lisait qu'une chose — cette liste est-elle *vide*,
c'est-à-dire la photo est-elle entièrement cachée. Sinon il la peignait **en entier**. Or des
photos empilées avec un léger décalage ne sont jamais entièrement cachées : il suffit d'une
bande de trois pixels qui dépasse.

Ce n'était pas un oubli : peindre une image restreinte à un rectangle demande à `tiny-skia`
un masque plein écran, qui coûte plus qu'il ne rapporte.

**La réponse : notre propre primitive de report**, dans le noyau, sans aucune dépendance. Son
clip n'est pas une option mais **son domaine d'itération** — on parcourt les pixels du
rectangle visible, jamais ceux de l'image. Une photo dont il ne reste qu'une bande de trois
pixels coûte trois colonnes.

Gain mesuré, photos opaques empilées, release : **100× au zoom 1**, 28× au dézoom, 1,6× en
zoom proche.

### 2.2 Le dessin ne coûtait plus rien, et l'image gelait quand même — CASCADE-1

REPORT-1 a mis à nu ce qui se cachait derrière :

```
478.00ms  repos  89 photos  ecrans 1.3x  vign 89
          dont vignettes 471.05ms, report 1.60ms
```

Quatre-vingt-dix-huit pour cent de l'image dans la **préparation**, zéro virgule trois dans le
dessin. Une vignette est indexée par sa forme, **phase sous-pixel comprise** : dès que la vue
bouge d'une fraction de pixel, les quatre-vingt-neuf deviennent caduques ; à la seconde où le
geste s'arrête, elles se reconstruisent toutes dans la même image.

Le cache ne construit donc plus rien de lui-même : il **note**, et l'atelier vient après la
présentation, dans ce que la période laisse. Trois questions ont été tranchées sans constante :

* **quoi en premier** — la surface visible décroissante, qui est le travail épargné, pas une
  préférence ;
* **quelle plus petite tranche** — la **ligne**, parce qu'étaler ne suffit pas si le grain
  reste une vignette entière ; et le découpage ne coûte rien à écrire, une bande n'étant qu'un
  clip pour REPORT-1 ;
* **et quand il ne reste rien** — on accorde alors *ce que l'image vient de coûter*, sans quoi
  la machine reste enfermée dans son régime dégradé.

### 2.3 Le plantage venait de la chaîne d'images

Trois sessions se sont terminées sur un code d'erreur sans message, et la sortie portait :

```
The `SurfaceOutput` returned by `get_current_texture` must be dropped before re-configuring
Queue::present — Surface is not configured for presentation
```

Le code reconfigurait la surface **en tenant encore l'image acquise**. Reconfigurer détruit la
chaîne et la recrée : on arrachait le sol sous les pieds de cette image. La réparation se
diffère désormais au début de la présentation suivante, seul instant où aucune image n'est
détenue. Les erreurs de validation ont disparu.

### 2.4 Le logiciel prévoit ce qu'une image coûtera — COUT-1

Demande de l'utilisateur, et elle fait loi : **cent images par seconde minimum, quoi qu'il se
passe** ; en dessous, on pixelise. Et **pré-optimiser** plutôt que recalculer les fps en
boucle.

Une boucle qui compte et corrige arrive toujours trop tard : elle dégrade après avoir raté,
puis raffine après avoir dégradé — la scène oscille et l'œil voit l'oscillation. Le modèle
sait ce qu'une image coûtera *pendant qu'on décide encore de ce qu'elle contiendra*.

Il n'interroge jamais le matériel : chaque nature de travail — pixel repris, rééchantillonné,
pixelisé — a un coût par unité que la machine vient de **démontrer** en le faisant. Le
bridage thermique s'y lit comme un débit qui baisse, sans qu'on ait demandé sa température.

**Le dernier débit, et pas une moyenne** : le coût par unité varie lentement, donc la dernière
mesure est le meilleur prédicteur de la suivante — et le seul qui ne demande aucune constante
de temps.

Résultat : **le modèle prévoit à 98 % de ce qui arrive.**

### 2.5 La carte graphique écrivait dans la texture qu'elle était en train de lire

Le rendu processeur ne coûtant plus rien, la carte est devenue le goulot :

```
2.5s  501.30ms  repos  1 noeud, 0 photo  dont blit 488.72ms
```

`write_texture` réécrivait **la même** texture à chaque image. Les textures tournent
maintenant sur un anneau dont le nombre se déduit de ce que la chaîne garde en vol, plus celle
qu'on écrit.

**Cela n'a pas suffi** : les pics persistent. Voir § 4.1.

---

## 3. Ce que la journée a appris sur la méthode, et qui a coûté cher

Trois erreurs de **mesure**, pas de code. Elles ont fait raisonner dans le vide pendant des
heures, et chacune a laissé un garde-fou.

### 3.1 Un compteur déclaré et jamais lu vaut zéro — et un zéro se lit comme une mesure

Quatre compteurs ont été ajoutés à la chronique sans que la ligne qui les remplit soit écrite.
Ils affichaient zéro, et tout un raisonnement s'est bâti dessus : « aucune photo ne passe par
une vignette », « les vignettes sont jetées », « il faut débrancher l'atelier ». Rien de cela
ne mesurait autre chose que l'absence de ces lignes.

> **Cliquet 9** — aucun champ de `Instantane` ne peut rester sans être rempli. Le test lit la
> structure, lit le fichier qui la remplit, et échoue si un champ n'y paraît pas.

### 3.2 Les images les plus lentes sont un échantillon biaisé

Conclure du `mip 0` des pires images que la vignette ne sert jamais était faux : une image
dont les photos ont leur vignette devient rapide, donc elle **quitte** cette liste. On n'y
regardait que les cas où le mécanisme avait échoué.

> Toute part se lit désormais sur **toutes** les images, dans le résumé, jamais sur les pires.

### 3.3 Comparer une prévision à autre chose que ce qu'elle prévoit

La justesse annonçait 218 %, puis 545 %. Deux fautes superposées : la prévision porte sur le
*report*, elle se comparait à la durée entière de l'image ; et elle porte sur le rendu *fin*,
elle se comparait à des images pixelisées — où l'on a exécuté autre chose.

> Elle ne se lit que sur les images rendues au plus fin, et contre le poste `report`.

### 3.4 Un remplacement de texte qui échoue en silence

Deux fois, une correction annoncée n'était pas branchée : l'atelier n'a jamais tourné de la
journée dans l'application, alors que le banc l'exécutait. Les éditions se font désormais avec
une assertion préalable, et se vérifient après coup.

---

## 4. Ce qui reste, par ordre de ce que la mesure désigne

### 4.1 La présentation — le goulot actuel

```
48.8s  481.61ms  zoomer  45 photos  dont present 471.23ms, report 3.00ms
```

**Hypothèse mesurée mais non confirmée** : on téléverse l'image **entière** à chaque frame.
En 4K, trente et un mébioctets ; à cinquante images par seconde, plus d'un gigaoctet par
seconde vers une carte qui partage sa mémoire avec le processeur. La colonne `envoi` de la
chronique le dira à la prochaine session.

Si c'est confirmé, la correction rejoint le § 4.2 : **ne téléverser que la région qui a
changé**.

### 4.2 Une scène immobile se repeint entièrement

Toutes les images portent `redessine 100%`. Le mécanisme de salissure (A.1) est écrit, testé,
et ne coupe **jamais rien** en pratique. Il faut savoir qui salit tout à chaque image — le
rendu par région existe mais n'est jamais emprunté.

C'est le même chantier que le § 4.1 : ce qui n'a pas été redessiné n'a pas besoin d'être
téléversé.

### 4.3 Le zoom proche — LE chantier que la charte nomme

En zoom proche une photo mesure plusieurs écrans. Elle n'a alors droit à **aucune vignette**
(`0/0` au banc, zoom 4), et tout passe par le rééchantillonnage à ~7,6 ns le pixel.

Deux directions, à peser par la mesure :

* une vignette **de la partie visible** seulement — elle tiendrait alors toujours dans la
  fenêtre. Pose deux questions : la mémoire (il faudrait une borne qui suive la machine, comme
  ADAPT-1 pour le cache d'images) et l'invalidation (ce qu'on voit change dès que la vue bouge
  d'un pixel) ;
* l'accélération du rééchantillonnage lui-même — réutiliser les lignes source entre lignes de
  destination quand on agrandit, ce qui ramènerait trois mélanges par pixel à un.

### 4.4 La pixelisation est-elle trop fréquente ?

**73 à 86 % des images.** Le modèle étant juste à 98 %, ce n'est plus une surestimation : la
scène coûte réellement plus de dix millisecondes au rendu fin. La question n'est donc plus
« le modèle se trompe-t-il » mais « est-ce que ça se voit » — et elle n'a pas de réponse
mesurable. **Elle appartient à l'œil de l'utilisateur.**

Si la réponse est oui, le § 4.3 devient prioritaire : c'est en rendant le rendu fin moins cher
qu'on pixelisera moins.

### 4.5 Le modèle ne peut pas apprendre ce qu'il évite de faire

Quand il pixelise, il n'observe plus le coût du rendu fin, qui reste donc à sa dernière
valeur. Une rétroaction, pas un défaut de calcul.

La sortie est mathématique : **le rapport entre les deux filtres est une propriété de
l'algorithme** — quatre lectures et trois mélanges contre une lecture et aucun — alors que le
débit absolu est une propriété de la machine. Séparer les deux permettrait de déduire l'un de
l'autre sans avoir à exécuter les deux.

### 4.6 Ce qui reste du logiciel lui-même

Rien de ce qui précède ne touche la fiche [`14`](14-ETAT-ET-RESTE.md), qui reste vraie :
**≈ 34 % des gestes, ≈ 25 % du logiciel**. Neuf domaines à zéro — rideaux, temporalité,
miroirs, export, storyboard, presets, collaboration, plugins, divers — soit ≈ 8 900 lignes de
TypeScript à recréer.

Et son § 5 reste entier : aucun document réel de Glucose Tauri n'a jamais été ouvert dans la
version Rust, et le canva Wikipédia — le but du projet — n'a toujours pas de premier pas
planifié.

---

## 5. Deux défauts vus et non corrigés

* **`card::tests::test_mode_1_editing_shows_the_markdown_signs_on_screen` est instable.** Il
  compare le dessin d'une carte *en édition*, dont le curseur clignote sur l'horloge. Il
  passait déjà par intermittence avant cette session.
* **`coller_image` écrit un PNG dans `%TEMP%` et ne le nettoie jamais** (fiche 15 § 8). Un
  projet rouvert après un nettoyage de Windows aura perdu ses images collées.
