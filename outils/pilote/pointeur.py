"""Un pointeur virtuel persistant pour un compositeur sans écran (fiche 99) — par `piloter.sh`.

    python3 pointeur.py <tuyau>

Lit des ordres sur un tuyau nommé, une ligne chacun :
  aller <x> <y>      (pixels de la sortie 1920 x 1080)
  presser | relacher (bouton gauche)
  fin                (le pointeur s'en va)
  droit              (clic droit complet)
Le pointeur vit tant que le programme vit : le siège garde sa capacité « pointeur », et le client
garde son wl_pointer entre deux ordres.
"""
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from pywayland.client import Display  # noqa: E402
from pywayland.protocol.wayland import WlSeat  # noqa: E402
from protocole import ZwlrVirtualPointerManagerV1  # noqa: E402

LARGEUR, HAUTEUR = 1920, 1080
BTN_GAUCHE, BTN_DROIT = 0x110, 0x111

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


if not os.path.exists(tuyau):
    os.mkfifo(tuyau)
print("pret", flush=True)
while True:
    with open(tuyau) as f:
        for ligne in f:
            mots = ligne.split()
            if not mots:
                continue
            if mots[0] == "aller":
                pointeur.motion_absolute(t(), int(float(mots[1])), int(float(mots[2])), LARGEUR, HAUTEUR)
                pointeur.frame()
                display.flush()
            elif mots[0] == "presser":
                bouton(BTN_GAUCHE, 1)
            elif mots[0] == "relacher":
                bouton(BTN_GAUCHE, 0)
            elif mots[0] == "droit":
                bouton(BTN_DROIT, 1)
                bouton(BTN_DROIT, 0)
            elif mots[0] == "fin":
                sys.exit(0)
            display.roundtrip()
