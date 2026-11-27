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
from loi import (AVERTIR, BATTEMENT, BLOQUE, CONSIGNE, FERMEE_ENDORMIE, INVITE_REPONSE, NOTIFICATIONS,  # noqa: E402
                 PASSATION, REGLES, REPONSE, SILENCE, TEXTE_DEPART, TEXTE_PASSATION, TEXTE_RELANCE, TEXTES,
                 contexte_plein, decider, depart, dernier_geste, dire_session, en_pause, endormie, heure, humaine, meme_etat,
                 il_ecrit, raison_continuer, raison_entendre, raison_passation, ses_fleches, ses_paroles, signee)
from loi import ENDORMIE, MODE_EMPLOI, POURQUOI, RAISON_PASSATION, SEUIL_JETONS, lire_ids  # noqa: E402,F401  (épreuves)
from machine import (DEPOT, MAISON, claude_binaire, dernier_signe, ecrire_json, jeton_github,  # noqa: E402
                     lire_json, travail)
import serveur  # noqa: E402

ETAT = MAISON / ".local/state/relais"
# a : le dossier par défaut de Claude Code (~/.claude) ; b : son propre dossier — sa connexion à lui,
# les mémoires et les journaux partagés par liens (installer).
COMPTES = {"a": None, "b": MAISON / ".claude-compte-b"}
# La passation coûte un tour : la demander avant le mur, pas au mur. 90 % des cinq heures (sa règle
# du 23/11) ; 97 % de la semaine (sa réponse du 25/11 dans `relais-reserve` : « plus que 90 %, peut-être
# 97 % si possible ») — le reste, jusqu'au bord, ne sert qu'à lui répondre (RESERVE-1).
SEUILS = {"five_hour": 90.0, "seven_day": 97.0}
# Le mur : une fenêtre pleine, et le serveur refuse le compte (« You've hit your … limit »).
MUR = {"five_hour": 100.0, "seven_day": 100.0}
# **Le bord : le mur moins une passation**, qui coûte au plus un point des cinq heures (quatorze
# passations mesurées, fiche 167). Une session ne va jamais au mur : coupée en route, elle perd son
# travail — le 24/11 à 19 h 37, poussée jusqu'au mur faute d'autre compte, elle a laissé du travail
# non commité et peut-être un sabotage dans le code ; il a dû l'arrêter lui-même.
BORD = {"five_hour": 99.0, "seven_day": 99.0}
# La plus courte des fenêtres de jauge : ce qu'un compte peut encore dépenser d'une traite.
FENETRE = 5 * 3600


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


def murs_de_l_abonnement(compte, maintenant, seuils=SEUILS):
    """Les remises des fenêtres de l'abonnement qui ont passé leur seuil — sans ses crédits."""
    return [remise for nom, (pct, remise) in charge(compte, maintenant).items()
            if nom != "credits" and pct >= seuils.get(nom, 100.0)]


def credits_libres(compte, maintenant, seuils=SEUILS):
    """**Des crédits en plus, achetés, sous leur plafond** (fiche 182) : l'abonnement au bout, le
    compte travaille encore — sur ce qu'il a payé pour ça, jusqu'au même seuil que sa semaine."""
    pct, _ = charge(compte, maintenant).get("credits", (100.0, 0.0))
    return pct < seuils.get("seven_day", 100.0)


def plein(compte, maintenant, seuils=SEUILS):
    """Ce compte a-t-il passé ses seuils ? Et si oui, quand redevient-il libre (None : il l'est).

    Libre seulement quand **toutes** ses fenêtres pleines sont remises : la plus lointaine de leurs
    remises, jamais la plus proche (le 23/11, une semaine pleine jusqu'à 16 h et cinq heures
    pleines jusqu'à 4 h ne rendaient pas le compte à 4 h). Ou tant que ses crédits en plus tiennent.

    Une jauge gardée peut se tromper dans les deux sens : trop basse (le compte servi ailleurs,
    claude.ai compris — le veilleur ferme une session que le serveur refuse) ; trop haute (une limite
    remise par Anthropic, des crédits achetés : le 27/11, 99 % gardé, 2 % au serveur). Celle-là, le
    veilleur la redemande au serveur (`relire`) : on ne la croit jamais pleine sans lui."""
    murs = murs_de_l_abonnement(compte, maintenant, seuils)
    return (True, max(murs)) if murs and not credits_libres(compte, maintenant, seuils) else (False, None)


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


