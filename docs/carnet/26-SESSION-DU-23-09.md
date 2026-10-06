# 26 — Le gel avait un nom, et le verdict le donnait à un autre

> **Rôle de ce document.** La fiche [`25`](25-SESSION-DU-22-09-SOIR.md) laissait le gel du
> canevas inexpliqué après deux hypothèses démenties, une piste principale — *la carte ouverte
> par bascule reste durablement dégradée* — et un poste « jamais instrumenté » depuis trois
> sessions. Cette fiche dit ce qu'une lecture attentive de la **même** chronique a tranché, ce
> qu'elle a démenti dans le plan qu'on me donnait, et les quatre chantiers qui en sont sortis.
>
> **Date** : 2026-09-23 · quatre commits, de `c134eea` à `d8d0499`.
> **État vérifié** : `cargo test --workspace` exit 0, **1 468 tests verts**, clippy strict à
> zéro, douze cliquets, aucun plafond relevé.
>
> **Le point de départ** : la chronique de dix-sept secondes du 22/09 au soir — la bascule à
> chaud vers la RTX, `present` à 406 ms au pire, un gel de 763 ms « dont 748,7 à ne pas
> dessiner », et un verdict dont le premier constat était *« tressaut x85 »*.

---

## 1. Ce qui a été fait, en une table

| Chantier | Ce que c'est | Vérifié par |
|---|---|---|
| **ENTRACTE-1** (`chronique/entracte.rs`) | Le temps « à ne pas dessiner » se découpe en cinq postes qui se **somment exactement** : attendre Windows, écouter la main, poser un dépôt, changer de carte, entretien. Le pire entracte est gardé décomposé | 7 tests, dont un qui porte la preuve à l'envers ; l'aperçu du rapport |
| **ARBITRE-3** (`app/terrain.rs`) | La bascule de carte attend que la vue s'immobilise | 3 tests ; garde retirée, le premier tombe (`left: None, right: Some(Rapide)`) |
| **TRESSAUT-1** (`chronique/rythme.rs`, `verdict.rs`) | Chaque saut se range du côté de ce qui a fait bouger l'intervalle ; le tressaut ne se juge plus que sur le rendu ; **le gel a son propre constat** | 2 tests, chacun le cas inverse de l'autre ; ancienne lecture remise, le premier tombe sur « tressaut x9,1 » |
| **DOCKS-1** (`dock/cache.rs`) | Les panneaux qui comptent la sélection la montraient **périmée** ; la clé dit désormais pourquoi elle refait un panneau | 2 tests ; le premier tombait sur 27 198 pixels faux avant la correction |

Et un banc, `bench_journal`, qui dit pourquoi un suspect n'en est pas un (§ 2.1).

---

## 2. Le gel de la dernière session a un nom, et deux chiffres le donnaient déjà

Deux lignes de la même sortie, jamais rapprochées :

```
    [Glucose] arbitre : la presentation passe sur la carte rapide ...
              (ancienne lachee en 138 ms, nouvelle ouverte en 606 ms)

    le pire gel : 763.3ms a la 10.4e seconde, dont 748.7ms a ne pas dessiner
```

138 + 606 = **744 ms**. Le pire gel en passe **748,7** à ne pas dessiner. Six millièmes
d'écart, et le reste est le tour de boucle qui entoure la bascule. L'image suivante, à la
10,8ᵉ seconde, paie encore **406 ms de `present`** : la première présentation sur une chaîne
neuve.

**Un peu plus d'une seconde de canevas figé, en plein zoom — c'est la bascule de carte de
l'arbitre, qui a lieu dans `about_to_wait`, de façon synchrone.** Rien ne pouvait le nommer :
la chronique ne découpait pas le temps passé hors du rendu.

### 2.1 Le suspect évident était innocent, et un banc l'a dit en une exécution

`about_to_wait` écrit le rapport **entier** sur disque dès qu'une image entre dans les
trente-deux plus lentes. J'y ai cru — c'est le seul accès disque de la boucle. `bench_journal`
rejoue la distribution du terrain et chronomètre les deux moitiés, là où l'application écrit
vraiment :

