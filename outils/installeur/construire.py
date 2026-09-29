"""Construit l'installeur Windows de Glucose Rust (fiche 48).

Usage : python outils/installeur/construire.py <glucose-desktop.exe> <dossier de sortie>

La version se lit dans l'exécutable lui-même (`--version`) : un installeur ne peut pas annoncer
une autre version que celle qu'il pose. NSIS se cherche dans `MAKENSIS`, dans le `PATH`, là où
son installeur le met, puis là où Tauri le garde.
"""
import os
import shutil
import subprocess
import sys

ICI = os.path.dirname(os.path.abspath(__file__))


def makensis():
    candidats = [
        os.environ.get("MAKENSIS"),
        shutil.which("makensis"),
        r"C:\Program Files (x86)\NSIS\makensis.exe",
        r"C:\Program Files\NSIS\makensis.exe",
        os.path.join(os.environ.get("LOCALAPPDATA", ""), "tauri", "NSIS", "makensis.exe"),
    ]
    for c in candidats:
        if c and os.path.isfile(c):
            return c
    sys.exit("makensis introuvable : installer NSIS, ou donner MAKENSIS")


def version_de(exe):
    sortie = subprocess.run([exe, "--version"], capture_output=True, text=True, check=True).stdout
    mots = sortie.split()
    if len(mots) != 2 or mots[0] != "Glucose":
        sys.exit(f"--version a rendu autre chose qu'une version : {sortie!r}")
    return mots[1]


def numerique(version):
    """Ce que Windows veut dans les propriétés du fichier : quatre nombres."""
    noyau = version.split("+")[0].split("-")[0].split(".")
    if len(noyau) != 3 or not all(n.isdigit() for n in noyau):
        sys.exit(f"version illisible : {version}")
    return ".".join(noyau + ["0"])


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    exe, dossier = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
    version = version_de(exe)
    os.makedirs(dossier, exist_ok=True)
    sortie = os.path.join(dossier, f"Glucose_{version}_x64-setup.exe")
    subprocess.run(
        [
            makensis(),
            "/V2",
            f"/DVERSION={version}",
            f"/DVERSION_NUMERIQUE={numerique(version)}",
            f"/DEXE={exe}",
            f"/DSORTIE={sortie}",
            os.path.join(ICI, "glucose.nsi"),
        ],
        check=True,
    )
    print(sortie)


main()
