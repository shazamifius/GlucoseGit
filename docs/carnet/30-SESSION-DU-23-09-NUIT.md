# 30 — De près, au repos, et en vagues

> **Rôle de ce document.** La session qui suit la fiche 29, et qui en a pris trois pistes : le
> texte qui disparaît de près (§ 4.2), l'application qui ne dort pas (§ 4.4), les images qui
> arrivent en vagues (§ 4.3). Et le retrait complet de Trans-domaines, demandé par
> l'utilisateur. Cette fiche dit ce qui a été établi, ce qui a été corrigé, ce que la mesure a
> démenti — et ce qui reste une hypothèse tant que l'utilisateur ne l'a pas vu à l'écran.
>
> **Date** : 2026-09-23, nuit · commits `f26ef5e` à `0c1a0bb`, et celui de cette fiche.
> **État vérifié** : `cargo test --workspace` exit 0, **1 519 tests verts** (1 509 au départ,
> 5 retirés avec Trans-domaines, 15 ajoutés), clippy strict à zéro, aucun plafond relevé —
> trois extractions à la place.
>
> **Rien de ceci n'a encore été vu à l'écran par l'utilisateur.** Tout est prouvé par des tests
> déterministes, chacun vérifié à l'envers ; rien n'est « réglé » au sens de la charte tant
> qu'il n'a pas joué une session (§ 9).

---

## 1. Ce qui a été fait, en une table

| Commit | Ce que c'est | Preuve |
|---|---|---|
| `f26ef5e` | **Trans-domaines retiré de bout en bout** — l'ajout de `bbc496b` et le faux état d'avant. Le bouton reste, ne s'allume jamais, et son clic ne fait rien | un test par la souris, sur le vrai `GlucoseApp` |
| `e1fcccb` | **DE-PRES-1** — une carte plus grande que l'écran se découpe en tuiles au lieu de disparaître | l'épreuve des deux voies à l'échelle 6 et 14, et cinq tests ; sept sabotages, tous attrapés |
| `cbec14b` | **L'instrument du repos était aveugle** — chaque image dit maintenant d'où elle vient ; la barre ne se redessine plus à chaque mouvement du pointeur | quatre tests ; quatre sabotages, tous attrapés |
| `0c1a0bb` | **NIVEAU-GPU-1** — la carte reçoit le niveau de la pyramide qui couvre la photo, prêté sans copie | `bench_televersement`, trois tests ; deux sabotages, tous attrapés |

---

## 2. Trans-domaines

L'utilisateur, en début de session : *« ne touche plus à Trans-domaines, laisse le bouton ne
rien faire »*, puis, quand je lui ai montré que « ne rien toucher » laissait des pointillés à
l'écran : *« il faut supprimer complet complet les systèmes »*.

Deux couches ont été retirées, et la seconde était plus ancienne que la session d'avant :

* **l'ajout de `bbc496b`**, annulé tel quel : `arrow::est_trans_domaine`, les pointillés 6-4,
  le masquage au dessin et à l'arbitre de clic, la structure `Affichage` et leurs cinq tests ;
* **le faux état d'origine** : `UiState::trans_domain` valait `true` et rien ne l'écrivait, le
  bouton s'affichait donc **allumé** en permanence, et chaque clic disait « Trans-domaines
  activé ». Retiré, avec sa place dans la clé de la bande.

Le bouton reste posé, au repos comme ses voisins (comparé au pixel sur les captures témoins :
seuls les 132 × 28 pixels du bouton changent), et son action s'appelle `TransDomain` — elle ne
bascule rien. Les fiches 03 et 06 ne décrivent plus les pointillés comme une fonction à
construire, pour qu'aucune session ne les refasse.

**Ce que Trans-domaines doit être reste la décision de l'utilisateur.** Et une remarque pour la
V1 publique, qu'il a entendue : la feuille de route (fiche 04, point 0.1) veut qu'un bouton sans
effet soit grisé, avec une infobulle « bientôt » ; celui-ci ne l'est pas, parce qu'il a demandé
qu'on n'y touche pas.

