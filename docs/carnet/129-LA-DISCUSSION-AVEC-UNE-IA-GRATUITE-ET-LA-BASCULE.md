# 129 — La discussion avec une IA gratuite, et la bascule qui dort le temps que Google dit

> Session du 22/11/2026, depuis 17 h 59, après la fiche 128 (IA-1, le 21/11 au soir). La passation :
> regarder le menu « L'API publique » à l'écran, la bascule entre modèles, la fenêtre de chat et le
> MCP, l'essai sous Windows, GitHub Models, pousser, l'index. Ses paroles du 21/11 : VISION § 3.

## 1. En arrivant

* **La machine** : 4,5 Go disponibles à 18 h, 2,2 à 18 h 26 (Brave, VS Code) — la borne refuse une
  compilation d'épreuves (3,58 Go).
* **GitHub** : `origin/main` à `596e783` ; **deux commits locaux non poussés** (`f9a8557` DETOURER-2,
  `87dfb5a` IA-1) ; le travail de la fiche 127 (sa fiche, ÉTAT, SUITE, l'index, `CLAUDE.md`) **jamais
  commité**, et sa fiche s'arrête sur « § 9 (à écrire) ».
* **La télémétrie** (18 h) : 228 sessions, 10 installations, **42 fins anormales** (+2 depuis le 20/11 :
  son téléphone, passé seul en 2.0.4-beta.4 le 21/11, et le Windows à carte AMD). Ce dernier dit une
  « pire image » de 4 294 967 ms : `u32::MAX` microsecondes, une veille de plus de 71 minutes saturée —
  la beta.4 n'a pas VEILLE-1.
* **Son Glucose** (par sa porte, l'onglet nommé) : rien de neuf depuis le 20/11 ; ses trois questions
  (la beta.5, les GitHub Pages, Cloudflare) attendent.

## 2. Remis en question

* **« La passation a tout dit de sa demande »** : non. Ses mots du 21/11 n'étaient nulle part dans
  VISION ; relus dans le journal de la session, ils disent l'essentiel que la passation taisait :
  *« j'aurais besoin de lui pour pas mal de choses, de manière très rapide »* — **le chat qui marche,
  vite**, pas un menu de test. L'ordre change : la fenêtre de chat et la bascule d'abord.
* **« Les 429 sont passagers »** (fiche 128 § 2) : **faux pour les Pro**. Le 429 de
  `gemini-3.1-pro-preview` et de `gemini-4-argon` dit `limit: 0` : aucun quota gratuit — depuis le
  1/04/2026, les Pro ne sont plus gratuits. Et Google dit lui-même quand revenir
  (`RetryInfo.retryDelay`, « 53509s » : minuit, heure du Pacifique).
* **« Huit essais au plus »** (fiche 128 § 6) : une constante qui peut **disparaître**. Le 3.8 Flash
  gratuit ne donne qu'une vingtaine de requêtes par jour ; huit essais à chaque « Tester » mangeaient
  le quota. La bascule fait l'essai elle-même : on s'arrête au premier qui répond.
* **« Le test GitHub Models »** (passation § 4.6) : **GitHub Models a fermé le 30/07/2026**
  (*« GitHub Models has been fully retired […] The playground, model catalog, inference API, and bring
  your own key (BYOK) are no longer available to any customer »*, docs.github.com). Le bouton « Donner
  la clé GitHub » d'IA-1 menait à un service fermé : un bouton qui ment. Retiré, avec le champ de la clé.
* **« Le chat se connecte comme client à l'adresse MCP de la porte »** (passation § 4.4) : un
  aller-retour par Cloudflare pour se parler à soi-même. Les outils de la porte se donnent **directement**
  à Gemini (ses `functionDeclarations`), et leurs appels entrent par **la boîte des agents**, comme ceux
  de Claude.ai : même accord, même signature, même témoin, aucune ligne d'exécution réécrite.
* **« L'essai sous Windows : WinHTTP, à brancher »** : IA-1 appelait `ureq` directement, que Windows
  n'a pas. La plateforme avait déjà une porte réseau par système (`telecharger`, `envoyer`) ; il lui
  manquait **une question avec ses en-têtes et une réponse entière** : `plateforme::echanger`, sur les
  deux voies. Le chat marche donc sur son PC de l'école — à prouver par la CI.
* **Le classement par le nom** (fiche 128) : il comptait `gemini-robotics-er-2-preview` comme une
  version 2 et `gemini-omni-1.2-flash` comme une 1.2 (les deux premiers nombres du nom), et laissait
  Gemma dehors. La version doit **suivre** `gemini-` ; Gemma entre, en dernier.
* **Le résultat de l'essai d'IA-1 ne réveillait pas la boucle** : il était relevé au tour suivant, et
  au repos Glucose ne tourne pas — il n'apparaissait qu'au premier mouvement de souris. Chaque
  événement de la discussion réveille la boucle (`Boite::reveiller`).
* **La passation demandait de « révoquer » des clés** : contraire à sa règle sur les jetons (aucune
  remarque) ; suivie, la règle.

## 3. La recherche (consultée le 22/11/2026)

* Google, [*Rate limits*](https://aistudio.google.com/docs/rate-limits) (l'ancienne adresse
  `ai.google.dev` renvoie là) : RPM, TPM, RPD ; *« Requests per day (RPD) quotas reset at midnight
  Pacific time »*. [*API errors*](https://aistudio.google.com/docs/api-errors) : 429
  `rate_limit_exceeded` / `quota_exceeded`, 503 *« temporarily overloaded »* — *« Wait and retry with
  exponential backoff »* ; ne pas réessayer sur 400, 402, 403. Rien d'officiel sur `RetryInfo` — mais
  la réponse réelle le porte (§ 4).
* Les limites gratuites de fin 2026, par des tiers (Google ne les publie plus que dans AI Studio) :
  3.8 Flash autour de 5 requêtes par minute et 20 par jour ; les Flash-Lite récents 15 et 500 ; les
  Pro, rien ([imisofts](https://imisofts.com/blog/gemini-api-free-tier-limits/),
  [aifreeapi](https://www.aifreeapi.com/en/posts/google-gemini-api-free-tier)).
* [*Thought signatures*](https://ai.google.dev/gemini-api/docs/thought-signatures) : une signature par
  appel d'outil, à rendre au tour suivant, faute de quoi un 400 ; [Codex
  #7519](https://github.com/openai/codex/issues/7519) et
  [VS Code #296713](https://github.com/microsoft/vscode/issues/296713) l'ont perdue.
* L'API « Interactions » (juin 2026) est la porte que Google met en avant, `generateContent` reste
  pleinement prise en charge ([the-decoder](https://the-decoder.com/google-makes-interactions-api-the-default-interface-for-gemini-models-and-agents/),
  [la page d'Interactions](https://ai.google.dev/gemini-api/docs/interactions/interactions-overview), qui se
  dit encore bêta) : `generateContent`, éprouvée avec sa clé.
* GitHub, [*Prototyping with AI models*](https://docs.github.com/en/github-models/use-github-models/prototyping-with-ai-models) :
  la page de fermeture.
* D'autres API gratuites parlent toutes le format d'OpenAI (Groq, OpenRouter, Mistral, Cerebras —
  [KDnuggets](https://www.kdnuggets.com/5-free-llm-api-providers-you-can-use-in-2026),
  [awesome-free-llm-apis](https://github.com/mnfst/awesome-free-llm-apis)) : **un seul branchement les
  couvrira**, à la place du bouton GitHub.
* tldraw, [*Agent starter kit*](https://tldraw.dev/starter-kits/agent) : un panneau de discussion à
  droite, l'agent qui agit sur le canevas par des actions typées, sous les yeux ; et
  [*Branching chat*](https://tldraw.dev/starter-kits/branching-chat), chaque message un nœud du
  canevas — l'objectif 3 de VISION (des graphes de questions et de réponses), pour plus tard.

## 4. L'essai réel, avant d'écrire (sa clé, le 22/11 à 18 h 07)

Quatre requêtes à `gemini-3.5-flash-lite` (500 par jour), deux refus :

* **Un appel d'outil** avec le schéma entier de `glucose_poser_texte` (`parametersJsonSchema`,
  `items: {}` compris) : `functionCall` avec un `id`, et sa `thoughtSignature` à côté.
* **La même histoire envoyée à un autre modèle** (`gemini-3.1-flash-lite`), signature comprise : il
  conclut (*« Le texte « bonjour » a été ajouté au tableau. »*). **La bascule peut se faire au milieu
  d'un appel d'outil.**
* **Le 429 de `gemini-4-argon`** : `limit: 0`, et `RetryInfo.retryDelay: "53509s"`. Le 404 de
  `gemini-2.5-flash` : *« no longer available to new users »*.
* **Gemma 4 31B** sait appeler l'outil — en 27 s, et en écrivant sa pensée en clair (`thought: true`).

## 5. Ce qui est écrit

* **`plateforme::echanger`** — une question (verbe, adresse, en-têtes, corps, attente) et sa réponse
  entière, refus compris : `ureq` hors de Windows, **WinHTTP** sous Windows (le délai de réponse posé
  sur la requête seule).
* **`ia/gemini.rs`**, pur : le corps, la lecture d'un tour (le `Content` gardé **intact**), la réponse
  d'un outil, la lecture d'un refus (sa première phrase, son `retryDelay`).
* **`ia/bascule.rs`**, pur — **BASCULE-1** : chaque requête au premier éveillé ; un 429 avec délai
  dort ce délai, à l'heure du mur (une machine qui dort compte) ; 503, 429 sans délai, 400 et 500
  cèdent la question ; 404 sort ; 401 et 403 arrêtent. **Aucune durée choisie.** **BASCULE-2** : le
  relais et le retour **se disent**, en ambre dans le tiroir.
* **`ia/discussion.rs`** — **DISCUSSION-1** : les outils de la porte à la portée d'une IA du nuage (le
  document ouvert seulement), chaque appel par la boîte des agents. **DISCUSSION-2** : une question
  ne s'arrête qu'en haut de la boucle, quand chaque appel a sa réponse ; la suivante rejoint le
  dernier message de l'utilisateur. Une conversation se décrit sans sa clé.
* **Le tiroir DISCUSSION** (`dock/discussion.rs`), le huitième, à droite : le fil coupé à l'arrivée
  (une seule géométrie, L4), le champ (`Entrée` envoie, `Échap` rend le clavier, `Ctrl+V` colle, un
  clic ailleurs rend le clavier), « Envoyer » / « Arrêter », « … réfléchit », et le pied : *« Ce que
  l'IA lit de ce document part chez Google. »*
* **Collaborer → « L'API publique… »** : « Discuter avec une IA », « Tester les modèles gratuits » (dans
  le tiroir : chaque modèle écarté dit pourquoi, le premier qui répond se nomme — une seule question
  au modèle qui répond), « Donner la clé Google AI Studio ».

## 6. Vérifié, et ce qui ne l'est pas

* **`cargo check` (épreuves comprises), Clippy strict `-D warnings`, `cargo fmt`** : zéro.
* **La bibliothèque du bureau, à 19 h 23 : 2 205 passent, 4 tombent** (toutes celles de la bascule,
  de Gemini, de la clé et de la discussion pure passent) :
  * `test_discussion_le_clavier_va_au_champ_puis_le_rend` et `test_discussion_sans_cle_dit_ou_la_donner` :
    l'épreuve envoie une lettre **sans le texte** que le vrai clavier joint toujours, et la loi de
    frappe (`Command::of`) ignore une touche sans texte — **l'épreuve est probablement fautive**
    (`taper` doit passer `Some(lettre)`), à confirmer ;
  * `test_un_mot_trop_long_se_coupe_et_la_coupe_avance_toujours` : **un vrai défaut** — un mot coupé
    jusqu'à son dernier caractère laisse une ligne vide de plus (`["a","b","c",""]`) : en fin de
    paragraphe, ne pousser la ligne que si elle n'est pas vide, ou si le paragraphe était vide ;
  * `test_tiroir_6_au_pire_cas_aucun_tiroir_ne_se_pose_sur_un_autre` (`rangement/tests.rs:468`) :
    8 boîtes pour 7 attendues — probablement un « 7 » écrit en dur depuis que les tiroirs sont huit ;
    le relire avant de le changer.
* **La mémoire** : le `rustc` des épreuves du bureau a monté à **2,62 Go** (tué par sa borne de 2,65 Go
  à 18 h 54, profil d'aujourd'hui). Relancé avec `debug = "line-tables-only"` pour le seul crate du
  bureau (`--config`), sous une borne de 3,0 Go : il a fini ; la pointe de toute la commande (Nix,
  cargo, `rustc`, puis les épreuves sur quatre fils) était de 2,9 Go — **ce n'est pas la pointe du
  `rustc` seul** : la mesure de SUITE reste à faire proprement.
* **Pas vérifié** : les suites d'intégration (les cliquets) ; les 23 sabotages
  (`outils/sabotages/discussion-129.json`, jamais joués) ; **le tiroir regardé au pilote** ; un essai
  réel de bout en bout par le programme (seulement en `curl`, § 4) ; **la voie WinHTTP**, jamais
  compilée.

## 8. Où la session s'arrête

Le 22/11 à 19 h 24, à sa demande. **Rien n'est commité** de ce travail, ni de celui de la fiche 127 :
tant que quatre épreuves tombent, il reste dans la copie de travail (les fichiers neufs marqués « A »
par `git add -N`), et en secours dans `~/.cache/glucose-bancs/session-129/discussion-129.patch`
(`git apply` le rend). `main` local a deux commits que GitHub n'a pas (`f9a8557`, `87dfb5a`) ; aucun
jeton dans cette session. Rien ne tourne sur sa machine.

## 7. Choix pris seul, à contester

* **Un tiroir plutôt qu'un graphe de questions sur le canevas** : sa demande dit « une fenêtre de
  chat », et *« presque temporaire »* ; un fil hors du document ne le remplit pas de chaque question.
  Le graphe (tldraw *Branching chat*, VISION § 1 objectif 3) reste la suite naturelle.
* **Deux minutes d'attente** par réponse : un nombre de ressenti (Gemma : 27 s pour un geste).
* **Le texte en clair** dans le tiroir (les marques du Markdown retirées) : le dessin riche des nœuds
  n'y est pas.
* **La conversation vit en mémoire** : rien n'est écrit sur le disque ; ce que la bascule apprend se
  réapprend au lancement suivant (quelques refus rapides à la première question).
* **La clé en clair à 0600** (fiche 128) : inchangé.

## 9. Fini par le relais (22/11, de 23 h 11 à 23 h 35)

Les deux premières sessions du relais (compte a), seules, sur la consigne qu'il a retouchée à
23 h 11 : *« D'abord finir le chantier 129 (les 4 épreuves qui tombent, les sabotages), tout commiter
et envoyer. »*

* **Les quatre épreuves** (la première session, `48094df`) : deux tapaient une lettre sans le texte
  que le vrai clavier joint toujours — **l'épreuve était fautive**, comme pressenti en § 6 ; une
  comptait sept tiroirs en dur (`TOUS.len()` désormais) ; et **le vrai défaut** : un mot coupé
  jusqu'à son dernier caractère laissait une ligne vide derrière lui — la coupe ne prend plus tout
  le mot, le dernier morceau reste la ligne en cours ; une ligne blanche voulue reste une ligne
  vide (une garde de plus). Toute la suite du dépôt : **3 474 épreuves, aucune ne tombe** ; Clippy
  strict et `cargo fmt` à zéro.
* **Les sabotages** (25 : les 23 de § 6, plus deux pour la coupe et contre la rustine « jamais de
  ligne vide ») : la première session les lançait quand le redémarrage du veilleur l'a coupée
  (23 h 26) ; l'arbre des sabotages était propre. La seconde les a rejoués en trois lots, au commit
  `48094df`, chacun avec son témoin : **25 sur 25 tombent**, chacun sur l'épreuve qu'il vise.
* **Toujours pas fait** : le tiroir regardé au pilote, l'essai de bout en bout par le programme, la
  voie WinHTTP par la CI — la suite est dans la fiche 130.
