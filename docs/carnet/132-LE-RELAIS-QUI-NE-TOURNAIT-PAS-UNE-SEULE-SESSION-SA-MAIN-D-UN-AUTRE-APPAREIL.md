# 132 — Le relais qui ne tournait pas : une seule session, sa main d'un autre appareil, ses notes lues

> Session de Claude dans VS Code (pas le relais), le 23/11/2026 depuis 13 h 44, à sa demande :
> *« tout le système autonome de Glucose qu'on a créé, qui a pour but de tourner H24, eh bien là, il
> ne fonctionne tout simplement PAS… c'est vraiment quelque chose à expressément corriger. Attention
> aussi à ne jamais avoir de double session !!!! »* Le veilleur a été arrêté pendant le travail (le
> battement le disait), puis relancé sur ce code.

## 1. En arrivant — ce qui s'était passé

* **Le veilleur tournait depuis 8 h 32, aucune session tmux.** Le journal : à 10 h 06, le compte a à
  90 % sur 5 h → passation demandée → session close à 10 h 08 ; le compte b plein de sa semaine
  (100 % jusqu'à 16 h). Rien d'autre jusqu'à 13 h 44.
* **10 h 08 → 13 h 20, trois heures perdues** : le relais a quitté le compte a à 91 % pour un compte
  qui ne pouvait pas le prendre, puis a attendu la remise de 13 h 20.
* **10 h 10 → 11 h 17, il écrivait depuis son autre appareil** (le co-working, signé « Aigle
  Émeraude », 127 gestes) : une membrane *URGENT IMPORTANT* — *« tu marches actuellement en
  autonomie, j'aimerais que tu me répondes et que tu poses tes questions ici sur le Glucose, mais
  surtout stp sois actif sur le Glucose car j'ai besoin de toi »* —, sept notes (les projets récents,
  VS Code, les scripts, le miroir de fichiers, le MCP, la pipette), la consigne retouchée (*« mets à
  jour le Glucose sur tous les points et réponds aux questions […] et aussi mets à jour le GitHub »*),
  et, à 11 h 13, les nœuds `relais-mode-emploi` et `relais-suivi` **retirés**.
* **13 h 20 → 13 h 44, le compte a libre, rien** : le veilleur répétait *« la consigne n'est pas signée
  par toi »*. `auteur_recent(…) == "humain"` : son geste venu de l'autre appareil porte le nom de
  l'appareil, pas « humain ». Sa propre consigne passait pour celle d'une IA.
* À 13 h 57, il répondait à Q08, Q10, Q18, Q19 depuis ce PC — que personne n'aurait lus non plus :
  rien dans le relais ne lisait ce qu'il écrit, hors du nœud `relais-reponse`.

## 2. Remis en question

* **« Seul l'humain signe »** : l'intention était *jamais une IA*. Elle se garde sans exclure ses autres
  appareils : une IA, même sur l'autre appareil, arrive « agent … » (`Lettre::main_chez_moi`), et l'IA
  de Glucose porte son nom. **Reconnaître SES appareils** plutôt que tout humain : écarté — la marque
  d'un participant (« f5fn ») est dans chacun de ses identifiants, son nom se choisit ; rien ne
  distingue son téléphone d'un autre humain qui aurait le code de la salle, et une liste de ses
  appareils serait une constante qui ne protège de rien de plus. Le risque qui reste est dit (§ 7).
* **Voir qu'une session à lui travaille** : par les processus (une conversation de VS Code vit des
  heures sans rien faire — la 9558, ouverte depuis 8 h 31), par les journaux de conversation seuls
  (une conversation du relais à peine close passerait pour la sienne) — écartés. **Son message est le
  signal** : le crochet « UserPromptSubmit », pour une session hors relais dans ce dépôt ; la durée,
  la loi qui existait (`SILENCE`, aucune commande de plus de 30 minutes), mesurée sur le journal de sa
  conversation, que Claude Code écrit à chaque outil ; `rendre` pour ne pas l'attendre.
* **Tuer la session du relais quand il arrive** : écarté — elle écrit sa passation (une à deux
  minutes), et sa session l'apprend, le crochet imprimant l'avertissement dans son contexte.
* **La relance par le crochet « Stop »** : le binaire de l'extension 2.1.330 coupe au 8ᵉ blocage de
  suite (`CLAUDE_CODE_STOP_HOOK_BLOCK_CAP ?? 8`) et le dit : *« check stop_hook_active in the input and
  return success while it's true »*. Lever le plafond (`=0`) : écarté, il protège d'une boucle. Le
  crochet relance une fois, le veilleur tape la suivante : le compte de Claude Code repart de zéro.
* **Les 200 derniers gestes** : une constante qui pouvait disparaître — le veilleur lit tous les gestes
  qu'il n'a pas vus (`vu`), deux lectures.
* **Le seuil de 90 %** : gardé (une passation coûte un tour), mais il ne ferme plus la session que si
  un autre compte peut la prendre ; sinon elle va jusqu'au mur, prévenue une fois.
* **Le mode d'emploi dans son Glucose** : un geste l'a effacé. Un fait, un seul endroit — dans le dépôt.

## 3. La recherche (consultée le 23/11/2026)

