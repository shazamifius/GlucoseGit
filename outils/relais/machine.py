"""**Ce que la machine dit**, en lecture seule : le Claude de VS Code, ce que Git sait du travail
(ce que la session a produit, ce que GitHub n'a pas encore), la présence du jeton — jamais sa
valeur. Extrait de `relais.py` le 23/11, qui passait ses six cents lignes.
"""
import json
import os
import subprocess
from pathlib import Path

from loi import heure

MAISON = Path.home()
DEPOT = MAISON / "Projets/GlucoseGit"
EXTENSIONS = MAISON / ".vscode/extensions"
JETON = MAISON / "Projets/github.md"


def lire_json(chemin, defaut):
    try:
        return json.loads(Path(chemin).read_text())
    except (OSError, ValueError):
        return defaut


def ecrire_json(chemin, donnees):
    chemin = Path(chemin)
    chemin.parent.mkdir(parents=True, exist_ok=True)
    provisoire = chemin.with_suffix(".tmp")
    provisoire.write_text(json.dumps(donnees, ensure_ascii=False, indent=1))
    os.replace(provisoire, chemin)


def dernier_signe(bail):
    """Le dernier signe de vie d'une session à lui : son dernier message, ou la dernière ligne de sa
    conversation, que Claude Code écrit à chaque réponse et à chaque outil."""
    try:
        ecrit = Path(bail["transcript"]).stat().st_mtime if bail.get("transcript") else 0.0
    except OSError:
        ecrit = 0.0
    return max(float(bail.get("depuis") or 0), ecrit)


def version(dossier):
    morceaux = dossier.name.split("-")[2] if dossier.name.count("-") >= 2 else "0"
    return tuple(int(x) if x.isdigit() else 0 for x in morceaux.split("."))


def claude_binaire():
    """**Le Claude de la dernière extension VS Code**, jamais celui de la ligne de commande : sa
    parole du 22/11 — le Claude en ligne de commande était en 2.1.223 quand l'extension était en
    2.1.330. L'extension embarque le sien et le met à jour seule : le relais suit VS Code."""
    candidats = sorted(EXTENSIONS.glob("anthropic.claude-code-*/resources/native-binary/claude"),
                       key=lambda p: version(p.parents[2]))
    return str(candidats[-1]) if candidats else "claude"


def jeton_github(maintenant):
    """Le jeton est-il là, et de quand (il le renouvelle tous les 3 jours) — jamais sa valeur."""
    try:
        present = any(l.strip() and not l.lstrip().startswith("#") for l in JETON.read_text().splitlines())
    except OSError:
        present = False
    if not present:
        return "absent — colle-le dans ~/Projets/github.md"
    jours = (maintenant - JETON.stat().st_mtime) / 86400
    return f"présent (posé il y a {jours:.0f} j)" + (" — à renouveler" if jours >= 3 else "")


def git(*arguments):
    """Une lecture de Git, jamais une écriture ; vide si elle échoue ou tarde (le réseau)."""
    try:
        r = subprocess.run(["git", "-C", str(DEPOT), *arguments], capture_output=True, text=True,
                           timeout=30, env={**os.environ, "GIT_TERMINAL_PROMPT": "0"})
    except (OSError, subprocess.TimeoutExpired):
        return ""
    return r.stdout.strip() if r.returncode == 0 else ""


def travail(depuis):
    """Ce que la session a produit : ses commits depuis son début, et ce que GitHub n'a pas encore.
    Lu dans Git, sans dépenser un jeton d'usage — de quoi juger ce que coûte chaque pour-cent.

    **Ce que GitHub a, demandé à GitHub** (`ls-remote`) : les envois passent par l'adresse qui porte
    le jeton, qui ne met jamais `origin/main` à jour ; compter contre lui comptait ce qui était déjà
    parti (le 23/11, « 1 pas encore sur GitHub » quand `c0d093a` y était)."""
    faits = git("log", f"--since=@{int(depuis)}", "--format=%h %s").splitlines()
    distant = git("ls-remote", "origin", "refs/heads/main").split()[:1]
    attente = (git("rev-list", "--count", f"{distant[0]}..main") if distant else "") or "?"
    liste = "".join(f"\n  - {c[:90]}" for c in faits[:5])
    return f"{len(faits)} commit(s) depuis {heure(depuis)} · {attente} pas encore sur GitHub{liste}"
