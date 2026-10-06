# 27 — L'image et sa provenance : la meilleure qualité, l'origine, l'auteur

> **Rôle de ce document.** L'utilisateur a posé, le 23/09, ce qu'il appelle *« l'objectif ultime
> et final de Glucose »*. Cette fiche dit ce qui en est fait, ce qui est faisable, ce qui ne
> l'est qu'en partie — avec les services qui existent réellement en 2026 et leurs limites —, et
> l'ordre dans lequel le construire. Elle clôt aussi la session du 23/09 après-midi (§ 1).
>
> **Date** : 2026-09-23 · commits `651086d` à `60893c2`.
> **État vérifié** : `cargo test --workspace` exit 0, **1 489 tests verts**, clippy strict à
> zéro, douze cliquets, aucun plafond relevé. Poussé sur `main`.

---

## 1. Ce que la fin de session a livré

| Chantier | Ce que c'est | Vérifié par |
|---|---|---|
| **ARBITRE-4, sur le terrain** | La session suivante sur l'Intel : l'arbitre a conclu dès la fin de l'échauffement, **écrit `rapide`** dans `carte.txt`, et la session n'a **pas figé** | La sortie de l'utilisateur, et le fichier lu sur son disque |
| **BORDURES-3** (`glucose_core::bordures`) | `Ctrl+B` emporte les **rangs de transition** qui mêlaient la bande à l'image — le liseré clair — et la couleur d'une bande se **fige** : l'ancienne boucle grignotait les dégradés, et mangeait du contenu | 3 tests bâtis sur les pixels **mesurés sur sa capture**, deux preuves à l'envers ; **34 images réelles** comparées ancien/nouveau, cadres tracés et regardés |
| **DEPOT-WEB-4** (`plateforme/sources`, `rapatrier`, `telechargement_windows`) | Une épingle Pinterest se pose **en image, dans sa pleine résolution d'origine** : Glucose télécharge ce qu'on dépose, par WinHTTP, sans caisse de plus | 9 tests purs ; **quatre épingles réelles** rapatriées sur la machine, de 512 × 512 à 2048 × 2048 |

### 1.1 Ce que la mesure a démenti

1. **Le liseré n'était pas le cadre de sélection** (fiche 25 § 9.5) : la capture montrait une
   image **non sélectionnée**, et ses bords, mesurés, portaient `243,239,218` puis
   `147,140,103` — deux rangs de fondu entre la bande blanche et l'image.
2. **Le détecteur mangeait du contenu depuis BORDURES-1**, et personne ne le savait : sur
   l'illustration au fond blanc, 37 colonnes et **la manche du kimono qui touche le bord** ;
   sur le mur de béton, 35 rangs. La boucle reprenait comme couleur de bande le rang où elle
   s'était arrêtée. Trouvé parce que le test du fondu est tombé **d'une façon que je
   n'attendais pas** — 21 rangs au lieu de 12.
3. **Le premier téléchargement réel a rapporté une icône.** Une page d'épingle porte des
   dizaines d'images ; seule sa balise `og:image` désigne la sienne. Le journal disait
   « rapatriée » : c'est en ouvrant l'image qu'on l'a vu.

---

## 2. La vision, dans les mots de l'utilisateur

> *« Avoir une recherche […] en étant sûr à 99 % qu'on a exactement la même image, mais dans sa
> qualité optimale ; avoir l'origine, la toute première fois qu'elle a été publiée sur
> Internet ; pouvoir retrouver aussi l'auteur de cette image ; puis […] quand on clique sur une
> image, avoir toutes les références de l'auteur qui a posté cette image, avec le classement
> de tous ses réseaux dans l'ordre où il est le plus souvent actif. »*
>
> *« Qualité optimum et origine, ça ne veut pas dire la même chose, mais il faut avoir les
> deux. »*

Trois demandes distinctes, et elles ne se construisent pas pareil :

| | question | difficulté |
|---|---|---|
| **A** | la même image, dans sa **meilleure qualité** | faisable, et faite pour Pinterest |
| **B** | son **origine** et son **auteur** | faisable pour l'illustration, partiel pour la photo |
| **C** | **tous les réseaux** de l'auteur, classés par activité | partiel : certains réseaux ferment leurs données |

---

## 3. Ce qui existe réellement, en 2026

