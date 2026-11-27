# Le relais — mode d'emploi

> Pour toute session de Claude qui touche à ce dépôt, celle du relais comme une session à lui.
> Il vivait dans son Glucose (le nœud `relais-mode-emploi`), d'où un geste l'a retiré le 23/11 :
> il vit ici, à un seul endroit. Le détail : fiches 129 à 132 ; le code : `relais.py` (les effets),
> `loi.py` (ce qui décide), `machine.py` (ce qui se lit de la machine).

## Ce qu'il fait

Des sessions de Claude Code travaillent seules sur Glucose, jour et nuit, d'un compte à l'autre
(a : `~/.claude`, b : `~/.claude-compte-b`). Un service, `relais-glucose` (systemd, utilisateur),
tourne `relais.py veiller` : chaque minute, il lit son Glucose, lance une session dans tmux
(`relais-a` ou `relais-b`), la relance, lui passe ce qu'il écrit, passe la main d'un compte à
l'autre, et bat son cœur dans le nœud ❤️ `relais-battement` (membrane « 🤖 Relais »).

## Les règles

1. **Une seule session à la fois** (sa parole du 23/11). Dès qu'il écrit à une session à lui (VS
   Code, un terminal, dans ce dépôt), le crochet « UserPromptSubmit » (`relais.py parole`) lui donne
   la main : la session du relais écrit sa passation et s'arrête, et aucune n'est lancée tant que la
   sienne donne signe de vie (un message, ou une ligne de sa conversation, depuis moins de 35
   minutes). Un seul veilleur (un verrou), jamais deux sessions tmux.
2. **Sa consigne** (`relais-consigne`) n'est suivie que si une main humaine l'a écrite en dernier :
   la sienne devant ce Glucose (« humain ») ou devant un autre appareil du canevas partagé (le nom
   de l'appareil, « Aigle Émeraude »). Jamais une IA (« agent … », « l'IA de Glucose »). **⏸ PAUSE**
   en première ligne arrête le relais.
3. **Ce qu'il écrit est lu.** Chaque nœud qu'il pose ou change, de n'importe quel appareil, est
   donné à la session qui travaille ou à la suivante (dans son premier message), **une fois qu'il a
   fini d'écrire** (un tour du veilleur sans geste de sa main : `loi.il_ecrit`). Un collage — un geste
   qui pose plusieurs nœuds, comme un rangement dans son archive — n'en est pas une (fiche 166).
   **Elle lui répond dans son Glucose avant toute autre tâche.**
