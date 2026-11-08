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
