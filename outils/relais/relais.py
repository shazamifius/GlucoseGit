#!/usr/bin/env python3
"""**Le relais** : Glucose avance seul, d'un compte à l'autre, sans qu'il tape « continue ».

Sa demande du 22/11/2026 : Claude travaille en autonomie ; une question s'écrit dans Glucose et
Claude passe à autre chose ; quand un compte arrive à sa limite, la passation s'écrit dans Glucose
et l'autre compte reprend. Un seul programme, la bibliothèque standard seulement :

  relais.py jauges       la barre d'état de Claude Code : garde les deux jauges du compte
  relais.py fin-de-tour  le crochet « Stop » : continuer selon SA consigne, ou écrire la passation
  relais.py veiller      le service : lance, relaie, réveille, bat le cœur dans Glucose
  relais.py installer    une fois : Glucose, les réglages de Claude Code, le second compte, le service
  relais.py etat         ce que le relais sait, en clair

Trois règles. **Seules les sessions lancées par le relais** (RELAIS=1) sont concernées : les siennes,
dans VS Code, ne voient rien changer. **Sa consigne n'est suivie que si c'est LUI qui l'a écrite en
dernier** (la Time Machine le dit) : une IA ne réécrit pas sa propre consigne ; dans le doute, on
s'arrête. **Rien n'attend en silence** : 35 minutes sans un changement à l'écran, et l'écran part
dans Glucose, où il répond.
"""
import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path

MAISON = Path.home()
ETAT = MAISON / ".local/state/relais"
DEPOT = MAISON / "Projets/GlucoseGit"
# a : le dossier par défaut de Claude Code (~/.claude) ; b : son propre dossier — sa connexion à lui,
# les mémoires et les journaux partagés par liens (installer).
COMPTES = {"a": None, "b": MAISON / ".claude-compte-b"}
# La passation coûte un tour : la demander avant le mur, pas au mur.
SEUILS = {"five_hour": 90.0, "seven_day": 95.0}
SILENCE = 35 * 60
EXTENSIONS = MAISON / ".vscode/extensions"


def nom_tmux(compte):
    """Une session tmux par compte : le VS Code de ce compte l'affiche (.vscode/tasks.json)."""
    return f"relais-{compte}"


def version(dossier):
    morceaux = dossier.name.split("-")[2] if dossier.name.count("-") >= 2 else "0"
    return tuple(int(x) if x.isdigit() else 0 for x in morceaux.split("."))


def claude_binaire():
    """**Le Claude de la dernière extension VS Code**, jamais celui de la ligne de commande : sa
    parole du 22/11 — le Claude en ligne de commande était en 2.1.223 quand l'extension était en
    2.1.330. L'extension embarque le sien et le met à jour seule : le relais suit VS Code."""
    candidats = sorted(EXTENSIONS.glob("anthropic.claude-code-*/resources/native-binary/claude"),
                       key=lambda p: version(p.parents[2]))
    return str(candidats[-1]) if candidats else "claude"
CONSIGNE, PASSATION, BATTEMENT, BLOQUE, REPONSE = (
    "relais-consigne", "relais-passation", "relais-battement", "relais-bloque", "relais-reponse")
MODE_EMPLOI = "relais-mode-emploi"  # ce qui est automatique, pour lui et pour toute session de Claude
INVITE_REPONSE = "✍️ Ta réponse :"

REGLES = (
    "Tu es une session du RELAIS de Glucose : personne ne lit ce terminal en direct. Une question "
    "pour lui s'écrit dans son Glucose (tableau=main, dans la membrane « relais »), un texte par "
    "question, et tu passes à une autre tâche du plan : ne t'arrête jamais pour attendre. "
    "N'écris jamais dans le nœud relais-consigne : c'est le sien. Jamais de publication de version. "
    "Aucune commande de plus de 30 minutes.")


def depart(compte):
    return (f"Session du relais (compte {compte}). Lis ~/.claude/CLAUDE.md, ~/Projets/LISEZ-MOI.md et "
            "toutes tes mémoires en entier, puis dans son Glucose (glucose lire tableau=main "
            f"ids={CONSIGNE} ids={PASSATION} ids={MODE_EMPLOI}) sa consigne, la passation et le mode d'emploi "
            "du relais. Puis travaille selon la consigne.")


