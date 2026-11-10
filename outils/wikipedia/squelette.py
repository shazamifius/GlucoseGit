#!/usr/bin/env python3
# **Un squelette de Wikipédia** (fiches 94 et 95) : les cent articles vitaux du niveau 2, rangés comme
# Wikipédia les range — neuf grands thèmes, et dans chacun des articles emboîtés par leurs puces —,
# posés dans le Glucose ouvert par les seules commandes `glucose`. Chaque article est une membrane,
# à son titre français, qui porte ses deux premières phrases en français ; ses sous-articles sont
# dedans. Puis « Eau » et « Hydrogène », entiers, à leur place, et tout se range en colonnes.
#
#   python3 -I squelette.py <programme glucose> <dossier de travail>
#
# La liste vient de `Wikipedia:Vital articles/Level 2` (anglais : il n'existe pas de liste française
# équivalente) ; les titres français, des liens entre langues de chaque article ; un article sans
# version française garde son titre anglais, et le compte rendu le dit. Wikimedia veut un nom de
# client : jamais une adresse de courriel.
import hashlib, json, os, re, subprocess, sys, time, urllib.parse, urllib.request

glucose, travail = sys.argv[1], sys.argv[2]
os.makedirs(travail, exist_ok=True)
CLIENT = 'Glucose-essai/1.0 (https://github.com/shazamifius/GlucoseGit)'
THEMES = {
    'History': 'Histoire', 'Geography': 'Géographie', 'Arts': 'Arts', 'Everyday life': 'Vie quotidienne',
    'Philosophy and religion': 'Philosophie et religion',
    'Society and social sciences': 'Société et sciences sociales',
    'Science and medicine': 'Sciences et médecine', 'Technology': 'Technologie',
    'Mathematics': 'Mathématiques',
}
# Les intitulés de la liste qui ne sont pas des articles.
INTITULES = {'Continents and regions': 'Continents et régions'}
# Les articles que Wikipédia en anglais ne relie à aucun article français (vérifié le 8/11/2026) :
# leur équivalent usuel.
SANS_LIEN = {'Post-classical history': 'Moyen Âge'}
appels, duree = 0, 0.0


def api(hote, **params):
    """Une requête à l'API de Wikipédia, gardée dans le dossier de travail."""
    requete = urllib.parse.urlencode({'format': 'json', 'formatversion': 2, **params})
    cache = os.path.join(travail, hashlib.sha1(f'{hote}?{requete}'.encode()).hexdigest()[:16] + '.json')
    if not os.path.exists(cache):
        req = urllib.request.Request(f'https://{hote}/w/api.php?{requete}', headers={'User-Agent': CLIENT})
        with urllib.request.urlopen(req, timeout=60) as r, open(cache, 'wb') as f:
            f.write(r.read())
        time.sleep(0.2)
    return json.load(open(cache, encoding='utf-8'))


def glucose_(*mots, entree=None):
    global appels, duree
    debut = time.monotonic()
    r = subprocess.run([glucose, *mots], input=entree, capture_output=True, text=True)
    duree += time.monotonic() - debut
    appels += 1
    if r.returncode != 0:
        sys.exit(f'glucose {" ".join(mots[:2])} : code {r.returncode}, {r.stderr.strip()}')
    return json.loads(r.stdout)


# 1. La liste, en arbre : (thème, [(profondeur, titre anglais, est un article)]).
texte = api('en.wikipedia.org', action='parse', page='Wikipedia:Vital_articles/Level_2',
            prop='wikitext')['parse']['wikitext']
themes = []
for ligne in texte.split('\n'):
    m = re.match(r'^==\s*(.*?)\s*==$', ligne.strip())
    if m and m.group(1) in THEMES:
        themes.append((m.group(1), []))
        continue
    m = re.match(r'^(\*+)\s*(.*)$', ligne)
    if not m or not themes:
        continue
    profondeur, reste = len(m.group(1)), m.group(2)
    lien = re.search(r"\[\[([^\]|]+)(?:\|[^\]]*)?\]\]", re.sub(r'\(\[\[Wikipedia:[^)]*\)', '', reste))
    if lien:
        themes[-1][1].append((profondeur, lien.group(1), True))
    else:
        intitule = re.sub(r"''+", '', reste).strip()
        themes[-1][1].append((profondeur, intitule, intitule not in INTITULES))

# 2. Les titres français, par les liens entre langues (cinquante titres par requête).
titres = sorted({t for _, articles in themes for _, t, est in articles if est})
francais = {}
for i in range(0, len(titres), 50):
    r = api('en.wikipedia.org', action='query', titles='|'.join(titres[i:i + 50]), redirects=1,
            prop='langlinks', lllang='fr', lllimit='max')['query']
    vers = {x['from']: x['to'] for x in r.get('normalized', []) + r.get('redirects', [])}

    def final(t):
        vus = set()
        while t in vers and t not in vus:
            vus.add(t)
            t = vers[t]
        return t

    for page in r['pages']:
        fr = next((l['title'] for l in page.get('langlinks', [])), None)
        for origine in [t for t in titres[i:i + 50] if final(t) == page['title']]:
            francais[origine] = fr
