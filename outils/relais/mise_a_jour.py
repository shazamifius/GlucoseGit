"""**Claude se met à jour seul** (MAJ-1, fiche 162) — sa demande du 24/11 : « ce serait bien que tu
arrives à faire fonctionner la mise à jour claude […] le problème c'est que j'ai pas accès à mon pc ».
L'extension de VS Code ne se met à jour que dans un VS Code ouvert ; le relais tourne sans lui.

Toutes les six heures, le veilleur demande au marché de VS Code la dernière version pour Linux. Plus
neuve que celle que VS Code dit installée, il l'installe (`code --install-extension …@version`, deux
fois au plus : le 25/11, le premier essai a échoué sur la signature du paquet, le second a réussi),
puis **l'essaie** : lancée dans le dépôt comme le relais la lance, sans un mot (aucun jeton), elle doit
arriver à son invite sans poser de question — une question au démarrage figerait une session que
personne ne regarde (règle 7). Refusée, la précédente est réinstallée, et la version refusée n'est
plus retentée. Le relais lance celle que VS Code dit installée (`machine.claude_binaire`).
"""
import json
import subprocess
import time
import urllib.request

from machine import DEPOT, EXTENSION, installee

PLATEFORME = "linux-x64"
MARCHE = "https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery"
# 0x1, les versions ; 0x200, la dernière seulement : 1,7 Ko au lieu de deux mille cinq cents versions.
REQUETE = {"filters": [{"criteria": [{"filterType": 7, "value": EXTENSION}]}], "flags": 0x201}
ESSAI = "essai-claude"  # jamais « relais-… » : session_tmux() le prendrait pour la session du relais
TOUTES_LES = 6 * 3600


def version(texte):
    return tuple(int(x) if x.isdigit() else 0 for x in str(texte).split("."))


def derniere_publiee():
    """La dernière version publiée pour Linux, ou rien si le marché ne répond pas."""
    demande = urllib.request.Request(MARCHE, data=json.dumps(REQUETE).encode(), headers={
        "Content-Type": "application/json", "Accept": "application/json;api-version=3.0-preview.1"})
    try:
        with urllib.request.urlopen(demande, timeout=30) as r:
            versions = json.load(r)["results"][0]["extensions"][0]["versions"]
    except (OSError, ValueError, KeyError, IndexError):
        return None
    pour_nous = [v["version"] for v in versions if v.get("targetPlatform") == PLATEFORME]
    return max(pour_nous, key=version, default=None)


def installer(v):
    """Installe cette version ; vrai si VS Code la dit installée ensuite."""
    for _ in range(2):
        try:
            subprocess.run(["code", "--install-extension", f"{EXTENSION}@{v}", "--force"],
                           capture_output=True, text=True, timeout=300)
        except (OSError, subprocess.TimeoutExpired):
            continue
        if installee()[0] == v:
            return True
    return False


def prete(ecran):
    """L'écran d'une session arrivée à son invite : le mode bypass dit sous la boîte de saisie, et
    aucune question à confirmer (celle de la confiance dans un dossier, le 25/11, se lisait ainsi)."""
    return "bypass permissions on" in ecran and "Enter to confirm" not in ecran


def essayer(dossier, attente=40):
    """Lance ce Claude dans le dépôt comme le relais le lance, sans lui écrire, et lit son écran : rend
    (prête, écran). Sans un message, il ne dépense rien ; il est fermé aussitôt."""
    binaire = dossier / "resources/native-binary/claude"
    subprocess.run(["tmux", "kill-session", "-t", ESSAI], capture_output=True)
    # Ni RELAIS (ses crochets le prendraient pour une session du relais), ni le compte b : celui
    # par défaut, où le dépôt est un dossier de confiance.
    subprocess.run(["tmux", "new-session", "-d", "-s", ESSAI, "-x", "200", "-y", "50", "-c", str(DEPOT),
                    "env", "-u", "RELAIS", "-u", "RELAIS_COMPTE", "-u", "CLAUDE_CONFIG_DIR", str(binaire),
                    "--ide", "--no-chrome", "--permission-mode", "bypassPermissions",
                    "--allow-dangerously-skip-permissions"], capture_output=True)
    ecran = ""
    fin = time.time() + attente
    while time.time() < fin:
        time.sleep(2)
        ecran = subprocess.run(["tmux", "capture-pane", "-p", "-t", ESSAI],
                               capture_output=True, text=True).stdout
        if prete(ecran):
            break
    subprocess.run(["tmux", "kill-session", "-t", ESSAI], capture_output=True)
    return prete(ecran), ecran


def veiller(memoire, maintenant, dire, journal):
    """Toutes les six heures : une version neuve s'installe et s'essaie. `dire` pose le nœud qui le
    raconte (le même, récrit à chaque fois) ; rend ce qui s'est passé, ou rien."""
    if maintenant - memoire.get("maj_vue", 0) < TOUTES_LES:
        return None
    memoire["maj_vue"] = maintenant
    neuve, (avant, _) = derniere_publiee(), installee()
    refusees = memoire.setdefault("maj_refusees", [])
    if not neuve or not avant or version(neuve) <= version(avant) or neuve in refusees:
        return None
    journal(f"Claude {neuve} publié ({avant} installé) : installation")
    if not installer(neuve):
        journal(f"Claude {neuve} : l'installation échoue, nouvel essai dans six heures")
        return None
    ok, ecran = essayer(installee()[1])
    quand = time.strftime("%d/%m à %H h %M", time.localtime(maintenant))
    if ok:
        fait = (f"🔄 **Claude {neuve} installé seul** le {quand} (avant : {avant}), puis essayé : il "
                "démarre jusqu'à son invite sans poser de question. La prochaine session du relais le "
                "prendra ; ton VS Code l'affichera quand tu le rouvriras.")
    else:
        refusees.append(neuve)
        # Le veilleur ne regarde plus ses sessions pendant ce temps : borné à cinq minutes par essai.
        garde = (f"Le relais garde {avant}" if installer(avant) else
                 f"**Le retour à {avant} a échoué** : la prochaine session lancera {neuve}")
        lignes = [l for l in ecran.splitlines() if l.strip()][-12:]
        fait = (f"⚠️ **Claude {neuve} refusé** le {quand} : à l'essai, il n'arrive pas à son invite sans "
                f"question. {garde}, et ne retentera pas {neuve}. Son écran :\n\n```\n"
                + "\n".join(lignes) + "\n```")
    journal(fait.split("\n")[0][:200])
    dire(fait)
    return fait
