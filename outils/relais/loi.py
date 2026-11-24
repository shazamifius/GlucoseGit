"""**La loi du relais**, sans un effet de bord : qui écrit, ce qu'il a écrit, ce que dit le crochet
« Stop », les textes que reçoit une session. Tout s'y éprouve sans Glucose, sans tmux, sans Claude
Code ; `relais.py` porte les effets (extrait de lui le 23/11, qui passait ses six cents lignes).
"""
import re
import time

MODE_EMPLOI = "outils/relais/MODE-EMPLOI.md"
CONSIGNE, PASSATION, BATTEMENT, BLOQUE, REPONSE = (
    "relais-consigne", "relais-passation", "relais-battement", "relais-bloque", "relais-reponse")
INVITE_REPONSE = "✍️ Ta réponse :"
# Aucune commande ne dure plus de 30 minutes (sa règle du 19/11) : une session qui travaille change
# son écran, ou écrit sa conversation, au moins aussi souvent. 35 minutes sans un signe : elle ne
# travaille plus.
SILENCE = 35 * 60
# **La mémoire du modèle** (son contexte) : à 90 %, la session écrit sa passation et une conversation
# neuve reprend — sa règle du 23/11 (text-kpll-351) : *« on vise toujours 90 %, que ce soit la
# semaine, les 5 heures ou l'utilisation de tokens […] et on switch ou on recrée un nouveau chat »*.
SEUIL_CONTEXTE = 90.0
# Ce qui réveille une session sans qu'il ait rien écrit : la fin d'une commande en arrière-plan (le
# 23/11 à 14 h 41, la vérification de Claude, réveillée ainsi, a pris la main au relais). Une session
# à lui qui travaille garde la main par son journal ; une session qui ne fait que veiller ne la prend plus.
NOTIFICATIONS = ("<task-notification>",)

REGLES = (
    "Tu es une session du RELAIS de Glucose : personne ne lit ce terminal en direct. Une question "
    "pour lui s'écrit dans son Glucose (tableau=main, dans la membrane « relais »), un texte par "
    "question, et tu passes à une autre tâche du plan : ne t'arrête jamais pour attendre. "
    "N'écris jamais dans le nœud relais-consigne : c'est le sien. Jamais de publication de version. "
    "Aucune commande de plus de 30 minutes.")


def heure(t):
    """L'heure ; et le jour quand ce n'est pas aujourd'hui (une semaine se remet dans deux jours)."""
    meme_jour = time.localtime(t)[:3] == time.localtime()[:3]
    return time.strftime("%H:%M" if meme_jour else "%d/%m %H:%M", time.localtime(t))


def lire_ids(ids):
    return "glucose lire tableau=main " + " ".join(f"ids={i}" for i in ids) + " niveau=complet"


# ── Ses textes : ce que le relais dit aux sessions, à lui de le récrire ──────────────────────
# Sa note du 23/11 (text-kpll-351) : *« un système qui me permet de voir le script de passation
# envoyé directement à l'ouverture d'une session Claude […] il ne me convient pas du tout ; de même
# pour le script de stop »*. Chaque texte vit dans un nœud de sa membrane « relais » ; le relais y
# ajoute les faits du moment (le compte, ses notes, le motif, sa consigne). Sa version ne vaut que
# si sa main l'a écrite en dernier — jamais une IA (`signee`) ; sinon, celle-ci.
TEXTE_DEPART, TEXTE_PASSATION, TEXTE_RELANCE = (
    "relais-texte-depart", "relais-texte-passation", "relais-texte-relance")
TEXTES = {
    TEXTE_DEPART: (
        f"Lis ~/.claude/CLAUDE.md, ~/Projets/LISEZ-MOI.md, toutes tes mémoires et le mode d'emploi du "
        f"relais ({MODE_EMPLOI}) en entier, puis dans son Glucose (glucose lire tableau=main "
        f"ids={CONSIGNE} ids={PASSATION}) sa consigne et la passation. Puis travaille selon la consigne."),
    TEXTE_PASSATION: (
        f"Écris MAINTENANT la passation dans son Glucose : remplace le texte du nœud {PASSATION} "
        f"(glucose poser-texte tableau=main id={PASSATION} texte=-) par l'état exact — fait, en cours, "
        "non commité, prochaine étape, pièges payés. Si tu n'as rien changé depuis celle qui y est, "
        "garde-la et ajoute seulement en tête une ligne qui la confirme, avec l'heure. Puis termine ton "
        "tour."),
    TEXTE_RELANCE: (
        "Ne t'arrête pas : continue selon sa consigne et le plan (SUITE § 2, la passation). Si tu "
        "attends une réponse de lui, écris la question dans son Glucose et prends une autre tâche."),
}


