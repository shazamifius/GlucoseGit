# 05 — Standards de code

> Tu dis : *« le code est vraiment extrêmement moche actuellement, il est horrible »*.
> Tu as raison, mais la laideur n'est pas où on la cherche d'habitude. Ce n'est ni le nommage
> ni l'indentation — ils sont corrects. **C'est la structure.**
>
> Ce document est la liste des règles qui empêchent le code de redevenir ce qu'il est
> aujourd'hui. Chacune vient d'un constat réel de l'audit, pas d'un manuel de style.

---

## Ce qui rend le code actuel « moche » — diagnostic précis

Trois causes, dans l'ordre de leur impact visuel :

### 1. La répétition structurelle (≈ 60 % de la laideur)

Créer une annotation, aujourd'hui :

```rust
let ann = Annotation::Text {
    id: aid.clone(), x: wx, y: wy,
    width: Some(240.0), height: Some(48.0),
    text: initial_str.clone(), font_size: Some(14.0),
    color: None, cursor_pos: None, source_file: None,
    membrane_id: None, domains: Vec::new(),
    mirror_of: None, temporal_anchor: None,
};
```

**14 lignes, dont 8 disent « rien ».** Ce bloc apparaît **4 fois** dans `app.rs` avec des
variations mineures. Ce n'est pas un problème de style : c'est R-23 (le modèle répète ses champs)
qui remonte à la surface.

**Après composition + constructeurs** :

```rust
let node = Node::text(id, Vec2::new(wx, wy)).with_size(240.0, 48.0).with_content(initial);
```

La laideur disparaît **parce que la structure a changé**, pas parce qu'on a reformaté.

### 2. L'imbrication profonde (≈ 30 %)

`window_event` : **490 lignes, 7 niveaux d'imbrication**. À ce stade, la moitié de l'écran est
occupée par des accolades, et on ne voit plus le code.

```rust
match event {                                   // 1
  WindowEvent::MouseInput { .. } => {           // 2
    match button {                              // 3
      MouseButton::Left => {                    // 4
        if state == Pressed {                   // 5
          if my < HEADER {                      // 6
            if let Some(action) = ... {         // 7
              match action { ... }              // 8
```

**Règle** : au-delà de **4 niveaux**, c'est une fonction à extraire. Toujours.

### 3. Les nombres magiques en double (≈ 10 %)

`cur_x += 32.0` recopié dans deux fonctions, `Color::from_rgba8(56,189,248)` recopié 11 fois.
Chaque littéral est une occasion de divergence, et R-07 en est la preuve vivante.

---

## Les deux règles qui passent avant les autres

Elles ne sont pas des conventions de style. Elles viennent du propriétaire du projet, et elles
priment sur tout ce qui suit.

### R1 — Rien à moitié

**Ce qui est fait doit être fait à fond.** Une fonctionnalité livrée à moitié est pire que pas
livrée du tout, parce qu'elle a l'air de marcher — et parce qu'elle consomme la confiance qu'on
place dans le reste.

Ce dépôt est un catalogue de ce que coûte l'inverse : douze modules du noyau écrits, testés, et
appelés par personne ; un panneau de domaines qui empile des lignes décoratives dans une liste
que rien ne lit ; dix boutons qui affichent un message décrivant une action qui n'a pas lieu ;
une suppression de domaine qui oublie ses assignations et les écrit sur disque.

En pratique :

- Une fonctionnalité n'est **pas** finie tant qu'elle n'est pas **branchée**, **visible**
  par l'utilisateur, **annulable** si elle modifie le document, et **conservée** à
  l'enregistrement. Quatre conditions, pas une.
- Une assignation qu'on ne voit nulle part n'est pas une assignation, c'est un champ.
- Si un morceau ne peut pas être fini proprement, **il se dit** — dans le rapport, dans l'audit,
  dans un `TODO` daté. Jamais en silence.

### R2 — L'ancien code TypeScript n'est pas un modèle

