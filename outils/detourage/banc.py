"""Le banc du détourage (DETOURER-1, fiche 121) : chaque image d'un dossier passée dans le moteur de
Glucose (l'exemple `bench_detourage`, compilé en release), le résultat posé sur un damier.

Usage : python3 -I outils/detourage/banc.py <binaire bench_detourage> <dossier d'images> <sortie> [fond]

Écrit `<nom>.glucose.png` (RVBA) et `<nom>.damier.png` ; la ligne du temps vient du binaire. Avec
« fond », retire le fond uni au lieu de détourer.
"""
import glob
import os
import subprocess
import sys

import numpy as np
from PIL import Image


def damier(l, h, c=16):
    y, x = np.mgrid[0:h, 0:l]
    return np.where(((x // c) + (y // c)) % 2, 200, 255).astype(np.float32)[..., None].repeat(3, 2)


def principal():
    binaire, dossier, sortie = sys.argv[1:4]
    mode = sys.argv[4:5]
    os.makedirs(sortie, exist_ok=True)
    for chemin in sorted(glob.glob(f"{dossier}/*")):
        if ".verite." in chemin:
            continue
        nom = os.path.splitext(os.path.basename(chemin))[0]
        im = Image.open(chemin).convert("RGBA")
        brut, fini = f"{sortie}/{nom}.entree.rvba", f"{sortie}/{nom}.sortie.rvba"
        open(brut, "wb").write(im.tobytes())
        arguments = [binaire, brut, str(im.width), str(im.height), fini] + (["1", "fond"] if mode else [])
        r = subprocess.run(arguments, capture_output=True, text=True)
        print(nom, r.stderr.strip().replace("\n", " | "), flush=True)
        if r.returncode:
            continue
        rvba = np.frombuffer(open(fini, "rb").read(), np.uint8).reshape(im.height, im.width, 4)
        Image.fromarray(rvba).save(f"{sortie}/{nom}.glucose.png")
        a = rvba[..., 3:].astype(np.float32) / 255
        compo = rvba[..., :3].astype(np.float32) * a + damier(im.width, im.height) * (1 - a)
        Image.fromarray(compo.astype(np.uint8)).save(f"{sortie}/{nom}.damier.png")
        v = f"{dossier}/{nom}.verite.png"
        if os.path.exists(v):
            vt = np.asarray(Image.open(v), np.float32) / 255
            print(f"   erreur moyenne de l'opacité contre la vérité : {np.abs(vt - a[..., 0]).mean():.4f}")
        os.remove(brut)
        os.remove(fini)


if __name__ == "__main__":
    principal()