def garder_l_histoire(compte, avant, limites, session):
    """**L'histoire des jauges** (fiche 147) : une ligne chaque fois qu'une jauge bouge — l'heure, le
    compte, la conversation, les deux pourcentages, la remise des 5 h. Croisée avec les jetons des
    conversations (leurs journaux), elle dit ce que l'abonnement compte vraiment et ce qu'un
    changement fait gagner ; sans elle, on n'avait que des relevés notés à la main (fiche 142)."""
    def pct(fenetres, nom):
        return (fenetres.get(nom) or {}).get("used_percentage")
    if all(pct(avant, nom) == pct(limites, nom) for nom in SEUILS):
        return
    ligne = {"t": round(time.time()), "compte": compte, "session": session,
             **{nom: pct(limites, nom) for nom in SEUILS},
             "remise": (limites.get("five_hour") or {}).get("resets_at")}
    with open(ETAT / "jauges-histoire.jsonl", "a") as f:
        f.write(json.dumps(ligne) + "\n")


def jauges():
    try:
        entree = json.load(sys.stdin)
    except ValueError:
        entree = {}
    limites = entree.get("rate_limits") or {}
    if limites:
        compte = compte_courant()
        garde = lire_json(ETAT / f"jauges-{compte}.json", {})
        avant = garde.get("rate_limits") or {}
        # Ses crédits, la barre d'état ne les dit pas : ceux que le serveur a dits restent.
        gardes = {**({"credits": avant["credits"]} if "credits" in avant else {}), **limites}
        ecrire_json(ETAT / f"jauges-{compte}.json", {"lu": time.time(), "rate_limits": gardes,
                                                     **({"serveur": garde["serveur"]} if "serveur" in garde else {})})
        garder_l_histoire(compte, avant, limites, entree.get("session_id"))
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


def relire(maintenant):
    """**Une jauge qui dit « plein », redemandée au serveur** (fiche 182), au plus toutes les dix
    minutes par compte : seule la barre d'état d'une session la relisait, et un compte cru plein ne
    reçoit plus de session — le 27/11, le relais a dormi six heures sur un compte remis à 2 %. Les
    crédits en plus aussi, tant qu'ils servent : la barre d'état ne les compte pas."""
    essais = lire_json(ETAT / "serveur.json", {})
    for compte in COMPTES:
        if not connecte(compte) or not murs_de_l_abonnement(compte, maintenant) \
                or maintenant - essais.get(compte, {}).get("t", 0) < serveur.INTERVALLE:
            continue
        limites, pourquoi = serveur.demander(COMPTES[compte] or MAISON / ".claude")
        if limites is None and pourquoi != essais.get(compte, {}).get("echec"):  # dit une fois, pas toutes les dix minutes
            journal(f"jauges du compte {compte} non relues au serveur : {pourquoi}")
        essais[compte] = {"t": maintenant, "echec": pourquoi}
        if limites is None:
            continue
        avant = lire_json(ETAT / f"jauges-{compte}.json", {}).get("rate_limits") or {}
        ecrire_json(ETAT / f"jauges-{compte}.json", {"lu": maintenant, "serveur": maintenant, "rate_limits": limites})
        garder_l_histoire(compte, avant, limites, "serveur")
        journal(f"jauges du compte {compte} relues au serveur : "
                + ", ".join(f"{n} {round(f['used_percentage'])} %" for n, f in limites.items()))
    ecrire_json(ETAT / "serveur.json", essais)


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
        lire_json(ETAT / "contexte.json", {}), session) else "renouveler" if session.get("renouveler") else (
        "limite" if plein(compte, maintenant)[0] and ailleurs(compte, maintenant)[0] else
        "repos" if plein(compte, maintenant, repli(session.get("notes")))[0] else
        "reserve" if gardee(compte, maintenant) else None)
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


def gardee(compte, maintenant):
    """**La fin de sa semaine, gardée pour ses notes** (RESERVE-1, fiche 163) : sa semaine a passé
    son seuil, et sa remise est à plus d'une fenêtre de cinq heures. Ce qui reste ne sert plus qu'à
    lui répondre quand il écrit : brûlé au travail, il laissait le relais sourd des jours entiers (le
    25/11, le compte a à 17 % de sa semaine en trois heures de travail, b plein jusqu'au 30/11 ;
    sa consigne : « h24 actif »). Dans la dernière fenêtre avant la remise, la réserve se dépense au
    travail : sinon elle serait perdue. Rendue (None) : l'heure où elle se rend au travail."""
    pct, remise = charge(compte, maintenant).get("seven_day", (0.0, 0.0))
    if credits_libres(compte, maintenant):  # ses crédits achetés : la semaine n'est plus la fin
        return None
    return remise - FENETRE if pct >= SEUILS["seven_day"] and remise - maintenant > FENETRE else None