Le dossier `src/` contient la version TS/TSX (32 423 lignes hors tests). Elle sert d'**inventaire
de ce qui existe** — quelles fonctionnalités, quel comportement attendu — et **à rien d'autre**.

**Son implémentation ne doit être ni portée, ni imitée, ni citée comme justification.** On
réécrit tout en Rust, et on le fait **mieux**. Une décision se défend par son raisonnement et par
une mesure, jamais par « c'est comme ça que le TypeScript faisait ».

Le corollaire vaut aussi dans l'autre sens : quand cette documentation invoquait le TSX pour
appuyer une règle, c'était un raccourci fautif. Les règles de ce document doivent tenir sans lui.

---

## Les règles

### § 1 — Structure

| Règle | Seuil | Vérification |
|---|---|---|
| **1.1** — Une fonction ne dépasse pas **60 lignes** | 60 | lint CI |
| **1.2** — Un fichier ne dépasse pas **500 lignes** | 500 | lint CI |
| **1.3** — L'imbrication ne dépasse pas **4 niveaux** | 4 | lint CI |
| **1.4** — Une fonction a **une** raison de changer | — | revue |
| **1.5** — Un `match` de plus de 5 bras avec du corps → chaque bras devient une fonction | 5 | revue |

**1.6 — Sortir tôt.** Un `if let Some(x) = … else { return }` en tête vaut mieux qu'un bloc
imbriqué de 200 lignes. `let … else` existe : l'utiliser.

**1.7 — Les gestionnaires d'événements ne contiennent aucune logique.** Ils traduisent un
événement en **intention**, et l'intention est traitée ailleurs :

```
WindowEvent  →  Input (normalisé, sans winit)  →  Intent  →  Command  →  Document
                                                      ↓
                                                   État d'UI
```

C'est la règle qui empêche à elle seule le retour de R-19.

---

### § 2 — Le modèle

**2.1 — Composition avant énumération.** Un champ présent dans plusieurs variantes remonte dans
le tronc commun. Corollaire : si tu écris le même nom de champ deux fois dans un `enum`, arrête-toi.

**2.2 — Aucun accesseur qui n'existe que pour traverser un `match`.** `fn x(&self) -> f64` qui
fait un `match` à 4 bras est le symptôme d'un modèle mal découpé.

**2.3 — Une seule convention de coordonnées** (loi L7) : origine haut-gauche, taille positive,
unités monde, `f64`. **Aucune exception.** Si un rendu a besoin du centre, il le calcule ;
le modèle ne le stocke jamais.

**2.4 — Les identifiants sont typés.** `NodeId`, `BoardId`, `AssetId` — jamais `String`.
Passer un `BoardId` où on attend un `NodeId` doit être une **erreur de compilation**.

**2.5 — Aucun id dérivé d'un autre id.** `format!("{}-dup", id)` est interdit (R-13). Les ids
viennent d'un générateur monotone, un point c'est tout.

**2.6 — Loi L9 : aucun champ sans consommateur.** Un champ arrive avec :
1. son rendu, 2. son édition, 3. sa sérialisation.
**Test automatique** : une liste des champs du modèle, comparée à une liste des champs lus par le
renderer et par le sérialiseur. Écart = build cassée.

**2.7 — Les invariants sont écrits en commentaire, au-dessus du champ.** Le code actuel le fait
déjà par endroits, et c'est excellent :

```rust
/// MEMB-1 — membrane propriétaire. STOCKÉE, jamais redérivée de la géométrie :
/// une membrane minimisée est plus petite que son contenu à l'échelle 1, donc un
/// test d'inclusion la lui ferait perdre au moment même où elle le réduit.
/// L'appartenance est un ÉVÉNEMENT (dépôt, conversion), pas un prédicat.
```

Ce commentaire vaut dix pages de documentation externe : il explique **pourquoi**, et il empêche
un futur toi de « simplifier » en cassant tout. **Il en faut plus, pas moins.**

---

### § 3 — Les mutations

