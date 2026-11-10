#!/usr/bin/env python3
# **Le sens des couleurs des membranes**, en prototype (fiche 99 ; sa parole du 9/11/2026 : « il faut
# que TOUT ait une signification ») : un agent colore les membranes par les seules commandes
# `glucose`, et chaque grandeur de la couleur dit quelque chose.
#
#   python3 -I teintes_sens.py <programme glucose> <membrane racine>
#
# * **La teinte dit la proximité des domaines** — « sciences, société, sciences sociales ont des
#   couleurs proches ». Chaque membrane est le sac des mots de ses textes (TF-IDF) ; deux régions se
#   ressemblent comme leurs sacs (le cosinus). Les thèmes, enfants de la racine, sont rangés sur le
#   cercle des teintes par une **sériation circulaire exacte** : l'ordre du cercle qui rend la plus
#   grande la somme des ressemblances entre voisins (tous les ordres essayés — neuf thèmes, 20 160
#   cercles), puis chaque écart de teinte proportionnel à la dissemblance des deux voisins. Les plus
#   proches sont voisins, et le plus près l'un de l'autre. (Le positionnement multidimensionnel,
#   essayé d'abord, rendait le bruit de neuf thèmes presque également dissemblables : les deux plus
#   proches tombaient à 200° l'un de l'autre.) Une sous-région prend la moyenne circulaire des
#   teintes des thèmes, pondérée par sa ressemblance à chacun.
# * **La clarté dit l'accessibilité** — « un sujet est sombre ou clair selon qu'il est accessible à
#   tous ou non ». La lisibilité de Kandel et Moles (1958, l'adaptation française de Flesch) :
#   207 − 1,015 × mots par phrase − 73,6 × syllabes par mot. Le plus lisible du document est le plus
#   clair, le moins lisible le plus sombre, les autres **rangés entre eux à égale distance** (par
#   rang : un texte aberrant, une liste, n'écrase pas l'échelle des autres). Le plus sombre garde un
#   titre lisible sur le fond : le contraste « AA » des guides d'accessibilité (4,5 : 1), une borne
#   de la norme et non un réglage.
# * **La saturation n'a pas de sens attribué** : elle est la même pour toutes les teintes à une
#   clarté donnée — la plus forte que toutes savent montrer —, pour qu'aucune ne paraisse plus vive.
# * **L'opacité dirait l'appartenance** (« un sujet pile dans sa matière est opaque, un sujet qui en
#   dérive plus transparent ») : une membrane n'a pas d'opacité dans Glucose ; elle est mesurée (la
#   ressemblance à sa parente) et écrite dans le compte rendu, pas simulée.
import json, math, re, subprocess, sys, unicodedata

glucose, racine = sys.argv[1], sys.argv[2]
FOND = (15, 15, 16)  # le fond du canevas (theme.rs, bg_canvas), pour le contraste des titres
MOTS_VIDES = set("""le la les un une des de du d l et ou en au aux à a ce ces cet cette son sa ses
leur leurs qui que quoi dont où par pour sur sous dans avec sans est sont été être avoir ont fait
plus moins très aussi comme mais ne pas se s il ils elle elles on nous vous je tu y elle lui
entre après avant depuis selon ainsi lors tout tous toute toutes autre autres même mêmes peut
deux trois bien dont cela celle celui ceux celles si non été sera ni chez vers""".split())
VOYELLES = "aeiouyàâäéèêëîïôöùûüœæ"


def g(*mots):
    r = subprocess.run([glucose, *mots], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"glucose {mots[0]} : {r.stderr.strip()}")
    return json.loads(r.stdout)


# ── La couleur : Oklch, le bord de l'écran (les mêmes lois que glucose_core::couleur) ──────────

