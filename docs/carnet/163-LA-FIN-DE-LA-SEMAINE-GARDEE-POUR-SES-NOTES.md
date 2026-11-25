# 163 — La fin de la semaine, gardée pour ses notes (25/11, compte a)

> Sa consigne : *« veille aussi au fonctionnement de l'automatisation de Claude afin qu'il soit h24
> actif »* ; sa réponse dans `relais-q-h24-quotas` : *« la perte de token est ma priorité actuelle »*,
> *« l'objectif est de tout marquer sur le Glucose et de faire fonctionner le système autonome »*.

## 1. Ce qui allait arriver

Le 25/11 à 5 h 46 : le compte a à **17 % de sa semaine** (remise le 2/12 à 0 h) après environ trois
heures de travail ; le compte b à **99 %** jusqu'au 30/11 à 16 h. Au rythme mesuré (une semaine ≈ 7 à
8 fenêtres de cinq heures, fiche 141 § 3), la semaine de a est pleine vers le 26/11 à midi. Jusque-là,
le relais allait **jusqu'au mur** quand aucun autre compte ne pouvait reprendre (« faute de mieux, un
compte sous le mur », 23/11) : puis **quatre jours sourd**, du 26/11 au 30/11 à 16 h. Une note de lui
pendant ces quatre jours n'aurait reçu aucune réponse.

Le travail d'une semaine est borné par sa jauge, quoi qu'on fasse : l'étaler n'en donne pas plus. Ce
qu'on peut choisir, c'est **à quoi sert sa fin**. La brûler au travail rapporte environ une heure de
chantier ; la garder rend le relais capable de lui répondre pendant les quatre jours.

## 2. RESERVE-1

**Quand la semaine d'un compte a passé son seuil (90 %, sa règle du 23/11) et qu'aucun autre ne peut
reprendre, ce qui en reste ne sert plus qu'à lui répondre** (`relais.gardee`) :

* une session au travail qui y entre écrit sa passation et s'arrête (`garder` → « reserve »), au lieu
  d'aller jusqu'au mur ;
* aucune session ne se lance pour le travail (`choisir`) ; le ❤️ dit : « la fin de la semaine du
  compte a est gardée pour tes notes : écris dans ton Glucose, une session te répondra — le travail
  reprend vers … » ;
* **une note de sa main** lance une session (`choisir(…, notes=True)`), marquée `notes` : son premier
  message lui dit de répondre à ses notes et à rien d'autre ; à la fin de son tour, le crochet lui
  demande la passation (motif « reserve »), puis elle est close ;
* **dans la dernière fenêtre de cinq heures avant la remise de la semaine**, la réserve se rend au
  travail : non dépensée, elle serait perdue (sa priorité : les jetons). La seule constante est la
  longueur de la plus courte fenêtre de jauge (`FENETRE`) — ce qu'un compte peut dépenser d'une traite ;
* sous le mur, le compte qui a **le plus de marge** passe d'abord : le 25/11, b gardait 1 % et a 10 %
  — une note confiée à b serait morte au mur avant sa réponse.

Seule la semaine est gardée : cinq heures au seuil se remettent dans moins de cinq heures, la session y
va encore jusqu'au mur (prévenue une fois).

## 3. Ce que ça coûte, et le choix qui reste le sien

Environ 10 % d'une semaine, soit à peu près une heure de chantier, déplacée du travail vers ses
réponses — rendue au travail si elle ne sert pas avant la dernière fenêtre. Une note qui ne demande
rien (un nœud qu'il agrandit) lance aussi une session : comme avant, ses gestes nommés sont des notes.
C'est un défaut argumenté, pas sa décision : s'il préfère le travail jusqu'au mur, `gardee` rend None
et tout redevient comme avant (`relais-reserve` dans sa membrane le lui dit).

Recherche : aucune source à chercher ici, les faits sont mesurés sur ses deux comptes (fiches 141
et 147 : la semaine, et non les cinq heures, borne le H24).

## 4. Les preuves

* 3 épreuves (`test_relais.py` : `Jauges`, `Veilleur`, `Lancement`) ; les 89 du relais vertes.
* **12 sabotages sur 12** (`outils/sabotages/reserve-163.json`). Un passage de `saboter.py` a vu une
  fois un sabotage finir en `-11` (une erreur de segmentation de Python, sans épreuve tombée) ; rejoué
  deux fois, il tombe comme les autres : un accident de la machine, pas de l'épreuve, à surveiller.
* Le veilleur relancé sur ce code le 25/11 à 5 h 51 (`systemctl --user restart relais-glucose`).
  **Pas encore vu en vrai** : la semaine de a doit atteindre 90 % (vers le 26/11). À vérifier alors
  dans le journal (« la fin de la semaine gardée pour ses notes ») et le ❤️.

## 5. Sans les connecteurs de claude.ai (même session)

Ses connecteurs de claude.ai (Gmail, Agenda, Drive, Docs, Sheets, Slides, Hugging Face, AWS, Claude
Docs, et « claude.ai glucose », sa porte publique, qui répondait 503 : sa porte fermée) se chargeaient
dans chaque session du relais : leurs ~110 outils nommés, les ~25 savoir-faire de Hugging Face et leurs
consignes. Mesuré sur cette session : **27 465 jetons au premier tour, 40 853 au second** — environ
13 000 de plus, relus à chaque réponse jusqu'à la passation, pour des outils qu'aucune session du relais
ne sert. Le relais lance maintenant ses sessions avec `ENABLE_CLAUDEAI_MCP_SERVERS=false` (lu dans le
binaire de Claude Code 2.1.332 : la variable, ou le réglage `disableClaudeAiConnectors`). Vérifié :
`claude mcp list` sous cette variable ne montre plus que `glucose --mcp`. Ses sessions à lui gardent
tout. 1 épreuve, son sabotage tombe (13 sur 13 dans `reserve-163.json`) ; veilleur relancé. **À
mesurer** sur la prochaine session lancée : la taille de son deuxième tour.

## 6. RENOUVELER-1 : une conversation neuve, quand la session change de chantier

Sa réponse du 24/11 : *« relire la conversation, c'est vraiment à ne jamais faire »*. Le seuil de
300 000 jetons reste (fiche 147 § 5 : la jauge compte une relecture ~0,02 d'un jeton neuf) ; mais une
session qui finit un chantier à 180 000 jetons et en prend un autre sans rapport relit, à chaque réponse,
tout ce qui ne lui sert plus — environ 3 600 jetons neufs par réponse au poids mesuré, quand une
conversation neuve en écrit ~30 000 une fois : passé une dizaine de réponses, la neuve coûte moins.
`relais.py renouveler` (seulement sous `RELAIS=1`) le note dans `session.json` ; le veilleur (geste
« renouveler ») et le crochet demandent la passation, puis le relais lance une conversation neuve, comme
au seuil des jetons. La session en juge seule, d'après ce que le chantier suivant doit lire (MODE-EMPLOI).
1 épreuve, 4 sabotages (17 sur 17 dans `reserve-163.json`) ; veilleur relancé. **Essayé en vrai par la
session qui l'a écrit**, le 25/11 à 6 h 08 : le journal du relais le dira (« une conversation neuve
demandée »).
