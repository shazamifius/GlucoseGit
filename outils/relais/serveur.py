"""**La jauge d'un compte, demandée au serveur d'Anthropic** (fiche 182).

La barre d'état de Claude Code donne les jauges d'un compte, mais seulement quand une session y
tourne. Le relais gardait la dernière lue et la croyait jusqu'à sa remise : « dans une fenêtre, l'usage
ne fait que s'ajouter ». Faux, et à sens unique : une limite remise à zéro par Anthropic, des crédits
achetés, un abonnement changé ne font que **baisser** la vraie jauge. Le 27/11, le compte a gardé à
99 % de sa semaine depuis 11 h 49 était à 2 % au serveur (sa semaine remise), et le relais dormait.

Le serveur est la vérité : la même adresse que la commande `/usage` de Claude Code
(`api/oauth/usage`), avec la connexion du compte (`.credentials.json`), lue sans jamais être écrite
ni dite. Une connexion expirée ne se renouvelle pas ici (ce serait réécrire la clé d'une session qui
tourne peut-être) : la jauge gardée reste alors ce qu'on sait, et le battement le dit.
"""
import json
import urllib.error
import urllib.request
from datetime import datetime

ADRESSE = "https://api.anthropic.com/api/oauth/usage?skip_spend=1"
# Pas plus d'une question au serveur par compte et par dix minutes : la jauge ne se relit que quand
# elle dit « plein », et une remise ne se rattrape pas à la minute près.
INTERVALLE = 600


def en_secondes(texte):
    """« 2026-11-30T14:59:59.69+00:00 » → l'instant ; rien (0) quand le serveur n'en dit pas."""
    try:
        return datetime.fromisoformat(texte).timestamp() if texte else 0.0
    except (TypeError, ValueError):
        return 0.0


def traduire(reponse):
    """La réponse du serveur, dans la forme de la barre d'état (`rate_limits`) ; plus « credits »
    quand des crédits en plus sont activés **avec un plafond mensuel** : jusqu'où il a accepté de
    payer. Sans plafond, aucune : un relais qui tourne jour et nuit ne dépense pas sans borne."""
    limites = {}
    for nom in ("five_hour", "seven_day"):
        f = reponse.get(nom)
        if isinstance(f, dict) and isinstance(f.get("utilization"), (int, float)):
            limites[nom] = {"used_percentage": float(f["utilization"]), "resets_at": en_secondes(f.get("resets_at"))}
    extra = reponse.get("extra_usage")
    plafond = extra.get("monthly_limit") if isinstance(extra, dict) else None
    if isinstance(extra, dict) and extra.get("is_enabled") and isinstance(plafond, (int, float)) and plafond > 0:
        pct = extra.get("utilization")
        if not isinstance(pct, (int, float)):
            pct = 100.0 * float(extra.get("used_credits") or 0) / plafond
        limites["credits"] = {"used_percentage": float(pct), "resets_at": 0}
    return limites


def demander(dossier, ouvrir=urllib.request.urlopen):
    """(les jauges, None) — ou (None, pourquoi pas) : pas connecté, connexion expirée, réseau."""
    try:
        cle = json.loads((dossier / ".credentials.json").read_text())["claudeAiOauth"]["accessToken"]
    except (OSError, ValueError, KeyError, TypeError):
        return None, "pas de connexion lisible"
    requete = urllib.request.Request(ADRESSE, headers={
        "Authorization": f"Bearer {cle}", "anthropic-beta": "oauth-2025-04-20", "User-Agent": "claude-code"})
    try:
        with ouvrir(requete, timeout=20) as r:
            return traduire(json.load(r)), None
    except urllib.error.HTTPError as e:
        return None, ("connexion expirée : une session sur ce compte la renouvelle" if e.code == 401
                      else "le serveur demande d'attendre quelques minutes" if e.code == 429
                      else f"le serveur répond {e.code}")
    except (OSError, ValueError) as e:
        return None, f"serveur injoignable ({e.__class__.__name__})"
