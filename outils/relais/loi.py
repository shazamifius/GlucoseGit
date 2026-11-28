"""**La loi du relais**, sans un effet de bord : qui écrit, ce qu'il a écrit, ce que dit le crochet
« Stop », les textes que reçoit une session. Tout s'y éprouve sans Glucose, sans tmux, sans Claude
Code ; `relais.py` porte les effets (extrait de lui le 23/11, qui passait ses six cents lignes).
"""
import re
import time

MODE_EMPLOI = "outils/relais/MODE-EMPLOI.md"
CONSIGNE, PASSATION, BATTEMENT, BLOQUE = (
    "relais-consigne", "relais-passation", "relais-battement", "relais-bloque")
# Sa réponse à une session figée s'écrit **dans l'alerte même**, sous cette ligne : le nœud à part
# (relais-reponse) qu'il a retiré le 27/11 à 14 h 29 laissait l'alerte de 22 h 07 renvoyer à un
# nœud qui n'existait plus (fiche 186).
INVITE_REPONSE = "✍️ Ta réponse :"
# Aucune commande ne dure plus de 30 minutes (sa règle du 19/11) : une session qui travaille change
# son écran, ou écrit sa conversation, au moins aussi souvent. 35 minutes sans un signe : elle ne
# travaille plus.
SILENCE = 35 * 60
# **La mémoire du modèle** (son contexte) : passé SEUIL_JETONS, la session écrit sa passation et une
# conversation neuve reprend. Sa règle du 23/11 visait 90 % (text-kpll-351) ; mais la fenêtre d'Opus
# est d'un million de jetons, et chaque réponse relit la conversation entière : mesuré le 24/11 sur
# 3 191 requêtes du relais, relire coûtait 78 % des jetons, à 318 000 jetons de contexte en moyenne
# (784 000 au plus). Sa réponse du 24/11 (relais-q-h24-quotas) : *« relire la conversation, c'est
# vraiment à ne jamais faire »*. Simulé sur ces sessions, le coût est au plus bas vers 300 000
# (fiche 142) ; remesuré le 28/11 sur 206 intervalles de jauge, le creux est à 277 000 et la
# fenêtre entière coûterait 80 % de plus par requête (fiche 147 § 7). Une fenêtre plus petite garde
# l'ancien seuil, en pourcentage.
SEUIL_JETONS = 300_000
SEUIL_CONTEXTE = 90.0
# **Endormie** (sa note du 25/11, img-utzs-2460 : « ne jamais réveiller une session Claude qui attend
# depuis plus de 50 minutes ») : le serveur garde la conversation en cache une heure (`ephemeral_1h`,
# lu dans les journaux de Claude Code le 25/11) ; passé ce délai, la réveiller la fait réécrire
# entière, au prix d'un jeton neuf — jusqu'à 300 000. Une session dont l'écran n'a pas bougé depuis
# ENDORMIE est close — arrêtée au bout de son tour **ou figée** : une session qui travaille anime son
# écran (le compteur de Claude Code tourne pendant qu'il pense comme pendant une commande, et aucune
# ne dure plus de 30 minutes). Le 27/11 à 21 h 31, figée sur un refus d'Anthropic, une session est
# restée ouverte dix-huit heures : seule l'arrêtée se fermait (fiche 186). Une neuve repart de la passation.
ENDORMIE = 50 * 60
FERMEE_ENDORMIE = "endormie depuis plus de 50 minutes"
FERMEE_FIGEE = "figée depuis plus de 50 minutes"
FERMEE_REFUSEE = "refusée par Anthropic"
FERMEES_SANS_PASSATION = (FERMEE_ENDORMIE, FERMEE_FIGEE, FERMEE_REFUSEE)

