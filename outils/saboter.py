"""Sabote une épreuve (fiche 46 § 5) : remplace un passage, lance les tests, exige qu'ils tombent, restaure.

Usage : python saboter.py <fichier_json>
Le JSON : [{"nom":..., "fichier":..., "avant":..., "apres":..., "cmd":[...]}]
Plusieurs passages cassés ensemble (deux gardes qui se couvrent l'une l'autre, fiche 100) :
          [{"nom":..., "remplacements":[{"fichier":..., "avant":..., "apres":...}, ...], "cmd":[...]}]
"""
import json
import os
import subprocess
import sys

# La racine du dépôt : le dossier parent de celui-ci.
RACINE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


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
        return "TOMBE" if r.returncode != 0 else "PASSE (épreuve aveugle !)"
    finally:
        for chemin, texte in originaux.items():
            with open(chemin, "w", encoding="utf-8", newline="") as f:
                f.write(texte)
            os.utime(chemin, None)


def main():
    with open(sys.argv[1], encoding="utf-8") as f:
        sabotages = json.load(f)
    for sab in sabotages:
        print(f"{sab['nom']:60s} {lancer(sab)}", flush=True)


main()