| service | ce qu'il sait | accès | limite |
|---|---|---|---|
| **SauceNAO** | l'illustration : pixiv, DeviantArt, ArtStation, Danbooru, Nijie… — **l'auteur**, le lien, un **score de ressemblance** | API JSON, **clé gratuite** avec un compte | quelques centaines de recherches par jour ; Twitter figé depuis 2019 |
| **Google Cloud Vision — Web Detection** | la photo comme l'illustration : **les mêmes images ailleurs sur le web** (`fullMatchingImages`), et les pages qui les portent | API, compte Google Cloud **avec facturation** | 1 000 recherches gratuites par mois, puis 3,50 $ les 1 000 |
| **TinEye** | l'image exacte, avec la date où il l'a vue | API **payante**, commerciale | — |
| **Bing Visual Search** | — | **fermée le 11 août 2025** | — |
| **Google Lens** | tout | **aucune API officielle** | — |

**Aucun service ne sait « la toute première publication ».** Chacun connaît ce qu'il a indexé,
et la vraie première publication peut être sur un réseau qu'aucun n'indexe. Ce qui se construit
honnêtement : **la plus ancienne publication connue**, datée, avec la liste de toutes les
autres.

---

## 4. Le « 99 % sûr », et c'est Glucose qui le garantit — pas le service

Un service de recherche renvoie des candidats et **un score à lui**. Ce score ne se croit pas :
il mélange « la même image » et « une image qui lui ressemble ». La garantie se construit donc
**chez nous**, par la mesure :

1. télécharger chaque candidat — le rapatriement de DEPOT-WEB-4 sait déjà le faire ;
2. le comparer à l'image du canevas par une **empreinte perceptuelle** : l'image réduite à
   32 × 32 en niveaux de gris, sa transformée en cosinus, les 64 coefficients basses fréquences
   comparés à leur médiane — 64 bits qui ne bougent pas quand l'image est recompressée,
   redimensionnée ou légèrement recadrée ;
3. ne retenir que ce dont l'empreinte diffère de **quelques bits** — et le seuil se **mesure**
   sur des paires réelles (la même image en 236, 736 et original ; deux images voisines d'un
   même auteur), il ne se choisit pas ;
4. parmi les versions retenues, la **plus grande** est la qualité optimale ; la **plus
   ancienne datée** est l'origine connue. Les deux ensemble, comme il l'a demandé.

C'est la même philosophie que tout le dépôt : on n'interroge pas, on **observe** — et deux
chemins vers le même résultat doivent rendre le même résultat.

---

## 5. L'ordre de construction

| # | Étape | Ce qui la prouve | Dépend de |
|---|---|---|---|
| 1 | **La meilleure qualité sur la plateforme d'où l'image vient** — Pinterest | quatre épingles réelles | **fait** |
| 2 | **L'empreinte perceptuelle** et son seuil, mesuré sur les images de ses documents | un banc sur des paires réelles | rien |
| 3 | **« Trouver l'origine »** au clic sur une image : SauceNAO, vérification par l'empreinte, remplacement par la meilleure version en **une entrée d'annulation** | une illustration de pixiv retrouvée en pleine taille, avec son auteur | **une clé SauceNAO** |
| 4 | **La provenance conservée** dans le document : source, auteur, date, versions — enregistrée, visible dans un panneau | la règle R1 : branché, visible, annulable, conservé | 3 |
| 5 | **Les photos** : Google Web Detection | une photo retrouvée à sa source | un compte Google Cloud |
| 6 | **Les réseaux de l'auteur** : les profils se citent entre eux (ArtStation et pixiv listent les autres comptes) ; l'activité se lit dans la date des dernières publications | un auteur réel, ses réseaux classés | 3 ; **X et Instagram ferment leurs données** — ils paraîtront sans classement |

**Le réseau ne part que sur un geste explicite** — déposer, cliquer « trouver l'origine ». Jamais
en fond. Envoyer une image à un service tiers est un choix que l'utilisateur fait image par
image, pas une habitude que Glucose prend pour lui.

---

## 6. Ce qu'il reste à trancher avec l'utilisateur

* **La clé SauceNAO** : un compte gratuit sur saucenao.com, dont la clé se colle dans Glucose.
  C'est à lui de le créer.
* **Les photos** : Google Web Detection demande un compte Google Cloud avec une carte bancaire,
  même pour les 1 000 recherches gratuites du mois. À décider quand l'étape 3 tournera.