---

## 3. DE-PRES-1 — le texte qui disparaissait de près

### 3.1 La cause, établie

`Regime::composer` refusait une texture plus grande que l'écran, et `composants_de_texte`
passait à la carte suivante sans rien dire. La documentation promettait un dessin « en direct »
que personne ne faisait : c'est la forme exacte de la fiche 22 § 9.1, *un commentaire qui décrit
une intention passe pour une description*. La fiche 24 § 13 l'avait prédit sans pouvoir
l'établir ; une épreuve des deux voies sur une carte à l'échelle 6 l'a établi en une exécution
— **746 519 canaux d'écart sur 1 920 000**, et l'image de la voie graphique reproduisait la
capture de l'utilisateur : la lueur, la grille, rien d'autre.

Deux variantes du même défaut, que personne n'avait nommées :

* **en mouvement**, une carte se rend au palier dyadique le plus proche, jusqu'à √2 fois sa
  taille à l'écran : une carte qui tenait à 80 % de l'écran le dépassait pendant le zoom, et
  disparaissait le temps du geste ;
* une **photo en chemin** vue de près passait par le même refus.

### 3.2 Ce que la mesure a dit avant d'écrire

Une carte rendue par tuiles, reposée, comparée à la carte dessinée en place et à la texture
entière d'avant, sur des images **regardées** :

* le texte et le fond sont identiques au bit près ; **aucune couture** aux frontières des tuiles ;
* les seuls écarts sont sur l'**arrondi des coins** et le liseré, un pixel de large, jusqu'à 48
  niveaux. Une marge autour de chaque tuile — même de mille pixels — n'y change rien : ce n'est
  pas le découpage, c'est la **position absolue** de la forme. `tiny-skia` approche un arc par
  des segments en virgule fixe dont l'arrondi en dépend ;
* **la texture entière d'avant a exactement le même écart** contre le rendu en place, dès
  l'échelle 2. Le test `test_une_carte_se_repose_au_bit_pres` ne regardait que l'échelle 1, où
  il est nul : « au bit près » n'était vrai qu'à l'échelle 1, et personne ne le savait. Le
  découpage n'ajoute rien à ce qui existait.

La borne n'est pas choisie : `tiny-skia` calcule quatre sous-lignes par pixel
(`SUPERSAMPLE_SHIFT = 2`, lu dans sa source), une sous-ligne qui bascule vaut 64 niveaux.

### 3.3 Ce qui a été construit

* **Les tuiles** : des carrés de `tuile::COTE` — le 256 que `bench_tuiles` avait mesuré pour
  TUILE-1, aucun nombre nouveau —, ancrés au coin du composant ; seuls ceux que l'écran montre
  existent. L'échelle est dans l'identité d'une tuile : une tuile d'une autre échelle montre un
  autre morceau, la poser à la place serait faux, pas flou.
* **La gouttière** : chaque tuile se rend avec un texel de ses voisines et se pose par sa
  fenêtre. En mouvement, le filtre bilinéaire lit le texel d'à côté ; sans elle, une couture
  d'un pixel paraîtrait pendant le geste. Un texel, parce que c'est le rayon du filtre.
* **Le repli** : la texture entière au plus haut palier dyadique où elle tient dans l'écran,
  sous l'**identité du composant entier**. La carte la pose à la place d'une tuile absente, et
  la garde pour le prochain zoom — c'est ce que fait un navigateur, qui garde toujours une
  version basse résolution. À l'entrée du régime découpé, la texture que la carte détenait déjà
  sert de repli.
* **Sa priorité** : toujours demandé tant qu'une carte est découpée, après tout le reste, et
  **jamais** par la règle « au moins une par image ». La première version ne le rendait que
  pour une tuile absente : ces tuiles passant avant lui, il n'aurait jamais eu de temps, et une
  carte ouverte de près n'en aurait jamais eu. Le test de la carte graphique l'a montré avant
  que l'utilisateur ne le voie.