# ── Petits outils ─────────────────────────────────────────────────────────────────────────────

def lire_json(chemin, defaut):
    try:
        return json.loads(Path(chemin).read_text())
    except (OSError, ValueError):
        return defaut


def ecrire_json(chemin, donnees):
    chemin = Path(chemin)
    chemin.parent.mkdir(parents=True, exist_ok=True)
    provisoire = chemin.with_suffix(".tmp")
    provisoire.write_text(json.dumps(donnees, ensure_ascii=False, indent=1))
    os.replace(provisoire, chemin)


def journal(message):
    ETAT.mkdir(parents=True, exist_ok=True)
    with open(ETAT / "journal.txt", "a") as f:
        f.write(time.strftime("%d/%m %H:%M:%S ") + message + "\n")


def compte_courant():
    if os.environ.get("RELAIS_COMPTE"):
        return os.environ["RELAIS_COMPTE"]
    dossier = os.environ.get("CLAUDE_CONFIG_DIR")
    for nom, chemin in COMPTES.items():
        if chemin and dossier and Path(dossier).resolve() == chemin.resolve():
            return nom
    return "a"


def heure(t):
    return time.strftime("%H:%M", time.localtime(t))


# ── Les jauges ────────────────────────────────────────────────────────────────────────────────

def charge(compte, maintenant):
    """Chaque fenêtre (« five_hour », « seven_day ») : (pourcentage, remise à zéro). Une fenêtre
    dont l'heure de remise est passée est vide ; une jauge jamais lue n'est pas comptée."""
    lu = lire_json(ETAT / f"jauges-{compte}.json", {}).get("rate_limits") or {}
    fenetres = {}
    for nom, f in lu.items():
        if not isinstance(f, dict) or "used_percentage" not in f:
            continue
        remise = float(f.get("resets_at") or 0)
        fenetres[nom] = (0.0 if remise and remise <= maintenant else float(f["used_percentage"]), remise)
    return fenetres


def plein(compte, maintenant):
    """Ce compte est-il au seuil ? Et sinon, quand le redevient-il libre (None : il l'est)."""
    murs = [remise for nom, (pct, remise) in charge(compte, maintenant).items()
            if pct >= SEUILS.get(nom, 100.0)]
    return (True, min(murs)) if murs else (False, None)


def jauges():
    try:
        entree = json.load(sys.stdin)
    except ValueError:
        entree = {}
    limites = entree.get("rate_limits") or {}
    if limites:
        ecrire_json(ETAT / f"jauges-{compte_courant()}.json", {"lu": time.time(), "rate_limits": limites})
    morceaux = [(entree.get("model") or {}).get("display_name", "")]
    for nom, titre in (("five_hour", "5 h"), ("seven_day", "semaine")):
        f = limites.get(nom) or {}
        if "used_percentage" in f:
            morceaux.append(f"{titre} {round(float(f['used_percentage']))} %")
    print(" · ".join(m for m in morceaux if m))


# ── Glucose, par sa porte ─────────────────────────────────────────────────────────────────────

def glucose(*arguments, entree=None):
    try:
        r = subprocess.run(["glucose", *arguments], input=entree, capture_output=True,
                           text=True, timeout=120)
    except (OSError, subprocess.TimeoutExpired) as e:
        journal(f"glucose {arguments[0]} : {e}")
        return None
    if r.returncode != 0:
        journal(f"glucose {arguments[0]} : {r.stderr.strip()[:300]}")
        return None
    try:
        return json.loads(r.stdout)
    except ValueError:
        return {}


def noeud(ident):
    lu = glucose("lire", "tableau=main", f"ids={ident}")
    return next((n for n in (lu or {}).get("noeuds", []) if n.get("id") == ident), None)


def ecrire(ident, texte, **place):
    reglages = [f"{k}={v}" for k, v in place.items()]
    return glucose("poser-texte", "tableau=main", f"id={ident}", *reglages, "texte=-", entree=texte)


