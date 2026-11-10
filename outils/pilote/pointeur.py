"""Un pointeur virtuel persistant pour un compositeur sans écran (fiche 99) — par `piloter.sh`.

    python3 pointeur.py <tuyau>

Lit des ordres sur un tuyau nommé, une ligne chacun :
  aller <x> <y>      (pixels de la sortie 1920 x 1080)
  presser | relacher (bouton gauche)
  milieu-presser | milieu-relacher (le bouton de la molette)
  glisser <x0> <y0> <x1> <y1> <ms> <hz>
                     (un mouvement continu, en ligne droite, à <hz> évènements par seconde :
                      la cadence d'une souris, 125 ou 1000 — le déplacement au clic molette)
  molette <crans> [<ms>]
                     (des crans de molette, un tous les <ms> (80 par défaut) : positifs vers
                      soi — on dézoome —, négatifs vers l'écran — on zoome)
  fin                (le pointeur s'en va)
  droit              (clic droit complet)
Le pointeur vit tant que le programme vit : le siège garde sa capacité « pointeur », et le client
garde son wl_pointer entre deux ordres. Comme une vraie souris, il s'arrête aux bords de la sortie ;
un ordre qu'il ne comprend pas s'écrit dans son journal, et il continue (fiche 100 § 10 : une
coordonnée négative le tuait, et le banc entier attendait ensuite un lecteur sur son tuyau).
"""
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from pywayland.client import Display  # noqa: E402
from pywayland.protocol.wayland import WlSeat  # noqa: E402
from protocole import ZwlrVirtualPointerManagerV1  # noqa: E402

LARGEUR, HAUTEUR = 1920, 1080
BTN_GAUCHE, BTN_DROIT, BTN_MILIEU = 0x110, 0x111, 0x112

tuyau = sys.argv[1]
display = Display()
display.connect()
registre = display.get_registry()
trouves = {}


def annonce(reg, nom, interface, version):
    if interface == "wl_seat":
        trouves["seat"] = reg.bind(nom, WlSeat, min(version, 7))
    elif interface == "zwlr_virtual_pointer_manager_v1":
        trouves["mgr"] = reg.bind(nom, ZwlrVirtualPointerManagerV1, min(version, 2))


registre.dispatcher["global"] = annonce
display.roundtrip()
pointeur = trouves["mgr"].create_virtual_pointer(trouves["seat"])
display.roundtrip()


def t():
    return int(time.monotonic() * 1000) & 0xFFFFFFFF


def bouton(b, etat):
    pointeur.button(t(), b, etat)
    pointeur.frame()
    display.flush()


def aller(x, y):
    """Va en (x, y), borné à la sortie : une souris ne sort pas de l'écran."""
    x = min(max(int(x), 0), LARGEUR - 1)
    y = min(max(int(y), 0), HAUTEUR - 1)
    pointeur.motion_absolute(t(), x, y, LARGEUR, HAUTEUR)
    pointeur.frame()
    display.flush()


def jouer(mots):
    """Joue un ordre."""
    if mots[0] == "aller":
        aller(float(mots[1]), float(mots[2]))
    elif mots[0] == "presser":
        bouton(BTN_GAUCHE, 1)
    elif mots[0] == "relacher":
        bouton(BTN_GAUCHE, 0)
    elif mots[0] == "milieu-presser":
        bouton(BTN_MILIEU, 1)
    elif mots[0] == "milieu-relacher":
        bouton(BTN_MILIEU, 0)
    elif mots[0] == "glisser":
        x0, y0, x1, y1, ms, hz = (float(m) for m in mots[1:7])
        pas = max(1, int(ms * hz / 1000))
        debut = time.monotonic()
        for i in range(1, pas + 1):
            # Chaque évènement part à son heure : la cadence est celle d'une main, pas celle de
            # la boucle Python.
            attente = debut + i / hz - time.monotonic()
            if attente > 0:
                time.sleep(attente)
            f = i / pas
            aller(x0 + (x1 - x0) * f, y0 + (y1 - y0) * f)
    elif mots[0] == "molette":
        # Un cran de souris : 15 unités et un cran discret, comme libinput les donne.
        crans = int(mots[1])
        ecart = float(mots[2]) / 1000 if len(mots) > 2 else 0.08
        sens = 1 if crans > 0 else -1
        for _ in range(abs(crans)):
            pointeur.axis_source(0)  # la roue
            pointeur.axis_discrete(t(), 0, 15.0 * sens, sens)
            pointeur.frame()
            display.flush()
            time.sleep(ecart)
    elif mots[0] == "droit":
        bouton(BTN_DROIT, 1)
        bouton(BTN_DROIT, 0)
    elif mots[0] == "fin":
        sys.exit(0)
    else:
        raise ValueError("ordre inconnu")


if not os.path.exists(tuyau):
    os.mkfifo(tuyau)
print("pret", flush=True)
while True:
    with open(tuyau) as f:
        for ligne in f:
            mots = ligne.split()
            if not mots:
                continue
            try:
                jouer(mots)
            except (ValueError, IndexError) as e:
                print(f"ordre refusé ({ligne.strip()!r}) : {e}", flush=True)
            display.roundtrip()
