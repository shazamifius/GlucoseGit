# 154 — Collaborer en trois portes (25/11 vers 1 h 45, le relais, compte a)

## 1. Sa demande

Sa proposition du 24/11 (`relais-rep-321`) : *« tu cliques sur collaborer tu vois : porte humain, porte
IA, sécurité »* — la porte humain avec les codes (travailler, lire, administrer), la porte IA avec le MCP
et l'API (*« et ensuite on retrouve toutes les fonctionnalités qu'on a actuellement »*), Sécurité avec
qui est connecté (humains et IA), les liens, leur remise à zéro, fermer la porte à quelqu'un. Ma réponse
(`relais-rep-321-collaborer`) l'a prise telle quelle, en quatre étapes ; l'étape 4 est faite (fiche 153),
les étapes 1 (« Changer les codes ») et 3 (le code administrateur) passent par le serveur, donc par
Cloudflare (`relais-cf-lien`). **Ici : l'étape 2, la fenêtre** (PORTES-1).

## 2. Ce qui est fait

`crates/glucose-desktop/src/app/collaborer/portes.rs` : **la même question sur deux niveaux** — toutes
ses réponses passent encore par `repondre_a_collaborer`, qui délègue les nouvelles à `repondre_aux_portes`.

* **Collaborer** : « Porte humain… », « Porte IA… », « Sécurité… », « Fermer » ; le texte dit d'un coup
  d'œil où en est chacune (partagé ou non — et alors le lien, qui est là, les photos qui attendent ; la
  porte du nuage ouverte ou fermée ; la clé de l'API publique branchée ou non).
* **Porte humain** : seul, « Partager ce canevas » (ou « Revenir dans sa salle »), « Rejoindre… », « Ton
  nom et ta couleur… » ; partagé, les deux codes à copier, « Quitter le partage », son nom. Tout ce qui
  se disait « là où on l'a fait » (un code copié, un partage raté, une salle vide) s'y dit désormais :
  `montrer_collaborer_avec` est devenu `montrer_la_porte_humain`, et « Annuler » après « Rejoindre… » ou
  « Ton nom… » y revient.
* **Porte IA** : « La porte aux IA publiques… » (le MCP du nuage) et « L'API publique… », chacune ouvrant
  sa fenêtre telle qu'elle était, avec sa mise en garde ; le texte rappelle que chaque geste d'une IA est
  signé et s'annule.
* **Sécurité** : les humains (les codes qui ouvrent la salle, ou « tu y es entré par le code qui
  regarde » ; qui est là ; les photos qui attendent), les IA du nuage (la porte, qui l'a et avec quels
  droits — `qui_a_la_porte`), **les programmes de cette machine** (Claude Code par `glucose --mcp` : qui a
  son accord, l'accord durable du document compris — `qui_a_ton_accord`, tiré de « Retirer l'accord des
  agents… » pour ne le dire qu'une fois). Deux gestes réels : « Les IA du nuage… » (retirer une IA) et
  « Les programmes de cette machine… » (retirer leur accord).
* « Retour » sur chaque porte remonte à Collaborer. Les réponses prennent les rangs libres : 7, 8, 9
  (les portes), 15 (Retour), 16 et 17 (les deux gestes de Sécurité).

**Ce qui n'y est pas, exprès** : ni « Changer les codes », ni les liens qui expirent, ni « fermer la
porte à une seule personne », ni le code administrateur — ils demandent le serveur. Aucun bouton ne les
annonce : un bouton qui ne fait rien ment (fiche 71).

## 3. Les doutes

* **Trois niveaux pour l'IA** (Collaborer → Porte IA → la porte du nuage). Nielsen Norman Group : au-delà
  de deux niveaux de dévoilement, l'usage chute ; Shopify conseille d'éviter plusieurs niveaux. C'est sa
  proposition telle quelle, et la porte du nuage ouverte a déjà quatre boutons : la fondre dans la porte
  IA ferait une fenêtre de huit. À revoir s'il s'y perd.
* « Retour » n'existe pas dans les fenêtres de la porte du nuage et de l'API : elles se ferment. Elles
  ont leurs propres réponses (`Suite::PortePublique`, `Suite::ApiPublique`) ; y ajouter « Retour » est un
  petit chantier à part, si le besoin se montre.
* **« Seul un administrateur ouvre Collaborer en entier »** (sa demande) attend le code administrateur
  (étape 3) : aujourd'hui, celui qui rejoint par un code voit les trois portes.

Sources : NN/g, *Progressive Disclosure* (https://www.nngroup.com/articles/progressive-disclosure/) ;
Shopify, *Progressive disclosure* (https://www.shopify.com/partners/blog/progressive-disclosure) ;
Microsoft, *Progressive Disclosure Controls*
(https://msdn.Microsoft.com/en-us/library/windows/desktop/dn742409(v=vs.85).aspx) — la stabilité : les
gestes de Sécurité sont toujours là, même quand ils n'ont rien à retirer.

## 4. La preuve

* Quatre épreuves neuves (`collaborer/portes/tests.rs`), par les libellés, comme sous sa main ; quatre
  anciennes adaptées (le bouton de la souris, la reprise, Rejoindre, Q18 inchangée).
* **9 sabotages sur 9** (`outils/sabotages/portes-154.json`) : une porte qui manque, « Retour » muet, la
  porte IA qui ne mène plus au MCP, Sécurité qui tait l'accord des programmes ou qui a la porte, qui
  confond le code qui regarde, un code qui regarde qui se dit capable d'écrire, « Rejoindre… » perdu,
  Sécurité qui mène ailleurs.
* clippy propre, **3 689 épreuves vertes**.
* **Regardé au pilote** (hors écran, sans document à lui) : Collaborer, puis Sécurité, tels que décrits.