def repli(notes):
    """**Jusqu'où un compte sert faute de mieux** (aucun sous ses seuils). Ses notes, jusqu'au bord :
    lui répondre passe avant tout. Le travail, jamais au-delà du seuil de ses cinq heures : ce qu'une
    fenêtre ne dépense pas reste à la semaine, qui borne tout (un compte la remplit en huit fenêtres
    sur trente-trois) — attendre la remise ne perd rien. Sa semaine, jusqu'au bord, dans sa seule
    dernière fenêtre (`gardee`) : ce qu'il en reste serait perdu."""
    return BORD if notes else {"five_hour": SEUILS["five_hour"], "seven_day": BORD["seven_day"]}


def choisir(courant, maintenant, notes=False):
    """Le compte qui travaille : le courant s'il est sous ses seuils, sinon un autre ; faute de
    mieux, un compte sous son repli (`repli`), **sauf la fin d'une semaine** (`gardee`), qui ne sert
    qu'à répondre à ses notes (`notes`). Aucun : (None, l'heure où le premier se libère ou se rend au
    travail)."""
    ordre = [courant] + [c for c in COMPTES if c != courant]
    murs = []
    for seuils in (SEUILS, repli(notes)):
        murs = []
        if seuils is not SEUILS:  # faute de mieux, d'abord celui qui a le plus de marge : ses notes y tiennent
            ordre.sort(key=lambda c: max((pct for pct, _ in charge(c, maintenant).values()), default=0.0))
        for compte in ordre:
            if not connecte(compte):
                continue
            au_seuil, libre_a = plein(compte, maintenant, seuils)
            rendue = None if au_seuil or notes or seuils is SEUILS else gardee(compte, maintenant)
            if not au_seuil and rendue is None:
                return compte, None
            murs.append(libre_a if au_seuil else rendue)
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


# **Les outils qu'une session du relais ne sert jamais** (fiche 147) : chaque requête relit leur mode
# d'emploi, et celui d'Artifact est le plus long de tous. Personne ne lit une page ni ne répond à une
# question en menu (tout va dans son Glucose), personne ne programme un réveil ni n'envoie d'avis.
# Agent reste : sans lui, aucune page web ne se lit (« chercher sur internet avant de concevoir »).
SANS_OUTILS = ("Artifact", "AskUserQuestion", "ScheduleWakeup", "SendFeedback", "ReportFindings")


