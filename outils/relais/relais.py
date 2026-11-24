#!/usr/bin/env python3
"""**Le relais** : Glucose avance seul, d'un compte à l'autre, sans qu'il tape « continue ».

Sa demande du 22/11/2026 : Claude travaille en autonomie ; une question s'écrit dans Glucose et
Claude passe à autre chose ; quand un compte arrive à sa limite, la passation s'écrit dans Glucose
et l'autre compte reprend. La bibliothèque standard seulement ; la loi sans effet de bord est dans
`loi.py`, le mode d'emploi dans `MODE-EMPLOI.md`.

  relais.py jauges       la barre d'état de Claude Code : garde les deux jauges du compte
  relais.py fin-de-tour  le crochet « Stop » : continuer selon SA consigne, ou écrire la passation
  relais.py parole       le crochet « UserPromptSubmit » : une session à lui prend la main
  relais.py rendre       une session à lui rend la main en partant
  relais.py veiller      le service : lance, relaie, réveille, lui répond, bat le cœur dans Glucose
  relais.py installer    une fois : Glucose, les réglages de Claude Code, le second compte, le service
  relais.py etat         ce que le relais sait, en clair

Quatre règles. **Une seule session travaille à la fois** (sa parole du 23/11) : dès qu'il écrit à
une session à lui, celle du relais écrit sa passation et s'efface ; un seul veilleur. **Sa consigne
n'est suivie que si c'est LUI qui l'a écrite en dernier** — de ce Glucose ou d'un autre appareil du
canevas partagé —, jamais une IA. **Ce qu'il écrit est lu** : donné à la session qui travaille, ou
à la suivante. **Rien n'attend en silence** : 35 minutes sans un changement à l'écran, et l'écran
part dans Glucose, où il répond.
"""
import fcntl
import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path

# Lancé par `python3 -I`, le dossier du programme n'est pas dans le chemin : `loi` y est. À la fin,
# pour que la bibliothèque standard passe toujours devant.
sys.path.append(str(Path(__file__).resolve().parent))
from loi import (AVERTIR, BATTEMENT, BLOQUE, CONSIGNE, INVITE_REPONSE, NOTIFICATIONS, PASSATION, REGLES,  # noqa: E402
                 REPONSE, SILENCE, TEXTE_DEPART, TEXTE_PASSATION, TEXTE_RELANCE, TEXTES, contexte_plein,
                 decider, depart, dernier_geste, dire_session, en_pause, heure, humaine, raison_continuer,
                 raison_entendre, raison_passation, raison_prevenir, ses_fleches, ses_paroles, signee)
from loi import MODE_EMPLOI, RAISON_PASSATION, SEUIL_JETONS, lire_ids  # noqa: E402,F401  (pour les épreuves)
from machine import (DEPOT, MAISON, claude_binaire, dernier_signe, ecrire_json, jeton_github,  # noqa: E402
                     lire_json, travail)

ETAT = MAISON / ".local/state/relais"
# a : le dossier par défaut de Claude Code (~/.claude) ; b : son propre dossier — sa connexion à lui,
# les mémoires et les journaux partagés par liens (installer).
COMPTES = {"a": None, "b": MAISON / ".claude-compte-b"}
# La passation coûte un tour : la demander avant le mur, pas au mur. 90 % partout (sa règle du 23/11).
SEUILS = {"five_hour": 90.0, "seven_day": 90.0}
# Le mur : une fenêtre pleine, et le serveur refuse le compte (« You've hit your … limit »).
MUR = {"five_hour": 100.0, "seven_day": 100.0}


def nom_tmux(compte):
    """Une session tmux par compte ; une seule vit à la fois, que montrent ses deux VS Code (voir.sh)."""
    return f"relais-{compte}"


# ── Petits outils ─────────────────────────────────────────────────────────────────────────────

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


