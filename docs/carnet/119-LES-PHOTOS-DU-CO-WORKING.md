# 119 — Les photos du co-working : ce que je pose part avec mon geste, ce qui manque se demande en entrant

> Session du 17/11/2026, le soir (21 h 15 – 23 h), après sa capture d'écran (fiche 118). Le premier
> chantier ouvert de SUITE § 2 : le co-working **sauf les photos** — une photo posée par un participant
> arrivait chez les autres en cadre gris, sans ses octets (B1, fiches 113 et 117 § 7 bis). Commencé en
> correctif le 17/11 au matin, repris ici, **fini et prouvé dans le vrai programme** contre un relais
> local ; le relais en ligne reste à redéployer avant la version qui s'en sert (§ 6).

## 1. Ce qui marche

* **Ce que je pose part avec mon geste** : chaque photo qu'un geste pose (`GestePartage::images_posees`)
  part juste après lui, par la salle, **sans y être gardée** (l'offre gratuite de Cloudflare tient 5 Go
  pour tous les canevas).
* **Ce qui manque se demande quand quelqu'un entre** : à chaque « à jour » de la salle — une entrée, un
  retour du réseau —, Glucose dit **bonjour**, avec les clés des photos qui lui manquent (même aucune).
  Ceux qui sont là répondent par les octets qu'ils ont ; et au bonjour d'un nouveau venu, chacun
  **redemande** ce qui lui manque, puisque le nouveau venu l'a peut-être. Une relance n'appelle pas de
  relance.
* **Qui l'a la renvoie, s'il écrit** : la salle refuse les octets d'un lecteur, qui sinon glisserait une
  autre image à la place d'une photo manquante. Une demande vient de n'importe qui.
* **À l'arrivée**, les octets entrent au registre (`Source::Memoire`), le scribe les scelle dans le
  document de celui qui les reçoit, et le magasin oublie l'échec de la photo.

Essai réel (`outils/porte/essai/essai_photos_pilote.sh`, deux vrais Glucose hors écran contre `wrangler
dev`) : Alice colle un damier **puis** partage ; Bob rejoint et **voit le damier** (son bonjour l'a
demandé). Alice colle des bandes vertes et jaunes, Bob déjà là : **Bob les voit**, comme Alice, sans rien
demander. Chaque capture regardée.

## 2. Ce qui a été remis en question (la conception du matin, fiche 117 § 7 bis)

* **La minuterie de 30 s a disparu.** Redemander toutes les 30 s exigeait une raison de réveil de plus
  (le masque des seize est plein) et réveillait la boucle pour rien des heures durant si personne n'a la
  photo. Or **seule une entrée peut apporter des octets qui n'étaient pas là** : on demande au bonjour,
  on redemande au bonjour d'un autre. La constante n'a pas rétréci, elle est partie, et le réveil avec
  elle. C'est l'échange « j'ai / il me manque » d'Automerge à chaque connexion, à l'échelle d'une clé de
  photo.
* **On ne demande pas ce qu'un geste en direct vient de poser** : son auteur l'envoie juste après. Le
  demander aussi l'aurait fait passer deux fois.
* **Les octets ne se lisent plus sur la boucle** : un *facteur*, fil à part, lit (une tranche sur le
  disque, quelques mégaoctets) et **attend les octets promis** — une photo collée part avec son geste
  avant d'être encodée en PNG ; le matin, elle ne partait jamais (`Objets::lire` d'une promesse pas
  encore tenue rend `None`).
* **Le facteur ne lit que le registre** (faille trouvée en écrivant, jamais en ligne) : `Objets::lire`
  d'une clé inconnue lit **le fichier de ce nom** — n'importe qui aurait demandé `~/.ssh/id_ed25519` et
  l'aurait reçu de chaque participant. Les clés d'un document vivent toutes au registre ; un chemin nu ne
  se sert plus.