| | |
|---|---:|
| sauvegardes sur 731 images | **98**, une image sur 7,5 |
| composer le rapport | 0,02 ms |
| l'écrire dans `%TEMP%`, antivirus compris | 0,05 ms |
| **sur toute la session** | **8 ms** |

Pas la cause. Le banc reste, pour que personne ne le resoupçonne.

### 2.2 Ce que l'entracte mesure désormais

Cinq postes, et à tout instant le contrôle est à un seul :

| poste | qui tient le contrôle |
|---|---|
| attendre Windows | winit et Windows — le sommeil demandé, la file de messages |
| écouter la main | nos gestionnaires : souris, clavier, molette |
| poser un dépôt | ce qu'un glisser-déposer apporte, décodage compris |
| changer de carte | lâcher une carte et en ouvrir une autre |
| entretien | le reste d'`about_to_wait` : empreinte, chronique, réveils |

**Leur somme vaut l'entracte au bit près**, et un test l'exige. C'est la garantie qui
manquait aux quatre marques mal posées du dépôt (`occlusion`, `recolte`, `blit`, `minimap`) :
ce qu'un poste ne prend pas, un autre le porte, rien ne disparaît.

Le rapport les range sous la ligne « dont à NE PAS dessiner », à laquelle ils répondent, et
termine par la ligne qui nomme un gel :

```
    la pire attente : 744.8ms a la 22.5e seconde -- dont changer de carte 744.0ms, ...
```

### 2.3 Le tableau trie par le PIRE, et c'est l'inverse de celui des postes du rendu

La fiche 24 § 2 a fait passer le tableau des postes du rendu de la médiane au p99. Je l'ai
repris par réflexe, et l'aperçu a montré la faute au premier coup d'œil : **« changer de
carte » arrivait dernier**. Un poste du rendu se paie à chaque image, donc son p99 désigne ce
qui gèle. La bascule de carte a lieu au plus deux fois dans la vie du processus : son p99 est
nul. Trier par lui reléguait le seul poste qu'on cherchait.

> **La leçon de la fiche 24 § 2, un cran plus loin** : la grandeur qui trie doit être celle
> qu'on cherche. Pour le tempo, c'est le p99. Pour un gel qui n'arrive qu'une fois, c'est le
> pire.

---

## 3. ARBITRE-3 — on ne peut pas rendre la bascule rapide, on peut la rendre invisible

**Ouvrir une carte graphique coûte une demi-seconde, et ce n'est pas un défaut.** Les mesures
publiées de `wgpu` donnent le même ordre ailleurs — instance 202 ms, adaptateur 143,
périphérique 85, configuration de la surface 140 — et c'est pour cette raison que les
logiciels professionnels (DaVinci Resolve, Blender) demandent un **redémarrage** pour changer
de carte.

Un gel d'une seconde pendant que rien ne bouge n'a rien à montrer de travers. L'arbitre
conclut quand il conclut — c'est pendant le mouvement qu'il mesure — ; **sa demande attend
le premier instant où la vue s'immobilise**. Il arrive toujours : l'élan s'éteint de lui-même
quand la main lâche.

**Aucune constante nouvelle.** L'immobilité est mot pour mot celle que le tempo juge déjà :
*« le tempo ne règle que ce qui BOUGE : c'est la seule situation où un intervalle irrégulier
se voit »*. Même loi, même raison, et un test exige que les deux jugements restent d'accord.

### 3.1 Ce que la mesure a démenti dans le plan qu'on me donnait

Le brief et la fiche 25 § 9.3 portaient une hypothèse forte : *« la RTX ouverte au lancement
ne gèle pas, la même RTX ouverte par bascule gèle — trois cents fois pire sur `present` »*,
parce que la fenêtre resterait associée à l'Arc et que la composition traverserait le bus.
La sortie proposée était de renoncer à la bascule à chaud et de persister le verdict.

**La chronique qui a inspiré cette hypothèse la dément.** Après la bascule, au zoom :

```
    present   median 0.13ms    p99 2.90ms    pire 406.14ms
```

La RTX présente **normalement**. La seule valeur à 406 ms est l'image qui suit immédiatement
la bascule. Et le tableau du § 9.3 comparait deux choses différentes :