4. **Les jauges** (la barre d'état, `relais.py jauges`) : un compte à 90 % sur 5 h ou à 97 % sur la semaine (sa réponse du 25/11)
   passe la main **si l'autre peut la prendre** ; sinon la session écrit sa passation et s'arrête, et le
   relais attend la remise — **jamais au mur** (fiche 167 : poussée jusqu'au mur le 24/11, une session a
   été coupée en plein travail ; ce qu'une fenêtre ne dépense pas reste à la semaine, qui borne tout).
   Ses notes vont jusqu'au bord (99 %, le mur moins une passation). Au mur, la session est close. **Sauf la fin
   d'une semaine** (RESERVE-1, fiche 163) : sa semaine au seuil et personne pour reprendre, le reste ne
   sert qu'à lui répondre — la session écrit sa passation, et une note de lui en lance une qui ne fait
   que lui répondre ; dans la dernière fenêtre de cinq heures avant la remise, la réserve se rend au travail, jusqu'au bord. **La mémoire
   du modèle** (le contexte de la conversation) passé **300 000 jetons** (90 % d'une fenêtre plus
   petite) : la passation, puis une conversation neuve. Chaque réponse relit la conversation entière :
   78 % des jetons au prix de l'API (fiche 142) — mais **la jauge la compte cinq fois moins** (~0,02 d'un jeton neuf, mesuré le 24/11 :
   fiche 147 § 5) : relire ~40 % de la jauge, écrire ~36 %, ajouter ~24 % ; le seuil reste.
5. **La relance** : le crochet « Stop » (`relais.py fin-de-tour`) relance une fois ; la suivante est
   tapée par le veilleur — Claude Code coupe au 8ᵉ blocage de suite (`CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`).
   **Jamais réveillée endormie** (sa note du 25/11) : une session arrêtée dont l'écran n'a pas bougé
   depuis 50 minutes a perdu son cache (une heure) — la réveiller lui ferait tout relire au prix fort.
   Le veilleur la ferme ; une neuve repart de la passation, avec ses notes.
6. **Rien n'attend en silence** : 35 minutes sans un changement à l'écran, l'écran part dans
   `relais-bloque`, et le ❤️ dit « figée » ; ce qu'il écrit dans `relais-reponse` est tapé dans la session.
7. **Jamais bloqué** (sa réponse du 23/11) : les sessions se lancent en **bypass** (jamais le mode
   auto) et avec `--no-chrome` — toute question que Claude Code pose au démarrage fige une session que
   personne ne regarde : la régler par une option à chaque lancement (fiche 134).
8. **Ses textes** : ce que le relais envoie au démarrage, à la passation et à la relance vit dans trois
   nœuds de sa membrane (`relais-texte-depart`, `-passation`, `-relance`) ; sa version part dès que sa
   main l'a écrite en dernier, jamais celle d'une IA.

## Pour une session du relais (RELAIS=1)

Lire sa consigne, la passation (`relais-passation`), puis ses notes si le premier message en donne.
Une question pour lui : un texte dans son Glucose, et passer à autre chose. Quand le relais demande
la passation, la **remplacer**, jamais l'allonger : ce qu'aucune autre source ne dit (le non commité, la
prochaine étape, un piège), sans le récit de ce qui est fait (sa note du 25/11, fiche 166) ; puis finir le tour. Ne jamais écrire
dans `relais-consigne`. **Un chantier fini, et le suivant n'a que faire de ce qu'elle a lu** : écrire la
passation, puis `python3 -I outils/relais/relais.py renouveler` — une conversation neuve reprend, au lieu de
relire à chaque réponse ce qui ne sert plus (RENOUVELER-1, fiche 163 § 6).

**Sobre en jetons** (sa priorité du 24/11) : ne relire ni ses règles ni l'index des mémoires (Claude
Code les charge), seulement la section qu'un travail demande ; aucun compte rendu dans le terminal,
que personne ne lit. **Ce que sa consigne désigne** : les flèches que sa main tire depuis `relais-consigne`,
puis depuis leurs cibles, sont données au démarrage (`loi.ses_fleches`) — son bloc important s'y lit.

## Pour une session à lui

Si une session du relais vivait, le premier message le dit (« [Relais] … ») : attendre qu'elle ait
écrit sa passation et soit close (`relais.py etat` : plus de session tmux) avant de toucher au
dépôt, puis lire sa passation. **En partant, rendre la main** : `python3 -I outils/relais/relais.py
rendre` — sinon le relais attend 35 minutes de silence.

## Les commandes

```bash
python3 -I outils/relais/relais.py etat      # comptes, session du relais, session à lui, mémoire
python3 -I outils/relais/relais.py rendre    # une session à lui rend la main
python3 -I outils/relais/relais.py renouveler  # une session du relais demande une conversation neuve
systemctl --user status relais-glucose       # le service ; restart : KillMode=process, tmux survit
tail ~/.local/state/relais/journal.txt       # ce qu'il a fait ; ses fichiers d'état à côté
tmux capture-pane -p -t relais-a             # l'écran de la session (lire, jamais taper à sa place)
bash outils/relais/voir.sh                   # ce que montrent ses deux VS Code : la session qui vit, ou pourquoi aucune
python3 -I outils/relais/relancer_glucose.py "<raison>"  # relancer SON Glucose (binaire neuf) : par son service, l'heure écrite chez lui
python3 -m unittest discover -s outils/relais -p 'test_*.py'   # ses épreuves
python3 outils/saboter.py outils/sabotages/relais-132.json        # et leurs sabotages
```

## Les pièges payés

