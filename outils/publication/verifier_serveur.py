"""Le serveur de télémétrie connaît-il ce que cette version écrira ? (CHARGE-1, fiche 86)

Usage : python verifier_serveur.py [adresse]

Le serveur refuse toute session qui porte une sorte de ligne inconnue. Une version publiée avant lui
perdrait donc tous ses journaux, sans que personne le voie. On lui envoie une session **qu'il refusera
de toute façon** : les lignes neuves, puis une ligne que personne ne connaît. Rien ne s'écrit dans sa
base, et sa réponse dit où il s'est arrêté : à la dernière ligne, il connaît les précédentes.
"""
import json
import sys
import urllib.error
import urllib.request

ADRESSE = "https://glucose-boite-noire.ferme-nilslamber.workers.dev"
DEBUT = (
    '{"type":"debut","epoque_ms":1,"version":"0.0.0","systeme":"linux",'
    '"architecture":"x86_64","demarrage_appareil_ms":null}'
)
# À compléter à chaque sorte de ligne neuve.
LIGNES_NEUVES = [
    '{"type":"charge","instant_ms":1,"machine_pct":1,"processus_pct":1,"fil_pct":1,'
    '"coeurs":1,"noeuds":1,"visibles":1}',
]
INCONNUE = '{"type":"la_garde_de_la_publication"}'


def probleme(statut, raison, nombre_de_lignes):
    """Ce qui ne va pas, ou None. Seule la dernière ligne doit avoir été refusée."""
    if statut != 422:
        return f"le serveur devait refuser cette session, il répond {statut} : {raison}"
    attendu = f"ligne {nombre_de_lignes}"
    if not raison.startswith(attendu):
        return f"le serveur ne connaît pas encore tout : « {raison} ». Le déployer avant de publier."
    return None


def demander(adresse, lignes):
    corps = json.dumps(
        {"installation": "0" * 32, "session": "0" * 32, "lignes": "\n".join(lignes) + "\n"}
    ).encode()
    requete = urllib.request.Request(
        f"{adresse}/v1/sessions",
        data=corps,
        headers={"Content-Type": "application/json", "User-Agent": "Glucose-publication"},
    )
    try:
        with urllib.request.urlopen(requete, timeout=30) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()


def principal():
    lignes = [DEBUT, *LIGNES_NEUVES, INCONNUE]
    adresse = sys.argv[1] if len(sys.argv) > 1 else ADRESSE
    statut, corps = demander(adresse, lignes)
    try:
        raison = json.loads(corps).get("raison", corps)
    except (ValueError, AttributeError):
        raison = corps
    p = probleme(statut, str(raison), len(lignes))
    if p:
        print(f"::error::{p}")
        return 1
    print("le serveur connaît ce que cette version écrira")
    return 0


if __name__ == "__main__":
    sys.exit(principal())
