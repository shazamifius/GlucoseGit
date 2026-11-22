# 130 — Une lecture n'attend jamais, aucun agent n'attend derrière un autre ; la 129 finie par le relais

> Session du relais (compte a), le 22/11/2026 depuis 23 h 26 — la deuxième session du relais, qui
> reprend la première, coupée à 23 h 26 par le redémarrage du veilleur. Sa consigne (retouchée par lui
> à 23 h 11) : *« Continue Glucose. D'abord finir le chantier 129 (les 4 épreuves qui tombent, les
> sabotages), tout commiter et envoyer ; puis le plan (SUITE § 2, la passation). Prouve chaque chose
> (épreuves, sabotages, captures du pilote). »*

## 1. En arrivant

* **La première session du relais** (lancée à 23 h 11) avait fait passer les quatre épreuves de la
  129 et commité (`48094df`), puis lancé les 25 sabotages ; le redémarrage du veilleur (`ce0324b`, la
  session interactive qui construit le relais) l'a coupée à 23 h 26, au milieu. Son journal le dit ;
  l'arbre des sabotages était propre (aucun passage resté cassé). La session interactive a depuis
  corrigé le service (`KillMode=process`) : un redémarrage du veilleur ne tue plus la session.
* **Ce qu'elle allait corriger** : de 23 h 12 à 23 h 25, son Glucose n'a répondu à **aucun** agent —
  ni à elle, ni au veilleur du relais, dont la lecture de la consigne a expiré quatre fois (deux
  minutes chacune, `~/.local/state/relais/journal.txt`).
* **Trois autres sessions de Claude tournaient** (VS Code) : la session interactive du relais (dans
  `~/Projets/glucosegit`), un essai du compte b. La règle du mode d'emploi du relais — *« si tu n'es
  pas lui, ne modifie pas le dépôt pendant qu'il tourne »* — est la leur ; je n'ai pas touché à
  `outils/relais`.

## 2. Remis en question

* **« Un geste resté ouvert retient toutes les demandes »** (la première session) : vrai, mais pas
  seul. La boucle ne regardait que **la tête de la file** : `Some(true) if in_live_edit() => return`
  arrêtait tout pendant un geste, et `None => { question ; return }` arrêtait aussi tout tant que
  l'utilisateur n'avait pas répondu à la question d'accord **d'un autre agent**. Deux causes
  possibles au blocage de 23 h 12 ; son Glucose a été relancé à 23 h 28 (la chronique de la session
  bloquée est écrasée) : **laquelle a joué ne se prouvera plus**. La loi neuve couvre les deux.
* **« Aucun texte en cours d'édition, sinon les agents attendent »** (le mode d'emploi du relais,
  dans son Glucose) : **probablement inexact**. La saisie n'ouvre le geste qu'au moment de valider
  (`text_edit.rs`, `begin_live_edit` puis `end_live_edit` dans le même appel) ; ce qui tient un geste
  ouvert longtemps, c'est **la roue des couleurs** (de la première couleur choisie jusqu'à « valider »
  ou « annuler »), une flèche qu'on tire, un nœud qu'on glisse.