**3.1 — Un seul chemin d'écriture.** Toute mutation du document passe par une `Command`.
`&mut Document` n'est jamais exposé hors du moteur de commandes.

**3.2 — Une commande sait s'inverser.** Sinon elle n'est pas annulable, donc elle n'existe pas.

**3.3 — Une commande est atomique.** Elle réussit entièrement ou ne s'applique pas.
Pas d'état intermédiaire observable.

**3.4 — Une commande maintient les index.** Index spatial, table id → handle, teintes
mémorisées : mis à jour **dans** la commande. Un index ne peut pas être périmé.

**3.5 — La navigation n'est pas une commande.** Pan, zoom, board actif, entrée dans un dossier
ne polluent jamais l'undo. *(Invariant déjà présent, à préserver.)*

**3.6 — Un geste = une entrée d'undo.** Un drag de 400 événements, c'est une entrée.
20 caractères tapés d'affilée, c'est une entrée.

---

### § 4 — Le rendu

**4.1 — Loi L1 : rien ne se dessine sans requête spatiale.** Toute passe de dessin commence par
une requête de visibilité. Une boucle `for x in &board.images` sans filtre est un bug.

**4.2 — Loi L2 : le coût d'une frame ne dépend pas de la taille du document.**
Toute violation est de gravité bloquante, pas « à optimiser plus tard ».

**4.3 — Aucune allocation dans la boucle de rendu.** Les tampons (`PathBuilder`, listes de
sommets, listes de widgets) sont réutilisés d'une frame à l'autre. Le HUD affiche les allocations
par frame ; l'objectif est **zéro**.

**4.4 — Aucun calcul de données dans le renderer.** Le renderer **lit**. La teinte symbiotique,
la mise en page du texte, les cibles d'alignement sont calculées ailleurs et mises en cache.
R-03 est né de la violation de cette règle.

**4.5 — Aucun accès disque dans le renderer.** `get_or_load_image` qui ouvre un fichier depuis
`draw_images` est un défaut de conception (R-29).

**4.6 — Toutes les couleurs viennent du `Theme`.** `Color::from_rgba8(…)` littéral est interdit
hors de la définition du thème.

---

**4.4 — Une seule transformation, jamais une borne par valeur.**
Tout ce qui appartient au monde subit **une seule** transformation d'échelle, ensemble : cadre,
police, marges, rayons, badges. Ce qui doit garder une taille constante **à l'écran** — guides,
poignées, traits d'un pixel — est divisé par l'échelle (`1 / scale`).

**Aucune valeur dérivée du zoom n'est bornée individuellement.**

```rust
// INTERDIT — la boîte se met à l'échelle librement, son contenu non.
let sw        = (width * vp.scale) as f32;
let font_size = (14.0 * vp.scale).clamp(8.0, 24.0) as f32;
let pad_x     = (18.0 * vp.scale as f32).clamp(4.0, 24.0);
```

Le raisonnement tient tout seul. Une boîte et son contenu forment **une seule** mise en page :
borner l'un sans borner l'autre, c'est décider que la mise en page se déforme. Et comme chaque
borne se déclenche à son propre seuil, N bornes cassent la mise en page à N niveaux de zoom
différents, chacun avec son symptôme — ici le texte débordait à 0,25, disparaissait à 0,50, et
n'occupait plus qu'un quart de la carte à 4,00.

**Ce n'est pas une opinion, c'est mesuré.** Rapport encre/boîte de la même carte, obtenu en la
rendant deux fois — avec et sans son texte — puis en différenciant les images :

| zoom | avec les bornes | avec une seule transformation |
|---:|---:|---:|
| 0,25 | 1,031 *(débordement)* | 0,615 |
| 0,50 | *aucune encre* | 0,608 |
| 1,00 | 0,596 | 0,600 |
| 2,00 | 0,490 | 0,596 |
| 4,00 | 0,245 | 0,593 |

3,5 % d'écart sur un facteur de zoom de 16, contre un rapport qui variait du simple au quadruple.
Le test est permanent : `renderer/card/proof.rs`.