def depart(compte, paroles=(), texte=None):
    """Le premier message d'une session du relais : son texte, et **d'abord ce qu'il a écrit
    depuis la dernière** — elle lui répond avant tout (sa parole du 23/11 : « soit actif sur le
    Glucose car j'ai besoin de toi », écrite quand aucune session ne tournait)."""
    ecrit = (f" Depuis la dernière session, il a écrit dans son Glucose ({lire_ids(paroles)}) : lis-le "
             "en premier et réponds-lui dans son Glucose — une réponse à côté de chaque note, ou reliée "
             "par une flèche — avant toute autre tâche. Une note que cette lecture dit introuvable est "
             "dans un dossier : chaque dossier dit le tableau qu'il ouvre (ouvre), lis-le avec "
             "tableau=…." if paroles else "")
    return f"Session du relais (compte {compte}). {texte or TEXTES[TEXTE_DEPART]}{ecrit}"


# ── Qui écrit, et ce qu'il a écrit ────────────────────────────────────────────────────────────

def dernier_geste(ident, histoire):
    """Le dernier geste de l'histoire qui nomme ce nœud, ou None."""
    for geste in reversed((histoire or {}).get("gestes", [])):
        if any(f"« {ident} »" in fait for fait in geste.get("fait", [])):
            return geste
    return None


def humaine(auteur):
    """**Une main humaine** : la sienne devant ce Glucose (« humain »), ou devant un autre appareil
    du canevas partagé, sous le nom que l'appareil se donne (le 23/11, « Aigle Émeraude » : sa
    consigne retouchée de là-bas ne valait rien, et le relais s'est tu). Jamais une IA : une IA, même
    sur l'autre appareil, arrive « agent … » (`Lettre::main_chez_moi`) ; l'IA de Glucose a son nom."""
    return bool(auteur) and not auteur.startswith("agent ") and auteur != "l'IA de Glucose"


def signee(n, memoire, histoire, ident=CONSIGNE):
    """Ce nœud (sa consigne, l'un de ses textes) est le sien : son empreinte est celle qu'une main
    humaine a écrite en dernier. Une fois vue d'elle, l'empreinte est gardée ; un geste d'agent la
    change, et elle ne vaut plus rien. `histoire` : les gestes que le veilleur n'a pas encore vus."""
    if not n or not n.get("empreinte"):
        return False
    cle = "consigne" if ident == CONSIGNE else f"signe:{ident}"
    if memoire.get(f"{cle}_signee") == n["empreinte"]:
        return True
    geste = dernier_geste(ident, histoire)
    if geste and humaine(geste.get("auteur")):
        memoire[f"{cle}_signee"] = n["empreinte"]
        memoire[f"{cle}_par"] = f"{geste['auteur']}, le {geste.get('date', '?')}"
        return True
    return False


def en_pause(texte):
    """⏸, « pause » ou « stop » en première ligne : son « stop » du 24/11 n'arrêtait rien."""
    premiere = (texte or "").strip().splitlines()[:1]
    return bool(premiere) and (premiere[0].startswith("⏸")
                               or any(mot in premiere[0].upper() for mot in ("PAUSE", "STOP")))


def ses_paroles(histoire, avant=()):
    """**Ce qu'il a écrit** : à la suite de `avant`, les nœuds que sa main pose ou change, dans
    l'ordre, chacun une fois ; un nœud qu'il retire en sort. Le battement est au relais ; sa réponse
    (relais-reponse) est tapée telle quelle (`surveiller`)."""
    ids = list(avant)
    for geste in (histoire or {}).get("gestes", []):
        if not humaine(geste.get("auteur")):
            continue
        for fait in geste.get("fait", []):
            # L'histoire nomme un dossier par son NOM (« Dossier ») : ce n'est pas un identifiant, et
            # le dossier se lit avec son tableau (fiche 133 ; le 23/11, « ids=Dossier » ne lisait rien).
            if " dossier " in fait:
                continue
            for ident in re.findall(r"«\s*([^»]+?)\s*»", fait):
                if fait.startswith("retire"):
                    ids = [i for i in ids if i != ident]
                elif ident not in ids and ident not in (BATTEMENT, REPONSE):
                    ids.append(ident)
    return ids


# ── Ce que dit le crochet « Stop » ────────────────────────────────────────────────────────────