* **« Une lecture peut passer pendant le geste »** : à vérifier, pas à supposer. Les quatre lecteurs
  (`LECTEURS`) : `glucose_lire` ne prend que `&self` ; `glucose_chercher` et `glucose_domaines`
  prennent `&mut self` pour leurs index ; `glucose_histoire` appelle `consigner()`, qui n'écrit que
  les gestes **publiés** — le geste ouvert n'est publié qu'à sa fermeture. Et la signature d'un geste
  (`journal.main`, que l'appel d'un agent remplace le temps de l'appel) se lit **à la publication**
  (`journal.rs`, `publier(tx, self.main.clone(), …)`), jamais pendant une lecture : une lecture ne
  peut ni écrire dans le geste, ni le fermer, ni le signer.
* **« Garder l'ordre par agent »** (ma première loi, écrite, éprouvée, sabotée 8 sur 8 — et
  **fausse**) : je voulais qu'un agent qui écrit puis lit lise ce qu'il a écrit, et j'ai gardé
  l'ordre des demandes **par nom d'agent**. Le pilote l'a démenti (§ 5) : chaque commande lancée
  d'un terminal se présente comme « le terminal » ; une écriture qui attendait le geste a retenu
  trois lectures d'une **autre** connexion, 20 secondes chacune. Et la garantie que je croyais
  devoir écrire existait déjà : **chaque connexion attend la réponse avant d'envoyer sa demande
  suivante** (`porte::servir`, `ia::discussion::executer`) ; la porte publique sert chaque requête
  HTTP pour elle-même, et deux requêtes JSON-RPC envoyées sans attendre ne se promettent aucun
  ordre. Une connexion n'a donc jamais deux demandes dans la boîte : *Read Your Writes* tient par
  construction, et l'ordre par nom n'ajoutait que de fausses dépendances — y compris derrière une
  demande que son client avait déjà abandonnée (une écriture expirée côté client reste dans la file
  et s'exécute à la fin du geste ; vu au pilote, rang 4 de l'histoire). **La loi juste est plus
  simple : aucune demande n'attend derrière une autre.**

## 3. La recherche (consultée le 22/11/2026)