def lancer(compte, paroles=(), texte=None, designes=()):
    """Lance la session du relais sur ce compte — **jamais une seconde** : s'il en vit une, rien."""
    if session_tmux():
        journal("une session du relais vit déjà : pas de seconde")
        return False
    # Sans les connecteurs de claude.ai (Gmail, Drive, Hugging Face, sa porte publique…) : aucune
    # session du relais ne s'en sert, et leurs outils, leurs savoir-faire et leurs consignes pesaient
    # ~13 000 jetons relus à chaque réponse (le 25/11 : 27 465 au premier tour, 40 853 au second).
    reglages = ["RELAIS=1", f"RELAIS_COMPTE={compte}", "ENABLE_CLAUDEAI_MCP_SERVERS=false"]
    if COMPTES[compte]:
        reglages.append(f"CLAUDE_CONFIG_DIR={COMPTES[compte]}")
    binaire = claude_binaire()
    garde = bool(gardee(compte, time.time()))
    orpheline = lire_json(ETAT / "session.json", {}).get("laissee") == FERMEE_ENDORMIE
    # --ide : branché sur le VS Code de ce compte. --no-chrome : sans lui, un compte qui n'a jamais
    # répondu à « Claude in Chrome » se fige au démarrage sur cette question (le 23/11, 1 h 48). Le
    # bypass, jamais le mode auto (sa réponse du 23/11 : « il faut que Claude ne soit JAMAIS bloqué ») :
    # le mode auto s'est figé deux fois sur une permission ; ses réglages ont déjà
    # skipDangerousModePermissionPrompt, sinon Claude Code poserait sa question au démarrage (fiche 134).
    fait = tmux("new-session", "-d", "-s", nom_tmux(compte), "-x", "200", "-y", "50", "-c", str(DEPOT),
                "env", *reglages, binaire, "--ide", "--no-chrome", "--permission-mode", "bypassPermissions",
                "--allow-dangerously-skip-permissions", "--disallowedTools", ",".join(SANS_OUTILS),
                "--remote-control", f"relais-{compte}",
                "--append-system-prompt", REGLES, depart(compte, paroles, texte, designes, garde, orpheline))
    if fait.returncode != 0:
        journal(f"tmux refuse la session : {fait.stderr.strip()[:200]}")
        return False
    ecrire_json(ETAT / "session.json", {"compte": compte, "depuis": time.time(), **({"notes": True} if garde else {})})
    notes = f", avec {len(paroles)} nœud(s) qu'il a écrits" if paroles else ""
    notes += ", sur la fin de sa semaine, pour ses notes seulement" if garde else ""
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
    * « repos » — un seuil passé, personne pour reprendre, et le compte au-delà de son repli
      (`repli`) : la passation, puis le relais attend la remise — **jamais au mur** (`BORD`) ;
    * « reserve » — sa semaine au seuil, personne pour reprendre, la fin de sa semaine gardée pour
      ses notes (`gardee`) : la passation, sauf pour une session lancée pour elles (`notes`), qui
      la fera à la fin de son tour (`fin_de_tour`) ;
    * « contexte » — la mémoire du modèle à 90 % : la passation, puis une conversation neuve ;
    * « renouveler » — la session l'a demandé (`renouveler`) : de même ;
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
    if session.get("renouveler"):
        return "renouveler"
    if not plein(compte, maintenant)[0]:
        return None
    if ailleurs(compte, maintenant)[0]:
        return "passation"
    if plein(compte, maintenant, repli(session.get("notes")))[0]:
        return "repos"
    if gardee(compte, maintenant):
        return None if session.get("notes") else "reserve"
    return None


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
    """Ce que le veilleur fait de la session du relais qui vit (`garder`) ; False s'il l'a close.
    Endormie, elle est close avant tout (`endormie`) : jamais réveillée, cache perdu."""
    if endormie(session, memoire.get("ecran_depuis", maintenant), maintenant):
        fermer(session, FERMEE_ENDORMIE)
        return False
    geste = garder(session, maintenant, humaine_la)
    if geste == "fermer" or (session.get("passation_demandee") and session.get("laissee") == "relayer"):
        retrait = geste != "fermer" and session.get("passation_pourquoi") == "retrait"
        fermer(session, "compte à sa limite" if geste == "fermer" else
               "en retrait devant sa session" if retrait else "passation demandée")
        if not retrait:
            memoire["compte"] = autre(session.get("compte"))
        return False
    if geste in ("passation", "retrait", "contexte", "reserve", "renouveler", "repos"):
        demander_passation(session, maintenant, geste if geste != "passation" else "limite")
        taper(raison_passation(session["passation_pourquoi"], memoire.get("textes", {}).get(TEXTE_PASSATION)))
        journal({"retrait": "une session à lui prend la main", "contexte": "la mémoire du modèle à 90 %",
                 "reserve": "la fin de la semaine gardée pour ses notes",
                 "renouveler": "une conversation neuve demandée",
                 "repos": "seuil passé, aucun compte pour reprendre"}
                .get(geste, "seuil passé en plein tour")
                + f" : passation demandée au compte {session.get('compte')}")
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
    if memoire.get("paroles") and not il_ecrit(histoire):
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
    mots = [f"{t} {round(f[n][0])} %" for n, t in (("five_hour", "5 h"), ("seven_day", "semaine"),
                                                    ("credits", "crédits achetés")) if n in f]
    au_seuil, libre_a = plein(compte, maintenant)
    if au_seuil and libre_a:
        mots.append(f"libre à {heure(libre_a)}")
    garde = lire_json(ETAT / f"jauges-{compte}.json", {})
    if garde.get("lu"):
        mots.append(f"lu {'au serveur' if garde.get('serveur') == garde['lu'] else 'en session'} à {heure(garde['lu'])}")
    echec = lire_json(ETAT / "serveur.json", {}).get(compte, {}).get("echec")
    if echec and au_seuil:
        mots.append(f"serveur : {echec}")
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


