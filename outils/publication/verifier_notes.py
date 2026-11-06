"""Les notes de version parlent-elles de cette version ? (NOTES-1, fiche 84)

Usage : python verifier_notes.py <notes.md> <dernière étiquette publiée>

La 2.0.3-beta.6 est partie avec les notes de la beta 5 : `notes.md` s'écrit à la main, et rien ne
disait qu'on l'avait oublié. Elles disent « depuis la <version d'avant> » : si ce n'est plus la
dernière version publiée, elles sont celles d'une version passée, et la publication s'arrête
avant d'avoir construit quoi que ce soit.
"""
import sys


def probleme(notes, etiquette):
    """Ce qui ne va pas, ou None."""
    if "{version}" not in notes:
        return "les notes ne portent pas {version}"
    precedente = etiquette.removeprefix("v")
    if f"depuis la {precedente}" not in notes:
        return f"les notes ne disent pas « depuis la {precedente} » : les réécrire"
    return None


def principal():
    with open(sys.argv[1], encoding="utf-8") as f:
        p = probleme(f.read(), sys.argv[2])
    if p:
        print(f"::error::{p}")
        return 1
    print("les notes parlent de cette version")
    return 0


if __name__ == "__main__":
    sys.exit(principal())