* Terry, Demers, Petersen, Spreitzer, Theimer, Welch, [*Session Guarantees for Weakly Consistent
  Replicated Data*](https://www.cs.cornell.edu/courses/cs734/2000FA/cached%20papers/SessionGuaranteesPDIS_1.html)
  (PDIS 1994, Bayou, Xerox PARC) : quatre garanties **par session**, dont *Read Your Writes* (les
  lectures d'une session voient ses écritures) et *Monotonic Reads* ; elles s'appliquent session par
  session, sans ordre imposé entre deux sessions. C'est la loi qu'il fallait : un agent est une
  session ; deux agents n'ont aucun lien de cause.
* [JSON-RPC 2.0](https://www.jsonrpc.org/specification) : une réponse se rapporte à sa requête par
  son `id` ; un lot de requêtes peut se traiter comme des tâches concurrentes, dans n'importe quel
  ordre (§ 6), et hors lot la spécification ne promet aucun ordre — un client qui a besoin d'un
  ordre attend la réponse avant d'envoyer la suite. C'est ce que font toutes les connexions de
  Glucose (et `Connexion` ne lit pas les lots).
* MCP, [les annotations d'outils](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/)
  (depuis la révision 2025-03-26) : `readOnlyHint` dit qu'un outil ne change rien (faux par défaut) ;
  les clients s'en servent pour l'accord et le parallélisme, et doivent les tenir pour non fiables
  hors d'un serveur de confiance. Glucose ne les déclarait pas.

## 4. Ce qui est écrit

* **LECTURE-1** (`app/agents.rs`, `Agents::prochaine`) : la boucle parcourt la file et sert **la
  première demande qui peut passer** — son agent a l'accord de l'utilisateur (ou c'est une IA du
  nuage, qui l'a déjà), **et** elle ne fait que lire, ou aucun geste n'est ouvert. Un agent sans
  réponse : la question se pose (pour le premier), ses demandes attendent, les autres passent.
  **Aucune demande n'attend derrière une autre.** Aucune constante. Et l'épreuve de la garantie
  là où elle vit : une connexion n'a jamais deux demandes dans la boîte (`porte/tests.rs`).
* **`outils::lit_seulement`** : la liste `LECTEURS`, celle de « Lire seulement », devient la seule
  source de « cet outil ne fait que lire » — pour la portée comme pour la boucle.
* **« Plus tard »** refusait la tête de la file en la croyant de l'agent questionné ; la tête peut
  désormais être celle d'un autre. Il refuse **la première demande de cet agent**.

## 5. Vérifié

* **La 129** : les 25 sabotages rejoués en trois lots au commit `48094df` (fiche 129 § 9) — **25 sur
  25**.
* **LECTURE-1 et 2** : Clippy strict du workspace ; **toute la suite, 3 481 épreuves, aucune ne
  tombe** ; **11 sabotages sur 11** (`outils/sabotages/lecture-130.json`, au commit `82d4ca1`), dont
  celui de la connexion qui n'attendrait plus sa réponse.
* **Au pilote, dans le vrai Glucose, en paire** (`essai_lecture.sh`, gardé dans
  `~/.cache/glucose-bancs/lecture-130/`) : le nœud d'accueil saisi et déplacé, bouton enfoncé — un
  geste ouvert —, puis une lecture seule, une écriture d'une connexion « le terminal » et trois
  lectures d'une autre :

  | | hors geste | lecture seule pendant le geste | 3 lectures, une écriture en attente | l'écriture |
  |---|---|---|---|---|
  | **avant** (le Glucose de 21 h 53) | 75 ms | **coupée à 10 s** | **coupées à 10 s** chacune | passe 22 ms après le relâchement |
  | **après** (`82d4ca1`) | 75 ms | **77 ms** | **76 à 78 ms** | passe 21 ms après le relâchement |

  L'ancien reproduit le blocage du 22/11 à 23 h 12 ; le nouveau ne fait attendre que l'écriture, le
  temps du geste. La capture pendant le geste montre le nœud dans la main et l'écriture pas encore
  posée.
* **Ma première loi, au pilote** : avec l'ordre par nom, les trois lectures d'une autre connexion
  ont attendu 20 s chacune derrière l'écriture (§ 2) — c'est ce qui l'a fait tomber.

## 6. La discussion de la 129, enfin regardée (au pilote, avec sa clé)

* Collaborer → « L'API publique… » dit *« Ta clé Google AI Studio est branchée »* ; « Discuter avec
  une IA » ouvre le tiroir DISCUSSION en haut à droite, le champ a le clavier, le pied dit *« Ce que
  l'IA lit de ce document part chez Google. »*
* *« Pose un texte qui dit : Bonjour depuis Gemini »*, `Entrée` : « Gemini 4 Hydrogen réfléchit… »
  (en vert), le bouton devient « Arrêter » ; douze secondes plus tard le fil dit « → a lu le
  tableau », « → a posé un texte », puis **Gemini 3.6 Flash** : *« J'ai posé le texte « Bonjour
  depuis Gemini » sur le canevas. »* Le nœud est là, signé « agent Gemini 3.6 Flash » dans
  l'histoire. **La discussion marche de bout en bout dans le vrai programme.**
* **Une surprise, pas un défaut de la loi écrite** : le tiroir a nommé Hydrogen, puis c'est Flash
  qui a répondu, sans un mot. BASCULE-2 ne dit un changement que par rapport à « celui d'avant » ;
  à la première réponse, il n'y en a pas. Ce que la bascule apprend (Hydrogen n'est pas gratuit) se
  réapprend à chaque lancement — la suite (5) de SUITE, le garder d'un lancement à l'autre, efface
  la surprise : le tiroir nommerait directement celui qui répond.
* Le texte du tiroir est petit (la taille des autres tiroirs) : à son écran de juger.

## 7. Choix pris seul, à contester

* **Une demande abandonnée par son client s'exécute quand même** à la fin du geste (vu : rang 1 de
  l'histoire de l'essai, après un délai dépassé côté client). Inchangé : une écriture d'agent est
  idempotente par son identifiant ; MCP a `notifications/cancelled` pour l'annuler, que Glucose ne
  lit pas encore.
* **Une écriture attend toujours tout le geste**, même long (la roue des couleurs restée ouverte) :
  ce qu'elle écrirait entrerait dans son geste à lui, et l'annuler l'emporterait (la roue repart des
  couleurs d'avant à chaque mouvement). Un geste d'agent posé « sous » celui de l'humain demanderait
  un journal à deux têtes : pas maintenant.
* **`readOnlyHint` sur les seuls lecteurs** ; `destructiveHint` (vrai par défaut) reste au défaut
  pour les autres, `glucose_retirer` compris.