Corollaire d'implémentation, et c'est lui la vraie cause du défaut : **la mise en page se calcule
avant la mise à l'échelle.** Dans l'autre ordre, la hauteur nécessaire dépend d'une police déjà
bornée, et le défaut revient par la fenêtre.

Si une borne de lisibilité est souhaitée, elle s'applique **à la transformation entière**, en un
seul endroit, et elle porte un nom. C'est alors un niveau de détail assumé, pas un accident.

### § 5 — L'interface

**5.1 — Loi L4 : layout une seule fois.** Une fonction produit `Vec<Widget { id, rect, visual }>`.
Dessin et hit-test lisent cette liste. Il devient impossible qu'un bouton ne clique pas là où il
est dessiné.

**5.2 — Aucune constante de mise en page recopiée.** Espacements, hauteurs et rayons viennent du
thème. `cur_x += 32.0` en dur est interdit.

**5.3 — Tout est en unités logiques.** La conversion en pixels physiques se fait au dernier
moment, une seule fois (R-16).

**5.4 — Un bouton dont l'action n'existe pas est absent ou grisé.** Jamais un toast qui simule
(R-33). C'est une règle d'honnêteté envers toi-même autant qu'envers l'utilisateur.

**5.5 — Chaque widget interactif a une infobulle et un raccourci documenté.**

---

### § 6 — Les erreurs

**6.1 — `let _ =` est interdit hors tests.** Lint CI.

**6.2 — `unwrap()` / `expect()` interdits hors tests et hors invariants prouvés.**
Quand un `expect()` est justifié, son message explique **pourquoi c'est impossible**, pas ce qui
s'est passé :

```rust
// ✗  .expect("Failed to load regular font")
// ✓  .expect("police intégrée par include_bytes! — corruption du binaire si ceci échoue")
```

**6.3 — Un type d'erreur par crate**, convertible vers celui du niveau supérieur.

**6.4 — Toute erreur atteignant l'utilisateur passe par la barre de statut.** Jamais un
`eprintln!`, jamais un échec silencieux.

**6.5 — Une erreur dit quoi faire.**
*« Image illisible : `C:\photos\x.jpg` — fichier tronqué ou format non pris en charge »*,
pas *« erreur de décodage »*.

---

### § 7 — Les tests

**7.1 — La logique métier est testée sans écran.** C'est le but du découpage en crates :
`glucose-model` se teste en `cargo test`, sans fenêtre.