# **Un tour refusé par Anthropic** (le crochet « StopFailure », qui part *à la place* de « Stop » :
# l'abonnement, la connexion, une limite, un serveur surchargé). Le 27/11 à 21 h 31, « can't use
# your Claude subscription, usually because a payment is past due » : passager — la même session a
# répondu le lendemain —, mais le relais n'écoutait que « Stop », et personne n'a réessayé. Un essai
# refusé ne coûte aucun jeton ; Claude Code a déjà réessayé dans le tour avant de renoncer. Le relais
# réessaie donc de plus en plus loin (5, 10, 20, 40 minutes), **jamais plus loin que la vie du cache**
# (ENDORMIE) : au-delà, attendre ne garde plus rien, et une panne longue est réessayée toutes les
# 50 minutes, sans compter. Un tour réussi efface le refus.
PREMIER_ESSAI = 5 * 60
ERREURS = {
    "billing_error": "abonnement refusé (un paiement en retard ?)",
    "authentication_failed": "connexion refusée (un /login à refaire ?)",
    "oauth_org_not_allowed": "connexion refusée par l'organisation",
    "account_on_hold": "compte suspendu par Anthropic",
    "verification_required": "Anthropic demande une vérification du compte",
    "rate_limit": "limite atteinte",
    "overloaded": "serveurs d'Anthropic surchargés",
    "server_error": "serveurs d'Anthropic en panne",
}


def delai_essai(fois):
    """Le temps avant le `fois`-ième nouvel essai d'un compte refusé : il double, jusqu'à ENDORMIE."""
    return min(PREMIER_ESSAI * 2 ** max(fois - 1, 0), ENDORMIE)


def refus_en_mots(refus):
    """Ce que le ❤️ dit d'un compte refusé : l'erreur en clair, et l'heure du prochain essai."""
    erreur = refus.get("erreur") or "unknown"
    return (f"refusé par Anthropic ({ERREURS.get(erreur, erreur)}, {refus.get('fois', 1)} fois) — "
            f"nouvel essai à {heure(refus.get('jusqu_a', 0))}")
# Ce qui réveille une session sans qu'il ait rien écrit : la fin d'une commande en arrière-plan (le
# 23/11 à 14 h 41, la vérification de Claude, réveillée ainsi, a pris la main au relais). Une session
# à lui qui travaille garde la main par son journal ; une session qui ne fait que veiller ne la prend plus.
NOTIFICATIONS = ("<task-notification>",)

REGLES = (
    "Tu es une session du RELAIS de Glucose : personne ne lit ce terminal en direct. Une question "
    "pour lui s'écrit dans son Glucose (tableau=main, dans la membrane « relais »), un texte par "
    "question, et tu passes à une autre tâche du plan : ne t'arrête jamais pour attendre. "
    "N'écris jamais dans le nœud relais-consigne : c'est le sien. Jamais de publication de version. "
    "Aucune commande de plus de 30 minutes. Les jetons sont sa priorité (24/11) : personne ne lit tes "
    "messages, n'y écris aucun compte rendu (une ligne au plus quand Claude Code en demande une) — ce "
    "qui compte va dans son Glucose et dans le dépôt ; lis au plus juste (une section plutôt qu'un "
    "fichier, grep avant cat, jamais deux fois la même chose).")


def meme_etat(dit, avant):
    """**Le même état, à l'heure près** : « en retrait : … son dernier signe : 17:52 » ne change pas
    d'état à 17:53. Le battement se récrivait à chaque minute qu'une session à lui travaillait — un
    geste de plus dans son document chaque minute (le 27/11, de 17:47 à 17:53) ; il ne se récrit
    plus que quand l'état change, ou au bout de dix minutes."""
    sans = lambda texte: re.sub(r"\d{1,2}:\d{2}", "", texte or "")  # noqa: E731
    return sans(dit) == sans(avant)


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
# Son bloc « URGENT IMPORTANT » (23/11) : ses flèches désignent ce qui presse (sa note du 24/11).
URGENT = "text-f5fn-345"
TEXTE_DEPART, TEXTE_PASSATION, TEXTE_RELANCE = (
    "relais-texte-depart", "relais-texte-passation", "relais-texte-relance")
