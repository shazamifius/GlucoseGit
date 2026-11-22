"""**Installer le relais**, une fois — et sans rien casser si on le relance : chaque étape regarde
d'abord ce qui existe. Quatre choses : sa membrane dans Glucose, la barre d'état et le crochet
« Stop » de Claude Code, le dossier du second compte, le service qui veille.
"""
import json
import os
import subprocess
from pathlib import Path

import relais as r

ICI = Path(__file__).resolve()
PROGRAMME = ICI.parent / "relais.py"
COMMANDE = f"python3 -I {PROGRAMME}"
UNITE = r.MAISON / ".config/systemd/user/relais-glucose.service"

CONSIGNE_PROPOSEE = (
    "📜 **Ta consigne au relais** — proposée par Claude Code le 22/11 : **le relais ne la suit que "
    "quand c'est toi qui l'as écrite ou retouchée en dernier.**\n\n"
    "Continue Glucose. D'abord finir le chantier 129 (les 4 épreuves qui tombent, les sabotages), tout "
    "commiter et envoyer ; puis le plan (SUITE § 2, la passation). Prouve chaque chose (épreuves, "
    "sabotages, captures du pilote). Tes questions pour moi : un texte chacune, dans cette membrane, "
    "et tu continues autre chose.\n\n"
    "*Pour arrêter le relais : écris ⏸ PAUSE en première ligne.*")

NOEUDS = [  # (id, texte, dx, dy, largeur)
    (r.CONSIGNE, CONSIGNE_PROPOSEE, 40, 90, 700),
    (r.BATTEMENT, "❤️ **Relais** · pas encore démarré", 790, 90, 520),
    (r.BLOQUE, "🆘 Rien pour l'instant : ici, l'écran d'une session qui ne bouge plus depuis 35 minutes.",
     790, 330, 520),
    (r.REPONSE, r.INVITE_REPONSE, 790, 620, 520),
    (r.PASSATION, "🔁 **Passation du relais** — vide jusqu'au premier relais ; la passation 130 "
     "(nœud passation-130) vaut jusque-là.", 40, 760, 700),
]


def membrane_dans_glucose():
    lu = r.glucose("lire", "tableau=main")
    if lu is None:
        return "Glucose ne répond pas : ouvre-le, puis relance l'installation"
    ids = {n["id"] for n in lu.get("noeuds", [])}
    if "relais" not in ids:
        boites = [n["boite"] for n in lu["noeuds"] if n.get("boite")]
        x0 = max(b["x"] + b["largeur"] for b in boites) + 300
        y0 = min(b["y"] for b in boites)
        r.glucose("poser-membrane", "tableau=main", "id=relais", "titre=🤖 Relais — Claude travaille seul",
                  f"x={x0}", f"y={y0}", "largeur=1360", "hauteur=1500", "couleur=#14b8a6")
    else:
        b = next(n["boite"] for n in lu["noeuds"] if n["id"] == "relais")
        x0, y0 = b["x"], b["y"]
    poses = []
    for ident, texte, dx, dy, largeur in NOEUDS:
        if ident not in ids:
            r.ecrire(ident, texte, dans="relais", x=x0 + dx, y=y0 + dy, largeur=largeur)
            poses.append(ident)
    reponse = r.noeud(r.REPONSE)
    memoire = r.lire_json(r.ETAT / "veilleur.json", {})
    memoire.setdefault("reponse_vue", (reponse or {}).get("empreinte"))
    r.ecrire_json(r.ETAT / "veilleur.json", memoire)
    return f"Glucose : {', '.join(poses) or 'déjà en place'}"


def reglages_claude():
    chemin = r.MAISON / ".claude/settings.json"
    reglages = json.loads(chemin.read_text())
    reglages["statusLine"] = {"type": "command", "command": f"{COMMANDE} jauges"}
    crochets = reglages.setdefault("hooks", {}).setdefault("Stop", [])
    voulu = f"{COMMANDE} fin-de-tour"
    if not any(h.get("command") == voulu for g in crochets for h in g.get("hooks", [])):
        crochets.append({"hooks": [{"type": "command", "command": voulu, "timeout": 300}]})
    provisoire = chemin.with_suffix(".relais.tmp")
    provisoire.write_text(json.dumps(reglages, ensure_ascii=False, indent=2) + "\n")
    os.replace(provisoire, chemin)
    return "Claude Code : barre d'état et crochet Stop"


def second_compte():
    b = r.COMPTES["b"]
    b.mkdir(mode=0o700, exist_ok=True)
    liens = []
    for nom in ("CLAUDE.md", "settings.json", "projects", "skills", "keybindings.json"):
        source, lien = r.MAISON / ".claude" / nom, b / nom
        if source.exists() and not lien.exists() and not lien.is_symlink():
            lien.symlink_to(source)
            liens.append(nom)
    env = dict(os.environ, CLAUDE_CONFIG_DIR=str(b))
    deja = subprocess.run(["claude", "mcp", "get", "glucose"], env=env, capture_output=True, text=True)
    if deja.returncode != 0:
        subprocess.run(["claude", "mcp", "add", "glucose", "-s", "user", "--", "glucose", "--mcp"],
                       env=env, capture_output=True, text=True)
        liens.append("MCP glucose")
    connecte = "connecté" if r.connecte("b") else "PAS ENCORE CONNECTÉ (un /login à faire une fois)"
    return f"second compte ({b}) : {', '.join(liens) or 'déjà prêt'} — {connecte}"


def service():
    UNITE.parent.mkdir(parents=True, exist_ok=True)
    UNITE.write_text(
        "[Unit]\nDescription=Le relais de Glucose : Claude travaille seul, d'un compte à l'autre\n\n"
        "[Service]\n"
        f"ExecStart=/run/current-system/sw/bin/python3 -I {PROGRAMME} veiller\n"
        "Environment=PATH=/run/current-system/sw/bin:/etc/profiles/per-user/%u/bin\n"
        "Restart=always\nRestartSec=30\n\n[Install]\nWantedBy=default.target\n")
    subprocess.run(["systemctl", "--user", "daemon-reload"], check=True)
    subprocess.run(["systemctl", "--user", "enable", "--now", UNITE.name], check=True)
    return f"service : {UNITE.name} actif"


def installer():
    r.ETAT.mkdir(parents=True, exist_ok=True)
    for etape in (membrane_dans_glucose, reglages_claude, second_compte, service):
        print("•", etape())
    return 0
