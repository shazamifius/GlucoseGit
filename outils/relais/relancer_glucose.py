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


def cette_session(depuis):
    """Les lignes de la sortie depuis l'ouverture de la fenêtre relancée (sa dernière chronique) —
    `None` tant qu'elle n'est pas ouverte. Avant, les lignes d'une session d'avant étaient lues :
    « repris au geste n° 442 », le 24/11 à 18 h 53, quand le Glucose relancé disait 814."""
    if not os.path.exists(SORTIE) or os.path.getmtime(SORTIE) < depuis:
        return None
    with open(SORTIE, encoding="utf-8", errors="replace") as f:
        lignes = f.read().splitlines()
    debuts = [i for i, l in enumerate(lignes) if "chronique de cette session" in l]
    return lignes[debuts[-1]:] if debuts else None


def partage_apres(depuis, attente=60):
    """Ce que le Glucose relancé dit du partage : il **revient** dans sa salle, puis, la salle
    arrivée, il y est **revenu** (REVENIR-2, fiche 148). On attend « revenu » au plus une minute."""
    dit = "fenêtre pas ouverte après une minute : à vérifier"
    for _ in range(attente):
        lignes = cette_session(depuis)
        if lignes is not None:
            partage = [l.replace("[Glucose] canevas partagé : ", "partage ") for l in lignes
                       if "canevas partagé" in l]
            if partage and "revenu" in partage[-1]:
                return partage[-1]
            dit = partage[-1] + " — la salle n'est pas encore arrivée" if partage else "aucun partage"
        time.sleep(1)
    return dit


def place(lu):
    """**Sa bulle se choisit à sa naissance** ; ensuite, la place qu'il lui a donnée (fiche 184 § 3).
    Réécrit « dans relais » à chaque relance, le nœud qu'il avait glissé dans « passation » y restait
    pour l'œil et retournait dans « relais » pour l'IA."""
    return [] if (lu or {}).get("noeuds") else ["dans=relais", "largeur=640"]


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
    pose = glucose("poser-texte", "tableau=main", f"id={NOEUD}", *place(lu), "texte=-", entree=texte)
    print(ligne if "erreur" not in pose else f"{ligne}\n(pas écrit dans son Glucose : {pose['erreur']})")


if __name__ == "__main__":
    main()
