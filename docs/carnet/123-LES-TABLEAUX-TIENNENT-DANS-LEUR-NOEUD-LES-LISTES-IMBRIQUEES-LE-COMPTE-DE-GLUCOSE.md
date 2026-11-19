# 123 — Les tableaux tiennent dans leur nœud, les listes s'imbriquent, le témoin dit l'onglet ; un compte Cloudflare à Glucose

> Session du 19/11/2026, depuis 12 h 04, après la fiche 122 (son travail resté dans l'arbre, non
> validé). Sa parole d'ouverture, avec une capture de son document « ce qui attend ma parole » :
> *« moi perso, j'ai juste des problèmes au niveau des MCP et de leur réaction face aux outils de
> Glucose : le texte dépasse d'un simple bloc texte »* ; *« hésite pas à accéder toi-même au Glucose
> pour voir tout ça »* ; puis, sur l'adresse des serveurs : *« tu as utilisé fermenislamber […] c'est
> un autre projet, qui n'a rien à voir ; il faudrait vraiment switch ça afin de ne pas se faire de
> croche-pied avec l'autre projet, afin que tout soit clean »* ; et *« regarde aussi les logs des
> utilisateurs afin de repérer les problèmes, l'optimisation, les plantages »*.

## 1. Ce que son document montrait