def decider(session, consigne_texte, pause, a_signee, relayer, maintenant, ecrite=False, suite=False):
    """(action, raison) — action : « laisser » (Claude s'arrête), « continuer » ou « passation ».

    `relayer` : la passation est due — le compte au seuil quand un autre peut reprendre, ou une
    session à lui qui prend la main. Une passation demandée ne laisse partir la session qu'**écrite**
    (`ecrite` : le nœud a changé depuis la demande). La demande peut arriver du veilleur en plein
    tour, tapée dans la session : le tour qui finit alors ne l'a peut-être pas encore lue, et la
    redemander ne coûte rien. Trois arrêts en deux minutes sans elle : la session ne peut plus
    l'écrire, on relaie quand même.

    `suite` : ce tour vient déjà d'une relance du crochet (`stop_hook_active`). Claude Code n'en
    accepte que 8 de suite, puis finit le tour de force — et le relais se taisait, la session prise
    pour vivante (`CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`, lu dans son binaire le 23/11). La relance passe
    alors par le veilleur, qui la tape comme un message : le compte de Claude Code repart de zéro."""
    recents = [t for t in session.get("arrets", []) if maintenant - t < 120]
    if session.get("passation_demandee"):
        return ("laisser", "relayer") if ecrite or len(recents) >= 3 else ("passation", "")
    if relayer:
        return "passation", ""
    if pause:
        return "laisser", "pause"
    if not a_signee or not (consigne_texte or "").strip():
        return "laisser", "consigne non signée"
    if len(recents) >= 3:
        return "laisser", "à vide"
    return ("laisser", "relancer") if suite else ("continuer", "")


def contexte_plein(contexte, session):
    """La mémoire du modèle de **cette** session a-t-elle passé le seuil ? La barre d'état l'écrit à
    chaque réponse (`contexte.json`) ; une lecture d'avant son lancement est celle d'une autre."""
    return (float(contexte.get("lu") or 0) >= float(session.get("depuis") or float("inf"))
            and float(contexte.get("pct") or 0) >= SEUIL_CONTEXTE)


def dire_session(session, memoire, maintenant):
    """Ce que le ❤️ dit d'une session vivante. **Figée, elle ne se dit pas « au travail »** : le
    23/11, une session est restée 1 h 48 devant une question de Claude Code au démarrage pendant que
    le ❤️ disait « au travail depuis 16:00 » (fiche 134)."""
    compte = session.get("compte")
    if memoire.get("bloque_signale"):
        return (f"🆘 session du compte {compte} figée : rien ne bouge à son écran depuis "
                f"{heure(memoire.get('ecran_depuis', maintenant))} — son écran est dans « {BLOQUE} »")
    dit = f"session du compte {compte} au travail depuis {heure(session.get('depuis', maintenant))}"
    return dit + (f" — arrêtée : {session['laissee']}" if session.get("laissee") else "")


def raison_continuer(consigne_texte, texte=None):
    return f"[Relais] {texte or TEXTES[TEXTE_RELANCE]} Sa consigne :\n{consigne_texte.strip()}"


POURQUOI = {
    "limite": "Ce compte arrive à sa limite.",
    "retrait": "Il travaille dans une session de Claude à lui, sur ce dépôt : une seule session à la "
               "fois (sa parole du 23/11), le relais s'efface et reprendra quand elle se taira.",
    "contexte": f"Ta conversation a rempli {SEUIL_CONTEXTE:.0f} % de la mémoire du modèle : une "
                "conversation neuve reprendra, avec ta passation (sa règle du 23/11).",
}


def raison_passation(motif=None, texte=None):
    return f"[Relais] {POURQUOI.get(motif, POURQUOI['limite'])} {texte or TEXTES[TEXTE_PASSATION]}"


RAISON_PASSATION = raison_passation()


def raison_prevenir(libre_a):
    return (f"[Relais] Ce compte arrive à sa limite, et aucun autre ne peut reprendre avant "
            f"{heure(libre_a) if libre_a else '?'} : continue, mais commite chaque étape finie et tiens "
            f"la passation à jour dans son Glucose ({PASSATION}) — au mur, la session sera close sans "
            "pouvoir l'écrire.")


def raison_entendre(paroles):
    return (f"[Relais] Il vient d'écrire dans son Glucose ({lire_ids(paroles)}). S'il te demande "
            "quelque chose ou te répond, réponds-lui dans son Glucose maintenant, puis reprends ton travail.")


AVERTIR = (
    "[Relais] Une session du relais travaille encore sur ce dépôt. Une seule session à la fois (sa "
    f"parole du 23/11) : elle écrit sa passation dans son Glucose ({PASSATION}) et s'arrête d'elle-même. "
    "Ne touche pas au dépôt avant (python3 -I outils/relais/relais.py etat : plus de session tmux). "
    f"Le relais reprendra {SILENCE // 60} minutes après le dernier signe de ta session, ou dès "
    "« python3 -I outils/relais/relais.py rendre » — à lancer en partant.")
