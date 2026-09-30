"""Prépare la source publiée de Glucose Tauri pour la répétition générale de la bascule (fiche 48).

Usage : python outils/installeur/preparer_tauri.py <source de Tauri> <clé publique d'essai>

Glucose Tauri reste celui que ses utilisateurs ont — la même source, étiquette v1.0.2-beta.1 ;
seuls changent :
* sa configuration : la clé d'essai au lieu de celle de Glucose, un serveur local au lieu de
  GitHub (en `http`, que l'updater n'accepte qu'avec `dangerousInsecureTransportProtocol`), et pas
  d'artefacts de mise à jour à signer pour lui-même ; sa caisse, son propre espace de travail ;
* **le clic de son popup** : au lieu d'attendre qu'on appuie sur « Installer », il fait, dès que
  la mise à jour est trouvée, les deux appels que fait ce bouton — `downloadAndInstall`, puis
  `relaunch`. Tout le reste — trouver, télécharger, vérifier, lancer l'installeur, quitter — est
  le code de l'updater de Tauri, tel quel.
"""
import json
import os
import sys


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    source, cle = sys.argv[1], sys.argv[2]

    conf_chemin = os.path.join(source, "src-tauri", "tauri.conf.json")
    with open(conf_chemin, encoding="utf-8") as f:
        conf = json.load(f)
    conf["plugins"]["updater"] = {
        "pubkey": cle,
        "endpoints": ["http://127.0.0.1:8765/latest.json"],
        "dangerousInsecureTransportProtocol": True,
    }
    conf["bundle"]["createUpdaterArtifacts"] = False
    with open(conf_chemin, "w", encoding="utf-8") as f:
        json.dump(conf, f, indent=2, ensure_ascii=False)

    popup = os.path.join(source, "src", "components", "UpdatePrompt.tsx")
    with open(popup, encoding="utf-8", newline="") as f:
        texte = f.read()
    avant = "if (!cancelled && u) setUpdate(u);"
    if texte.count(avant) != 1:
        sys.exit(f"le popup de Tauri a changé : « {avant} » introuvable")
    apres = (
        "if (!cancelled && u) { setUpdate(u); await u.downloadAndInstall(); await relaunch(); }"
    )
    with open(popup, "w", encoding="utf-8", newline="") as f:
        f.write(texte.replace(avant, apres))

    # Extraite dans le dépôt de Glucose Rust, sa caisse se croirait membre de notre espace de
    # travail : elle se déclare le sien. Rien du programme ne change.
    manifeste = os.path.join(source, "src-tauri", "Cargo.toml")
    with open(manifeste, encoding="utf-8", newline="") as f:
        cargo = f.read()
    if "[workspace]" not in cargo:
        with open(manifeste, "a", encoding="utf-8", newline="") as f:
            f.write("\n[workspace]\n")
    print("Glucose Tauri préparé : clé d'essai, serveur local, le clic fait par le programme")


main()