def plein(compte, maintenant, seuils=SEUILS):
    """Ce compte a-t-il passé ses seuils ? Et si oui, quand redevient-il libre (None : il l'est).

    Libre seulement quand **toutes** ses fenêtres pleines sont remises : la plus lointaine de leurs
    remises, jamais la plus proche (le 23/11, une semaine pleine jusqu'à 16 h et cinq heures
    pleines jusqu'à 4 h ne rendaient pas le compte à 4 h).

    Une jauge lue il y a longtemps reste vraie de ce côté-là : dans une fenêtre, l'usage ne fait que
    s'ajouter jusqu'à sa remise. Elle ne peut que **sous-estimer** (le compte servi ailleurs,
    claude.ai compris) — d'où le veilleur, qui ferme une session que le serveur refuse."""
    murs = [remise for nom, (pct, remise) in charge(compte, maintenant).items()
            if pct >= seuils.get(nom, 100.0)]
    return (True, max(murs)) if murs else (False, None)


def ailleurs(compte, maintenant):
    """Un autre compte peut-il reprendre maintenant ? (oui, None) — ou (non, l'heure où le premier
    se libère). Sans lui, la passation fermerait la session pour rien : le 23/11 à 10 h 08, la session
    a quitté le compte a à 91 % quand b était plein jusqu'à 16 h — trois heures sans personne."""
    murs = []
    for autre_compte in COMPTES:
        if autre_compte == compte or not connecte(autre_compte):
            continue
        au_seuil, libre_a = plein(autre_compte, maintenant)
        if not au_seuil:
            return True, None
        murs.append(libre_a)
    return False, min(murs, default=None)


def jauges():
    try:
        entree = json.load(sys.stdin)
    except ValueError:
        entree = {}
    limites = entree.get("rate_limits") or {}
    if limites:
        ecrire_json(ETAT / f"jauges-{compte_courant()}.json", {"lu": time.time(), "rate_limits": limites})
    fenetre = entree.get("context_window") or {}
    if os.environ.get("RELAIS") == "1" and fenetre.get("used_percentage") is not None:
        ecrire_json(ETAT / "contexte.json", {"lu": time.time(), "pct": fenetre["used_percentage"],
                                             "taille": fenetre.get("context_window_size")})
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


def ses_textes(memoire, histoire):
    """Ses versions des textes du relais (`TEXTES`), celles que sa main a écrites en dernier."""
    lus = (glucose("lire", "tableau=main", *(f"ids={i}" for i in TEXTES)) or {}).get("noeuds", [])
    return {n["id"]: n["texte"].strip() for n in lus if n.get("id") in TEXTES
            and (n.get("texte") or "").strip() and signee(n, memoire, histoire, n["id"])}


def gestes_neufs(memoire):
    """Les gestes du document que le veilleur n'a pas encore vus (ceux d'après `vu`), **tous** —
    le 23/11, 127 gestes en une heure depuis son autre appareil, quand le relais n'en relisait que
    les 200 derniers. Deux lectures : combien il y en a, puis ceux-là. Rend {"gestes", "total"} ;
    None si Glucose ne répond pas. La première fois, rien d'ancien."""
    tete = glucose("histoire", "nombre=1")
    if tete is None:
        return None
    total = int(tete.get("total") or 0)
    vu = min(int(memoire.get("vu", total)), total)
    if vu == total:
        return {"gestes": [], "total": total}
    lu = glucose("histoire", f"nombre={total - vu}")
    return None if lu is None else {"gestes": lu.get("gestes", []), "total": total}


# ── Une seule session ─────────────────────────────────────────────────────────────────────────

def main_humaine(maintenant):
    """Une session à lui tient-elle le dépôt ? Son dernier signe, ou None après SILENCE sans un."""
    dernier = max(map(dernier_signe, lire_json(ETAT / "baux.json", {}).values()), default=0.0)
    return dernier if maintenant - dernier < SILENCE else None


def parole():
    """Le crochet « UserPromptSubmit » : il écrit à une session à lui (VS Code, un terminal) dans ce
    dépôt — **elle prend la main**, et le relais s'efface : une seule session à la fois (sa parole du
    23/11). Si une session du relais vit encore, elle l'apprend (le texte imprimé lui est donné)."""
    if os.environ.get("RELAIS") == "1":
        return 0  # le relais se parle à lui-même : ses propres sessions ne prennent pas la main
    try:
        entree = json.load(sys.stdin)
    except ValueError:
        return 0
    if (entree.get("prompt") or "").lstrip().startswith(NOTIFICATIONS):
        return 0  # réveillée par sa propre commande, une session ne prend pas la main : il n'a rien écrit
    ici = Path(entree.get("cwd") or "/").resolve()
    if DEPOT.resolve() not in (ici, *ici.parents):
        return 0
    maintenant = time.time()
    baux = {s: b for s, b in lire_json(ETAT / "baux.json", {}).items()
            if maintenant - dernier_signe(b) < SILENCE}
    baux[entree.get("session_id") or "?"] = {"transcript": entree.get("transcript_path"), "depuis": maintenant}
    ecrire_json(ETAT / "baux.json", baux)
    if session_tmux():
        print(AVERTIR)
    return 0


