"""Sabote une épreuve (fiche 46 § 5) : remplace un passage, lance les tests, exige qu'ils tombent, restaure.

Usage : python saboter.py <fichier_json>
Le JSON : [{"nom":..., "fichier":..., "avant":..., "apres":..., "cmd":[...]}]
"""
import json
import os
import subprocess
import sys

# La racine du dépôt : le dossier parent de celui-ci.
RACINE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def lancer(sab):
    chemin = os.path.join(RACINE, sab["fichier"])
    with open(chemin, encoding="utf-8", newline="") as f:
        original = f.read()
    if original.count(sab["avant"]) != 1:
        return f"INTROUVABLE ({original.count(sab['avant'])} occurrences)"
    try:
        with open(chemin, "w", encoding="utf-8", newline="") as f:
            f.write(original.replace(sab["avant"], sab["apres"], 1))
        r = subprocess.run(sab["cmd"], cwd=RACINE, capture_output=True, text=True,
                           encoding="utf-8", errors="replace")
        sortie = r.stdout + r.stderr
        if "error[" in sortie or "could not compile" in sortie:
            return "NE COMPILE PAS"
        return "TOMBE" if r.returncode != 0 else "PASSE (épreuve aveugle !)"
    finally:
        with open(chemin, "w", encoding="utf-8", newline="") as f:
            f.write(original)
        os.utime(chemin, None)


def main():
    with open(sys.argv[1], encoding="utf-8") as f:
        sabotages = json.load(f)
    for sab in sabotages:
        print(f"{sab['nom']:60s} {lancer(sab)}", flush=True)


main()