* Claude Code, [*Hooks*](https://code.claude.com/docs/en/hooks) : « UserPromptSubmit » reçoit
  `session_id`, `transcript_path`, `cwd`, `prompt` ; *« Claude Code adds plain-text stdout as context
  that Claude can see and act on »* ; délai par défaut ramené à 30 s. « SessionEnd » : un budget de
  1,5 s, rien sur la fenêtre de VS Code qu'on ferme — non retenu.
* Le binaire de l'extension (2.1.330) : `s.CLAUDE_CODE_STOP_HOOK_BLOCK_CAP??8`, *« A hook blocked the
  turn from ending N consecutive times—overriding and ending turn »* ; `--session-id <uuid>` existe
  (non retenu : le bail n'en a pas besoin).
* `flock(2)` : le verrou suit le fichier ouvert et meurt avec le processus.

## 4. Ce qui est écrit (`outils/relais/`)

* **`loi.py`** (ce qui décide, sans effet de bord) et **`machine.py`** (Git, le jeton, le Claude de VS
  Code), extraits de `relais.py`, qui passait ses six cents lignes (598).
* **RELAIS-MAINS** : `humaine()` — « humain » ou le nom d'un appareil du canevas partagé, jamais
  « agent … » ni « l'IA de Glucose » ; pour la consigne et pour `relais-reponse`. Le ❤️ dit de qui
  vient la consigne suivie.
* **UNE-SEULE** : `relais.py parole` (le crochet, posé par l'installeur), `rendre`, `main_humaine()` ;
  le veilleur demande la passation (« retrait »), ferme sans changer de compte, ne lance rien ; le
  crochet « Stop » cède aussi ; un verrou pour le veilleur ; `lancer()` refuse s'il en vit une.
* **SES-NOTES** : `gestes_neufs()` (tous), `ses_paroles()` (ses nœuds posés ou changés, chacun une fois,
  ceux qu'il retire en sortent) ; donnés au premier message de la session suivante (*« réponds-lui
  avant toute autre tâche »*), ou tapés dans celle qui travaille quand il a fini d'écrire.
* **RELAIS-PLAFOND**, **JUSQU-AU-MUR** (`ailleurs()`, `choisir()` en deux passes, « prevenir ») ; la
  relance tapée **une fois** (avant, « Reprends selon sa consigne » se retapait à chaque minute) ;
  l'histoire lue avant les nœuds ; l'heure porte le jour d'une remise lointaine.
* `MODE-EMPLOI.md` ; le `CLAUDE.md` du dépôt y renvoie, avec la règle d'une seule session.

## 5. Vérifié

* **57 épreuves** (31 avant) ; **32 sabotages neufs sur 32** (`relais-132`) — le premier jeu en
  laissait un aveugle (le crochet au seuil sans personne pour reprendre), son épreuve est écrite ;
  les **15 d'avant**, portés à leur nouvelle adresse, **15 sur 15** (`relais`, `relais-131`).
* **Le crochet, dans un vrai Claude Code** (2.1.330, `-p`, une session tmux factice `relais-b`) : le
  bail pris sous son identifiant et son journal ; l'avertissement reçu mot pour mot dans son contexte.
* **Le vrai veilleur relancé** (14 h 12) : le verrou ; la consigne *« écrite par Aigle Émeraude, le
  23/11/2026 11:17 »* ; **31 nœuds de lui relevés** depuis le geste 213 ; « en retrait » devant cette
  session. La suite — `rendre`, la session lancée avec ses notes — se regarde après le commit.

## 6. Vu, pas réglé

* **Le H24 reste borné par les deux quotas** : b plein de sa semaine jusqu'au 23/11 à 16 h, a à 77 % de
  la sienne. Le relais ne dort plus pour rien ; il ne fabrique pas de quota.
* Les nœuds d'un autre tableau : la liste dit `tableau=main` (« Dossier » n'y est pas).
* Un participant dont le Glucose mentirait (déclarer « humain » pour un agent) : seul le code de la
  salle l'en sépare.
* Une session à lui que VS Code laisse ouverte sans lui écrire ne prend pas la main : juste.

## 7. Choix pris seul, à contester

* **Tout humain du canevas partagé signe la consigne** : quiconque a le code de la salle de ce
  document peut diriger le relais. Le ❤️ dit toujours de qui vient la consigne suivie.
* **Une session à lui prend la main dès son premier message**, même pour une simple question.
* **35 minutes de silence** avant que le relais reprenne, sauf `rendre`.

## 8. Après le commit — la première session, regardée

* **14 h 15**, la main rendue : la session est lancée sur le compte a avec ses 31 nœuds. **De 14 h 23 à
  14 h 38, elle lui répond dans son Glucose**, une réponse et une flèche par note (`relais-rep-…`),
  puis reprend le plan : MCP-DOSSIER (fiche 133, `9accdfe`), dont `ses_paroles()` qui ne prend plus le
  nom d'un dossier (« Dossier ») pour un identifiant.
* **14 h 41, un défaut vu en vrai** : ma vérification, réveillée par la fin de sa propre commande en
  arrière-plan, est passée par « UserPromptSubmit » — **une notification de tâche prenait la main** et
  aurait fait s'effacer le relais à chaque vérification. Rendue à 14 h 41 min 26 s, avant le tour du
  veilleur. **Corrigé** (`NOTIFICATIONS`, `loi.py`) : seul ce qu'il écrit prend la main ; une session à
  lui qui travaille la garde par son journal. Épreuve et sabotage (33 sur 33).
* **Corrigé sans double session** : la main reprise exprès à 14 h 44, **la passation demandée,
  écrite, la session close à 14 h 50** — elle avait commité son travail d'abord, copie de travail
  propre. Le passage de main d'une session à l'autre, regardé de bout en bout.
* **Un piège de `saboter.py`** : Python croit sa copie compilée tant que la date (à la seconde) et la
  taille du source n'ont pas changé ; « min » pour « max », rétabli dans la même seconde, jouait le
  sabotage sur le code intact (le « libre à la plus proche remise » passait pour aveugle, puis le
  témoin tombait). `jouer()` compile désormais dans un dossier vierge (`PYTHONPYCACHEPREFIX`) :
  **les 50 sabotages du relais (132, 131, relais, 133) tombent, trois fois de suite**.
