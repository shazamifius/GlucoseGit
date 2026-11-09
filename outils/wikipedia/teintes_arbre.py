#!/usr/bin/env python3
# **Une teinte par grande région, ses régions en nuances** — la proposition D de la fiche 95, en
# prototype (fiche 98) : un agent colore l'arbre des membranes comme un cartographe, par les seules
# commandes `glucose`, sans rien changer à Glucose. Rien ne se cache : seule la couleur s'ajoute.
#
#   python3 -I teintes_arbre.py <programme glucose> <membrane racine>
#
# L'algorithme est Tree Colors (Tennekes et de Jonge, IEEE TVCG 2014) : la racine reçoit toute la
# roue des teintes ; chaque membrane partage la sienne en parts égales entre ses enfants, les
# permute pour qu'aucun ordre ne se lise (deux frères voisins à 144° l'un de l'autre), retourne les
# parts paires (deux feuilles voisines de branches différentes ne se ressemblent pas), et n'en garde
# que le milieu, f = 0,75 — l'écart qui sépare les branches. Chaque région est donc proche de la
# teinte de son parent, et ses sœurs s'en écartent. La racine est grise. Sur un canevas sombre,
# c'est la méthode « additive » de l'article : la clarté monte avec la profondeur et la saturation
# descend (ses défauts, L₁ = 40, βL = +10, C₁ = 75, βC = −5 en HCL, valent pour trois ou quatre
# niveaux ; le squelette en a huit : la pente se déduit de la profondeur, entre deux bornes). Les
# teintes se calculent en Oklch, l'espace de `glucose_core::couleur`.
import json, math, subprocess, sys

glucose, racine = sys.argv[1], sys.argv[2]
FRACTION = 0.75
# Les bornes de la clarté et de la saturation (Oklch), du premier niveau au plus profond.
L_HAUT, L_BAS, C_HAUT, C_BAS = 0.66, 0.86, 0.15, 0.08


def g(*mots):
    r = subprocess.run([glucose, *mots], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f'glucose {mots[0]} : {r.stderr.strip()}')
    return json.loads(r.stdout)


def ordre(n):
    """La permutation des frères de l'article : à 144° l'un de l'autre sur le cercle."""
    if n == 3:
        return [0, 2, 1]
    if n == 4:
        return [0, 2, 1, 3]
    pas = int(0.4 * n)
    if pas == 0:
        return list(range(n))
    vus, k, suite = set(), 0, []
    while len(suite) < n:
        while k in vus:
            k = (k + 1) % n
        suite.append(k)
        vus.add(k)
        k = (k + pas) % n
    return suite


def srgb(l, c, h):
    """Oklch vers #rrggbb (Björn Ottosson, 2020), ramené dans le gamut par la saturation."""
    for _ in range(40):
        a, b = c * math.cos(math.radians(h)), c * math.sin(math.radians(h))
        l_ = (l + 0.3963377774 * a + 0.2158037573 * b) ** 3
        m_ = (l - 0.1055613458 * a - 0.0638541728 * b) ** 3
        s_ = (l - 0.0894841775 * a - 1.2914855480 * b) ** 3
        rgb = (4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_,
               -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_,
               -0.0041960863 * l_ - 0.7034186147 * m_ + 1.7076147010 * s_)
        if all(-1e-6 <= x <= 1 + 1e-6 for x in rgb):
            break
        c *= 0.95
    gamma = [12.92 * x if x <= 0.0031308 else 1.055 * x ** (1 / 2.4) - 0.055 for x in rgb]
    return '#' + ''.join(f'{round(min(max(x, 0), 1) * 255):02x}' for x in gamma)


plan = g('lire', 'niveau=plan')
enfants = {}
for n in plan['noeuds']:
    if n['sorte'] == 'membrane' and n.get('membrane'):
        enfants.setdefault(n['membrane'], []).append(n['id'])


def profondeur(v):
    return 1 + max((profondeur(e) for e in enfants.get(v, [])), default=0)


d = profondeur(racine) - 1
teintes = {}


def assigner(v, debut, fin, niveau):
    teintes[v] = ((debut + fin) / 2, niveau)
    fils = enfants.get(v, [])
    if not fils:
        return
    part = (fin - debut) / len(fils)
    parts = [(debut + i * part, debut + (i + 1) * part) for i in range(len(fils))]
    parts = [parts[i] for i in ordre(len(fils))]
    parts = [(b, a) if i % 2 == 1 else (a, b) for i, (a, b) in enumerate(parts)]
    for fil, (a, b) in zip(fils, parts):
        marge = (b - a) * (1 - FRACTION) / 2
        assigner(fil, a + marge, b - marge, niveau + 1)


assigner(racine, 0.0, 360.0, 0)
for v, (h, niveau) in teintes.items():
    if niveau == 0:
        couleur = srgb(0.70, 0.0, 0.0)
    else:
        t = (niveau - 1) / max(d - 1, 1)
        couleur = srgb(L_HAUT + t * (L_BAS - L_HAUT), C_HAUT + t * (C_BAS - C_HAUT), h % 360)
    g('poser-membrane', f'id={v}', f'couleur={couleur}')
print(f'{len(teintes)} membranes colorées, profondeur {d}')
