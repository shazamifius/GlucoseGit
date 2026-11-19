# 122 — Ses réponses dans Glucose, la porte à l'épreuve du réel, la borne qui le laisse respirer, les tiroirs du côté de leur bouton

> Session du 19/11/2026, depuis 9 h 18, après la fiche 121 (restée dans l'arbre, non validée). Sa
> parole d'ouverture, avec l'adresse de sa porte publique : *« moi de mon côté je commence à remplir
> ça de manière progressive ; l'idée, c'est de répondre à toutes les questions ; évite de créer des
> pages complètes en artefact, crée plutôt sur Glucose ; voici le lien MCP pour toutes les
> réponses ; pourrais-tu commencer par ça ? »*

## 1. Ses réponses, lues par sa porte

* **Le premier vrai client de la porte publique** : Claude Code, sous son identité vérifiée
  (`https://claude.ai/oauth/claude-code-client-metadata`, « Claude Code », un retour en boucle
  locale) — le chemin de `faux_claude.mjs`, sans se faire passer pour claude.ai. Son clic,
  « Lire et écrire », devant le code 2 7 2. Devenu un outil : `outils/porte/claude_code.mjs`
  (`accord`, `outils`, `appeler`), la clé dans `~/.local/share/glucose-cles/porte-claude-code.json`.
* **Son document** (« ce qui attend ma parole ») : trois pages recopiées en textes — les tiroirs
  (fiche 121), le co-working (fiche 113), la porte publique (fiche 111) — et sa réponse à côté de
  chacune, reliée par une flèche. Ses mots sont dans VISION (§ 1, § 3, § 9). Mes réponses posées
  dessous, signées « agent Claude Code (claude.ai) » dans sa Time Machine, avec un écrit.
* **Sa question « 272 ? »**, posée en retour dans le canevas : *« j'ai l'impression que c'est
  toujours le même code et ça n'a jamais fonctionné ; j'ai testé aussi avec Mistral ; à chaque fois
  le message reste uniquement : voulez-vous que « tel domaine » ait accès en lecture-écriture de
  votre document, et des boutons pour autoriser ou non — donc pas de code »*. Deux défauts réels :
  le code n'est pas vu (§ 3), et **Claude.ai et Mistral ne sont jamais entrés** (§ 4).

## 2. Remis en question

* **« Le chemin du document ne part plus au nuage »** (fiche 116 § 6) : faux. `glucose_lire` rend
  deux champs, `fichier` (tu depuis la fiche 116) et `document` — l'identité du document, qui est
  son chemin quand il vient d'un fichier. L'essai de la fiche 116 lisait un brouillon, dont
  l'identité est un instant : il ne pouvait pas le voir. Le premier vrai client a reçu
  `/home/…/Downloads/ce qui attend ma parole.glucose`.
* **« Toute commande lourde bornée ne gêne plus la machine »** (fiche 110) : faux quand il
  travaille à côté — § 5.
* **Mon hypothèse sur l'échec de Claude.ai** (une fenêtre cachée sous Wayland ne dessine plus, et
  l'échange du code attendrait le dessin) : fausse à la lecture du code — l'échange se fait dans le
  fil du lien, hors de la boucle (`http::echanger`).

## 3. PUBLIQUE-1 : aucun chemin au nuage, par l'identité non plus

`document_vu()` (`agents/executer/domaines.rs`), à côté de `fichier_vu()` : pour un agent de cette
machine, l'identité telle quelle ; pour une IA du nuage, `doc-` et les seize premiers chiffres du
SHA-256 de l'identité — la même d'une session à l'autre (l'agent range sous elle ce qu'il a vu,
fiche 80), muette sur le disque (C1). L'épreuve de bout en bout (`publique_1_…`) pose l'identité
d'un document venu d'un fichier, `/home/quelqu-un/Téléchargements/ses réponses.glucose`, et exige
qu'aucune lecture ne dise ni le dossier ni le nom, que `document` commence par `doc-`, et que deux
lectures rendent la même. Deux sabotages (`outils/sabotages/porte-122.json`) : l'identité brute,
une empreinte qui change à chaque lecture.