def auteur_recent(ident, histoire):
    """L'auteur du dernier geste de l'histoire qui nomme ce nœud (« humain », « agent … »)."""
    for geste in reversed((histoire or {}).get("gestes", [])):
        if any(f"« {ident} »" in fait for fait in geste.get("fait", [])):
            return geste.get("auteur")
    return None


def signee(n, memoire, histoire):
    """La consigne est la sienne : son empreinte est celle qu'il a écrite en dernier. Une fois vue
    de lui, l'empreinte est gardée ; un geste d'agent la change, et elle ne vaut plus rien."""
    if not n or not n.get("empreinte"):
        return False
    if memoire.get("consigne_signee") == n["empreinte"]:
        return True
    if auteur_recent(CONSIGNE, histoire) == "humain":
        memoire["consigne_signee"] = n["empreinte"]
        return True
    return False


def en_pause(texte):
    premiere = (texte or "").strip().splitlines()[:1]
    return bool(premiere) and (premiere[0].startswith("⏸") or "PAUSE" in premiere[0].upper())


# ── Le crochet « Stop » ───────────────────────────────────────────────────────────────────────

def decider(session, consigne_texte, pause, a_signee, au_seuil, maintenant):
    """(action, raison) — action : « laisser » (Claude s'arrête), « continuer » ou « passation »."""
    if session.get("passation_demandee"):
        return "laisser", "relayer"
    if au_seuil:
        return "passation", ""
    if pause:
        return "laisser", "pause"
    if not a_signee or not (consigne_texte or "").strip():
        return "laisser", "consigne non signée"
    recents = [t for t in session.get("arrets", []) if maintenant - t < 120]
    if len(recents) >= 3:
        return "laisser", "à vide"
    return "continuer", ""


def raison_continuer(consigne_texte):
    return ("[Relais] Ne t'arrête pas : continue selon sa consigne et le plan (SUITE § 2, la passation). "
            "Si tu attends une réponse de lui, écris la question dans son Glucose et prends une autre "
            f"tâche. Sa consigne :\n{consigne_texte.strip()}")


RAISON_PASSATION = (
    "[Relais] Ce compte arrive à sa limite. Écris MAINTENANT la passation dans son Glucose : remplace "
    f"le texte du nœud {PASSATION} (glucose poser-texte tableau=main id={PASSATION} texte=-) par "
    "l'état exact — fait, en cours, non commité, prochaine étape, pièges payés. Puis termine ton tour.")


def fin_de_tour():
    if os.environ.get("RELAIS") != "1":
        return 0  # une session à lui : le relais n'y touche pas
    try:
        json.load(sys.stdin)
    except ValueError:
        pass
    maintenant = time.time()
    session = lire_json(ETAT / "session.json", {})
    memoire = lire_json(ETAT / "veilleur.json", {})
    n = noeud(CONSIGNE)
    texte = (n or {}).get("texte", "")
    action, pourquoi = decider(session, texte, en_pause(texte),
                               signee(n, memoire, glucose("histoire", "nombre=200")),
                               plein(compte_courant(), maintenant)[0], maintenant)
    session["arrets"] = [t for t in session.get("arrets", []) if maintenant - t < 120] + [maintenant]
    if action == "passation":
        session["passation_demandee"] = maintenant
        session["passation_avant"] = (noeud(PASSATION) or {}).get("empreinte")
        print(json.dumps({"decision": "block", "reason": RAISON_PASSATION}, ensure_ascii=False))
    elif action == "continuer":
        session.pop("laissee", None)
        print(json.dumps({"decision": "block", "reason": raison_continuer(texte)}, ensure_ascii=False))
    else:
        session["laissee"] = pourquoi
    ecrire_json(ETAT / "session.json", session)
    journal(f"fin de tour : {action} {pourquoi}".strip())
    return 0


# ── Le veilleur ───────────────────────────────────────────────────────────────────────────────

def tmux(*arguments):
    return subprocess.run(["tmux", *arguments], capture_output=True, text=True)


def connecte(compte):
    dossier = COMPTES[compte] or MAISON / ".claude"
    return (dossier / ".credentials.json").exists()