| | ce que la colonne mesurait vraiment |
|---|---|
| « rapide au lancement » | 437 s **entièrement** sur la RTX |
| « bascule à chaud », 17 s | **10 s sur l'Arc**, puis la bascule, puis 7 s sur la RTX |

L'écart attribué à « la façon d'ouvrir la carte » est l'écart entre les deux cartes, plus le
coût ponctuel de la bascule. `soumettre` à 48,82 ms le confirme : cette image est à la 6,2ᵉ
seconde, donc **avant** la bascule, donc sur l'Arc.

**L'arbitrage que le brief voulait soumettre n'a donc pas lieu d'être** — ni renoncer à
l'adaptation en temps réel, ni persister le choix d'une session à l'autre. Une reconstruction
d'une seconde ne doit simplement pas tomber au milieu d'un geste.

> **Ce qui reste vrai, et il faut le garder** : la persistance du verdict éviterait de payer
> cette seconde au repos à *chaque* lancement. C'est un raffinement, plus une nécessité — et
> elle introduirait un état hors du document, ce qui se décide avec l'utilisateur.

---

## 4. TRESSAUT-1 — le premier du verdict depuis quatre sessions était un gel compté deux fois

Le tressaut empirait : x76, puis x25, puis **x85**. Le brief proposait de l'attaquer par la
prédiction de pose.

### 4.1 Le calcul

Le saut d'une image vaut `vitesse × |intervalle − pas|`, et depuis la fiche 19 le pas vaut
l'intervalle de l'image **précédente**. Un gel produit donc **deux** sauts géants : l'image
figée, dont le contenu s'arrête, puis l'image de rattrapage, dont le contenu bondit. Dès qu'un
pour cent des intervalles gèle, le p99 du saut tombe forcément dans les gels.

Et la gravité du tressaut **était** ce p99 rapporté à l'avance : **1 024 / 12 = 85,3** — le
nombre exact de la chronique.

Pendant ce temps, **le gel n'avait aucun constat à lui**. La cadence compte les images dont le
*rendu* dépasse le plancher ; 748 ms passées à ne pas dessiner n'en sont pas une. Le seul
endroit où le gel paraissait dans le verdict, c'était sous le nom d'un défaut de mouvement.

### 4.2 Ce qui change, sans un seuil

* Chaque intervalle se décompose en ce que l'application a passé à **ne pas dessiner** et ce
  qu'elle a passé à **dessiner**. La part qui a le plus changé depuis l'image précédente est
  celle qui a déplacé le contenu : le saut se range de son côté. **Deux mesures qu'on compare,
  pas une constante qu'on choisit.**
* Le tressaut ne se juge plus que sur les sauts que le **rendu** a causés.
* **Le gel a son constat**, avec la grandeur qu'ARBITRE-2 a dû apprendre pour `present` : le
  temps passé à ne pas dessiner au-delà du plancher, ramené en images perdues, comparé à la
  part tolérée. `BUDGET_TOTAL` et `PART_TOLEREE` — aucun nombre neuf.
* La cadence cite enfin les **durées de rendu** qu'elle compte. Elle citait les intervalles,
  qui contiennent les gels, et annonçait « c'est un gel » sur ce que le nouveau constat porte.

Les deux tests portent chacun le cas inverse de l'autre — sans quoi le premier ne prouverait
rien, puisqu'une séparation qui rangerait tout du côté de l'attente le passerait :

| session rejouée | tressaut | gel |
|---|---|---|
| trois balayages tenus, six gels de 110 ms et un de 748 | **aucun** | **en tête, x19** |
| le rendu déborde d'une image sur trente, jamais de pause | **oui** | **aucun** |

Vérifié à l'envers : l'ancienne lecture remise, le premier test tombe sur « tressaut x9,1 »
pour une session où le rendu n'a pas bougé d'une microseconde.

> **Ce que cela retire du plan** : la prédiction de pose reste un chantier légitime pour la
> *latence* (fiche 23 § 4.2), mais **pas pour le tressaut du verdict**. Depuis la fiche 19,
> une machine qui tient son tempo ne produit aucun saut : tout le tressaut vient des ratés, et
> les ratés se soignent à leur cause.

