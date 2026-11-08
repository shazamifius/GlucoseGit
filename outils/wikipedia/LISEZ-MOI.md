# Un article de Wikipédia dans Glucose, par les commandes (fiche 91)

Rien ne s'ouvre sur son écran : le vrai Glucose sans fenêtre (`porte_sans_fenetre`), et les commandes
`glucose` contre sa porte.

```bash
nix develop -c cargo build -p glucose-desktop --bin glucose-desktop --example porte_sans_fenetre
# L'article (Wikimedia veut un nom de client ; jamais une adresse de courriel) :
curl -s -A 'Glucose-essai/1.0 (https://github.com/shazamifius/GlucoseGit)' \
  'https://fr.wikipedia.org/w/api.php?action=query&prop=extracts|categories|links&explaintext=1&titles=Eau&format=json&cllimit=max&pllimit=max&clshow=!hidden' \
  -o eau.json
# Un Glucose sans fenêtre, dans un dossier jetable ; une prise Unix tient en 108 caractères.
W=$(mktemp -d); mkdir -p $W/glucose $W/images; : > $W/images/main.jsonl
prise=$(mktemp -u /tmp/glucose-wiki.XXXXXX)
GLUCOSE_DOSSIER_ESSAI=$W/glucose target/debug/examples/porte_sans_fenetre $prise $W/arret $W/images &
export GLUCOSE_PORTE=$prise
python3 -I outils/wikipedia/poser_un_article.py eau.json target/debug/glucose-desktop eau
python3 -I outils/wikipedia/en_colonnes.py target/debug/glucose-desktop eau 1.6   # le prototype
target/debug/glucose-desktop lire niveau=plan                                     # la carte
: > $W/arret   # Glucose finit, et rend le tableau en images dans $W/images
```

## Le squelette (fiche 95)

Les cent articles vitaux du niveau 2, rangés comme Wikipédia les range (neuf thèmes, des articles
emboîtés), chacun une membrane à son titre français avec ses deux premières phrases ; « Eau » et
« Hydrogène » entiers à leur place ; puis `glucose disposer wiki`. Les réponses de Wikipédia sont
gardées dans le dossier de travail : une seconde pose ne redemande rien.

```bash
python3 -I outils/wikipedia/squelette.py target/debug/glucose-desktop <dossier de travail>
```

**En une commande, dans le vrai Glucose sans fenêtre** (fiche 97) — la pose, le tableau entier, les
neuf thèmes et les deux articles en images (`GLUCOSE_VUES` nomme d'autres membranes à rendre), le
plan, et le document à ouvrir dans un essai isolé :

```bash
nix develop -c bash outils/wikipedia/essai_squelette.sh <dossier de sortie>
```

Ce qu'un agent pose `dans=` une membrane lui appartient, même si un voisin grandi la recouvre
(MEMB-8, fiche 97) : c'est ce qui rend le squelette juste.
