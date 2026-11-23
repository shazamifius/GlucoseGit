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


def depart(compte, paroles=()):
    """Le premier message d'une session du relais : ce qu'elle lit, et **d'abord ce qu'il a écrit
    depuis la dernière** — elle lui répond avant tout (sa parole du 23/11 : « soit actif sur le
    Glucose car j'ai besoin de toi », écrite quand aucune session ne tournait)."""
    ecrit = (f" Depuis la dernière session, il a écrit dans son Glucose ({lire_ids(paroles)}) : lis-le "
             "en premier et réponds-lui dans son Glucose — une réponse à côté de chaque note, ou reliée "
             "par une flèche — avant toute autre tâche. Une note que cette lecture dit introuvable est "
             "dans un dossier : chaque dossier dit le tableau qu'il ouvre (ouvre), lis-le avec "
             "tableau=…." if paroles else "")
    return (f"Session du relais (compte {compte}). Lis ~/.claude/CLAUDE.md, ~/Projets/LISEZ-MOI.md, "
            f"toutes tes mémoires et le mode d'emploi du relais ({MODE_EMPLOI}) en entier, puis dans son "
            f"Glucose (glucose lire tableau=main ids={CONSIGNE} ids={PASSATION}) sa consigne et la "
            f"passation.{ecrit} Puis travaille selon la consigne.")


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


def signee(n, memoire, histoire):
    """La consigne est la sienne : son empreinte est celle qu'une main humaine a écrite en dernier.
    Une fois vue d'elle, l'empreinte est gardée ; un geste d'agent la change, et elle ne vaut plus
    rien. `histoire` : les gestes que le veilleur n'a pas encore vus."""
    if not n or not n.get("empreinte"):
        return False
    if memoire.get("consigne_signee") == n["empreinte"]:
        return True
    geste = dernier_geste(CONSIGNE, histoire)
    if geste and humaine(geste.get("auteur")):
        memoire["consigne_signee"] = n["empreinte"]
        memoire["consigne_par"] = f"{geste['auteur']}, le {geste.get('date', '?')}"
        return True
    return False


def en_pause(texte):
    premiere = (texte or "").strip().splitlines()[:1]
    return bool(premiere) and (premiere[0].startswith("⏸") or "PAUSE" in premiere[0].upper())


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


def raison_continuer(consigne_texte):
    return ("[Relais] Ne t'arrête pas : continue selon sa consigne et le plan (SUITE § 2, la passation). "
            "Si tu attends une réponse de lui, écris la question dans son Glucose et prends une autre "
            f"tâche. Sa consigne :\n{consigne_texte.strip()}")


POURQUOI = {
    "limite": "Ce compte arrive à sa limite.",
    "retrait": "Il travaille dans une session de Claude à lui, sur ce dépôt : une seule session à la "
               "fois (sa parole du 23/11), le relais s'efface et reprendra quand elle se taira.",
}


def raison_passation(motif=None):
    return (f"[Relais] {POURQUOI.get(motif, POURQUOI['limite'])} Écris MAINTENANT la passation dans son "
            f"Glucose : remplace le texte du nœud {PASSATION} (glucose poser-texte tableau=main "
            f"id={PASSATION} texte=-) par l'état exact — fait, en cours, non commité, prochaine étape, "
            "pièges payés. Si tu n'as rien changé depuis celle qui y est, garde-la et ajoute seulement en "
            "tête une ligne qui la confirme, avec l'heure. Puis termine ton tour.")


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