**Le code de la question d'accord, qu'il ne voit pas** : il est dans le texte (`Code : 2 7 2 — le
même que sur la page de ton IA ? Sinon, refuse.`), au milieu d'un paragraphe, sans rien qui le
distingue — la page web, elle, l'écrit en 44 px. La question ne coupe son texte que si l'écran
manque de hauteur : dans sa fenêtre (936 × 996), rien n'est coupé. Le code passe dans le titre.

## 4. Claude.ai et Mistral n'entrent pas — en cours

* **Ce qui est sûr** : sa liste des IA qui ont la porte (lue sur une copie de
  `porte-publique.json`) n'a que « Claude Code ». L'accord ne s'inscrit qu'à l'échange du code
  contre une clé (`oauth::echanger`) : pour Claude.ai et Mistral, cet échange n'a jamais réussi. La
  question est apparue dans Glucose, donc la découverte, la page et la demande passent : le chemin
  casse après son clic — la page qui renvoie le navigateur vers l'IA, ou l'échange.
* **Écarté** : `iss` (RFC 9207) égale l'émetteur annoncé — vérifié sur sa porte en ligne ; le dessin
  d'une fenêtre cachée (l'échange se fait dans le fil du lien).
* **La documentation d'Anthropic** ([l'authentification des
  connecteurs](https://claude.com/docs/connectors/building/authentication), lue le 19/11) : rien que
  la porte ne remplisse — `401` et `resource_metadata`, le premier `authorization_servers`, CIMD
  avec `none`, le formulaire au `/token`, 10 s par point d'accès.
* **Mistral** s'inscrit par DCR ([MCP Connectors](https://docs.mistral.ai/le-chat/knowledge-integrations/connectors/mcp-connectors),
  [Configuring a custom connector](https://help.mistral.ai/en/articles/393572-configuring-a-custom-connector)) ;
  une piste, à confirmer : s'il présente son client par `Authorization: Basic` plutôt que dans le
  formulaire, `echanger` ne trouve pas `client_id` et refuse.
* **L'outil du diagnostic** : `wrangler tail glucose-porte`, résumé par un script qui ne garde que
  l'heure, la méthode, le chemin (Cloudflare masque l'identifiant de la porte), les noms des
  paramètres et le statut — ni code, ni clé, ni adresse. Il attend son essai.

## 5. La borne qui le laisse respirer

* **9 h 36** : la suite entière, bornée à 60 % de la mémoire (4,8 Go), pendant que Brave, Glucose et
  VS Code tenaient 3 Go. 1,7 Go de son bureau dans le swap compressé, la mémoire en pression 11 % du
  temps (`/proc/pressure/memory`, `full avg10`) : *« mon ordinateur est à 1 fps depuis tout à
  l'heure »*. Arrêtée à 9 h 52.
* **La loi** (`outils/borne.sh`) : la plus petite de 60 % de la mémoire et de `MemAvailable` — *« an
  estimate of how much memory is available for starting new applications, without swapping »*
  ([la documentation du noyau](https://docs.kernel.org/filesystems/proc.html)) — moins 1 Go laissé à
  son bureau (un nombre de ressenti). Trop peu pour une compilation : rien ne part, et c'est dit.
* **Mesurée** par une sonde (la pression toutes les 5 s) : la suite suivante, 0,34 % en moyenne (pire
  7 %, quatre fois au-dessus de 2 % en neuf minutes) ; les épreuves du bureau, 0,20 %. Les pics
  restants viennent surtout de son bureau qui reprend ce que la première suite avait poussé au
  swap (`swappiness` 180, zram).
* **10 h 46** : un `rustc` tué par sa borne — celui des épreuves du bureau monte à **2,4 Go**
  (`anon-rss:2417076kB`), pas 1,08 comme le 12/11 ; deux dans 4 Go ne tenaient pas. Une compilation
  par **2,5 Go** de borne désormais. Le crate qu'il écrivait est resté à moitié écrit (des
  « undefined reference » à l'édition de liens suivante) : `cargo clean -p glucose-desktop` — 114 Go
  libérés, le disque de 37 à 113 Go libres.

## 6. Mistral et Claude.ai entrent — la porte marchait, l'adresse était fausse

* **L'écoute du relais l'a montré** (aucune des trois hypothèses d'avant ne l'avait trouvé) : à
  10 h 39 et 11 h 34, Mistral (`MistralAI-MCPClient/1.0`) frappait à `/g/…/mcp` et recevait **503
  « le Glucose de cette adresse n'est pas ouvert »**, quand la vraie porte, à la même minute,
  répondait 401. Son connecteur portait **une autre adresse** que celle de ce Glucose — chacun de
  ses Glucose (l'école, le téléphone) a la sienne. Sa question posée dans le canevas ; il a corrigé.
* **11 h 37, Mistral** : `/.well-known/mcp/server-card/…` (404 : un format en discussion),
  `/mcp` 401, la découverte (200, 200), **DCR** (`/register` 201), la page, la demande, son clic,
  **`/token` 200**, puis `/mcp` 200. Inscrit sous le nom qu'il se donne, « mistral-mcp-client »,
  depuis `callback.mistral.ai`. Le Chat gratuit était ensuite **« à pleine capacité »** : il n'a
  rien posé. Sa conclusion — *« un connecteur ne fonctionne absolument pas pour Mistral car il est
  juste beaucoup trop mauvais dans sa version rapide ; MAIS… dans la théorie ça fonctionne »* —
  contestée : le service était saturé, on ne l'a pas vu travailler.
* **11 h 45, Claude.ai, depuis son compte** : le dialogue « Add custom connector » a **détecté** seul
  l'OAuth (« Sign in now — Detected ») et le CIMD (« Use Claude's published identity — Detected ») ;
  la découverte, la page (sans inscription : CIMD), son clic (code 3 4 6), `/token` 200, puis les
  outils (`Claude-User`). Inscrit « Claude », `claude.ai`. **Il a posé, signé « agent Claude
  (claude.ai) »** : un onglet « Textes optimisés », trois membranes colorées (`#3b82f6`, `#10b981`,
  `#fb923c`) et ses trois pages restructurées (titres, tableaux), les originaux intacts, un « f »
  égaré retiré. **Le critère de SUITE pour la porte est rempli.** Sa remarque ensuite : *« moi, mon
  Glucose n'a juste pas changé du tout »* — son Glucose installé date du 17/11 ; et l'onglet neuf
  est-il visible dans sa barre d'onglets ? **À vérifier.**
* **Les émojis** : Claude.ai dit colorer par « les membranes colorées et des émojis 🟢 🔴 🟠 🔵 ».
  Glucose les dessine en **noir et blanc** (`NotoEmoji-Regular.ttf`, EMOJI-1) : ses ronds « fait » et
  « reste à faire » sont, très probablement, identiques à son écran — le pont humain-IA cassé (pas
  encore regardé à l'image). Fait : la description de `glucose_poser_texte` le dit aux IA. Reste :
  les émojis en couleur (Tauri, un navigateur, les avait) — sa parole : *« je suis d'accord avec tes
  propositions »*.

## 7. Les tiroirs, écrits pour de vrai (TIROIR-4) — NON VALIDÉS

* `TabId::cote()` : Ordonner, Timer, Time Machine, Storyboard à gauche ; Plugins, Mood board,
  Domaines à droite — le **groupe** de la bande où est le bouton (l'épreuve par la souris le lit
  dans la vraie bande : un bouton avant « Collaborer » est à gauche ; la moitié de l'écran était
  fausse — à 1440 pixels, Storyboard est à 724).
* `compute_panel_layouts` : collés au bord sous la bande ; **first fit** (Johnson, 1974) : chaque
  tiroir dans la première colonne de son côté où il tient, sinon une colonne voisine ; une colonne
  a la largeur de son plus large ; une colonne de trop reste dans l'écran (contre l'autre bord).
* Plat : ni ombre ni coin (`panel_shadow*` retirées du thème) ; la poignée en bas pour tous, la
  poignée comptée dans chaque hauteur.
* Le glisser (`glisser_le_tiroir`, `lacher_le_tiroir` dans `interactions/panels.rs`) : le tiroir
  prend la place de celui dont le centre est le plus proche du sien (le « closestCenter » de
  dnd-kit, *« generally the recommended default for sortable lists »*), sans sauter ; il se ferme
  **tiré au-delà du bord de l'écran de son côté** de plus de 80 pixels — un tiroir de la colonne
  voisine traverse la première sans se fermer.
* `panneau_sous` cherche du dernier dessiné au premier : là où deux se croisent, le clic va à
  celui qu'on voit.
* **La limite, mesurée** : six tiroirs en 1440 × 900 avec **huit domaines** — Domaines 494 pixels,
  sous aucun autre ; trois colonnes à droite, 1 450 pixels pour 1 440 : Storyboard et Domaines se
  croisent de 10 pixels. Tranché : l'épreuve garde le cas courant (deux domaines) ; le cas extrême
  passe par l'épreuve « trop étroit ». **La vraie loi serait des tiroirs qui défilent quand la
  hauteur manque.**
* Épreuves : 107 vertes (tiroirs, dock, souris, porte), la suite entière pas encore sur cet état ;
  **jamais regardés au pilote** ; sabotages à écrire.

## 8. Sa règle des 30 minutes

*« 3 heures… j'aimerais que toutes les 30 minutes tu mettes un timer, tu te réveilles, tu vérifies
à chaque fois si tout fonctionne ; je ne souhaite qu'aucune commande ne dure 3 heures »* — une
écoute du relais lancée pour trois heures. Mémoire `trente-minutes-au-plus`. Une suite entière
après `cargo clean` en a duré 39 : à couper en étapes (compiler, puis éprouver).

## 9. La recherche (consultée le 19/11/2026)

* Anthropic, [l'authentification des connecteurs](https://claude.com/docs/connectors/building/authentication) :
  CIMD, DCR, le `401` et `resource_metadata`, 10 s par point d'accès.
* Mistral, [MCP Connectors](https://docs.mistral.ai/le-chat/knowledge-integrations/connectors/mcp-connectors)
  et [Configuring a custom connector](https://help.mistral.ai/en/articles/393572-configuring-a-custom-connector) :
  OAuth 2.1 par DCR.
* Le noyau Linux, [`/proc/meminfo`](https://docs.kernel.org/filesystems/proc.html) : `MemAvailable`,
  *« an estimate of how much memory is available for starting new applications, without swapping »*.
* dnd-kit, [les algorithmes de collision](https://dndkit.com/api-documentation/context-provider/collision-detection-algorithms) :
  le « closestCenter », *« generally the recommended default for sortable lists »*.
* Le « first fit » du rangement en boîtes : D. S. Johnson, *Near-optimal bin packing algorithms*
  (thèse, MIT, 1973) et *Fast algorithms for bin packing* (1974).

## 10. Les leçons

* **Mesurer avant de théoriser** : trois hypothèses sur l'échec de Claude.ai et de Mistral, toutes
  fausses ; l'écoute du relais (`wrangler tail`) a montré la vérité en cinq minutes — une autre
  adresse que celle de ce Glucose, qui répondait 503.
* **Une borne fixe protège la machine, pas lui** : ce qui compte est ce que la machine laisse
  quand on lance (`MemAvailable`), pas sa taille.
* **La pression mémoire du système mélange tout** : une compilation qui touche sa borne y monte à
  20 % pendant que son bureau est à 0 ; mesurer ses cgroups.
* **Une mesure ancienne se périme** : le `rustc` des épreuves du bureau tenait 1,08 Go le 12/11,
  2,4 Go le 19/11.
* **Le côté d'un tiroir se lit dans la bande**, pas dans la moitié de l'écran : l'épreuve passe
  par la vraie bande.
* **Une épreuve qui s'arrête au premier cas faux coûte une compilation par cas** : la faire
  collecter tous les cas et imprimer les nombres.
* **`pkill -f` sur un motif présent dans sa propre commande la tue elle-même** : tuer par numéro
  de processus.

Le travail des § 3, 6 (la phrase des émojis) et 7 est resté dans l'arbre, non validé, à la fin de
la session ; il est validé et envoyé par la fiche 123.

**Correction du 19/11 après-midi (fiche 124)** : la fiche 123 n'a rien compilé non plus (sa mémoire
pleine) ; ce travail est validé et envoyé par la fiche 124.

**Correction du 19/11 à 15 h (fiche 125)** : le tour 3 de la fiche 124 est tombé ; ce travail est
validé par GitHub au tour 4 (`d46f5e0`, dix tâches sur dix), et dans `main` depuis.