def srgb_lin(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def gamma(c):
    return 12.92 * c if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055


def oklch_srgb(l, c, h):
    a, b = c * math.cos(h), c * math.sin(h)
    l_ = (l + 0.3963377774 * a + 0.2158037573 * b) ** 3
    m_ = (l - 0.1055613458 * a - 0.0638541728 * b) ** 3
    s_ = (l - 0.0894841775 * a - 1.2914855480 * b) ** 3
    return [gamma(v) for v in (4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_,
                               -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_,
                               -0.0041960863 * l_ - 0.7034186147 * m_ + 1.7076147010 * s_)]


def chroma_maximale(l, h):
    dedans, dehors = 0.0, 0.5
    for _ in range(50):
        m = (dedans + dehors) / 2
        if all(-1e-9 <= v <= 1 + 1e-9 for v in oklch_srgb(l, m, h)):
            dedans = m
        else:
            dehors = m
    return dedans


def luminance(rgb):
    r, g_, b = (srgb_lin(v) for v in rgb)
    return 0.2126 * r + 0.7152 * g_ + 0.0722 * b


def clarte_aa():
    """La plus petite clarté Oklch d'un gris dont le contraste avec le fond atteint 4,5 : 1."""
    y_fond = luminance([v / 255 for v in FOND])
    bas, haut = 0.0, 1.0
    for _ in range(50):
        m = (bas + haut) / 2
        y = luminance(oklch_srgb(m, 0.0, 0.0))
        if (y + 0.05) / (y_fond + 0.05) >= 4.5:
            haut = m
        else:
            bas = m
    return haut


def hex_de(l, h, c):
    rgb = [round(min(max(v, 0.0), 1.0) * 255) for v in oklch_srgb(l, c, h)]
    return "#%02x%02x%02x" % tuple(rgb)


# ── Les textes : les mots, la lisibilité ────────────────────────────────────────────────────

def mots_de(texte):
    return [m for m in re.findall(r"[a-zàâäéèêëîïôöùûüçœæ]+", texte.lower())
            if len(m) >= 3 and m not in MOTS_VIDES]


def syllabes(mot):
    groupes = len(re.findall(f"[{VOYELLES}]+", mot))
    if mot.endswith("e") and groupes > 1:
        groupes -= 1
    return max(groupes, 1)


def lisibilite(texte):
    """Kandel et Moles (1958) : 207 − 1,015 × mots par phrase − 73,6 × syllabes par mot."""
    phrases = max(len(re.findall(r"[.!?…]+", texte)), 1)
    mots = re.findall(r"[A-Za-zÀ-ÿœæŒÆ]+", texte)
    if not mots:
        return None
    s = sum(syllabes(m.lower()) for m in mots)
    return 207 - 1.015 * len(mots) / phrases - 73.6 * s / len(mots)


def cosinus(u, v):
    num = sum(p * v.get(k, 0.0) for k, p in u.items())
    nu = math.sqrt(sum(p * p for p in u.values()))
    nv = math.sqrt(sum(p * p for p in v.values()))
    return num / (nu * nv) if nu and nv else 0.0


def cercle_des_teintes(sim):
    """La sériation circulaire exacte : l'ordre du cercle qui maximise la somme des ressemblances
    entre voisins, puis des écarts de teinte proportionnels à la dissemblance des voisins (un tour
    entier). Rend la teinte de chaque thème, en radians."""
    import itertools
    n = len(sim)
    if n == 1:
        return [0.0]
    meilleur, ordre = -1.0, list(range(n))
    for reste in itertools.permutations(range(1, n)):
        if n > 2 and reste[0] > reste[-1]:
            continue  # un cercle et son miroir ne font qu'un
        tour = (0,) + reste
        somme = sum(sim[tour[k]][tour[(k + 1) % n]] for k in range(n))
        if somme > meilleur:
            meilleur, ordre = somme, list(tour)
    ecarts = [1 - sim[ordre[k]][ordre[(k + 1) % n]] for k in range(n)]
    total = sum(ecarts)
    teintes, angle = [0.0] * n, 0.0
    for k in range(n):
        teintes[ordre[k]] = angle
        angle += math.tau * ecarts[k] / total
    return teintes


# ── L'arbre et ses textes ───────────────────────────────────────────────────────────────────

lu = g("lire", "niveau=complet")
noeuds = lu["noeuds"]
membranes = {n["id"]: n for n in noeuds if n.get("sorte") == "membrane"}
parent = {n["id"]: n.get("membrane") for n in noeuds}
enfants = {}
for i, p in parent.items():
    enfants.setdefault(p, []).append(i)


def textes_sous(mid):
    """Les textes que porte une membrane, à toute profondeur, et son titre."""
    out = [membranes[mid].get("titre") or membranes[mid].get("texte") or membranes[mid].get("debut") or ""]
    for e in enfants.get(mid, []):
        n = next(x for x in noeuds if x["id"] == e)
        if n.get("sorte") == "membrane":
            out.extend(textes_sous(e))
        else:
            out.append(n.get("texte") or n.get("debut") or "")
    return out


ids = [m for m in membranes if m != racine]
corps = {m: " ".join(textes_sous(m)) for m in ids}
sacs = {m: {} for m in ids}
for m in ids:
    for mot in mots_de(corps[m]):
        sacs[m][mot] = sacs[m].get(mot, 0) + 1
presence = {}
for m in ids:
    for mot in sacs[m]:
        presence[mot] = presence.get(mot, 0) + 1
tfidf = {m: {mot: (1 + math.log(f)) * math.log(len(ids) / presence[mot]) for mot, f in sacs[m].items()}
         for m in ids}

# **La proximité, version 2 (fiche 102) : les liens que la carte porte.** Deux régions se ressemblent
# comme les articles qu'elles citent et qui les citent — la mesure par vecteurs de liens de Milne et
# Witten (*An effective, low-cost measure of semantic relatedness obtained from Wikipedia links*, 2008),
# où un lien vers un article que tous citent compte moins (le poids log(N / degré)). Chaque région se
# compte elle-même parmi ses voisins, pour qu'un lien direct entre deux régions les rapproche aussi.
# L'agent lit tout dans la carte (`glucose lire`) : ce qu'un humain voit relié, il le voit relié. Les
# mots (TF-IDF des résumés, la version 1 : des cosinus de 0,05 à 0,14) restent la mesure d'une carte
# sans liens, ou demandée par `GLUCOSE_PROXIMITE=mots`.
def article_de(nid):
    """La membrane qui porte ce bout de lien : elle-même, ou celle qui contient le texte."""
    return nid if nid in membranes else parent.get(nid)


liens = [(article_de(n.get("source")), article_de(n.get("cible"))) for n in noeuds
         if n.get("sorte") == "fleche" and n.get("source") and n.get("cible")]
liens = [(a, b) for a, b in liens if a in membranes and b in membranes and a != b]
voisins = {}
for a, b in liens:
    voisins.setdefault(a, set()).add(b)
    voisins.setdefault(b, set()).add(a)
par_les_liens = bool(liens) and __import__("os").environ.get("GLUCOSE_PROXIMITE") != "mots"


def sous(mid):
    """Les membranes d'une région, elle comprise, à toute profondeur."""
    out = [mid]
    for e in enfants.get(mid, []):
        if e in membranes:
            out.extend(sous(e))
    return out


N = max(len(voisins), 1)
idf = {x: math.log(N / len(v)) + 1.0 / N for x, v in voisins.items()}
profil = {}
for m in ids:
    v = {}
    for a in sous(m):
        for x in voisins.get(a, set()) | ({a} if a in voisins else set()):
            v[x] = v.get(x, 0.0) + idf[x]
    profil[m] = v
ressemblance = (lambda a, b: cosinus(profil[a], profil[b])) if par_les_liens \
    else (lambda a, b: cosinus(tfidf[a], tfidf[b]))
print(f"proximité : {'les liens de la carte' if par_les_liens else 'les mots des textes'}"
      + (f" ({len(liens)} liens entre {len(voisins)} membranes)" if par_les_liens else ""))

themes = [m for m in enfants.get(racine, []) if m in membranes]
sim = [[ressemblance(a, b) for b in themes] for a in themes]
angles = cercle_des_teintes(sim)
teinte = {t: angles[i] for i, t in enumerate(themes)}

def teinte_de(m):
    """La moyenne circulaire des teintes des thèmes, pondérée par la ressemblance à chacun ; une
    région dont la mesure ne dit rien (aucun lien, aucun mot) prend la teinte de sa parente : elle
    est dans son domaine."""
    if m in teinte:
        return teinte[m]
    poids = [(max(ressemblance(m, t), 0.0), teinte[t]) for t in themes]
    x = sum(p * math.cos(h) for p, h in poids)
    y = sum(p * math.sin(h) for p, h in poids)
    if x or y:
        teinte[m] = math.atan2(y, x)
    else:
        teinte[m] = teinte_de(parent[m]) if parent.get(m) in membranes and parent[m] != racine else 0.0
    return teinte[m]


for m in ids:
    teinte_de(m)

lisib = {m: lisibilite(corps[m]) for m in ids}
connues = sorted(v for v in lisib.values() if v is not None)
bas_lu, haut_lu = connues[0], connues[-1]
rang = lambda v: connues.index(v) / max(len(connues) - 1, 1)
L_BAS, L_HAUT = clarte_aa(), 0.92
print(f"clarté : de {L_BAS:.3f} (contraste AA sur le fond) à {L_HAUT} ; lisibilité de {bas_lu:.0f} à {haut_lu:.0f}")
for m in ids:
    v = lisib[m] if lisib[m] is not None else connues[len(connues) // 2]
    frac = rang(v)
    l = L_BAS + frac * (L_HAUT - L_BAS)
    c = min(chroma_maximale(l, i * math.tau / 72) for i in range(72))
    couleur = hex_de(l, teinte[m], c)
    appartenance = ressemblance(m, parent[m]) if parent[m] in tfidf else 1.0
    g("poser-membrane", f"id={m}", f"couleur={couleur}")
    titre = (membranes[m].get("titre") or membranes[m].get("debut") or m)[:40]
    print(f"{titre:40s} teinte {math.degrees(teinte[m]) % 360:5.0f}°  lisibilité {v:5.0f}  "
          f"clarté {l:.2f}  appartenance {appartenance:.2f}  {couleur}")

print("\nles thèmes, leurs ressemblances (cosinus), dans l'ordre du cercle :")
for i, a in enumerate(themes):
    voisins = sorted(((sim[i][j], themes[j]) for j in range(len(themes)) if j != i), reverse=True)[:2]
    nom = lambda t: (membranes[t].get("debut") or t)[:22]
    print(f"  {nom(a):22s} → " + ", ".join(f"{nom(t)} {s:.2f}" for s, t in voisins))