* **Leur propre file, d'une place** (`lien::Passe`, `sync_channel(1)`) : les photos passaient dans la
  file des gestes ; un lot de deux cents photos aurait fait attendre chaque geste d'Alice derrière lui
  des minutes, et tenu toutes ses photos en mémoire à la fois. Le fil fait partir à chaque tour les
  lettres et le curseur, **puis une seule chose qui passe** ; le facteur attend que la précédente soit
  partie : trois photos en mémoire au plus.
* **`passe` est devenu `photo` et `demande`** en clair dans le message, pour que la salle — qui ne lit
  rien de ce qu'elle passe — puisse refuser les octets d'un lecteur.

## 3. La course que seul l'essai réel a montrée

Toutes les épreuves passaient ; dans le vrai programme, la photo collée **restait grise chez Bob**. Pas
de supposition : des traces provisoires (le facteur, l'arrivée) ont montré que **les octets arrivaient**
(3 306 octets, au registre). La cause : **le geste arrive avant ses octets** — l'atelier de Bob cherche
aussitôt la photo, ne trouve rien, et l'échec qu'il rend **après** l'arrivée des octets la condamnait
pour toujours (le cache négatif R-29 ne retente jamais). `Magasin::octets_arrives` : la photo n'est plus
un échec, et si un décodage est en chantier, l'échec qu'il rendra ne compte pas. La base marchait par
chance : sa réponse arrivait avant le premier décodage.

## 4. Les preuves

* **Épreuves** (`cargo test -p glucose-desktop --lib -- b1_`, onze) : le facteur attend une promesse ;
  il ne lit ni un fichier ni une promesse abandonnée ; le bonjour à l'« à jour », une fois, même sans
  manque ; la relance au bonjour d'un autre, jamais à une relance ; qui attend la base garde son
  bonjour ; un écrivain répond, un lecteur non ; ma photo part avec mon geste ; les octets arrivés
  entrent au registre ; un échec d'avant les octets ne compte pas ; les lettres passent devant les
  photos ; ce qui passe avant l'« à jour » se perd sans bloquer. Le noyau : la lettre se relit, bonjour
  compris.
* **Le relais** : `node --test outils/porte/test/*.test.js` (7/7 — `photo` et `demande` lus, `passe`
  refusé) ; `node outils/porte/essai/salle_essai.mjs` contre `wrangler dev` : **12 OUI**, dont « la
  photo d'Alice passe à Bob, la demande de Bob à Alice — ni gardées ni numérotées » et « Bob, lecteur, ne
  fait pas passer d'octets » — **saboté à la main** (le refus retiré) : NON, puis remis : 12 OUI.
* **Sabotages** : `outils/sabotages/photos-119.json`, quatorze gardes cassées une à une — voir § 7.
* **Toute la suite** : 3 338 épreuves, cliquets compris, aucune ne tombe ; clippy sans avertissement.

## 5. Ce qui reste, et ses limites assumées

* **Redéployer le relais** (`cd outils/porte && npx wrangler deploy`) **avant toute version qui
  s'en sert**, puis l'essai en ligne (`salle_essai.mjs <racine en ligne>` et l'essai des photos avec la
  racine en ligne). Un Glucose plus ancien ignore `photo` et `demande` (son fil ignore un `t` inconnu) :
  le déployer d'abord ne casse rien.
* **Son essai à son écran** : deux Glucose dans le même canevas, une photo collée chez l'un.
* Une photo dont l'auteur est parti avant de l'envoyer attend qu'il revienne (son bonjour fera
  redemander) ; tant qu'elle manque, elle reste un cadre gris — rien ne dit encore « en route ».
* Les octets reçus ne se vérifient pas contre une empreinte : un **écrivain** pourrait envoyer d'autres
  octets pour une photo qui manque — il peut déjà remplacer l'image elle-même. Un lecteur, non (la salle).