TEXTES = {
    # Sans relire ce que Claude Code charge déjà (ses règles, celles du dépôt, l'index des mémoires) :
    # le démarrage d'une conversation lisait 110 000 jetons, et chaque réponse les relisait (fiche 142).
    # Son texte à lui (relais-texte-depart, 27/11), corrigé à sa demande (relais-rep-texte-depart) :
    # le même, qu'il l'ait retouché ou non.
    TEXTE_DEPART: (
        "Bonjour ! Tu reprends Glucose, un projet qui vise à créer un canevas incroyable. Ce qui est "
        "particulier, c'est que tu dois utiliser Glucose pour créer et améliorer Glucose.\n\n"
        f"Lis dans son Glucose (tu as la porte MCP) sa consigne et la passation (glucose lire "
        f"tableau=main ids={CONSIGNE} ids={PASSATION}). Tes règles et l'index de tes mémoires sont déjà "
        f"chargés : du mode d'emploi du relais ({MODE_EMPLOI}, seulement si ton travail touche au "
        "relais), de ~/Projets/LISEZ-MOI.md, d'une mémoire, de VISION, ÉTAT, SUITE ou MÉTHODE, ne lis "
        "que la section dont ton travail a besoin — les jetons sont sa priorité (24/11).\n\n"
        f"Lis aussi son bloc URGENT IMPORTANT : chaque flèche qui en part désigne un point à traiter en "
        f"priorité (glucose lire tableau=main ids={URGENT} fleches=true). Puis travaille selon la consigne."),
    # Sa note du 25/11 (text-bmgs-2413) : *« c'est nul le fait qu'il met les récaps de ce qui s'est
    # passé, ça tu le vois par toi-même grâce au MCP »*. Ce qui est fait, `git log` et son Glucose le
    # disent ; la passation de ce matin-là, allongée de session en session, pesait 1 500 mots relus à
    # chaque démarrage. Elle ne garde que ce qui n'est écrit nulle part ailleurs, et se remplace.
    TEXTE_PASSATION: (
        f"Écris MAINTENANT la passation dans son Glucose : remplace tout le texte du nœud {PASSATION} "
        f"(glucose poser-texte tableau=main id={PASSATION} texte=-), sans jamais l'allonger. Une "
        "première ligne (le compte, le jour, l'heure lue par date), puis seulement ce que la session "
        "suivante ne voit nulle part ailleurs : le travail non commité (où, ce qui manque), la prochaine "
        "étape, un piège payé — dix lignes au plus. Ni ce qui est fait (git log le dit), ni ses questions "
        "en attente (son Glucose les montre). Rien de changé : récris-la avec sa première ligne remise à "
        "l'heure. Puis termine ton tour."),
    TEXTE_RELANCE: (
        "Ne t'arrête pas : continue selon sa consigne et le plan (SUITE § 2, la passation). Si tu "
        "attends une réponse de lui, écris la question dans son Glucose et prends une autre tâche."),
}


def ses_notes(paroles, depuis=None):
    """**Où lire ce qu'il a écrit** : « quoi de neuf » depuis son premier geste (sa note du 27/11,
    text-kix1-2714 : la liste « ids=… » de chaque nœud touché s'allongeait sans fin quand une session
    à lui tenait la main) — chaque nœud une fois, son tableau, ce qui a changé ligne à ligne, les
    dossiers compris. La liste ne reste que faute de rang connu (un veilleur d'avant ce changement)."""
    if depuis is not None:
        return (f"glucose_quoi_de_neuf depuis={depuis} : chaque nœud une fois, ce qui a changé, où ; "
                "relis ensuite en entier seulement ceux dont tu as besoin")
    return lire_ids(paroles)


def depart(compte, paroles=(), texte=None, importants=(), garde=False, orpheline=False, depuis=None):
    """Le premier message d'une session du relais : son texte, et **d'abord ce qu'il a écrit
    depuis la dernière** — elle lui répond avant tout (sa parole du 23/11 : « soit actif sur le
    Glucose car j'ai besoin de toi », écrite quand aucune session ne tournait). Sur un compte qui a
    passé son seuil, lancée pour ses notes (`garde`, RESERVE-1, fiche 163 ; fiche 186), elle ne fait
    que lui répondre. Chaque morceau son paragraphe : son texte finit sans point (« …en prioriter »),
    et la suite s'y collait."""
    ecrit = (f"Depuis la dernière session, il a écrit dans son Glucose ({ses_notes(paroles, depuis)}) : "
             "lis-le en premier et réponds-lui dans son Glucose — une réponse à côté de chaque note, ou "
             "reliée par une flèche — avant toute autre tâche. Une note dans un dossier se lit avec "
             "tableau=… (chaque dossier dit le tableau qu'il ouvre)." if paroles else "")
    designe = (f"Sa consigne désigne par ses flèches ce qu'il juge le plus important "
               f"({lire_ids(importants)}) : lis-le juste après elle." if importants else "")
    reserve = (f"{POURQUOI['reserve']} Réponds à ses notes, à rien d'autre, puis termine ton tour."
               if garde else "")
    # Close endormie ou figée (`endormie`), la session d'avant n'a pas récrit sa passation : son
    # travail peut attendre dans l'arbre, non commité.
    avant = ("La session d'avant a été close sans récrire la passation : `git status` avant de t'y fier."
             if orpheline else "")
    morceaux = (f"Session du relais (compte {compte}). {(texte or TEXTES[TEXTE_DEPART]).strip()}",
                designe, ecrit, reserve, avant)
    return "\n\n".join(m for m in morceaux if m)