sans = [t for t in titres if not francais.get(t)]
francais.update({t: f for t, f in SANS_LIEN.items() if not francais.get(t)})

# 3. Les deux premières phrases de chaque article, en français (vingt par requête).
noms = sorted({f for f in francais.values() if f})
phrases = {}
for i in range(0, len(noms), 20):
    r = api('fr.wikipedia.org', action='query', titles='|'.join(noms[i:i + 20]), redirects=1,
            prop='extracts', exintro=1, explaintext=1, exsentences=2, exlimit=20)['query']
    vers = {x['to']: x['from'] for x in r.get('redirects', []) + r.get('normalized', [])}
    for page in r['pages']:
        phrases[vers.get(page['title'], page['title'])] = page.get('extract', '').strip()

# 4. La pose : un thème, puis ses articles, chacun dans celui que sa puce désigne.
glucose_('poser-membrane', 'id=wiki', 'titre=Wikipédia — les cent articles vitaux')
ids, n = {}, 0
for k, (theme, articles) in enumerate(themes):
    tid = f'theme{k}'
    glucose_('poser-membrane', f'id={tid}', f'titre={THEMES[theme]}', 'dans=wiki')
    pile = [(0, tid)]
    for profondeur, titre, est in articles:
        while pile[-1][0] >= profondeur:
            pile.pop()
        n += 1
        aid = f'va{n}'
        nom = (francais.get(titre) if est else INTITULES.get(titre)) or titre
        glucose_('poser-membrane', f'id={aid}', f'titre={nom}', f'dans={pile[-1][1]}')
        if est and phrases.get(nom):
            glucose_('poser-texte', f'id={aid}-t', f'dans={aid}', 'texte=-', entree=phrases[nom])
        ids[titre] = aid
        pile.append((profondeur, aid))
print(f'{len(themes)} thèmes, {n} articles ; sans lien vers le français : {sans or "aucun"}')

# 5. « Eau » et « Hydrogène », entiers, à leur place.
ici = os.path.dirname(os.path.abspath(__file__))
for titre, prefixe, dans in [('Eau', 'eau', ids['Water']), ('Hydrogène', 'hydrogene', ids['Chemical element'])]:
    article = api('fr.wikipedia.org', action='query', prop='extracts|categories|links', explaintext=1,
                  titles=titre, cllimit='max', pllimit='max', clshow='!hidden', formatversion=1)
    chemin = os.path.join(travail, f'{prefixe}.json')
    json.dump(article, open(chemin, 'w', encoding='utf-8'), ensure_ascii=False)
    subprocess.run([sys.executable, '-I', os.path.join(ici, 'poser_un_article.py'), chemin, glucose,
                    prefixe, f'dans={dans}'], check=True)

# 6. Tout se range, en colonnes, à la proportion de l'écran.
debut = time.monotonic()
print('disposer :', glucose_('disposer', 'wiki'), f'en {time.monotonic() - debut:.1f} s')

# 7. Sur demande (`GLUCOSE_LIENS=1`, fiche 102) : les liens de Wikipédia entre ces articles. Chaque
# lien d'un article vers un autre des cent devient une flèche, du texte de l'article (là où un
# lecteur le trouve) vers la membrane de l'autre. Un bloc de cinquante articles contre un bloc de
# cinquante cibles par requête (`pltitles`) : quelques requêtes pour tout le graphe.
if os.environ.get('GLUCOSE_LIENS') == '1':
    membrane = {francais.get(t) or INTITULES.get(t) or t: aid for t, aid in ids.items()}
    avec_texte = {nom for nom in membrane if phrases.get(nom)}
    noms = sorted(membrane)
    liens = set()
    for i in range(0, len(noms), 50):
        for j in range(0, len(noms), 50):
            suite = {}
            while True:
                r = api('fr.wikipedia.org', action='query', titles='|'.join(noms[i:i + 50]),
                        prop='links', pltitles='|'.join(noms[j:j + 50]), pllimit='max', **suite)
                for page in r['query']['pages']:
                    for lien in page.get('links', []):
                        if page['title'] in membrane and lien['title'] in membrane \
                                and lien['title'] != page['title']:
                            liens.add((page['title'], lien['title']))
                if 'continue' not in r:
                    break
                suite = {k: v for k, v in r['continue'].items() if k != 'continue'}
    for k, (de, vers) in enumerate(sorted(liens)):
        source = f'{membrane[de]}-t' if de in avec_texte else membrane[de]
        glucose_('relier', f'id=lien{k}', f'source={source}', f'cible={membrane[vers]}')
    print(f'{len(liens)} liens entre les articles')
print(f'{appels} commandes du squelette, {duree:.1f} s')