* Une photo de 20 Mo passe en vingt morceaux d'1 Mio, sans barre de progression.

## 6. Ce qui change dans le code

`crates/glucose-core/src/salle/lettre.rs` (`Corps::Objet`, `Corps::Demande { cles, bonjour }`) ;
`crates/glucose-desktop/src/app/collaborer/photos.rs` (le facteur, le bonjour, la relance, la réponse,
l'arrivée) et `photos/tests.rs` ; `collaborer.rs` (`CanevasPartage::neuf`, le bonjour à l'« à jour », le
facteur dans `faire_partir`) ; `salle/lien.rs` (`Passe`, sa file, `un_tour`, `relier`) et `lien/tests.rs` ;
`renderer/magasin.rs` (`octets_arrives`) et `atelier.rs` (`en_cours`) ; `outils/porte/src/messages.js`,
`salle.js` ; `outils/pilote/piloter.sh presse-image` (une image au presse-papiers du compositeur hors
écran) ; `outils/porte/essai/essai_photos_pilote.sh`.

## 7. Les sabotages

`outils/sabotages/photos-119.json` : **14 sur 14 tombent** — le facteur qui lit un fichier qu'une clé
nommerait, qui n'attend pas une promesse ; l'« à jour » sans bonjour ; toute demande qui fait relancer ;
un lecteur qui répond ; le bonjour avant la base ; celui qui a ouvert le partage sans bonjour ; un
bonjour sans manque tu ; ma photo qui ne part pas avec mon geste ; l'échec d'avant les octets qui
condamne ; ce qui passe avant les lettres, avant l'« à jour » ; une demande dite « photo » ; le bonjour
absent de la lettre. Et la salle, à la main (§ 4).

## 8. La recherche (consultée le 17/11/2026)

* Excalidraw : la scène passe par un relais chiffré de bout en bout, **les fichiers (images) sont
  stockés à part** (Firebase Storage) et chargés à la demande — [son blog sur le chiffrement de bout en
  bout](https://plus.excalidraw.com/blog/end-to-end-encryption), [la collaboration
  P2P](https://plus.excalidraw.com/blog/building-excalidraw-p2p-collaboration-feature). Glucose ne garde
  rien (5 Go pour tous) : les octets passent de pair à pair par la salle.
* Automerge, [le protocole de synchronisation](https://cran.r-project.org/web/packages/automerge/vignettes/sync-protocol.html)
  et [`automerge::sync`](https://docs.rs/automerge/latest/automerge/sync/index.html) : à chaque connexion,
  chaque pair dit ce qu'il a et ce qu'il lui manque (`have`, `need`), et l'on échange jusqu'à ce que
  personne n'ait plus rien à envoyer — le bonjour de Glucose, à l'échelle d'une photo.

## 9. Les leçons

* **Toutes les épreuves vertes, la photo grise à l'écran** : l'ordre réel des arrivées (le geste, puis
  ses octets, pendant un décodage) n'existait dans aucune épreuve. L'essai réel n'est pas une formalité.
* **Tracer avant de supposer** : trois hypothèses plausibles (la promesse jamais tenue, le relais qui
  refuse, la clé renommée au scellement) étaient fausses ; deux lignes de trace ont tranché en une
  compilation.
* **Douter après le succès** : les captures étaient bonnes, les treize sabotages tombaient — et
  **celui qui ouvre le partage ne disait jamais bonjour** : il reste « Partage », jamais « Dedans », et
  le bonjour n'attendait que « Dedans ». Au retour de son réseau, il n'aurait rien demandé de ce qu'on
  avait posé pendant son absence. Trouvé en relisant l'énumération des entrées ; la garde dit
  maintenant la seule attente légitime — celle de la base —, et son sabotage remet l'ancien défaut.
* **Une file partagée est une priorité implicite** : mettre les photos dans la file des gestes, c'était
  décider sans le dire qu'un geste attend derrière un lot.