---

## 5. DOCKS-1 — la clé était trop étroite avant d'être trop large

La fiche 24 § 14.1 concluait que la clé des panneaux est « trop large », sans pouvoir dire
laquelle de ses parties bougeait. En allant la lire, j'ai trouvé l'inverse avant la réponse.

« Domaines » écrit *« N nœud(s) sélectionné(s) »* et grise ses boutons d'assignation quand
rien n'est sélectionné ; « Ordonner » compte les images visées. La clé ne connaissait du
document que sa **version** — et sélectionner n'en change pas, parce que c'est de la
navigation (fiche 05 § 3.5). **Les deux panneaux restaient sur leur ancien compte, boutons
grisés, tant que la souris ne passait pas dessus.** 27 198 pixels faux, jusqu'à 197 niveaux,
dans le test écrit avant la correction.

La clé retient désormais le **nombre** de nœuds sélectionnés — exactement ce que les deux
panneaux lisent. Comparer les identifiants copierait la sélection entière par panneau et par
image, pour un « tout sélectionner » sur dix millions de nœuds.

**Et la question de la fiche 24 reste ouverte, mais elle a maintenant son instrument** : le
cache nomme la partie de sa clé qui a changé chaque fois qu'il refait un panneau, jusqu'à une
section du rapport, *« Pourquoi les panneaux se redessinent »*. En lisant le code, **aucune
partie ne devrait bouger pendant un zoom** ; la prochaine chronique dira laquelle le fait.

---

## 6. Ce que cette session a démenti — la liste

1. **Le journal écrit sur disque** comme cause du gel : 8 ms sur toute une session.
2. **La piste principale du brief et de la fiche 25 § 9.3** : la RTX ouverte par bascule
   n'est pas durablement dégradée. C'est la bascule qui coûte, pas son résultat.
3. **Le tableau du § 9.3** comparait une session sur une carte à une session sur deux.
4. **Le tressaut comme défaut indépendant** : c'était le gel, compté deux fois, rangé premier.
5. **Mon tri de l'entracte par le p99** : repris par réflexe, il reléguait la bascule en dernier.
6. **`fermer` rangeait un entracte jamais ouvert** : deux tests ont protesté, ils avaient raison.
7. **La clé des panneaux « trop large »** : sur la sélection, elle était trop étroite.

---

## 7. Ce qui reste, chiffré

| # | Ce que c'est | Chiffre | Statut |
|---|---|---|---|
| 1 | **Le gel restant, hors bascule** | « à ne pas dessiner » au p99 : 110 ms | **L'entracte le dira** à la prochaine session |
| 2 | **Ce qui refait les panneaux pendant un zoom** | `docks` jusqu'à 9,91 ms | La section « Pourquoi les panneaux se redessinent » le dira |
| 3 | **La mémoire : 633 Mo au pire, 325 à la fin** | 307 Mo sur la RTX dès le lancement ; 461 à 598 sur l'Arc | **Piste, non établie** : une carte intégrée n'a pas de mémoire à elle, elle emprunte la RAM — ses textures et sa chaîne d'images pourraient compter dans la mémoire du processus, celles d'une carte dédiée non. Se teste en deux lancements du même document : `GLUCOSE_CARTE=econome` puis `=rapide` |
| 4 | **`soumettre` à 48 ms** | sur l'Arc, avant la bascule | Le chemin hybride de la fiche 22 § 12, probablement ; à confirmer |
| 5 | **`Ctrl+B`** | défaut non reproduit | **Le fichier d'une image en défaut est demandé** — sans lui, toute correction est une invention |
| 6 | **Pinterest** | `GLUCOSE_DEPOT=1` jamais lu | **La sortie est demandée** |
| 7 | **La persistance du verdict de l'arbitre** | une seconde au repos, à chaque lancement | Raffinement ; engage un état hors du document |
| 8 | Le reste de la fiche 25 § 7 | — | Inchangé |

---

## 8. Ce qu'il faut tester à l'écran, dans l'ordre

