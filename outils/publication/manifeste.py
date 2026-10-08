"""Écrit le latest.json d'une publication de Glucose Rust (fiche 48).

Usage : python outils/publication/manifeste.py <dossier> <version> <dépôt>

<dossier> porte les cinq fichiers de la version et leurs signatures (.sig), comme le
signataire de Tauri les écrit :
    Glucose_<v>_x64-setup.exe    Glucose_<v>_amd64.AppImage
    Glucose_<v>_amd64.deb        Glucose-<v>-1.x86_64.rpm
    Glucose_<v>_android.apk
<dépôt> : propriétaire/nom, comme GitHub le donne (GITHUB_REPOSITORY).

Les clés sont celles du latest.json de Glucose Tauri, que ses programmes de mise à jour lisent :
celle de chaque forme d'installation (windows-x86_64-nsis, linux-x86_64-appimage, -deb, -rpm),
puis celle de la plateforme seule, qui sert l'installeur NSIS et l'AppImage. Les adresses sont
celles de la release v<version> de ce dépôt. Android (fiche 58) a les siennes, à la façon de
Tauri : android-aarch64 et android-armv7, avec et sans « -apk » — un seul APK porte les deux
processeurs. Aucune entrée pour le Mac : Glucose Rust n'y a pas
encore de paquet, et un Mac qui ne trouve pas sa clé ne se voit rien proposer.

Rien d'autre que la version et les plateformes : l'updater de Tauri lit ce fichier d'un bloc, et
une date qui ne serait pas au format RFC 3339 le lui ferait refuser tout entier. `pub_date` et
`notes` sont facultatives chez lui : elles ne s'écrivent pas.

Ce script ne prouve rien : l'exemple `publication manifeste` relit ce qu'il écrit avec le code
même de Glucose, et vérifie chaque fichier par sa signature.
"""
import json
import os
import sys


def main():
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    dossier, version, depot = sys.argv[1], sys.argv[2], sys.argv[3]
    adresse = f"https://github.com/{depot}/releases/download/v{version}/"

    def entree(nom):
        chemin = os.path.join(dossier, nom)
        if not os.path.isfile(chemin):
            sys.exit(f"{nom} manque dans {dossier}")
        with open(chemin + ".sig", encoding="ascii") as f:
            return {"signature": f.read().strip(), "url": adresse + nom}

    nsis = entree(f"Glucose_{version}_x64-setup.exe")
    appimage = entree(f"Glucose_{version}_amd64.AppImage")
    apk = entree(f"Glucose_{version}_android.apk")
    manifeste = {
        "version": version,
        "platforms": {
            "windows-x86_64": nsis,
            "windows-x86_64-nsis": nsis,
            "linux-x86_64": appimage,
            "linux-x86_64-appimage": appimage,
            "linux-x86_64-deb": entree(f"Glucose_{version}_amd64.deb"),
            "linux-x86_64-rpm": entree(f"Glucose-{version}-1.x86_64.rpm"),
            "android-aarch64": apk,
            "android-aarch64-apk": apk,
            "android-armv7": apk,
            "android-armv7-apk": apk,
        },
    }
    sortie = os.path.join(dossier, "latest.json")
    with open(sortie, "w", encoding="utf-8", newline="\n") as f:
        json.dump(manifeste, f, indent=2)
        f.write("\n")
    print(sortie)


main()
