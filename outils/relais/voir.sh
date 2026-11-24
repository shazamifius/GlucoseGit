#!/usr/bin/env bash
# **La vue du relais** : ce que montre l'onglet de terminal de VS Code (.vscode/tasks.json, local à ce
# poste, relance ce fichier toutes les 20 secondes et le relit à chaque fois : un changement ici vaut
# sans recharger VS Code).
#
# La session du relais qui vit, **quel que soit son compte** : il n'y en a qu'une à la fois (sa parole
# du 23/11), et le relais passe d'un compte à l'autre. Jusqu'au 24/11, chaque VS Code ne suivait que
# la session de son propre compte : celui du compte a est resté vide de 23 h 25 à 9 h (sa semaine
# pleine) pendant que le compte b travaillait, et il a cru le relais en panne.
# Sans session : ce que dit le ❤️ (`relais-battement`), lu du veilleur — pourquoi, et jusqu'à quand.
session=$(tmux ls -F '#S' 2>/dev/null | grep -m1 '^relais-')  # le nom : `nom_tmux`, dans relais.py
if [ -n "$session" ]; then
    exec tmux attach -t "$session"
fi
printf '\033[H\033[2J'
python3 -I - "${RELAIS_ETAT:-$HOME/.local/state/relais}/veilleur.json" <<'FIN'
import json, sys, time
try:
    memoire = json.load(open(sys.argv[1]))
except (OSError, ValueError):
    memoire = {}
print("🤖 Le relais — aucune session ne travaille en ce moment.\n")
print("❤️ ", memoire.get("dit") or "le veilleur n'a encore rien dit")
if memoire.get("battu"):
    print(f"\n(son dernier battement : {time.strftime('%H:%M', time.localtime(memoire['battu']))} ; "
          "cette vue se rafraîchit toutes les 20 secondes, et montre la session dès qu'elle part)")
FIN
