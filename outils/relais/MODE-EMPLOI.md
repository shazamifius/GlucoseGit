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
   donné à la session qui travaille (tapé quand il a fini d'écrire) ou à la suivante (dans son
   premier message). **Elle lui répond dans son Glucose avant toute autre tâche.**
4. **Les jauges** (la barre d'état, `relais.py jauges`) : un compte à 90 % sur 5 h ou sur la semaine
   passe la main **si l'autre peut la prendre** ; sinon il continue jusqu'au mur (100 %), prévenu une
   fois — commiter chaque étape, tenir la passation à jour. Au mur, la session est close. **La mémoire
   du modèle** (le contexte de la conversation) à 90 % : la passation, puis une conversation neuve.
5. **La relance** : le crochet « Stop » (`relais.py fin-de-tour`) relance une fois ; la suivante est
   tapée par le veilleur — Claude Code coupe au 8ᵉ blocage de suite (`CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`).
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
la passation, la récrire (ou la confirmer d'une ligne datée), puis finir le tour. Ne jamais écrire
dans `relais-consigne`.

## Pour une session à lui

Si une session du relais vivait, le premier message le dit (« [Relais] … ») : attendre qu'elle ait
écrit sa passation et soit close (`relais.py etat` : plus de session tmux) avant de toucher au
dépôt, puis lire sa passation. **En partant, rendre la main** : `python3 -I outils/relais/relais.py
rendre` — sinon le relais attend 35 minutes de silence.

## Les commandes

```bash
python3 -I outils/relais/relais.py etat      # comptes, session du relais, session à lui, mémoire
python3 -I outils/relais/relais.py rendre    # une session à lui rend la main
systemctl --user status relais-glucose       # le service ; restart : KillMode=process, tmux survit
tail ~/.local/state/relais/journal.txt       # ce qu'il a fait ; ses fichiers d'état à côté
tmux capture-pane -p -t relais-a             # l'écran de la session (lire, jamais taper à sa place)
bash outils/relais/voir.sh                   # ce que montrent ses deux VS Code : la session qui vit, ou pourquoi aucune
python3 -m unittest discover -s outils/relais -p test_relais.py   # ses épreuves
python3 outils/saboter.py outils/sabotages/relais-132.json        # et leurs sabotages
```

## Les pièges payés

- **La vue de VS Code suit la seule session qui vit, quel que soit son compte** (`voir.sh`, relu toutes
  les 20 secondes par `.vscode/tasks.json`) : jusqu'au 24/11, chaque VS Code ne suivait que la session de
  son compte, et celui du compte a, vide toute une nuit (sa semaine pleine), lui a fait croire le relais
  en panne (fiche 141).
- `python3 -I` ne met pas le dossier du programme dans le chemin : `relais.py` l'ajoute à la fin.
- Les crochets se lisent au lancement d'une session : un crochet ajouté ne vaut que pour les suivantes.
- Le veilleur relancé (`systemctl --user restart`) lit le code neuf ; les crochets le lisent à
  chaque appel — un `relais.py` cassé casse aussi la barre d'état.
- Une fenêtre de jauge pleine se vide à sa remise ; une jauge jamais lue compte pour libre : le
  lancement la lit (la barre d'état ne la donne qu'après la première réponse).
- La fin d'une commande en arrière-plan réveille une session par « UserPromptSubmit » : elle ne prend
  pas la main (`NOTIFICATIONS`) — sinon une session qui ne fait que veiller interromprait le relais.
- `saboter.py` compile Python dans un dossier vierge : une copie compilée périmée jouait le
  sabotage sur le code intact quand le remplacement avait la même taille.