**7.2 — Un test par invariant nommé.** MEMB-1, UNDO-1, PICK-1, SNAP-1 : chaque invariant du code
a un test qui porte son nom. *(Le dépôt actuel le fait bien : `undo_redo_suite.rs` fait 707 lignes
et couvre de vrais cas limites. C'est un modèle à suivre.)*

**7.3 — Un test de non-régression visuelle par phase.** Une scène de référence rendue en PNG,
comparée octet à octet. C'est ce qui aurait attrapé R-06 immédiatement.

**7.4 — Un banc de performance par loi.** L2 a un banc qui casse la build si le seuil est dépassé.

**7.5 — Un test de round-trip pour la persistance.** Sauvegarder, charger, comparer.
Y compris avec interruption brutale.

**7.6 — Aucun module n'est mergé sans son appelant.** Un test d'intégration prouve que le module
est **atteignable depuis l'application**. C'est la règle qui empêche R-18 de revenir, et c'est la
plus importante du document.

---

### § 8 — Les dépendances

**8.1 — Maximum 2 dépendances directes** hors FFI système (voir `02-ARCHITECTURE-CIBLE.md` § 0.2).

**8.2 — Toute dépendance exige une note de décision** dans `docs/architecture/decisions/`,
répondant à : que fait-elle ? combien de lignes pour la remplacer ? que tire-t-elle
transitivement ? comment on s'en défait ?

**8.3 — Toute dépendance est derrière un trait.** `glucose-codec` expose `decode_image` ;
personne n'importe le crate sous-jacent directement. On doit pouvoir le remplacer sans toucher
un appelant.

**8.4 — `unsafe` uniquement dans `glucose-platform`.** Vérifié en CI par `#![forbid(unsafe_code)]`
dans tous les autres crates.

---

### § 9 — Le dépôt

**9.1 — Loi L10 : le message de commit décrit ce que fait le code.**
Si le commit dit « suppression de X », `grep X` doit ne rien retourner (R-32).

**9.2 — Un commit = un changement cohérent.** Pas de commit « re-création intégrale » de
3 000 lignes : il devient impossible à relire, à bisecter, à annuler.

**9.3 — Le `README` décrit l'état réel**, pas l'état souhaité. Il porte la parité du moment et
un lien vers `03-PARITE-FONCTIONNELLE.md`.

**9.4 — La roadmap est mise à jour à chaque fin de phase**, avec les vrais chiffres mesurés.

---

## La checklist avant de merger

```
STRUCTURE
[ ] Aucune fonction > 60 lignes, aucun fichier > 500 lignes
[ ] Imbrication ≤ 4 niveaux
[ ] Aucune logique dans un gestionnaire d'événement

MODÈLE
[ ] Aucun champ recopié entre variantes
[ ] Aucun id dérivé d'un autre id
[ ] Tout nouveau champ a son rendu + son édition + sa sérialisation
[ ] Les invariants non évidents sont commentés avec leur POURQUOI

MUTATIONS
[ ] Toute mutation passe par une Command inversible
[ ] Les index sont mis à jour dans la commande
[ ] Un geste utilisateur = une entrée d'undo

RENDU
[ ] Toute passe de dessin commence par une requête spatiale
[ ] Aucune allocation, aucun calcul, aucun accès disque dans la frame
[ ] Aucune couleur littérale hors du thème

INTERFACE
[ ] Dessin et hit-test lisent la MÊME liste de widgets
[ ] Aucune constante de layout recopiée
[ ] Aucun bouton qui simule une action inexistante

ERREURS
[ ] Aucun let _ = , aucun unwrap() hors test
[ ] Toute erreur utilisateur atteint la barre de statut, et dit quoi faire

TESTS
[ ] Un test par invariant nommé
[ ] Un test d'intégration prouve que le code est ATTEIGNABLE depuis l'app
[ ] Les bancs de performance passent

DÉPÔT
[ ] Le message de commit est vérifiable par grep
[ ] Aucune dépendance ajoutée sans note de décision
```

---

## Une note sur la beauté du code

La beauté d'un code Rust ne vient pas du formatage — `cargo fmt` s'en charge. Elle vient de
**trois qualités**, dans cet ordre :

1. **On peut le lire de haut en bas sans sauter.** Une fonction raconte une histoire, avec un
   début, un milieu et une fin. `window_event` et ses 490 lignes n'est pas une histoire, c'est
   un annuaire.

2. **Les types disent la vérité.** Si une fonction prend un `&Document` et retourne une
   `Command`, on sait sans lire qu'elle ne modifie rien. Si elle prend `&mut GlucoseApp`, on ne
   sait rien du tout. C'est pour ça que les handles typés (§ 2.4) et la séparation des crates
   comptent plus que n'importe quelle convention de nommage.

3. **On peut supprimer une partie sans tout casser.** C'est le test ultime. Aujourd'hui,
   supprimer la minimap demande de toucher `ui.rs`, `renderer.rs` et `app.rs`. Après le
   découpage, c'est retirer une ligne de la description de l'UI.

Le code actuel échoue sur les trois — non par manque de soin, mais parce que **l'architecture ne
permet pas de faire mieux**. C'est pour ça que la réponse n'est pas « refactoriser », c'est
« restructurer ». Et c'est aussi pourquoi ça vaut la peine : une fois la structure juste, écrire
du code beau devient le chemin le plus facile, pas un effort supplémentaire.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