1. **Lancer sans variable, zoomer et se déplacer une minute.** Si l'arbitre bascule, la
   bannière le dit — et le canevas ne doit plus figer **pendant** le geste : la bascule
   attend qu'on lâche. Fermer, coller la sortie entière.
2. **Dans la même chronique**, lire sous « dont à NE PAS dessiner » la section *« et voici OÙ
   il est allé »*, et la ligne *« la pire attente »*.
3. **Sélectionner des nœuds** avec le panneau Domaines ouvert, la souris sur le canevas : le
   compte doit suivre, et les boutons d'assignation s'allumer.
4. **Deux lancements du même document pour la mémoire** : `GLUCOSE_CARTE=econome`, puis
   `GLUCOSE_CARTE=rapide` — et comparer la ligne « mémoire de travail ».

---

## 9. La première session de terrain avec les nouveaux instruments, et ce qu'elle a tranché

L'utilisateur a joué vingt-huit secondes sur la carte rapide — 1 588 images, 277 pincements —,
testé l'enregistrement sur les deux cartes (*« les save .glucose fonctionnent, que ce soit
rapide ou économe »*), et glissé deux épingles depuis Pinterest. **Sa sortie console ne m'est
pas parvenue** : seule la chronique, écrite sur son disque, a pu être relue. Elle ne garde que
la **dernière** session, donc la mémoire sur l'Intel est perdue.

### 9.1 La clé des panneaux n'était pas trop large — démenti de la fiche 24 § 14.1

```
    Pourquoi les panneaux se redessinent
      document modifie              7 images
      souris dans le panneau        6 images
      selection                     5 images
      premiere fois                 1 images
      place ou taille               1 images
```

**Vingt images sur 1 588, toutes pour une raison légitime.** Les panneaux ne se refont
**jamais** pendant un zoom. La fiche 24 avait vu `dock=2` sur dix des douze images les plus
lentes et en avait conclu que la clé était trop large ; c'était l'inverse de la causalité :
les images qui refont un panneau sont celles où le document change, et ces images-là sont
chères pour d'autres raisons aussi. **Ce qui reste à gagner est le coût d'un rendu légitime** —
6 ms pour deux panneaux à 150 % —, pas la clé.

Et les cinq images « sélection » sont exactement celles qui montraient, avant DOCKS-1, un
compte périmé.

### 9.2 Pinterest donne un raccourci, pas une image — DEPOT-WEB-2

Les deux fichiers déposés étaient encore sur le disque, soixante-quatorze octets chacun :

```
    [InternetShortcut]
    URL=https://fr.pinterest.com/pin/288441551156395185/
```

Chrome promet un **raccourci** pour un lien glissé, et le pont le prenait au deuxième rang,
**devant** le bitmap — une adresse habillée en fichier, posée en carte portant son nom. Un
raccourci se lit désormais pour ce qu'il est et se pose en lien qu'on suit, d'où qu'il vienne ;
il passe **après** le bitmap. Et deux `.jpg` déposés la veille par le même chemin prouvent que
le pont fonctionne quand la page livre l'image.

**Ce qui n'est pas résolu** : si Pinterest ne pose aucun bitmap, on obtient le lien de
l'épingle, pas son image. L'image demanderait de télécharger la page — HTTP, donc une décision
qui engage la charte. La liste des formats que `GLUCOSE_DEPOT=1` écrit pour ce dépôt tranchera
entre les deux, et elle n'est toujours pas lue.

### 9.3 Le gel du démarrage cachait les autres — ENTRACTE-2

Le verdict : **[gel] x14,6**, 2 314 ms perdues à ne pas dessiner. L'entracte n'en décomposait
qu'une attente — la pire, **le démarrage**, 607 ms à la 0,6ᵉ seconde. Les trois quarts restants
n'avaient pas de nom, et « écouter la main » avait un pire à **162,66 ms** — un seul
gestionnaire d'événement, sans doute un enregistrement — qu'aucune ligne ne datait.

C'est la faute de la fiche 25 § 7 — une valeur extrême qui cache les autres —, refaite dans
l'instrument écrit pour la réparer. Chaque attente qui a mangé au moins une image est
désormais gardée, datée et décomposée ; la liste et le constat du gel se recoupent par
construction.

