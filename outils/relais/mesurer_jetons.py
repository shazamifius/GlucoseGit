"""**Ce que la jauge compte** (fiche 147) — l'histoire des jauges croisée avec les jetons des conversations.

    python3 -I outils/relais/mesurer_jetons.py [pas]   # pas : les points de jauge par intervalle (5)

Pour chaque intervalle de la fenêtre de 5 h (au moins `pas` points, pour noyer l'arrondi au point près) :
les requêtes au modèle, ce qu'elles ont relu (lecture du cache), ajouté (écriture du cache) et écrit
(pensée comprise). Puis, pour chaque poids possible de la relecture, l'« équivalent » d'un point de
jauge et sa dispersion d'un intervalle à l'autre : le poids qui disperse le moins est ce que
l'abonnement compte (fiche 142 : 0,1, le prix de l'API ; un relevé public d'avril : 0). Les heures de
pointe (14 h - 20 h à Paris en semaine, où les 5 h se vident plus vite) sont marquées d'une étoile.

Limite : toutes les conversations du dossier comptent — une seule session à la fois, mais une session
à lui sur le même compte, ou claude.ai, gonflent un intervalle.
"""
import datetime
import glob
import json
import os
import statistics
import sys
from pathlib import Path

MAISON = Path.home()
HISTOIRE = MAISON / ".local/state/relais/jauges-histoire.jsonl"
JOURNAUX = (MAISON / ".claude/projects", MAISON / ".claude-compte-b/projects")
POIDS = (0, 0.02, 0.05, 0.1, 0.2)  # la relecture ; ajouter au cache : 2 ; écrire : 5 (prix de l'API)


def requetes(depuis):
    """Chaque requête depuis cet instant : (instant, relu, ajouté, écrit), une fois par message."""
    vues = {}
    for racine in JOURNAUX:
        for f in glob.glob(f"{racine}/**/*.jsonl", recursive=True):
            if os.path.getmtime(f) < depuis:
                continue
            for ligne in open(f, errors="ignore"):
                try:
                    e = json.loads(ligne)
                except ValueError:
                    continue
                m = e.get("message") or {}
                u = m.get("usage")
                if e.get("type") == "assistant" and u and m.get("id"):
                    t = datetime.datetime.fromisoformat(e["timestamp"].replace("Z", "+00:00")).timestamp()
                    vues[m["id"]] = (t, u.get("cache_read_input_tokens", 0),
                                     u.get("cache_creation_input_tokens", 0) + u.get("input_tokens", 0),
                                     u.get("output_tokens", 0))
    return sorted(vues.values())


def intervalles(points, pas):
    """Les intervalles d'une même fenêtre de 5 h, d'au moins `pas` points de jauge chacun."""
    debut = None
    for p in points:
        if debut is None or p["remise"] != debut["remise"] or p["five_hour"] < debut["five_hour"]:
            debut = p
        elif p["five_hour"] - debut["five_hour"] >= pas:
            yield debut, p
            debut = p


def pointe(t):
    h = datetime.datetime.fromtimestamp(t, datetime.timezone(datetime.timedelta(hours=1)))
    return h.weekday() < 5 and 14 <= h.hour < 20


def main(pas=5):
    points = [json.loads(l) for l in HISTOIRE.read_text().splitlines() if l.strip()]
    points = [p for p in points if p.get("five_hour") is not None]
    if not points:
        return print("aucune histoire de jauge")
    toutes = requetes(points[0]["t"] - 60)
    lignes = []
    print("intervalle (Paris)       compte  Δ%  requêtes  req/%  relu(M)  ajouté(k)  écrit(k)")
    for a, b in intervalles(points, pas):
        dedans = [r for r in toutes if a["t"] < r[0] <= b["t"]]
        d = b["five_hour"] - a["five_hour"]
        relu, ajoute, ecrit = (sum(r[i] for r in dedans) for i in (1, 2, 3))
        lignes.append((d, relu, ajoute, ecrit))
        heure = lambda t: datetime.datetime.fromtimestamp(t).strftime("%d/%m %H:%M")
        etoile = "*" if pointe(a["t"]) else " "
        print(f"{heure(a['t'])}-{heure(b['t'])[6:]}{etoile} {a['compte']:>5} {d:4d} {len(dedans):8d} "
              f"{len(dedans) / d:6.1f} {relu / 1e6:8.2f} {ajoute / 1e3:9.0f} {ecrit / 1e3:9.0f}")
    if len(lignes) < 3:
        return print("trop peu d'intervalles pour juger (il en faut trois)")
    print("\npoids de la relecture   un point de jauge (k-équiv.)   dispersion")
    for w in POIDS:
        v = [(relu * w + ajoute * 2 + ecrit * 5) / d / 1e3 for d, relu, ajoute, ecrit in lignes]
        moyenne = statistics.mean(v)
        print(f"{w:>20}   {moyenne:27.1f}   {statistics.pstdev(v) / moyenne:9.1%}")


if __name__ == "__main__":
    main(int(sys.argv[1]) if len(sys.argv) > 1 else 5)
