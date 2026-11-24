"""**Relancer son Glucose** — le seul geste pour lui donner un binaire neuf (sa note du 24/11 :
« j'ai encore été déconnecté de la sync le 24/11 à 12 h 10 », l'heure d'une relance du relais).

Son Glucose tourne en service (`glucose.service`, `Restart=always`) : il se relance par systemd,
jamais en le tuant puis en le lançant à la main — le service en rouvre un second dix secondes plus
tard (le 24/11 à 13 h 30 : deux fenêtres sur le même document, treize minutes). Chaque relance
s'écrit dans son Glucose (`relais-relances`) avec l'heure, la raison, et ce qu'a fait le partage :
ce PC quitte sa salle et la rejoint, et chez lui, de loin, quelque chose coupe — l'heure exacte
de chaque relance est ce qui le trouvera.

    python3 -I outils/relais/relancer_glucose.py "<raison>"
"""
import json
import os
import subprocess
import sys
import time

NOEUD = "relais-relances"
SORTIE = os.path.expanduser("~/sortie-glucose.txt")
TITRE = ("🔄 **Les relances de ton Glucose sur ce PC** (le relais les écrit, la plus récente en "
         "haut ; ta note du 24/11 sur la synchronisation)")
GARDEES = 12  # les plus récentes ; l'histoire de Glucose garde les autres


def glucose(*args, entree=None):
    r = subprocess.run(["glucose", *args], input=entree, capture_output=True, text=True, timeout=60)
    try:
        return json.loads(r.stdout)
    except ValueError:
        return {"erreur": (r.stdout + r.stderr).strip()}


def partage_apres(depuis):
    """Ce que la sortie du Glucose relancé dit du partage, une fois sa fenêtre ouverte."""
    for _ in range(60):
        if os.path.exists(SORTIE) and os.path.getmtime(SORTIE) >= depuis:
            with open(SORTIE, encoding="utf-8", errors="replace") as f:
                lignes = f.read().splitlines()
            if any("chronique de cette session" in l for l in lignes):
                for l in lignes:
                    if "canevas partagé" in l:
                        return l.replace("[Glucose] canevas partagé : ", "partage ")
                return "aucun partage repris"
        time.sleep(1)
    return "fenêtre pas ouverte après une minute : à vérifier"


def main():
    raison = " ".join(sys.argv[1:]).strip()
    if not raison:
        sys.exit("dire la raison de la relance")
    debut = time.time()
    subprocess.run(["systemctl", "--user", "restart", "glucose.service"], check=True)
    partage = partage_apres(debut)
    lu = glucose("lire", "tableau=main", f"ids={NOEUD}", "niveau=complet")
    ancien = (lu.get("noeuds") or [{}])[0].get("texte", "")
    lignes = [l for l in ancien.splitlines() if l.startswith("•")]
    ligne = f"• {time.strftime('%d/%m %H h %M')} — {raison} — {partage}"
    texte = "\n\n".join([TITRE, "\n".join([ligne, *lignes][:GARDEES])])
    pose = glucose("poser-texte", "tableau=main", f"id={NOEUD}", "dans=relais", "largeur=640",
                   "texte=-", entree=texte)
    print(ligne if "erreur" not in pose else f"{ligne}\n(pas écrit dans son Glucose : {pose['erreur']})")


if __name__ == "__main__":
    main()