- **Son Glucose est un service** (`glucose.service`, `Restart=always`) : le relancer par
  `relancer_glucose.py`, jamais `kill` puis `glucose` à la main — le service en rouvre un second dix
  secondes après (le 24/11 à 13 h 30, deux fenêtres ; la seconde n'avait rouvert aucun document).
  Le relais MCP et `glucose lire` rouvrent Glucose **par son service** quand sa porte se tait (SERVICE-1,
  fiche 144) : avant, ils lançaient une seconde fenêtre pendant chaque relance (14 h 02).
  **Chaque relance coupe peut-être son partage de loin** (sa note du 24/11 : « déconnecté de la sync […]
  12 h 10 », la relance de 12 h 18) : le moins souvent possible — une relance pour plusieurs chantiers —,
  et chacune s'écrit dans `relais-relances`, avec ce qu'a fait le partage (fiche 144).
- **La vue de VS Code suit la seule session qui vit, quel que soit son compte** (`voir.sh`, relu toutes
  les 20 secondes par `.vscode/tasks.json`) : jusqu'au 24/11, chaque VS Code ne suivait que la session de
  son compte, et celui du compte a, vide toute une nuit (sa semaine pleine), lui a fait croire le relais
  en panne (fiche 141).
- **Les cinq nœuds du relais ne s'archivent jamais** (`relais-consigne`, `-passation`, `-battement`,
  `-bloque`, `-reponse` : `loi.py`, ligne 9). Un rangement de sa carte a mis `relais-bloque` dans une
  archive (`board-f5fn-356`) et effacé `relais-reponse` : le 24/11 à 10 h 00, l'alerte d'une session figée
  n'a pas pu se poser (« l'identifiant est déjà pris ailleurs », journal). Remis dans sa membrane le
  24/11 à 10 h 33 ; l'ancienne alerte reste à l'archive sous `relais-bloque-archive-2311`.
- **`glucose_quoi_de_neuf` sans `depuis`, dans une session neuve, rend 200 gestes** (≈ 8 000 jetons,
  24/11 à 14 h 12) : son agent n'a encore rien fait. Lui donner le rang `jusqu_a` que la passation
  écrit ; les notes de sa main, le relais les donne déjà au premier message.
- **Une sonde `claude -p` lancée d'une session du relais hérite `RELAIS=1`** (le crochet de fin de tour la
  relance) ; sans lui, dans le dépôt, elle prendrait la main : `env -u RELAIS`, hors du dépôt. Elle coûte
  (deux sondes, 6 % de la fenêtre de 5 h, fiche 147) : l'histoire des jauges suffit d'ordinaire.
- **Claude se met à jour seul** (MAJ-1, fiche 162) : toutes les six heures, le veilleur installe la
  version neuve de l'extension, l'essaie sans lui écrire (l'invite, sans question), la refuse sinon ;
  `relais-maj-claude` le dit. `code --install-extension anthropic.claude-code` a échoué une fois sur la
  signature du paquet le 25/11 ; nommer la version (`…@2.1.332`) a réussi. Le relais lance la version
  que VS Code dit installée (`extensions.json`), pas le plus grand dossier.
- **Sans les connecteurs de claude.ai** (`ENABLE_CLAUDEAI_MCP_SERVERS=false` au lancement, fiche 163
  § 5) : Gmail, Drive, Hugging Face… pesaient ~13 000 jetons relus à chaque réponse, jamais servis.
- `python3 -I` ne met pas le dossier du programme dans le chemin : `relais.py` l'ajoute à la fin.
- Les crochets se lisent au lancement d'une session : un crochet ajouté ne vaut que pour les suivantes.
- Le veilleur relancé (`systemctl --user restart`) lit le code neuf ; les crochets le lisent à
  chaque appel — un `relais.py` cassé casse aussi la barre d'état.
- Une fenêtre de jauge pleine se vide à sa remise ; une jauge jamais lue compte pour libre : le
  lancement la lit (la barre d'état ne la donne qu'après la première réponse).
- **Une jauge qui dit « plein » se redemande au serveur d'Anthropic** (`serveur.py`, fiche 182), au
  plus toutes les dix minutes par compte : la barre d'état ne la relit que dans une session, et un
  compte cru plein n'en reçoit plus (le 27/11, a gardé à 99 % était remis à 2 % : six heures de
  sommeil). L'adresse de `/usage` (`api/oauth/usage`), avec la connexion du compte, lue jamais écrite ;
  expirée, la jauge gardée reste et le battement le dit. **Des crédits en plus**, activés avec un
  plafond mensuel, font travailler un compte au-delà de son abonnement jusqu'à 97 % du plafond ; sans
  plafond, ils ne comptent pas.
- La fin d'une commande en arrière-plan réveille une session par « UserPromptSubmit » : elle ne prend
  pas la main (`NOTIFICATIONS`) — sinon une session qui ne fait que veiller interromprait le relais.
- `saboter.py` compile Python dans un dossier vierge : une copie compilée périmée jouait le
  sabotage sur le code intact quand le remplacement avait la même taille.
