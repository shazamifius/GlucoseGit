#!/usr/bin/env python3
# Pose un article de Wikipédia dans le Glucose ouvert, par les seules commandes `glucose` (fiche 90) :
# une membrane pour l'article, une par section et sous-section, emboîtées ; un texte par paragraphe ;
# les catégories de l'article comme domaines de ses paragraphes. Les positions : celles que Glucose
# choisit lui-même (sous le contenu de la membrane).
#
#   python3 -I poser_un_article.py <article.json de l'API> <programme glucose> <préfixe des ids>
import json, re, subprocess, sys, time

source, glucose, prefixe = sys.argv[1], sys.argv[2], sys.argv[3]
page = list(json.load(open(source, encoding='utf-8'))['query']['pages'].values())[0]
titre, texte = page['title'], page['extract']
IGNOREES = {'Notes et références', 'Voir aussi', 'Bibliographie', 'Liens externes', 'Articles connexes',
            'Références', 'Notes'}
appels, duree = 0, 0.0


def glucose_(*mots, entree=None):
    global appels, duree
    debut = time.monotonic()
    r = subprocess.run([glucose, *mots], input=entree, capture_output=True, text=True)
    duree += time.monotonic() - debut
    appels += 1
    if r.returncode != 0:
        sys.exit(f'glucose {" ".join(mots[:2])} : code {r.returncode}, {r.stderr.strip()}')
    return json.loads(r.stdout)


glucose_('poser-membrane', f'id={prefixe}', f'titre={titre} — Wikipédia')
pile = [(1, prefixe)]  # (niveau, membrane)
paragraphes, n, ignore = [], 0, False
courant = prefixe
for ligne in texte.split('\n'):
    m = re.match(r'^(=+)\s*(.*?)\s*=+$', ligne.strip())
    if m:
        niveau, nom = len(m.group(1)), m.group(2)
        ignore = nom in IGNOREES and niveau == 2
        if ignore:
            continue
        while pile and pile[-1][0] >= niveau:
            pile.pop()
        n += 1
        courant = f'{prefixe}-s{n}'
        glucose_('poser-membrane', f'id={courant}', f'titre={nom}', f'dans={pile[-1][1]}')
        pile.append((niveau, courant))
        continue
    if ignore or not ligne.strip():
        continue
    n += 1
    ident = f'{prefixe}-p{n}'
    glucose_('poser-texte', f'id={ident}', f'dans={courant}', 'texte=-', entree=ligne.strip())
    paragraphes.append(ident)

categories = [c['title'].split(':', 1)[1] for c in page.get('categories', [])]
for cat in categories:
    r = subprocess.run([glucose, 'poser-domaine', f'nouveau={cat}', *[f'ids={i}' for i in paragraphes]],
                       capture_output=True, text=True)
    appels += 1
    print(f'  domaine « {cat} » : code {r.returncode} {r.stderr.strip()[:160]}')
print(f'{len(paragraphes)} paragraphes, {n - len(paragraphes)} membranes de sections, '
      f'{len(categories)} domaines ; {appels} commandes, {duree:.1f} s, {1000 * duree / max(appels, 1):.0f} ms chacune')