def choisir(courant, maintenant):
    """Le compte qui travaille : le courant s'il le peut, sinon un autre ; aucun : (None, l'heure)."""
    ordre = [courant] + [c for c in COMPTES if c != courant]
    murs = []
    for compte in ordre:
        if not connecte(compte):
            continue
        au_seuil, libre_a = plein(compte, maintenant)
        if not au_seuil:
            return compte, None
        murs.append(libre_a)
    return None, (min(murs) if murs else None)


def lancer(compte):
    reglages = ["RELAIS=1", f"RELAIS_COMPTE={compte}"]
    if COMPTES[compte]:
        reglages.append(f"CLAUDE_CONFIG_DIR={COMPTES[compte]}")
    binaire = claude_binaire()
    # --ide : branché sur le VS Code de ce compte (ses outils, ses différences affichées).
    tmux("new-session", "-d", "-s", nom_tmux(compte), "-x", "200", "-y", "50", "-c", str(DEPOT),
         "env", *reglages, binaire, "--ide", "--remote-control", f"relais-{compte}",
         "--append-system-prompt", REGLES, depart(compte))
    ecrire_json(ETAT / "session.json", {"compte": compte, "depuis": time.time()})
    journal(f"session lancée sur le compte {compte} ({binaire})")


def session_tmux():
    """Le nom de la session du relais qui vit, s'il y en a une."""
    for compte in COMPTES:
        if tmux("has-session", "-t", nom_tmux(compte)).returncode == 0:
            return nom_tmux(compte)
    return None


def taper(texte):
    """Tape un texte dans la session qui vit, puis Entrée."""
    nom = session_tmux()
    if nom:
        tmux("send-keys", "-t", nom, "-l", texte)
        tmux("send-keys", "-t", nom, "Enter")


def ecran():
    nom = session_tmux()
    r = tmux("capture-pane", "-p", "-t", nom) if nom else None
    return r.stdout if r and r.returncode == 0 else None


def surveiller(memoire, histoire, maintenant):
    vu = ecran()
    if vu is None:
        return
    empreinte = hashlib.sha256(vu.encode()).hexdigest()
    if empreinte != memoire.get("ecran"):
        memoire.update(ecran=empreinte, ecran_depuis=maintenant, bloque_signale=False)
    elif maintenant - memoire.get("ecran_depuis", maintenant) >= SILENCE and not memoire.get("bloque_signale"):
        lignes = "\n".join(vu.rstrip().splitlines()[-40:])
        ecrire(BLOQUE, f"🆘 **Relais : rien ne bouge depuis 35 minutes** ({heure(maintenant)}). "
               f"L'écran de la session :\n```\n{lignes}\n```\nRéponds dans le nœud « {REPONSE} » : "
               "ton texte sera tapé dans la session.")
        memoire["bloque_signale"] = True
        journal("silence de 35 minutes : écran posé dans Glucose")
    r = noeud(REPONSE)
    if not r or r.get("empreinte") == memoire.get("reponse_vue"):
        return
    memoire["reponse_vue"] = r.get("empreinte")
    texte = r.get("texte", "").replace(INVITE_REPONSE, "").strip()
    if texte and auteur_recent(REPONSE, histoire) == "humain":
        taper(texte)
        ecrire(REPONSE, INVITE_REPONSE)
        memoire["reponse_vue"] = (noeud(REPONSE) or {}).get("empreinte")
        journal("sa réponse tapée dans la session")


JETON = MAISON / "Projets/github.md"


def jeton_github(maintenant):
    """Le jeton est-il là, et de quand (il le renouvelle tous les 3 jours) — jamais sa valeur."""
    try:
        present = any(l.strip() and not l.lstrip().startswith("#") for l in JETON.read_text().splitlines())
    except OSError:
        present = False
    if not present:
        return "absent — colle-le dans ~/Projets/github.md"
    jours = (maintenant - JETON.stat().st_mtime) / 86400
    return f"présent (posé il y a {jours:.0f} j)" + (" — à renouveler" if jours >= 3 else "")