Deux extractions pour tenir 600 lignes — `scene_gpu/cascade.rs` et `composants/contenu.rs` ; le
découpage lui-même vit dans un module neuf, `composants/decoupe.rs`.

### 3.4 Ce qui n'est pas vérifié

* **Le flou du repli** à l'arrêt d'un zoom : les tuiles de l'échelle exacte se rendent sur le
  budget, et le repli — jusqu'à huit fois moins fin à l'échelle maximale — se voit le temps
  qu'elles arrivent. Combien d'images, sur sa machine : la chronique le dira (`textures_kpx`,
  `textures_reportees`). Si ça se voit, la suite est connue : le repli par l'échelle
  précédente, comme un navigateur qui garde deux découpages.
* **Une carte en saisie vue de près** : sa clé suit le curseur, donc toutes ses tuiles visibles
  se refont deux fois par seconde. Chacune coûte ~0,3 ms ; jamais mesuré sur un écran entier.
* **L'arrondi des coins à l'échelle 2 à 6**, présent depuis COMPOSANT-1 : invisible à l'œil,
  jamais borné par un test hors de celui du découpage.

---

## 4. Le repos — l'instrument était aveugle

La fiche 29 § 4.4 relevait la contradiction : *« il dessine 13,4 images par seconde pendant qu'on
ne le touche pas »*, et deux lignes plus bas *« aucune : chaque image a été demandée par un
geste »*. Trois défauts, deux dans l'instrument et un dans l'application.

1. **Le masque des raisons de réveil était effacé avant d'être lu.** Il se notait comme un
   compteur de l'image dans `about_to_wait` — entre deux images — et `perf::frame_begin` vide
   les compteurs au début de la suivante. La section répondait « aucune » **à toutes les
   sessions depuis sa création**. Un test qui rejoue l'ordre réel (endormissement, début
   d'image, enregistrement) fait tomber l'ancien code sur les trois tests de provenance.
2. **La veille comptait la main en images classées sous un geste.** Un survol n'en a pas — il
   est classé « repos » —, donc les images qu'il faisait redessiner passaient pour dessinées
   sans utilisateur. La main se compte maintenant en **événements** : souris, molette, clavier,
   focus.
3. **Au-dessus de la barre, chaque mouvement du pointeur redessinait l'image entière**, même
   sur le même bouton ; la quitter ne redessinait rien, si bien qu'un bouton restait éclairé
   sous un pointeur parti. Le survol a maintenant **une** définition (`ui::bande::Survol`), lue
   par le dessin pour sa clé et par la souris pour savoir s'il change.

Chaque image porte désormais sa provenance (`app::reveil::Provenance`) : une raison de réveil,
**la main**, **un dépôt arrivé**, ou **le système** quand rien de connu ne l'a demandée. La
section du rapport devient « D'où viennent les images ». La ligne « le système » est celle qui
accusera — Windows qui demande de repeindre, ou une raison qu'on aurait oublié d'écrire.

**La cause des 13,4 images par seconde n'est pas établie.** La piste la plus probable est le
survol de la barre : dans la chronique du dépôt (copiée, 23/09 17 h 14), les 816 images « au
repos » redessinent souvent la bande (`bande` à 1,72 ms au p99). La prochaine chronique
tranchera, ligne par ligne.

Au passage : le champ mort `UiState::hovered_btn` (`#[allow(dead_code)]`) remplacé par le vrai
survol ; la taille d'une fenêtre sans fenêtre valait 1280 × 720 dans le clic et 1440 × 900 à sa
création — une seule constante, `TAILLE_INITIALE` ; `Chronique::rendues_sous_la_main`, sans
appelant, retirée.

---

## 5. NIVEAU-GPU-1 — les vagues, et ce que coûtait un envoi

### 5.1 Le banc d'abord

`bench_televersement`, sur la machine de l'utilisateur, en médiane de 24 envois :

