"""Sabote une épreuve (fiche 46 § 5) : remplace un passage, lance les tests, exige qu'ils tombent, restaure.

Usage : python saboter.py <fichier_json>
Chaque commande se joue d'abord sur le code intact (le témoin, fiche 125) : si elle tombe déjà, ses
sabotages ne sont pas joués.
Le JSON : [{"nom":..., "fichier":..., "avant":..., "apres":..., "cmd":[...]}]
Plusieurs passages cassés ensemble (deux gardes qui se couvrent l'une l'autre, fiche 100) :
          [{"nom":..., "remplacements":[{"fichier":..., "avant":..., "apres":...}, ...], "cmd":[...]}]
"""
import json
import os
import re
import subprocess
import sys

# La racine du dépôt : le dossier parent de celui-ci.
RACINE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def epreuve_tombee(sortie):
    """**La preuve qu'une épreuve est tombée** : la ligne d'échec de l'exécuteur — libtest de Rust
    (« test result: FAILED », « panicked at »), `node --test` (« not ok »), un essai qui écrit ses
    verdicts (« NON … », `outils/porte/essai/`).

    Un code de sortie non nul ne suffit pas (fiche 125) : une édition de liens ratée, un processus
    tué par la borne (137), Nix qui échoue sortent aussi en erreur, et ne disent rien de la garde.
    """
    return ("test result: FAILED" in sortie or "panicked at" in sortie
            or re.search(r"^(not ok |NON\b)", sortie, re.MULTILINE) is not None)


def lancer(sab):
    remplacements = sab.get("remplacements") or [sab]
    originaux = {}
    for r in remplacements:
        chemin = os.path.join(RACINE, r["fichier"])
        if chemin not in originaux:
            with open(chemin, encoding="utf-8", newline="") as f:
                originaux[chemin] = f.read()
    actuels = dict(originaux)
    for r in remplacements:
        chemin = os.path.join(RACINE, r["fichier"])
        n = actuels[chemin].count(r["avant"])
        if n != 1:
            return f"INTROUVABLE ({n} occurrences dans {r['fichier']})"
        actuels[chemin] = actuels[chemin].replace(r["avant"], r["apres"], 1)
    try:
        for chemin, texte in actuels.items():
            with open(chemin, "w", encoding="utf-8", newline="") as f:
                f.write(texte)
        r = subprocess.run(sab["cmd"], cwd=RACINE, capture_output=True, text=True,
                           encoding="utf-8", errors="replace")
        sortie = r.stdout + r.stderr
        if "error[" in sortie or "could not compile" in sortie:
            return "NE COMPILE PAS"
        if r.returncode != 0:
            if epreuve_tombee(sortie):
                return "TOMBE"
            dernieres = [l for l in sortie.splitlines() if l.strip()][-1:] or [""]
            return f"ÉCHOUE SANS ÉPREUVE TOMBÉE (code {r.returncode}) : {dernieres[0][:120]}"
        # Une épreuve graphique qui n'obtient pas la carte se saute, et compte comme réussie :
        # en parallèle, plusieurs se la disputent (fiche 110). Avant de dire une épreuve aveugle,
        # la rejouer seule, un fil à la fois.
        if "--" in sab["cmd"]:
            seul = subprocess.run(sab["cmd"] + ["--test-threads=1"], cwd=RACINE,
                                  capture_output=True, text=True, encoding="utf-8",
                                  errors="replace")
            if seul.returncode != 0:
                return "TOMBE (un fil à la fois : une épreuve graphique s'était sautée)"
        return "PASSE (épreuve aveugle !)"
    finally:
        for chemin, texte in originaux.items():
            with open(chemin, "w", encoding="utf-8", newline="") as f:
                f.write(texte)
            os.utime(chemin, None)


def temoin(cmd):
    """Le témoin : la commande sur le code intact. Elle doit passer, sinon « TOMBE » ne prouve rien.

    Le 19/11 (fiche 125), une épreuve tombait déjà sur sa précondition, partout : ses sabotages ont
    dit « TOMBE » huit fois sur huit, et le reste de l'épreuve n'avait jamais tourné. MÉTHODE § 4.1
    le demandait (« toute épreuve neuve se lance d'abord sur le code intact ») ; l'outil l'exige.
    Rend None si le témoin passe, sinon les lignes qui disent pourquoi il tombe.
    """
    r = subprocess.run(cmd, cwd=RACINE, capture_output=True, text=True,
                       encoding="utf-8", errors="replace")
    if r.returncode == 0:
        return None
    sortie = (r.stdout + r.stderr).splitlines()
    return [l for l in sortie if "panicked at" in l or "error[" in l or "FAILED" in l][:6] or sortie[-6:]


def main():
    with open(sys.argv[1], encoding="utf-8") as f:
        sabotages = json.load(f)
    temoins = {}
    for sab in sabotages:
        cle = tuple(sab["cmd"])
        if cle not in temoins:
            temoins[cle] = temoin(sab["cmd"])
            if temoins[cle] is not None:
                print(f"TÉMOIN QUI TOMBE — le code intact échoue déjà sous « {' '.join(cle)} » :",
                      *("    " + l for l in temoins[cle]), sep="\n", flush=True)
        verdict = "SANS TÉMOIN (le code intact tombe)" if temoins[cle] is not None else lancer(sab)
        print(f"{sab['nom']:60s} {verdict}", flush=True)
    sys.exit(1 if any(t is not None for t in temoins.values()) else 0)


main()