def travail(depuis):
    """Ce que la session a produit : ses commits depuis son début, et ce que GitHub n'a pas encore.
    Lu dans Git, sans dépenser un jeton d'usage — de quoi juger ce que coûte chaque pour-cent."""
    def git(*a):
        return subprocess.run(["git", "-C", str(DEPOT), *a], capture_output=True, text=True).stdout.strip()
    faits = git("log", f"--since=@{int(depuis)}", "--format=%h %s").splitlines()
    attente = git("rev-list", "--count", "origin/main..main") or "?"
    liste = "".join(f"\n  - {c[:90]}" for c in faits[:5])
    return f"{len(faits)} commit(s) depuis {heure(depuis)} · {attente} pas encore sur GitHub{liste}"


def battre(memoire, etat, maintenant):
    compte = etat.get("compte") or memoire.get("compte") or "a"
    f = charge(compte, maintenant)
    jauge = " · ".join(f"{t} : {round(f[n][0])} %" for n, t in (("five_hour", "5 h"), ("seven_day", "semaine"))
                       if n in f) or "jauges pas encore lues"
    depuis = lire_json(ETAT / "session.json", {}).get("depuis") or maintenant
    ecrire(BATTEMENT, f"❤️ **Relais** · {heure(maintenant)}\n- compte {compte} · {jauge}\n- {etat['dit']}"
                      f"\n- travail : {travail(depuis)}\n- jeton GitHub : {jeton_github(maintenant)}")
    memoire["battu"] = maintenant


def un_tour():
    maintenant = time.time()
    memoire = lire_json(ETAT / "veilleur.json", {})
    session = lire_json(ETAT / "session.json", {})
    n = noeud(CONSIGNE)
    if n is None:
        journal("Glucose ne répond pas, ou la consigne manque")
        return
    histoire = glucose("histoire", "nombre=200")
    ok, pause = signee(n, memoire, histoire), en_pause(n.get("texte"))
    vivante = ecran() is not None
    if vivante and session.get("passation_demandee") and session.get("laissee") == "relayer":
        tmux("kill-session", "-t", session_tmux() or "")
        journal("passation écrite : session close, relais")
        memoire["compte"] = "b" if session.get("compte") == "a" else "a"
        vivante = False
    if vivante:
        surveiller(memoire, histoire, maintenant)
        if session.get("laissee") in ("pause", "consigne non signée") and ok and not pause:
            taper("[Relais] Reprends selon sa consigne.")
        dit = f"session en cours depuis {heure(session.get('depuis', maintenant))}" + (
            f" — arrêtée : {session['laissee']}" if session.get("laissee") else "")
    elif not ok:
        dit = "en attente : la consigne n'est pas signée par toi (écris-la ou retouche-la)"
    elif pause:
        dit = "en pause (première ligne de la consigne)"
    else:
        compte, libre_a = choisir(memoire.get("compte", "a"), maintenant)
        if compte:
            lancer(compte)
            memoire["compte"] = compte
            dit = f"session lancée sur le compte {compte}"
        else:
            dit = "les comptes sont à leur limite : reprise vers " + (heure(libre_a) if libre_a else "?")
    if maintenant - memoire.get("battu", 0) >= 600 or dit != memoire.get("dit"):
        memoire["dit"] = dit
        battre(memoire, {"compte": memoire.get("compte"), "dit": dit}, maintenant)
    ecrire_json(ETAT / "veilleur.json", memoire)


def veiller():
    journal("veilleur démarré")
    while True:
        try:
            un_tour()
        except Exception as e:  # le service ne meurt pas d'une erreur : il la dit, et recommence
            journal(f"erreur : {e!r}")
        time.sleep(60)


def etat():
    maintenant = time.time()
    for compte in COMPTES:
        print(f"compte {compte} : connecté={connecte(compte)} plein={plein(compte, maintenant)} "
              f"jauges={charge(compte, maintenant)}")
    print("session :", lire_json(ETAT / "session.json", {}))
    print("veilleur :", {k: v for k, v in lire_json(ETAT / "veilleur.json", {}).items() if k != "ecran"})


if __name__ == "__main__":
    geste = sys.argv[1] if len(sys.argv) > 1 else "etat"
    if geste == "installer":
        sys.path.insert(0, str(Path(__file__).parent))
        import installer
        sys.exit(installer.installer())
    sys.exit({"jauges": jauges, "fin-de-tour": fin_de_tour, "veiller": veiller, "etat": etat}[geste]() or 0)