| taille | Mo | copier la source | `write_texture` | RTX, jusqu'à la fin | Arc 140T, jusqu'à la fin |
|---|---:|---:|---:|---:|---:|
| 2297 × 3062 | 26,8 | **10,5 ms** | 2,9 ms | 4,0 ms | 5,4 ms |
| 1149 × 1531 | 6,7 | 2,1 ms | 0,8 ms | 1,5 ms | 2,2 ms |
| 575 × 766 | 1,7 | 0,5 ms | 0,2 ms | 0,6 ms | 1,3 ms |
| 288 × 383 | 0,4 | 0,01 ms | 0,06 ms | 0,4 ms | 0,9 ms |

Deux choses que personne n'avait mesurées :

* **la copie coûtait plus que l'envoi** — l'application clonait la texture native avant de la
  téléverser : 10,5 ms sur 13,4 ;
* **le niveau qu'il faut coûte deux cents fois moins** qu'un original, pour une vignette.

Deux originaux dans une image, et le budget de CASCADE-2 est mangé : le reste attend l'image
suivante. C'est l'explication la plus probable des vagues — à confirmer à l'écran.

### 5.2 Ce qui a été construit

* La carte reçoit **le niveau de la pyramide qui couvre encore la taille posée**, par la règle
  de la voie processeur (MIP-1), lue au même endroit : sur la largeur où la **source entière**
  se pose (`Recadrage::source_pour`), qu'un recadrage rend plus grande que la boîte. C'est ce que
  fait Chromium (`cc/tiles/gpu_image_decode_cache` : une puissance de deux plus petite, jamais
  sous la taille rendue).
* La clé porte le facteur (`src@f`), l'identité reste le fichier : quand le niveau change,
  l'ancien se pose pendant que le nouveau arrive.
* Les bornes de BORDURES-4 se calculent sur ce niveau (`Pose::bornes_de`, trois arguments).
* **Les pixels se prêtent** : `scene_gpu::Source` rend un `Cow`, et `Confie::pixels` — la seule
  définition, lue par la présentation, les bancs et les épreuves — prête le niveau du magasin.

### 5.3 Un défaut de qualité qu'on ne voyait pas

Lue un texel sur huit par le filtre bilinéaire, une photo réduite **crénelait**. Sur un damier
d'un pixel posé à 60 pixels, les deux voies divergeaient de **111 niveaux sur 9 216 canaux** —
du moiré sur la carte, du gris au processeur. Elles tiennent maintenant dans l'écart admis.

### 5.4 La mémoire — ce qui n'a pas été touché, et pourquoi

