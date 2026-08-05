# Glucose — ce que c'est, vraiment

Ce fichier est la boussole du projet pour toute session Claude Code qui travaille ici. Pour
l'historique détaillé et les guides pratiques, voir `HANDOFF.md`, `ROADMAP.md`, `GUIDE.md` et
la mémoire persistante (`~/.claude/projects/.../memory/`, notamment
`indestructible-incorruptible-north-star.md` et `glucose-notes-plugin.md`).

## La vision (posée par l'user le 2026-08-03, à ne jamais perdre de vue)

Glucose n'est pas un outil de stockage. **Glucose est un système de mémoire en LATENT SPACE**,
pas basé sur du stockage pur. L'objectif final — une étoile polaire, pas une release proche —
est de donner à une IA un contexte **absolument infini** : une capacité à TOUT comprendre et
TOUT corréler dans une masse de contenu, et à faire des rappels pile au bon moment (des "petits
clins d'œil"), au lieu de simplement re-parcourir un grand fichier.

**Glucose sert à partager de l'information** : humain → IA, IA → humain, humain → humain, et
**surtout IA → IA**. L'ambition dépasse l'usage personnel : inventer une nouvelle manière de
contextualiser/"former" une IA dans le monde, en s'appuyant sur Glucose comme substrat de
mémoire spatiale. Toute décision de conception se juge à l'aune de cette question : *est-ce
que ça rapproche Glucose d'une mémoire dense, corrélée, consultable par une IA à contexte
infini — ou est-ce que ça produit juste un joli résumé épars ?*

Le pilier historique et le plus aimé de l'user pour tenir cette promesse est le système
git/historique de Glucose (undo, jalons durables, compaction) — voir
`indestructible-incorruptible-north-star.md`. « Mieux qu'une feuille » = indestructible et
incorruptible ; c'est la condition nécessaire, pas suffisante, de la vision ci-dessus.

## Principe d'architecture central (ne jamais le violer implicitement)

**L'IA décide le SENS, le CODE décide la GÉOMÉTRIE.** Jamais de coordonnées x,y demandées à un
modèle. Tout free-form (position, anti-collision, routage de flèches) est déterministe, côté
code. Voir `user-loves-elegant-math` : privilégier une solution géométrique/mathématique qui
fait DISPARAÎTRE une constante arbitraire plutôt qu'une qui la rétrécit ou la patche.

## Design visuel de l'app (glucose-design-monochrome)

UI = noir/blanc/gris **uniquement**. La couleur n'est JAMAIS décorative — elle porte un
statut (vert = ok, rouge = pas ok). Jamais de violet/bleu comme choix esthétique dans
l'interface de Glucose elle-même.

**Point de vigilance identifié le 2026-08-03** : le générateur `glucose-notes` (pipeline
texte→carte, voir plus bas) attribue aux MEMBRANES de thème une couleur par teinte HSL répartie
sur le cercle chromatique (`build_glucose.rs::theme_color`). Sur le premier vrai test à grande
échelle, l'user a trouvé "les couleurs se marient mal ensemble" — piste principale : ce n'est
probablement pas un bug de calcul de teinte (les hues sont bien distinctes en théorie), mais un
désaccord de principe avec le design monochrome de l'app elle-même. Réfléchir à un jeu de
couleurs de membrane plus sobre/cohérent avec l'ADN visuel de Glucose plutôt qu'un arc-en-ciel
saturé, avant de re-toucher `theme_color`.

**Jamais de LOD / semantic zoom.** L'user déteste ce concept, ne jamais l'implémenter ni le
proposer, quel que soit le prétexte performance.

## Le pipeline texte→carte (`glucose-notes`, plugin séparé)

Projet Rust dans `~/Documents/glucose-notes` (privé sur GitHub), intégré nativement dans
Glucose comme système de plugin (sidecar + manifest, voir `glucose-notes-plugin.md`). Doctrine
technique complète dans cette mémoire — invariants durables : CAPTURE fidèle par défaut
(recopie vérifiée mot-pour-mot, zéro hallucination) > SYNTHESE (reformulation libre,
réservée à un gros modèle) ; agnostique au sujet et à la langue par construction.

**Pivot architectural en cours (2026-08-03), directement issu de la vision ci-dessus :** un
premier run "focus-first" (filtrer le corpus par sujet AVANT extraction) sur un vrai corpus
massif (37 000 messages) a produit une carte beaucoup trop pauvre — 15 831 paragraphes filtrés
à 201, puis seulement 53 unités, 10 bulles : "presque rien" selon l'user, qui attendait un
arbre dense, quasi message par message, richement relié. **Diagnostic : le focus-first jette
la majorité du corpus avant même que l'extraction ait une chance de le comprendre — c'est
l'inverse de "TOUT comprendre TOUT corréler".**

**Nouvelle direction (pas encore construite, prochain grand chantier) :**
1. Construire d'abord une carte **complète et dense** sur l'intégralité du corpus — l'IA
   propose sa propre architecture complète, pas un sous-ensemble présélectionné.
2. Le filtre/la recherche par sujet (`--focus`) devient une **vue appliquée après coup** sur ce
   graphe déjà riche, pas un pré-filtre qui le détruit avant construction.
3. Deux plafonds codés en dur dans `architect.rs` doivent tomber pour supporter cette densité :
   `MAX_EDGES=28` (liens à noms propres partagés, plafond global arbitraire sur toute la carte)
   et `themes.truncate(9)` (au plus 9 bulles). Piste pour les liens : réutiliser le mécanisme
   embeddings + arbre couvrant minimal déjà bâti et testé dans `mapspace.rs` (mode carte
   sémantique) — aucune constante arbitraire, ça scale naturellement avec la taille du corpus.
4. Bug de rendu trouvé en même temps : les arêtes d'"épine" (`theme_spine_edges`, chaînage
   narratif déterministe sans IA) n'ont jamais de description — sur le test réel, 43 des 44
   flèches de la carte étaient des épines sans description, d'où un fléchage perçu illisible.
   Le placement géométrique (`layout.rs`) n'a par ailleurs aucune notion de l'ordre d'épine —
   les lignes zigzaguent car l'ordre logique du chaînage et l'ordre géométrique du placement ne
   sont pas alignés.

Coût à anticiper : un run comprehensive-first sur un corpus de dizaines de milliers de messages
est un ordre de grandeur plus cher qu'un run focus-first (des heures, pas des minutes) — à
chiffrer et confirmer avant de lancer un run complet, pas à découvrir après coup.

## Ce que ce fichier n'est pas

Pas un changelog. L'historique détaillé de CE qui a été fait vit dans `git log` et dans les
mémoires spécifiques (`glucose-notes-plugin.md` pour le pipeline, etc.) — ce fichier ne garde
que ce qui doit orienter une décision future.