def ses_fleches(noeuds, gestes, depuis=CONSIGNE, profondeur=2):
    """**Ce que sa consigne désigne** : les nœuds où mènent les flèches que sa main a tirées depuis
    elle, puis depuis ceux-là. Sa note du 24/11 (text-g420-2321) : que le démarrage « regarde le
    block important » — un bloc d'où partent ses flèches vers tout ce qui presse. Une flèche tirée
    par une IA ne compte pas : la consigne ne s'étend que de sa main. `gestes` : toute l'histoire."""
    siennes = {i for g in gestes if humaine(g.get("auteur")) for fait in g.get("fait", [])
               if fait.startswith("pose la flèche") for i in re.findall(r"«\s*([^»]+?)\s*»", fait)}
    fleches = [n for n in noeuds if n.get("sorte") == "fleche" and n.get("id") in siennes]
    vus, front = [], [depuis]
    for _ in range(profondeur):
        front = [f["cible"] for f in fleches if f.get("source") in front
                 and f.get("cible") not in vus and f.get("cible") != depuis]
        vus += [c for i, c in enumerate(front) if c not in front[:i]]
    return vus


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
    à une alerte (relais-bloque) est tapée telle quelle (`surveiller`)."""
    ids = list(avant)
    for geste in (histoire or {}).get("gestes", []):
        if not humaine(geste.get("auteur")):
            continue
        # **Un collage n'est pas une note** : ce qu'il écrit se pose un nœud à la fois ; un geste qui en
        # pose plusieurs colle ce qui existait (le 25/11, treize nœuds rangés dans son archive furent
        # donnés à lire comme neufs). Un nœud collé qu'il retouche ensuite redevient sa parole.
        collage = sum(f.startswith("pose ") for f in geste.get("fait", [])) > 1
        for fait in geste.get("fait", []):
            if collage and fait.startswith("pose "):
                continue
            # L'histoire nomme un dossier par son NOM (« Dossier ») : ce n'est pas un identifiant, et
            # le dossier se lit avec son tableau (fiche 133 ; le 23/11, « ids=Dossier » ne lisait rien).
            if " dossier " in fait:
                continue
            for ident in re.findall(r"«\s*([^»]+?)\s*»", fait):
                if fait.startswith("retire"):
                    ids = [i for i in ids if i != ident]
                elif ident not in ids and ident not in (BATTEMENT, BLOQUE):
                    ids.append(ident)
    return ids


def il_ecrit(histoire):
    """**Il écrit encore** : un geste de sa main dans ce tour du veilleur. Ce qu'il écrit attend qu'il
    ait fini — tapé dans la session qui vit (`entendre`), ou la session lancée pour lui (`attendre`) :
    le 25/11 à 8 h 15, une session partit sur le seul titre de sa note, une seconde trois minutes plus
    tard sur son texte, deux démarrages sur la fin d'une semaine."""
    return any(humaine(g.get("auteur")) for g in (histoire or {}).get("gestes", []))


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


def endormie(session, immobile_depuis, maintenant):
    """L'écran immobile depuis ENDORMIE : son cache est perdu, la réveiller coûterait sa relecture
    entière — arrêtée au bout d'un tour (`laissee`) ou figée (sans rien de `laissee` : un refus
    d'Anthropic, une question au démarrage). Rend la raison de la fermer, ou None. L'écran, pas
    l'heure de l'arrêt : une note tapée dans une session arrêtée la fait travailler sans lever
    `laissee`, et son écran bouge."""
    if maintenant - immobile_depuis < ENDORMIE:
        return None
    return FERMEE_ENDORMIE if session.get("laissee") else FERMEE_FIGEE


