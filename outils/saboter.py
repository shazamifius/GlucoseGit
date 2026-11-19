"""Sabote une épreuve (fiche 46 § 5) : remplace un passage, lance les tests, exige qu'ils tombent, restaure.

Usage : python saboter.py <fichier_json>
        python saboter.py --temoins <fichier_json>…   (le code intact seul, chaque commande une fois)
Chaque commande se joue d'abord sur le code intact (le témoin, fiche 125) : si elle tombe déjà, ou si
aucune épreuve ne répond à son filtre, ses sabotages ne sont pas joués (fiche 126).
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


def jouer(cmd):
    r = subprocess.run(cmd, cwd=RACINE, capture_output=True, text=True,
                       encoding="utf-8", errors="replace")
    return r.returncode, r.stdout + r.stderr


def tombees(sortie):
    """**Les épreuves tombées, par leur nom** : la ligne d'échec de l'exécuteur — libtest de Rust
    (« test x ... FAILED »), `node --test` (« not ok 3 - x »), un essai qui écrit ses verdicts
    (« NON … », `outils/porte/essai/`).

    Un code de sortie non nul ne suffit pas (fiche 125) : une édition de liens ratée, un processus
    tué par la borne (137), Nix qui échoue sortent aussi en erreur, et ne disent rien de la garde.
    Et le nom se dit (fiche 126) : une épreuve fragile, étrangère au sabotage, qui tombe au même
    moment ne doit pas passer pour la garde qui tient.
    """
    noms = re.findall(r"^test (\S+) \.\.\. FAILED$", sortie, re.MULTILINE)
    noms += re.findall(r"^\s*not ok \d+ - (.+)$", sortie, re.MULTILINE)
    noms += re.findall(r"^(NON\b.{0,80})", sortie, re.MULTILINE)
    if not noms and ("test result: FAILED" in sortie or "panicked at" in sortie):
        noms = ["(une épreuve, sans nom lisible)"]
    return list(dict.fromkeys(noms))


def passees(sortie):
    """Combien d'épreuves ont passé, d'après l'exécuteur ; None s'il ne le dit pas (un essai)."""
    rust = re.findall(r"test result: ok\. (\d+) passed", sortie)
    node = re.findall(r"^\s*(?:#|ℹ) pass (\d+)", sortie, re.MULTILINE)
    return sum(int(n) for n in rust + node) if rust or node else None


def dire(noms):
    return ", ".join(noms[:3]) + (f" (+{len(noms) - 3})" if len(noms) > 3 else "")


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
        code, sortie = jouer(sab["cmd"])
        if "error[" in sortie or "could not compile" in sortie:
            return "NE COMPILE PAS"
        if code == 0 and "--" in sab["cmd"]:
            # Une épreuve graphique qui n'obtient pas la carte se saute, et compte comme réussie :
            # en parallèle, plusieurs se la disputent (fiche 110). Avant de dire une épreuve
            # aveugle, la rejouer seule, un fil à la fois — et exiger la même preuve (fiche 126).
            code, sortie = jouer(sab["cmd"] + ["--test-threads=1"])
            if code != 0 and tombees(sortie):
                return f"TOMBE, un fil à la fois (une épreuve graphique s'était sautée) : {dire(tombees(sortie))}"
        if code == 0:
            return "PASSE (épreuve aveugle !)"
        if tombees(sortie):
            return f"TOMBE : {dire(tombees(sortie))}"
        dernieres = [l for l in sortie.splitlines() if l.strip()][-1:] or [""]
        return f"ÉCHOUE SANS ÉPREUVE TOMBÉE (code {code}) : {dernieres[0][:120]}"
    finally:
        for chemin, texte in originaux.items():
            with open(chemin, "w", encoding="utf-8", newline="") as f:
                f.write(texte)
            os.utime(chemin, None)


def temoin(cmd):
    """Le témoin : la commande sur le code intact. Elle doit passer, et jouer au moins une épreuve.

    Le 19/11 (fiche 125), une épreuve tombait déjà sur sa précondition, partout : ses sabotages ont
    dit « TOMBE » huit fois sur huit, et le reste de l'épreuve n'avait jamais tourné. MÉTHODE § 4.1
    le demandait (« toute épreuve neuve se lance d'abord sur le code intact ») ; l'outil l'exige.
    Un filtre qui ne trouve aucune épreuve passe aussi : chaque sabotage coûterait une compilation
    pour dire « aveugle » d'une épreuve qui n'a pas tourné (fiche 126).
    Rend (None, combien d'épreuves ont passé) si le témoin tient, sinon (pourquoi, les lignes).
    """
    code, sortie = jouer(cmd)
    if code != 0:
        lignes = sortie.splitlines()
        return "tombe", [l for l in lignes if "panicked at" in l or "error[" in l or "FAILED" in l][:6] or lignes[-6:]
    n = passees(sortie)
    if n == 0:
        return "vide", ["aucune épreuve ne répond à ce filtre"]
    return None, n


def main():
    seulement_les_temoins = sys.argv[1] == "--temoins"
    fichiers = sys.argv[2:] if seulement_les_temoins else sys.argv[1:2]
    sabotages = []
    for fichier in fichiers:
        with open(fichier, encoding="utf-8") as f:
            sabotages += [dict(s, fichier_json=os.path.basename(fichier)) for s in json.load(f)]
    temoins = {}
    for sab in sabotages:
        cle = tuple(sab["cmd"])
        if cle not in temoins:
            temoins[cle] = temoin(sab["cmd"])
            raison, detail = temoins[cle]
            commande = " ".join(cle)
            if raison is None:
                combien = "un nombre d'épreuves qu'il ne dit pas" if detail is None else f"{detail} épreuve(s)"
                print(f"TÉMOIN QUI TIENT — {combien} sous « {commande} » ({sab['fichier_json']})", flush=True)
            else:
                quoi = "TOMBE DÉJÀ" if raison == "tombe" else "NE JOUE AUCUNE ÉPREUVE"
                print(f"TÉMOIN QUI {quoi} — le code intact, sous « {commande} » ({sab['fichier_json']}) :",
                      *("    " + l for l in detail), sep="\n", flush=True)
        if seulement_les_temoins:
            continue
        verdict = lancer(sab) if temoins[cle][0] is None else "SANS TÉMOIN (voir plus haut)"
        print(f"{sab['nom']:60s} {verdict}", flush=True)
    sys.exit(1 if any(t[0] is not None for t in temoins.values()) else 0)


main()