Lu par sa porte (`glucose_lire`, les deux onglets). Claude.ai avait posé, le 19/11 vers 11 h 45,
**un onglet neuf**, « Textes optimisés » : trois membranes et trois très grands textes (900 à 1 000
de large, jusqu'à 5 089 de haut), ses trois pages restructurées — 33 lignes de tableau, 30 titres,
8 blocs de code (des schémas en caractères), 4 lignes de **listes imbriquées**, des émojis de
couleur comme signaux (🟢 🔴 🟠 🔵). Sur sa capture :

* **les tableaux sortaient de leur nœud** — la colonne de droite passait sur la membrane voisine ;
* **les listes imbriquées** (`   - ⬅️ Gauche` sous `1. Côté du bouton`) étaient dessinées en
  paragraphe, **tiret compris** ;
* les émojis, en noir et blanc : 🟢 et 🔴 se ressemblent (fiche 122 § 6).

Rien de neuf de sa part dans l'onglet principal depuis 11 h 59. Son *« mon Glucose n'a juste pas
changé du tout »* de la fiche 122 s'explique : l'IA travaillait dans un onglet qu'il ne regardait
pas, et le témoin disait « travaille dans ton canevas ».

## 2. Remis en question

* **TABLE-1 : « un tableau ne se reflue pas […] c'est ce que fait un tableau »** — faux. Un
  navigateur coupe les cellules en lignes ; Glucose Tauri, un navigateur, le faisait. La règle
  laissait déborder exprès.
* **« La carte se redimensionne »** (le même commentaire) : la hauteur, oui — jamais la largeur,
  qu'une IA fixe en posant le nœud.
* **L'en-tête d'un tableau était mesuré en romain et dessiné en gras** : plus étroite que lui, sa
  colonne laissait le gras mordre sur la gouttière.
* **Le piège de la passation, payé quand même** : mon arrêt de l'attente de clippy cherchait les
  processus par un motif présent dans sa propre ligne de commande, et s'est tué lui-même. Rien de
  cassé ; désormais, par numéro de processus seulement.

## 3. TABLE-3 — un tableau tient dans la largeur de son nœud

`renderer/richtext/table.rs`. La largeur se répartit comme dans *CSS Tables 3* (§ 3.9.3) : chaque
colonne a une largeur **minimale** (son mot le plus large, entre deux espaces — là où la coupe ne
coupe pas) et une **naturelle** (sa plus longue cellule sur une ligne). La place suffit : les
naturelles. Sinon, *« la combinaison linéaire (de poids qui somment à 1) des deux répartitions
voisines »* qui remplit exactement la place. **En deçà des minimales** — la CSS laisse déborder —,
les minimales réduites en proportion, et les mots se coupent entre deux lettres comme partout
(`wrap_paragraph`) : la fonction reste continue, et **un texte ne sort jamais de son nœud**.
Aucun nombre à choisir.

Chaque cellule se coupe dans sa colonne ; une rangée a la hauteur de sa plus haute cellule, et
chacune de ses lignes visuelles porte la ligne de même rang de chaque cellule, sur son taquet.

**En édition**, une rangée est sa source, barres comprises en signes gris, coupée comme un
paragraphe (MODE-1 : les signes reviennent quand on corrige) — le curseur, qui suit des lignes
sans trou, y marche comme partout ; la séparation reste un filet, comme un `---`.

## 4. LISTE-1 — les listes imbriquées

`glucose-core/src/text/block.rs`. `Bullet` et `Ordered` portent une **profondeur**. La règle de
CommonMark (*« indenting subsequent lines of Ls by W + N spaces »* : la largeur de la marque plus
ses espaces) : un élément est l'enfant du précédent quand sa marque commence au moins à la colonne
où commence le texte de celui-ci ; sinon, le frère du premier ancêtre dont il atteint le texte. Une
ligne vide, ou décalée, garde la liste ouverte ; une ligne au bord la ferme. Les espaces de tête
sont des signes, comme la marque. Le texte se décale d'un retrait par niveau (`indent_of`) ; la
puce et le numéro, d'un retrait de moins (`retrait_d_ornement`, déduit de `indent_of`), sur les
trois dessins — en pixels, de loin, en vecteurs.

## 5. TEMOIN-2 — le témoin dit l'onglet

`app/porte_publique.rs` : une IA du nuage qui travaille dans un autre onglet que celui qu'il
regarde le dit — « Claude (claude.ai) travaille dans l'onglet « Textes optimisés » » —, et le
redit quand elle change d'onglet.

## 6. Ce que la télémétrie dit (19/11, 12 h 10)

**205 sessions, 10 installations, 40 fins anormales** — aucune neuve depuis le 17/11.

* **Son téléphone** : aucune fin anormale depuis la 2.0.4-beta.1 (6 sessions, finies en arrière-plan,
  ce qu'Android fait). Prometteur pour MEMOIRE-4 ; pas encore une preuve.
* **Un Windows à AMD Radeon 660M** (un utilisateur, en beta.1 puis beta.2) : une image de
  **415,7 s** en « éditer du texte ». Les lignes de charge, toutes les 5 s, s'arrêtent sept minutes,
  et la première d'après dit la machine entière à **0 %** sur l'intervalle : **très probablement
  une mise en veille**, pas un gel. La télémétrie ne sait pas les distinguer — la « pire image » en
  est faussée.
* Son PC de l'école (GTX 1660 SUPER) en beta.2 : rien d'anormal (pire image 1,1 s au repos).

## 7. Un compte Cloudflare à Glucose — proposé, à son geste

`ferme-nilslamber` est **le sous-domaine `workers.dev` de tout son compte**, posé en juillet par un
autre projet (`ferme-tiles`) ; la télémétrie (depuis la fiche 55) et la porte (fiche 116) s'y sont
ajoutées. Le renommer casserait l'autre projet et la télémétrie de chaque version installée.
Cloudflare permet, depuis le 4/08/2026, de créer jusqu'à cinq comptes gratuits sous le même
identifiant (*« select Create Account from either the account switcher in the top left »*). Le
plan : son clic ; les deux serveurs sur le compte neuf ; l'historique recopié ; l'ancien serveur
de télémétrie qui transmet au nouveau, pour les versions déjà installées ; les adresses neuves dans
la version suivante ; ses connecteurs (Claude.ai, Mistral) réajoutés avec l'adresse neuve.

## 8. La recherche (consultée le 19/11/2026)

* [CSS Tables 3, § 3.9.3](https://drafts.csswg.org/css-tables-3/#width-distribution-algorithm) : les
  répartitions minimale et naturelle, la combinaison linéaire entre deux répartitions voisines.
* [CommonMark 0.31.2, les éléments de liste](https://spec.commonmark.org/0.31.2/#list-items) : W + N.
* Cloudflare, [Create Free accounts from the dashboard](https://developers.cloudflare.com/changelog/post/2026-08-04-free-dashboard-button/)
  (4/08/2026 : cinq comptes gratuits, sept jours d'ancienneté).
* Les émojis en couleur, pour la suite : [COLR](https://learn.microsoft.com/typography/opentype/spec/colr)
  (v0 : des couches, chacune une forme et une couleur de [CPAL](https://learn.microsoft.com/typography/opentype/spec/cpal) ;
  v1 : dégradés et compositions) ; [Twemoji Mozilla](https://github.com/mozilla/twemoji-colr) (COLRv0,
  1,4 Mo, dessins sous CC-BY 4.0, code sous Apache 2.0) ; Noto Color Emoji (OFL 1.1), aujourd'hui en
  COLRv1 ; [Emoji-COLRv0](https://github.com/Emoji-COLRv0/Emoji-COLRv0) (Twemoji, OpenMoji, EmojiTwo
  convertis).

## 9. Sa machine pleine

À 12 h 15, la mémoire ne laissait que 2 Go (Brave 2,7 Go, VS Code 1,2, une application Electron 1,
trois sessions de Claude 0,8) : sous la borne, aucune compilation ne part — il en faut 3,6 Go.

## 10. Où la session s'est arrêtée (12 h 40, à sa demande)

* **Rien n'est compilé ni éprouvé** : ni le travail de la fiche 122, ni celui-ci. Une épreuve du
  noyau, bornée à 1 Go, compilait encore après onze minutes : arrêtée (un `rustc` tué — si besoin,
  `cargo clean -p glucose-core`).
* **Écrits et prêts** : les épreuves (`renderer/richtext/table/tests.rs` — dont ses deux vrais
  tableaux dans leur nœud de 900, et la preuve à l'envers ; `text/block/tests.rs` ;
  `renderer/richtext/tests.rs` ; `app/porte_publique/tests.rs`) et les sabotages
  (`table-123.json` 10, `liste-123.json` 8, `temoin-123.json` 2, `tiroirs-122.json` 6), dont les
  motifs ont été vérifiés après `cargo fmt`.
* **Pas fait** : le compte Cloudflare (son geste d'abord), rien regardé au pilote, ÉTAT et SUITE
  toujours en retard (ils parlent du 18/11), l'index du carnet sans les fiches 121 à 123.
* **Une voie si sa mémoire reste pleine** : la vérification de GitHub se lance à la main sur une
  branche (`ci.yml` a `workflow_dispatch`) — sur leurs machines, rien sur la sienne ; la branche se
  retire dès la fusion.

## 11. Les leçons

* **Une règle qui dit « c'est ce que fait X » se vérifie contre X** : TABLE-1 décrivait un tableau
  qu'aucun navigateur ne fait.
* **Ce qu'une IA écrit est un banc d'essai** : trois textes de Claude.ai ont montré trois défauts
  (tableaux, listes imbriquées, émojis) qu'aucune de nos épreuves ne cherchait.
* **Une machine occupée par son propriétaire ne compile pas** : regarder `MemAvailable` avant de
  promettre une vérification, et garder la voie de GitHub en réserve.