### 9.4 Ce que la session dit d'autre

| | |
|---|---|
| mémoire de travail, carte rapide | **311 Mo**, 313 au pire — le chiffre de l'Intel manque pour tester la piste du § 7 |
| aucune bascule de carte | `changer de carte` 0,01 ms au pire : ARBITRE-3 n'a pas été exercé |
| le tressaut, enfin le vrai | **x7,7** sur les seuls sauts du rendu : 3 balayages 88,5 %, 2 : 8,4 %, 4 : 3,1 % — les pics de `textures` à 8-9 ms |
| **`bande` à 32,28 ms** | une seule image, à la 4,6ᵉ seconde, quand le document change : la barre du haut refaite coûte trente fois son remplissage. Le facteur vingt de la fiche 24 § 14.2 a doublé, et reste inexpliqué |

---

## 10. Deux sessions de plus, et une correction à ce que ce document affirmait

L'utilisateur a joué deux sessions, sorties écrites dans un fichier : `GLUCOSE_CARTE=econome`
avec `GLUCOSE_DEPOT=1`, puis sans carte imposée. **La seconde a « COMPLÈTEMENT » gelé.**

### 10.1 La mémoire : la piste du § 7 est établie

| même document | mémoire de travail |
|---|---:|
| NVIDIA, ouverte au lancement | **311 Mo** |
| Intel Arc, imposée | **636 Mo** |

Deux sessions, une seule différence. Une carte intégrée n'a pas de mémoire à elle : ce
qu'elle détient vit dans la RAM du processus. Les 461 à 633 Mo des fiches 25 et 26 étaient
l'Intel, pas une fuite. Ce que les deux cartes détiennent pour vingt-deux nœuds — environ
325 Mo — reste à mesurer poste par poste, et c'est un chantier d'empreinte à part.

### 10.2 Pinterest ne donne presque rien — DEPOT-WEB-3

Dix dépôts sur l'Intel. **Six** ne portaient **aucun** format standard : `DragContext`,
`DragImageBits` — la vignette qui suit le curseur —, `chromium/x-renderer-taint`, et
`Chromium Web Custom MIME Data Format`, les données que la page pose elle-même. **Quatre**
portaient le lien de l'épingle — le raccourci `.url` — et un fragment HTML. **Aucun** ne
portait l'image.

L'instrument écrit désormais les adresses cachées dans chaque format. Mais quelle qu'elle
soit, **obtenir l'image demande de la télécharger** : c'est la décision que DEPOT-WEB-1 avait
évitée, et la mesure l'impose maintenant. Elle est soumise à l'utilisateur.

### 10.3 La moitié des gels mesurés étaient faux — GEL-1

Sur l'Intel, le verdict mettait **[gel] x310** en tête : 11 020 ms à la 25,8ᵉ seconde, puis
1 773, 1 197, 1 006 ms, tous « attendre Windows », tous finis par « poser un dépôt ».
L'utilisateur n'a rien vu geler dans cette session. Il était **dans son navigateur**.

La mesure partait de l'image précédente dès qu'une image était « attendue », et « attendue »
se décidait au dernier moment : le dépôt lance un décodage, l'image suivante devient attendue,
et tout le temps passé dans le navigateur devenait un gel. **C'était la dette de la fiche 24
§ 7, et le constat du gel que j'ai ajouté au § 4 l'avait portée en tête du verdict.**

Un gel se compte désormais depuis l'instant où l'image est **devenue nécessaire** — posé par
`mark_dirty` et `salir`, pris au début du rendu. Et « attendre Windows » devient décisif :
une attente chez Windows **alors qu'une image était due** veut dire que Windows tenait le fil.

### 10.4 Le gel qui ne finit pas était invisible

La session sans carte imposée a gelé « complètement », et sa chronique finissait sur une
image normale. Elle ne pouvait pas faire autrement : **un gel ne se mesurait qu'entre deux
images présentées**, et un gel qui ne finit jamais n'a pas d'image suivante. Fermer pendant
qu'une image est due se range désormais comme un gel, et le rapport l'écrit en première ligne.

### 10.5 La correction : j'avais écarté trop vite ce que le brief proposait

