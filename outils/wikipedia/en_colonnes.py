#!/usr/bin/env python3
# Prototype : disposer un article posé dans Glucose EN COLONNES DE JOURNAL, de bas en haut, dans l'ordre
# de lecture, par les seules commandes `glucose`.
#
# Chaque membrane range ses enfants (textes et sous-membranes, dans l'ordre du document) en k colonnes
# contiguës ; la partition linéaire (programmation dynamique, exacte) rend la plus haute colonne la plus
# courte possible ; k est choisi pour que la membrane ait la proportion visée (celle de l'écran).
#
#   python3 -I en_colonnes.py <programme glucose> <préfixe> <proportion>
import json, math, re, subprocess, sys

glucose, prefixe, cible = sys.argv[1], sys.argv[2], float(sys.argv[3])
MARGE, ECART, BANDEAU = 32.0, 24.0, 56.0  # le bord, entre deux colonnes ou deux nœuds, le titre


def g(*mots):
    r = subprocess.run([glucose, *mots], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f'glucose {mots[0]} : {r.stderr.strip()}')
    return json.loads(r.stdout)


noeuds = {n['id']: n for n in g('lire')['noeuds'] if n['id'].startswith(prefixe)}
rang = lambda i: int(re.search(r'(\d+)$', i).group(1)) if i != prefixe else -1
enfants = {}
for i, n in sorted(noeuds.items(), key=lambda kv: rang(kv[0])):
    if n.get('membrane'):
        enfants.setdefault(n['membrane'], []).append(i)
taille = {}   # id -> (largeur, hauteur) une fois disposé
place = {}    # id -> (x, y) relatif au coin de sa membrane parente


def partition(h, k):
    """Coupe la suite h en k morceaux contigus : la plus grande somme (écarts compris) la plus petite."""
    n = len(h)
    pre = [0.0]
    for x in h:
        pre.append(pre[-1] + x)
    s = lambda a, b: pre[b] - pre[a] + ECART * (b - a - 1)
    INF = float('inf')
    D = [[INF] * (n + 1) for _ in range(k + 1)]
    C = [[0] * (n + 1) for _ in range(k + 1)]
    D[0][0] = 0.0
    for j in range(1, k + 1):
        for b in range(j, n + 1):
            for a in range(j - 1, b):
                v = max(D[j - 1][a], s(a, b))
                if v < D[j][b]:
                    D[j][b], C[j][b] = v, a
    coupes, b = [], n
    for j in range(k, 0, -1):
        coupes.append((C[j][b], b))
        b = C[j][b]
    return list(reversed(coupes))


def disposer(m):
    fils = enfants.get(m, [])
    for f in fils:
        if f in enfants:
            disposer(f)
        else:
            b = noeuds[f]['boite']
            taille[f] = (b['largeur'], b['hauteur'])
    if not fils:
        taille[m] = (240.0, BANDEAU)
        return
    meilleur = None
    for k in range(1, len(fils) + 1):
        cols = partition([taille[f][1] for f in fils], k)
        largeurs = [max(taille[f][0] for f in fils[a:b]) for a, b in cols]
        hauteurs = [sum(taille[f][1] for f in fils[a:b]) + ECART * (b - a - 1) for a, b in cols]
        L = sum(largeurs) + ECART * (k - 1) + 2 * MARGE
        H = max(hauteurs) + BANDEAU + 2 * MARGE
        ecart = abs(math.log((L / H) / cible))
        if meilleur is None or ecart < meilleur[0]:
            meilleur = (ecart, cols, largeurs, L, H)
    _, cols, largeurs, L, H = meilleur
    x = MARGE
    for (a, b), lc in zip(cols, largeurs):
        y = MARGE + BANDEAU
        for f in fils[a:b]:
            place[f] = (x, y)
            y += taille[f][1] + ECART
        x += lc + ECART
    taille[m] = (L, H)


disposer(prefixe)
# Du haut vers le bas : chaque membrane prend sa boîte, puis ce qu'elle contient y vient.
racine = noeuds[prefixe]['boite']
absolu = {prefixe: (racine['x'], racine['y'])}
ordre, file = [], [prefixe]
while file:
    m = file.pop(0)
    ordre.append(m)
    for f in enfants.get(m, []):
        absolu[f] = (absolu[m][0] + place[f][0], absolu[m][1] + place[f][1])
        file.append(f)
for i in ordre:
    x, y = absolu[i]
    if i in enfants or noeuds[i]['sorte'] == 'membrane':
        L, H = taille[i]
        titre = noeuds[i].get('debut', '')
        g('poser-membrane', f'id={i}', f'titre={titre}', f'x={x}', f'y={y}', f'largeur={L}',
          f'hauteur={H}')
    else:
        g('deplacer', f'id={i}', f'x={x}', f'y={y}')
L, H = taille[prefixe]
print(f'{len(ordre)} nœuds disposés ; l\'article : {L:.0f} × {H:.0f} (proportion {L / H:.2f}, visée {cible})')