Le 1,3 Go de sa session n'a pas été attaqué de front, et c'est délibéré. Le magasin se borne
déjà à **la moitié de la mémoire disponible**, relue à chaque image (ADAPT-1), et la charte dit
*« utiliser 650 Mo lorsqu'on a 32 Go ? »* — ne pas se priver de ce que la machine offre. Chaque
photo y garde sa pyramide entière (≈ 1,33 × l'original) : l'original sert au zoom proche et à
`Ctrl+B`, qui lit la texture native.

Ce qui a changé est ailleurs : la carte ne reçoit plus que des niveaux, et les copies de 27 Mo
par envoi ont disparu. Combien la mémoire du processus a baissé, la prochaine chronique le dira.
Si elle reste trop haute pour l'utilisateur, la question devient une question de produit :
**garder les originaux en mémoire ou les relire au zoom** — cent à trois cents millisecondes de
décodage à chaque fois.

### 5.5 Ce qui reste des vagues

* **À l'ouverture d'un document**, les photos paraissent à mesure qu'elles se **décodent** : ce
  n'est plus le bus, c'est l'atelier. Garder toujours un petit niveau résident — ce que font les
  moteurs de jeu — ne sert qu'à ce qui a déjà été décodé une fois.
* Une photo qui sort de l'écran est oubliée de la carte à la fin de l'image : y revenir la
  téléverse de nouveau — mais au niveau voulu, donc en une fraction de milliseconde.

---

## 6. La gravure

Son fichier n'a pas été retrouvé : **486 images distinctes** examinées — celles de tous ses
documents `.glucose` (extraites de leurs sections de nature 3), les 105 dépôts de
`glucose_depose`, les images récentes de ses dossiers — et classées par saturation, orientation
et bordure claire. Aucune n'est un paysage à la pointe sèche. Elle a probablement été glissée
d'une page sans être rapatriée. **Il faut la lui demander.**

---

## 7. Ce que cette session a démenti — la liste

1. **« Une carte texturée se repose au bit près »** — vrai à l'échelle 1 seulement ; jusqu'à 48
   niveaux sur l'arrondi des coins dès l'échelle 2, depuis COMPOSANT-1 (§ 3.2).
2. **« La section des réveils dit pourquoi l'application ne dort pas »** — elle n'a jamais rien
   dit : le masque était effacé avant d'être lu (§ 4).
3. **« Un intervalle sans image classée est un intervalle sans main »** — un survol n'est classé
   sous aucun geste (§ 4).
4. **« Le coût d'une photo sur la carte, c'est son envoi »** — c'était d'abord sa copie (§ 5.1).
5. **« Une photo réduite est nette sur la carte »** — elle crénelait (§ 5.3).
6. **Ma propre première version du repli** — il n'aurait jamais eu de temps (§ 3.3). Le test
   l'a vu, pas l'utilisateur.

Et une leçon de méthode, la même que la fiche 29 : **tous les tests de cette session passaient
du premier coup**. C'est une raison de douter, pas de conclure — les quatorze sabotages l'ont
confirmé, un par un.

---

## 8. Ce qui reste, chiffré

| # | Piste | Ce qu'on sait | Ce qu'il faut |
|---|---|---|---|
| 1 | Les 13 images par seconde au repos | la provenance les nommera | **sa prochaine chronique** |
| 2 | Les gels hors rendu, la session fermée pendant un gel (fiche 29 § 4.5) | rien de nouveau | **sa prochaine longue session** |
| 3 | La cadence à 164-194 nœuds (fiche 29 § 4.6) | rien de nouveau | idem |
| 4 | `docks` à 5,8 ms sur une sélection | la chronique du dépôt : « sélectionner » à 11,6 ms en médiane, **au-dessus du plancher** à cause des panneaux | un banc des deux panneaux à 150 % |
| 5 | Le flou du repli à l'arrêt d'un zoom proche | non mesuré | l'écran |
| 6 | La mémoire | le magasin n'a pas changé | sa chronique, puis sa décision (§ 5.4) |
| 7 | La gravure | introuvable | **son fichier** |
| 8 | Trans-domaines | retiré | **sa définition** |
| 9 | La recherche d'origine (fiche 27, étape 3) | pHash prêt | **une clé SauceNAO** — reposée une fois, pas plus |

---

## 9. Ce qu'il faut tester à l'écran

Une longue session, avec son document d'environ 190 nœuds, la sortie dans un fichier :

```text
cargo run --release > sortie-longue.txt 2>&1
```

1. **De près** : zoomer sur une carte de texte jusqu'à ce qu'elle dépasse l'écran, s'y déplacer,
   s'y arrêter. Le texte doit rester là ; dire s'il se voit flou un instant à l'arrêt.
2. **Les images** : glisser une dizaine d'épingles, puis dézoomer d'un coup et rezoomer. Dire si
   elles arrivent encore en vagues, et si une photo réduite paraît plus propre qu'avant.
3. **Le repos** : laisser l'application sans y toucher une minute, la souris hors de la fenêtre.
4. **Trans-domaines** : cliquer dessus — il ne doit rien se passer.

Puis fermer par la croix, et **copier la chronique** (`%TEMP%\glucose-chronique\derniere-session.txt`)
avant de relancer quoi que ce soit : la section « D'où viennent les images » et le processeur
« pendant qu'on ne le touche pas » diront ce que le repos cache.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