def contexte_plein(contexte, session):
    """La mémoire du modèle de **cette** session a-t-elle passé le seuil ? La barre d'état l'écrit à
    chaque réponse (`contexte.json`) ; une lecture d'avant son lancement est celle d'une autre."""
    if float(contexte.get("lu") or 0) < float(session.get("depuis") or float("inf")):
        return False
    pct, taille = float(contexte.get("pct") or 0), float(contexte.get("taille") or 0)
    return pct >= SEUIL_CONTEXTE or pct * taille / 100 >= SEUIL_JETONS


def dire_session(session, memoire, maintenant, refus=None):
    """Ce que le ❤️ dit d'une session vivante. **Figée, elle ne se dit pas « au travail »** : le
    23/11, une session est restée 1 h 48 devant une question de Claude Code au démarrage pendant que
    le ❤️ disait « au travail depuis 16:00 » (fiche 134). **Refusée par Anthropic, elle le dit**, en
    clair, avec l'heure du prochain essai (fiche 186)."""
    compte = session.get("compte")
    if refus:
        return f"⛔ session du compte {compte} {refus_en_mots(refus)}"
    if memoire.get("bloque_signale"):
        return (f"🆘 session du compte {compte} figée : rien ne bouge à son écran depuis "
                f"{heure(memoire.get('ecran_depuis', maintenant))} — son écran est dans « {BLOQUE} »")
    dit = f"session du compte {compte} au travail depuis {heure(session.get('depuis', maintenant))}"
    return dit + (f" — arrêtée : {session['laissee']}" if session.get("laissee") else "")


def raison_continuer(consigne_texte, texte=None):
    return f"[Relais] {texte or TEXTES[TEXTE_RELANCE]} Sa consigne :\n{consigne_texte.strip()}"


def raison_reessayer(refus, consigne_texte, texte=None):
    """Le nouvel essai d'un tour refusé (« Once fixed, resend your message », dit Claude Code)."""
    erreur = refus.get("erreur") or "unknown"
    return (f"[Relais] Ton dernier tour a été refusé par Anthropic ({ERREURS.get(erreur, erreur)}) : "
            f"reprends où tu en étais. {raison_continuer(consigne_texte, texte)[len('[Relais] '):]}")


POURQUOI = {
    "limite": "Ce compte arrive à sa limite.",
    "retrait": "Il travaille dans une session de Claude à lui, sur ce dépôt : une seule session à la "
               "fois (sa parole du 23/11), le relais s'efface et reprendra quand elle se taira.",
    "contexte": f"Ta conversation a passé {SEUIL_JETONS // 1000} 000 jetons : chaque réponse la relit "
                "entière, et c'est ce qui use ses comptes. Une conversation neuve reprendra, avec ta "
                "passation (sa réponse du 24/11).",
    "renouveler": "Tu as demandé une conversation neuve (relais.py renouveler) : la suivante reprendra "
                  "avec ta passation.",
    "repos": "Ce compte arrive à sa limite et aucun autre ne peut reprendre : écris ta passation et "
             "arrête-toi, le relais reprendra seul à sa remise. Jamais au mur : coupée en route, une "
             "session perd son travail (le 24/11 à 19 h 37, il a dû l'arrêter lui-même).",
    "reserve": "Ce compte a passé son seuil et aucun autre ne peut reprendre : ce qui en reste est gardé "
               "pour lui répondre quand il écrit (RESERVE-1, fiche 163 ; fiche 186) — une session se "
               "lancera pour ses notes.",
}


def raison_passation(motif=None, texte=None):
    return f"[Relais] {POURQUOI.get(motif, POURQUOI['limite'])} {texte or TEXTES[TEXTE_PASSATION]}"


RAISON_PASSATION = raison_passation()


def raison_entendre(paroles, depuis=None):
    return (f"[Relais] Il vient d'écrire dans son Glucose ({ses_notes(paroles, depuis)}). S'il te demande "
            "quelque chose ou te répond, réponds-lui dans son Glucose maintenant, puis reprends ton travail.")


AVERTIR = (
    "[Relais] Une session du relais travaille encore sur ce dépôt. Une seule session à la fois (sa "
    f"parole du 23/11) : elle écrit sa passation dans son Glucose ({PASSATION}) et s'arrête d'elle-même. "
    "Ne touche pas au dépôt avant (python3 -I outils/relais/relais.py etat : plus de session tmux). "
    f"Le relais reprendra {SILENCE // 60} minutes après le dernier signe de ta session, ou dès "
    "« python3 -I outils/relais/relais.py rendre » — à lancer en partant.")