def rendre():
    """Une session à lui rend la main en partant : le relais reprend au tour suivant."""
    ecrire_json(ETAT / "baux.json", {})
    journal("la main rendue au relais")
    return 0


def un_seul_veilleur():
    """Le verrou du veilleur : deux veilleurs lanceraient deux sessions. Un second attend que le
    premier s'arrête — le système rend le verrou avec le processus, même tué."""
    ETAT.mkdir(parents=True, exist_ok=True)
    verrou = open(ETAT / "veilleur.lock", "w")
    try:
        fcntl.flock(verrou, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        journal("un veilleur tourne déjà : celui-ci attend qu'il s'arrête")
        fcntl.flock(verrou, fcntl.LOCK_EX)
    return verrou


# ── Le crochet « Stop » ───────────────────────────────────────────────────────────────────────

def demander_passation(session, maintenant, motif="limite"):
    """Retient la demande, son motif et l'empreinte de la passation d'avant (la première fois
    seulement) : c'est à elle qu'on saura si la session l'a récrite."""
    if not session.get("passation_demandee"):
        session["passation_demandee"] = maintenant
        session["passation_pourquoi"] = motif
        session["passation_avant"] = (noeud(PASSATION) or {}).get("empreinte")


def passation_ecrite(session):
    """Le nœud de la passation a-t-il changé depuis la demande ?"""
    if not session.get("passation_demandee"):
        return False
    maintenant = (noeud(PASSATION) or {}).get("empreinte")
    return maintenant is not None and maintenant != session.get("passation_avant")


def fin_de_tour():
    if os.environ.get("RELAIS") != "1":
        return 0  # une session à lui : le relais n'y touche pas
    try:
        entree = json.load(sys.stdin)
    except ValueError:
        entree = {}
    maintenant = time.time()
    session = lire_json(ETAT / "session.json", {})
    memoire = lire_json(ETAT / "veilleur.json", {})
    n = noeud(CONSIGNE)
    texte = (n or {}).get("texte", "")
    compte = compte_courant()
    motif = "retrait" if main_humaine(maintenant) else "contexte" if contexte_plein(
        lire_json(ETAT / "contexte.json", {}), session) else (
        "limite" if plein(compte, maintenant)[0] and ailleurs(compte, maintenant)[0] else None)
    textes = memoire.get("textes", {})
    action, pourquoi = decider(session, texte, en_pause(texte), signee(n, memoire, gestes_neufs(memoire)),
                               motif, maintenant, passation_ecrite(session),
                               bool((entree or {}).get("stop_hook_active")))
    session["arrets"] = [t for t in session.get("arrets", []) if maintenant - t < 120] + [maintenant]
    if action == "passation":
        demander_passation(session, maintenant, motif or "limite")
        print(json.dumps({"decision": "block", "reason": raison_passation(session.get("passation_pourquoi"),
                                                                                textes.get(TEXTE_PASSATION))},
                         ensure_ascii=False))
    elif action == "continuer":
        session.pop("laissee", None)
        print(json.dumps({"decision": "block", "reason": raison_continuer(texte, textes.get(TEXTE_RELANCE))},
                         ensure_ascii=False))
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
    """Le compte qui travaille : le courant s'il est sous ses seuils, sinon un autre ; faute de
    mieux, un compte encore sous le mur — mieux vaut finir sa jauge que dormir (le 23/11, trois
    heures perdues à 91 %). Aucun : (None, l'heure où le premier se libère)."""
    ordre = [courant] + [c for c in COMPTES if c != courant]
    murs = []
    for seuils in (SEUILS, MUR):
        murs = []
        for compte in ordre:
            if not connecte(compte):
                continue
            au_seuil, libre_a = plein(compte, maintenant, seuils)
            if not au_seuil:
                return compte, None
            murs.append(libre_a)
    return None, (min(murs) if murs else None)


def session_tmux():
    """Le nom de la session du relais qui vit, s'il y en a une."""
    for compte in COMPTES:
        if tmux("has-session", "-t", nom_tmux(compte)).returncode == 0:
            return nom_tmux(compte)
    return None


def importants():
    """Ce que sa consigne désigne par ses flèches (`ses_fleches`), lu de toute l'histoire."""
    tete = glucose("histoire", "nombre=1")
    lu = glucose("lire", "tableau=main")
    if not tete or not lu:
        return []
    histoire = glucose("histoire", f"nombre={int(tete.get('total') or 1)}") or {}
    return ses_fleches(lu.get("noeuds", []), histoire.get("gestes", []))


def lancer(compte, paroles=(), texte=None, designes=()):
    """Lance la session du relais sur ce compte — **jamais une seconde** : s'il en vit une, rien."""
    if session_tmux():
        journal("une session du relais vit déjà : pas de seconde")
        return False
    reglages = ["RELAIS=1", f"RELAIS_COMPTE={compte}"]
    if COMPTES[compte]:
        reglages.append(f"CLAUDE_CONFIG_DIR={COMPTES[compte]}")
    binaire = claude_binaire()
    # --ide : branché sur le VS Code de ce compte. --no-chrome : sans lui, un compte qui n'a jamais
    # répondu à « Claude in Chrome » se fige au démarrage sur cette question (le 23/11, 1 h 48). Le
    # bypass, jamais le mode auto (sa réponse du 23/11 : « il faut que Claude ne soit JAMAIS bloqué ») :
    # le mode auto s'est figé deux fois sur une permission ; ses réglages ont déjà
    # skipDangerousModePermissionPrompt, sinon Claude Code poserait sa question au démarrage (fiche 134).
    fait = tmux("new-session", "-d", "-s", nom_tmux(compte), "-x", "200", "-y", "50", "-c", str(DEPOT),
                "env", *reglages, binaire, "--ide", "--no-chrome", "--permission-mode", "bypassPermissions",
                "--allow-dangerously-skip-permissions", "--remote-control", f"relais-{compte}",
                "--append-system-prompt", REGLES, depart(compte, paroles, texte, designes))
    if fait.returncode != 0:
        journal(f"tmux refuse la session : {fait.stderr.strip()[:200]}")
        return False
    ecrire_json(ETAT / "session.json", {"compte": compte, "depuis": time.time()})
    notes = f", avec {len(paroles)} nœud(s) qu'il a écrits" if paroles else ""
    journal(f"session lancée sur le compte {compte}{notes} ({binaire})")
    return True


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


def garder(session, maintenant, humaine_la=None):
    """Ce que le veilleur fait d'une session vivante, à chaque tour :

    * « fermer » — **le mur** : une fenêtre pleine, le serveur la refuse. Elle ne peut plus rien
      écrire, et tmux la tiendrait ouverte sur son message de limite, prise pour vivante (le 23/11,
      la session b, lancée sur une semaine pleine que personne n'avait lue, a figé le relais) ;
    * « retrait » — une session à lui tient le dépôt (`humaine_la`) : une seule à la fois ;
    * « passation » — un seuil passé quand un autre compte peut reprendre, et rien encore demandé :
      **la passation n'attend pas la fin du tour** (le 23/11 à 0 h 15, un tour durait depuis 48
      minutes, le compte à 97 %) ;
    * « prevenir » — un seuil passé et personne pour reprendre : elle continue jusqu'au mur, prévenue
      une fois ;
    * « contexte » — la mémoire du modèle à 90 % : la passation, puis une conversation neuve ;
    * None : rien à faire."""
    compte = session.get("compte")
    if not compte:
        return None
    if plein(compte, maintenant, MUR)[0]:
        return "fermer"
    if session.get("passation_demandee"):
        return None
    if humaine_la:
        return "retrait"
    if contexte_plein(lire_json(ETAT / "contexte.json", {}), session):
        return "contexte"
    if not plein(compte, maintenant)[0]:
        return None
    if ailleurs(compte, maintenant)[0]:
        return "passation"
    return None if session.get("prevenue") else "prevenir"


def fermer(session, raison):
    """Ferme la session du relais et le dit : avec ou sans sa passation écrite."""
    tmux("kill-session", "-t", session_tmux() or "")
    dite = "écrite" if passation_ecrite(session) else "PAS récrite (la précédente reste)"
    journal(f"{raison} : session close, passation {dite}, relais")
    session["laissee"] = raison
    ecrire_json(ETAT / "session.json", session)


def autre(compte):
    return next((c for c in COMPTES if c != compte), compte)


def tenir(session, memoire, humaine_la, maintenant):
    """Ce que le veilleur fait de la session du relais qui vit (`garder`) ; False s'il l'a close."""
    geste = garder(session, maintenant, humaine_la)
    if geste == "fermer" or (session.get("passation_demandee") and session.get("laissee") == "relayer"):
        retrait = geste != "fermer" and session.get("passation_pourquoi") == "retrait"
        fermer(session, "compte à sa limite" if geste == "fermer" else
               "en retrait devant sa session" if retrait else "passation demandée")
        if not retrait:
            memoire["compte"] = autre(session.get("compte"))
        return False
    if geste in ("passation", "retrait", "contexte"):
        demander_passation(session, maintenant, geste if geste != "passation" else "limite")
        taper(raison_passation(session["passation_pourquoi"], memoire.get("textes", {}).get(TEXTE_PASSATION)))
        journal({"retrait": "une session à lui prend la main", "contexte": "la mémoire du modèle à 90 %"}
                .get(geste, "seuil passé en plein tour")
                + f" : passation demandée au compte {session.get('compte')}")
    elif geste == "prevenir":
        session["prevenue"] = maintenant
        taper(raison_prevenir(ailleurs(session["compte"], maintenant)[1]))
        journal(f"seuil passé, aucun compte pour reprendre : le compte {session['compte']} va jusqu'au mur")
    if geste:
        ecrire_json(ETAT / "session.json", session)
    return True


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
    geste = dernier_geste(REPONSE, histoire)
    if geste is None:
        return  # son geste n'est pas encore dans l'histoire lue : il le sera au tour suivant
    memoire["reponse_vue"] = r.get("empreinte")
    texte = r.get("texte", "").replace(INVITE_REPONSE, "").strip()
    if texte and humaine(geste.get("auteur")):
        taper(texte)
        ecrire(REPONSE, INVITE_REPONSE)
        memoire["reponse_vue"] = (noeud(REPONSE) or {}).get("empreinte")
        journal("sa réponse tapée dans la session")


def entendre(memoire, histoire):
    """Ce qu'il écrit pendant qu'une session travaille lui est tapé **une fois qu'il a fini
    d'écrire** — un tour du veilleur sans un geste de lui —, en un seul message."""
    if memoire.get("paroles") and not any(humaine(g.get("auteur")) for g in histoire.get("gestes", [])):
        taper(raison_entendre(memoire.pop("paroles")))
        journal("ses notes tapées dans la session")


def relancer(session, texte, ok, pause, sien=None):
    """Une session laissée à l'arrêt repart quand rien ne l'arrête plus : la pause levée, la
    consigne de nouveau la sienne, ou une relance que le crochet ne pouvait plus faire (`decider`).
    Une fois : la demande tapée, l'arrêt est oublié (avant, il se retapait à chaque tour)."""
    if session.get("laissee") in ("pause", "consigne non signée", "relancer") and ok and not pause:
        taper(raison_continuer(texte, sien))
        session.pop("laissee")
        ecrire_json(ETAT / "session.json", session)
        journal("session relancée selon sa consigne")


def jauge_en_mots(compte, maintenant):
    f = charge(compte, maintenant)
    mots = [f"{t} {round(f[n][0])} %" for n, t in (("five_hour", "5 h"), ("seven_day", "semaine")) if n in f]
    au_seuil, libre_a = plein(compte, maintenant)
    if au_seuil and libre_a:
        mots.append(f"libre à {heure(libre_a)}")
    return ", ".join(mots) or "jauges pas encore lues"


def battre(memoire, compte, dit, maintenant):
    """Le ❤️ : ce que fait le relais, les deux comptes, de qui vient la consigne suivie, le travail."""
    comptes = " · ".join(f"{c}{' (au travail)' if c == compte else ''} : {jauge_en_mots(c, maintenant)}"
                         for c in COMPTES if connecte(c))
    par = f"\n- consigne suivie : écrite par {memoire['consigne_par']}" if memoire.get("consigne_par") else ""
    depuis = lire_json(ETAT / "session.json", {}).get("depuis") or maintenant
    ecrire(BATTEMENT, f"❤️ **Relais** · {heure(maintenant)}\n- {dit}\n- comptes — {comptes}{par}"
                      f"\n- travail : {travail(depuis)}\n- jeton GitHub : {jeton_github(maintenant)}")
    memoire["battu"] = maintenant


def attendre(memoire, ok, pause, humaine_la, maintenant):
    """Aucune session du relais ne vit : en lancer une, ou dire pourquoi pas."""
    if humaine_la:
        return (f"en retrait : une session de Claude à toi travaille sur le dépôt (son dernier signe : "
                f"{heure(humaine_la)}) — je reprends {SILENCE // 60} minutes après, ou dès qu'elle me rend "
                "la main")
    if not ok:
        return "en attente : une IA a changé la consigne après toi — retouche-la pour qu'elle reparte"
    if pause:
        return "en pause (première ligne de la consigne)"
    compte, libre_a = choisir(memoire.get("compte", "a"), maintenant)
    if not compte:
        return "les comptes sont à leur limite : reprise vers " + (heure(libre_a) if libre_a else "?")
    if not lancer(compte, memoire.get("paroles", []), memoire.get("textes", {}).get(TEXTE_DEPART),
                  importants()):
        return "une session du relais vit déjà : jamais une seconde"
    memoire["compte"], memoire["paroles"] = compte, []
    return f"session lancée sur le compte {compte}"


def un_tour():
    maintenant = time.time()
    memoire = lire_json(ETAT / "veilleur.json", {})
    session = lire_json(ETAT / "session.json", {})
    # L'histoire avant les nœuds : un geste qui tombe entre les deux attend le tour suivant, au lieu
    # d'être pris pour l'auteur d'une empreinte qu'il n'a pas écrite.
    neufs = gestes_neufs(memoire)
    n = noeud(CONSIGNE)
    if n is None:
        journal("Glucose ne répond pas, ou la consigne manque")
        return
    histoire = neufs or {"gestes": []}
    memoire["textes"] = ses_textes(memoire, histoire)
    ok, pause = signee(n, memoire, histoire), en_pause(n.get("texte"))
    memoire["paroles"] = ses_paroles(histoire, memoire.get("paroles", []))
    humaine_la = main_humaine(maintenant)
    vivante = ecran() is not None and tenir(session, memoire, humaine_la, maintenant)
    if vivante:
        surveiller(memoire, histoire, maintenant)
        if not humaine_la and not session.get("passation_demandee"):
            entendre(memoire, histoire)
            relancer(session, n.get("texte", ""), ok, pause, memoire["textes"].get(TEXTE_RELANCE))
        dit = dire_session(session, memoire, maintenant)
    else:
        dit = attendre(memoire, ok, pause, humaine_la, maintenant)
    if neufs is not None:
        memoire["vu"] = neufs["total"]
    if maintenant - memoire.get("battu", 0) >= 600 or dit != memoire.get("dit"):
        memoire["dit"] = dit
        battre(memoire, session.get("compte") if vivante else None, dit, maintenant)
    ecrire_json(ETAT / "veilleur.json", memoire)


def veiller():
    verrou = un_seul_veilleur()  # gardé ouvert tant qu'il vit : le verrou meurt avec lui
    journal(f"veilleur démarré (verrou {verrou.name})")
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
    humaine_la = main_humaine(maintenant)
    print("session du relais :", session_tmux() or "aucune", lire_json(ETAT / "session.json", {}))
    print("session à lui :", f"dernier signe à {heure(humaine_la)}" if humaine_la else "aucune",
          lire_json(ETAT / "baux.json", {}))
    print("veilleur :", {k: v for k, v in lire_json(ETAT / "veilleur.json", {}).items() if k != "ecran"})
    return 0


if __name__ == "__main__":
    geste = sys.argv[1] if len(sys.argv) > 1 else "etat"
    if geste == "installer":
        import installer
        sys.exit(installer.installer())
    sys.exit({"jauges": jauges, "fin-de-tour": fin_de_tour, "parole": parole, "rendre": rendre,
              "veiller": veiller, "etat": etat}[geste]() or 0)