def attendre(memoire, ok, pause, humaine_la, maintenant, ecrit=False):
    """Aucune session du relais ne vit : en lancer une, ou dire pourquoi pas — ses notes, une fois
    qu'il a fini de les écrire (`ecrit` : un geste de sa main dans ce tour, `il_ecrit`)."""
    if humaine_la:
        return (f"en retrait : une session de Claude à toi travaille sur le dépôt (son dernier signe : "
                f"{heure(humaine_la)}) — je reprends {SILENCE // 60} minutes après, ou dès qu'elle me rend "
                "la main")
    if not ok:
        return "en attente : une IA a changé la consigne après toi — retouche-la pour qu'elle reparte"
    if pause:
        return "en pause (première ligne de la consigne)"
    if ecrit and memoire.get("paroles"):
        return "tu écris : une session partira quand tu auras fini (une minute sans geste de toi)"
    compte, libre_a = choisir(memoire.get("compte", "a"), maintenant, bool(memoire.get("paroles")))
    if not compte:
        garde = [c for c in COMPTES if connecte(c) and gardee(c, maintenant) and not plein(c, maintenant, MUR)[0]]
        if garde:
            return (f"la fin de la semaine du compte {garde[0]} est gardée pour tes notes : écris dans ton "
                    f"Glucose, une session te répondra — le travail reprend vers "
                    f"{heure(libre_a) if libre_a else '?'}")
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
    relire(maintenant)
    vivante = ecran() is not None and tenir(session, memoire, humaine_la, maintenant)
    if vivante:
        surveiller(memoire, histoire, maintenant)
        if not humaine_la and not session.get("passation_demandee"):
            entendre(memoire, histoire)
            relancer(session, n.get("texte", ""), ok, pause, memoire["textes"].get(TEXTE_RELANCE))
        dit = dire_session(session, memoire, maintenant)
    else:
        dit = attendre(memoire, ok, pause, humaine_la, maintenant, il_ecrit(histoire))
    if neufs is not None:
        memoire["vu"] = neufs["total"]
    if maintenant - memoire.get("battu", 0) >= 600 or not meme_etat(dit, memoire.get("dit")):
        memoire["dit"] = dit
        battre(memoire, session.get("compte") if vivante else None, dit, maintenant)
    ecrire_json(ETAT / "veilleur.json", memoire)


def mettre_a_jour():
    """Claude se met à jour seul, toutes les six heures (MAJ-1, `mise_a_jour.py`) : son état à part,
    que `un_tour` ne récrit pas ; ce qu'il fait se dit dans un seul nœud de sa membrane, récrit."""
    import mise_a_jour
    memoire = lire_json(ETAT / "mise-a-jour.json", {})
    dire = lambda texte: ecrire("relais-maj-claude", texte, dans="relais", largeur=700)  # noqa: E731
    mise_a_jour.veiller(memoire, time.time(), dire, journal)
    ecrire_json(ETAT / "mise-a-jour.json", memoire)


def veiller():
    verrou = un_seul_veilleur()  # gardé ouvert tant qu'il vit : le verrou meurt avec lui
    journal(f"veilleur démarré (verrou {verrou.name})")
    while True:
        for geste in (un_tour, mettre_a_jour):
            try:
                geste()
            except Exception as e:  # le service ne meurt pas d'une erreur : il la dit, et recommence
                journal(f"erreur ({geste.__name__}) : {e!r}")
        time.sleep(60)


def renouveler():
    """**Une conversation neuve, demandée par la session** (RENOUVELER-1, fiche 163 § 6) : un chantier
    fini, et le suivant n'a que faire de ce qu'elle a lu — chaque réponse relirait pourtant toute sa
    conversation (sa réponse du 24/11 : « relire la conversation, c'est vraiment à ne jamais faire »).
    Le veilleur demande la passation, puis lance une conversation neuve, comme au seuil des jetons."""
    if os.environ.get("RELAIS") != "1":
        print("seule une session du relais renouvelle sa conversation")
        return 1
    session = lire_json(ETAT / "session.json", {})
    session["renouveler"] = time.time()
    ecrire_json(ETAT / "session.json", session)
    journal("la session demande une conversation neuve")
    return 0


def etat():
    maintenant = time.time()
    relire(maintenant)
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
              "renouveler": renouveler, "veiller": veiller, "etat": etat}[geste]() or 0)