Le § 3.1 conclut que *« l'arbitrage que le brief voulait soumettre n'a pas lieu d'être »*.
**C'était aller trop loin.** J'ai démenti un **mécanisme** — la carte ouverte par bascule reste
lente, `present` p99 2,90 ms le réfute — et j'en ai conclu contre la **corrélation**, que la
mesure n'a jamais démentie :

| session | carte | gel ressenti |
|---|---|---|
| 22/09, 437 s | rapide **au lancement** | aucun |
| 23/09 12 h 53, 28 s | rapide **au lancement** | aucun |
| 22/09, 37 s | **bascule** | 13 s |
| 22/09, 17 s | **bascule** | 763 ms |
| 23/09 13 h 37, 18 s | **bascule**, au repos (ARBITRE-3) | « complètement » |

Cinq sessions, et la ligne de partage n'a pas bougé. ARBITRE-3 a fait basculer la carte **au
repos** — 586 ms, puis 467 ms sur la première présentation —, et la session a gelé quand même.
Et **les trois dépôts qui ont suivi la bascule ont affiché une liste de formats vide**, ce
qu'aucun des dix dépôts de la session sans bascule n'a fait.

**Ce qui n'est pas établi** : le mécanisme du gel complet. Les instruments de GEL-1 et
DEPOT-WEB-3 le diront à la prochaine session sans carte imposée. **Ce qui l'est assez pour
agir** : ouvrir une carte au lancement n'a jamais gelé, en changer en cours de route a gelé
trois fois sur trois. Si la prochaine session le confirme, la sortie est celle que le brief
proposait — l'arbitre retient son verdict et le lancement suivant ouvre la bonne carte —, et
c'est une décision qui engage la vision.

---

## 11. Le gel complet, enfin nommé — ARBITRE-4

La session du 23/09 à 14 h 44, sans carte imposée, a « RE freeze total ». Pour la première
fois, les instruments de GEL-1 étaient en place, et ils ont dit ce qu'aucune chronique
n'avait pu dire :

```
    8,3 s    l'arbitre passe sur la NVIDIA (89 + 475 ms)
    8,8 s    première image sur la nouvelle carte : 500 ms de present
    9,4 s    Glucose dessine et PRÉSENTE des centaines d'images, à 103 par seconde ;
    → 14 s   latence du geste p99 16,4 ms ; à la fermeture, aucune image en retard
             de plus de 46 ms — et l'utilisateur voyait un canevas FIGÉ
```

**Les images partaient, et n'arrivaient pas à l'écran.** Chaque présentation répondait
« réussie ». C'est ce que l'utilisateur décrit depuis trois jours — *« le canva freeze, mais
Ctrl+O, Ctrl+S fonctionnent »* : ces fenêtres-là sont dessinées par Windows, pas par Glucose.

Cela dément la fiche 25 § 9.3 — la carte ne présente pas lentement, elle présente vite et
pour personne — **et ma réfutation du § 3.1** : j'avais lu `present` à 2,90 ms au p99 comme la
preuve que la RTX présentait normalement.

> **La leçon, et elle vaut pour tout instrument de ce dépôt : un accusé de réception n'est pas
> une livraison.** Une présentation qui répond vite dit que la carte a accepté l'image, pas
> que l'œil l'a vue. Rien, de l'intérieur de l'application, ne mesure ce que l'écran montre —
> seul l'utilisateur le peut, et c'est pour cela que son *« c'est figé »* devait primer sur
> mes chiffres.

**La correction** : l'arbitre ne change plus de carte en cours de route. Quand il conclut, il
écrit son choix dans `carte.txt` (dossier local de l'application), la bannière le dit, et le
lancement suivant ouvre directement cette carte — le chemin qui n'a jamais figé en cinq
sessions. Le poste « changer de carte » de l'entracte disparaît avec ce qu'il mesurait.

**Ce qui reste à confirmer à l'écran** : premier lancement sans variable sur l'Intel, jusqu'à
ce que la bannière dise *« la carte rapide s'ouvrira au prochain lancement »* ; puis un second
lancement, qui doit s'ouvrir sur la NVIDIA et ne jamais figer.
