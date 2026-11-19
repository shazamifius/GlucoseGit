# 121 — Le détourage, par un moteur à nous ; la beta.3 ; les tiroirs du côté de leur bouton

> Session du 18/11/2026, depuis 12 h 20, après la fiche 120. Le prompt de passation demandait de
> vérifier `main`, de publier ou refaire la beta, de prendre la question des tiroirs, l'échelle de
> l'interface et **le détourage** (sa demande du 18/11, VISION § 3). Sa fiche de questions ne s'ouvre
> pas depuis cette session (les outils de Claude Docs la refusent : « pas partagée, ou introuvable ») :
> ses réponses, s'il y en a, ne sont pas lues.

## 1. Ce qu'on a trouvé en arrivant

* **La 2.0.4-beta.2 était publiée** — son clic, le 18/11 à 11 h 49 —, construite sur `c8d7585` :
  **sans** le travail du matin (le texte net, les modes de la fenêtre, la barre de Windows, renommer
  un dossier). La télémétrie montre son PC de l'école déjà en beta.2 (58 sessions, aucune fin anormale
  neuve).
* **`main` vérifié** : la suite entière verte sur `b7470299` (**3 359** épreuves, 7 ignorées) ; le
  sabotage du script de KWin, jamais rejoué, **tombe** (le second chantier de la fiche 120 : 19 sur
  19) ; la vérification de GitHub **verte, dix tâches**, Windows compris, sur `15beb94` (les notes de
  la beta.3).
* **La 2.0.4-beta.3 en brouillon** (le flux « Publier », onze fichiers, sur `15beb94`) : à son clic.

## 2. Le détourage — la recherche (consultée le 18/11/2026)

Les sources et la décision : [`decisions/17`](decisions/17-LE-DETOURAGE-PAR-U2NETP.md). L'essentiel :

* **Les modèles ouverts** : U²-Net et u2netp (Apache 2.0), ISNet (Apache 2.0), BiRefNet (MIT), BEN2
  base (MIT), MVANet (MIT) ; RMBG-1.4 et 2.0 de BRIA **exclus** (non commerciaux, et le défaut de
  `rembg`).
* **Les données d'entraînement** : DIS5K (ISNet, BiRefNet) interdit l'usage commercial « même après
  traitement » — lu dans le PDF de ses auteurs ; DUTS-TR (U²-Net) n'a pas de licence publiée.
  **Aucun jeu de détourage n'a de licence commerciale claire** : c'est dit dans la note, et accepté
  pour des poids que leurs auteurs donnent sous licence permise.
* **En Rust**, tout ce qui existe passe par ONNX Runtime (`ort`) : du C++ précompilé.

## 3. Mesurer avant de choisir

Hors du dépôt (`~/.cache/glucose-bancs/detourage-121/`), sur des images dont on connaît la vérité —
**sa boule sur un fond multicolore** (fabriquée, son masque exact), une boule et un anneau sur blanc,
un dessin au trait — et cinq photos libres de Wikimedia Commons :

| | u2netp (référence ONNX) | ce que ça dit |
|---|---|---|
| la boule sur fond multicolore | 0,32 % d'erreur moyenne d'opacité | la forme principale, presque parfaite |
| l'anneau sur blanc | 1,0 % | **le trou retiré** |
| le dessin au trait | **32 %** | il garde le blanc entre les traits : le modèle cherche un objet, le dessin entier en est un |
| le vélo devant des graffitis | la façade entière gardée | « la forme principale » y est ambiguë |
| le coût | 19,8 G multiplications, 0,8 à 1,2 s sur un cœur | |

Et de près : **un liseré vert** au bord de la boule (la couleur du fond dans les pixels mêlés), de
faibles fantômes des rayures. Les poids en demi-précision changent le masque jusqu'à 0,1 sur des
bords : refusés.

**Deux gestes, donc, pas un** : « Détourer » (l'IA, les photos) et « Retirer le fond uni » (exact,
les dessins, les logos, les scans).

## 4. DETOURER-1 — un moteur à nous, le modèle comme donnée

* **Le format** (`outils/detourage/convertir.py`) : le graphe ONNX figé à sa taille d'entrée, les
  calculs de forme retirés (la taille de chaque agrandissement demandée à ONNX Runtime, qui exécute
  le graphe une fois), les ReLU fondus, les sorties de côté retirées : 253 opérations, sept sortes.
* **L'étalon** (`outils/detourage/reference.py`) : le format exécuté en numpy, **à 2·10⁻⁵ d'ONNX
  Runtime** — la conversion n'a rien changé.
* **Le moteur** (`glucose-core/src/detourage/`, sans dépendance) : la lecture vérifie chaque forme
  avant tout calcul ; la convolution par blocs de quatre canaux, ligne par ligne, les fils sur les
  canaux, **AVX2 et FMA choisis à l'exécution** (la charte) ; **à 4·10⁻⁵ de l'étalon** — et
  4,0·10⁻⁵ encore avec la convolution par tuiles, remesuré le 19/11 (fiche 122 ; la moyenne à
  9·10⁻⁸, le maximum exact).
* **Le coût, mesuré** sur son VivoBook : le masque en **1,0 s** (7 fils), 1,2 à 1,5 s avec la
  couleur du premier plan ; la référence ONNX fait 0,85 s sur un seul cœur — notre convolution est
  encore environ huit fois moins rapide par cœur (§ 9).
* **La justesse sur les images** : la boule 0,29 %, l'anneau 0,47 %, la boule sur blanc 0,30 % —
  égale ou meilleure que la référence (le prétraitement diffère : le nôtre réduit par le filtre
  triangle élargi, celle de `rembg` par Lanczos).

## 5. DETOURER-2 — la couleur du premier plan

La méthode de Marco Forte (ICIP 2021), réécrite depuis l'article : deux passes de flou en boîte
(rayons 90 puis 6, ceux de l'article), par sommes glissantes. **Le liseré vert a disparu** (regardé
agrandi huit fois) ; sur un bord synthétique, l'écart à la vraie couleur passe de 0,72 à 0,023.

## 6. DETOURER-3 — retirer le fond uni

La couleur du fond est la médiane du cadre de l'image ; chaque pixel devient aussi transparent que
possible tel que, reposé sur ce fond, il redonne l'original — la « couleur vers alpha » de GIMP à
ses réglages par défaut : **aucun seuil**. Un trait gris d'anticrénelage devient du noir à demi.

## 7. Le geste dans Glucose

« **Détourer : la forme principale** » et « **Retirer le fond uni** », au menu d'une image et à la
palette — offerts seulement sur une image déverrouillée qui a des octets (l'épreuve d'APPLIQUE-1 l'a
exigé : offrir un geste qui ne fait rien est un bouton qui ment). Les pixels natifs attendus comme
ceux de `Ctrl+B` ; **un fil** détoure, puis réveille la boucle (comme l'export) ; le curseur
`Progress` du système dit le travail — aucun message ajouté. **Un seul geste** pose tout : une clé
neuve par image, dont le magasin adopte les pixels et promet le PNG au scribe (COLLER-3) ; `Ctrl+Z`
rend l'original, que le document garde ; le co-working envoie les octets neufs (B1). Une image
changée, retirée ou verrouillée pendant le calcul n'est pas touchée.

## 8. DEVANT-1, essayé sous sway et sous KDE

La fiche 120 n'avait essayé « toujours devant » que sous niri ; les autres bureaux étaient lus dans
leur documentation. Essayés ici, **hors de son écran**, par le vrai Glucose et le scénario
(`4 mode-reference`, puis de nouveau à 10 s) — les scripts dans `~/.cache/glucose-bancs/devant-121/` :

* **sway 1.12** (le sway sans écran du pilote ; un lanceur donne à Glucose sa prise, `SWAYSOCK`) :
  avant, la fenêtre en mosaïque, 1920 × 1080 ; après `Ctrl+Maj+A`, **flottante**, 1440 × 900 au
  milieu ; au second geste, de nouveau en mosaïque — lu dans `swaymsg -t get_tree`.
* **KDE Plasma 6, KWin 6.7.5** (`kwin_wayland --virtual`, son propre bus par `dbus-run-session`,
  son propre dossier d'exécution : rien de sa session touché) : la fenêtre **garde-au-dessus false,
  puis true, puis false** — lu par un script de KWin qui écrit `keepAbove`. Deux pièges payés : le
  KWin sans écran n'importe pas les images que la carte graphique présente par `dmabuf` (Glucose y
  tombait) — la voie du processeur, forcée par `moteurs.txt`, y présente ; et les messages des
  scripts de KWin partent au journal du système tant que `QT_FORCE_STDERR_LOGGING=1` ne les rend pas.
* **Hyprland 0.56.2** : pas essayé — imbriqué dans sway, il exige `xdg_wm_base` 6, que sway 1.12
  n'offre pas (5), et s'arrête. Il reste lu dans sa documentation (`hyprctl setfloating pid:`).
* **GNOME** : Glucose y dit le geste à faire (`Alt+Espace`) ; rien à essayer.

## 9. Les tiroirs : le prototype, sur une page, à sa parole

Le prototype de la fiche 120 (un tiroir du côté de son bouton) **empilait quatre tiroirs dans une
colonne qui débordait de l'écran** en 1080p, et chaque tiroir gardait sa largeur (des bords
dentelés). Refait (`~/.cache/glucose-bancs/tiroirs-121/proto_cote.py`, appliqué dans un arbre
détaché, sa propre cible) : du côté de son bouton, **collé au bord**, plat (ni ombre ni coin),
**une colonne a la largeur de son tiroir le plus large**, et quand la hauteur manque, **une colonne
voisine** vers l'intérieur. Capturé dans le vrai Glucose hors écran : sept tiroirs ouverts, puis le
cas courant (Time Machine et Domaines — aujourd'hui **croisés** : le bouton de la Time Machine est
à gauche et elle s'ouvre à droite, celui de Domaines à droite et il s'ouvre à gauche).

La page, avec les trois choix de la fiche 120 à contester (`F11`, les icônes d'un pixel, aucune
correction de contraste) : <https://claude.ai/artifact/2JjKzEP2BTGDZLpYh5gK83>. **Rien n'est écrit
dans `main`** avant sa parole.

## 10. ECHELLE-1 — l'échelle de l'interface, à son choix

Les 42 lectures de l'échelle : toutes celles de l'interface lisaient `UiState::scale_factor`, celles
du matériel (les bords de la fenêtre en mode référence, la vitesse perçue d'un geste, le pincement)
`GlucoseApp::scale_factor` ; **le canevas ne lit ni l'une ni l'autre**. Le chemin le plus sûr en
découle : `UiState::scale_factor` **devient l'échelle de l'interface** — celle de l'écran fois son
choix —, et rien d'autre ne change. Tout ce qui dessine la chrome et tout ce qui y reçoit un clic la
lit déjà : ils restent d'accord par construction (l'épreuve de BARRE-2 les compare déjà de 100 à
200 %). Aucune des 36 lectures n'a eu à être réécrite ; les rassembler en une fonction était le plan
d'hier — inutile, mesuré sur le code.

* **Les crans** : ceux du zoom de Chrome, de 50 à 300 %. Comme la « Resolution Scale » de Blender
  (*« Adjusts the size of fonts and buttons relative to the automatically detected DPI »*), on
  multiplie l'échelle du système.
* **Où** : le menu du vide (« Agrandir l'interface », « Réduire l'interface »), à côté du plein écran,
  et la palette (avec « L'interface à 100 % »). **Pas de raccourci par défaut** : `Ctrl` `+` zoome le
  canevas dans Figma et Miro ; Blender n'en donne pas.
* **Honnête** : un cran qui ne changerait rien — au bout des crans, ou l'échelle bornée de
  l'interface (4) atteinte, un téléphone à 2,75 n'a pas de 150 % — ne s'offre pas.
* **Retenu** d'une session à l'autre (`echelle-interface`, à côté du mode de la fenêtre) ; un
  souvenir abîmé vaut 100 %, jamais une interface géante.

## 11. Les épreuves et les sabotages

* **La suite entière verte avant l'envoi de `aa72ab8c`** (3 385 épreuves, clippy strict ; fiche 122) —
  la convolution par tuiles, ECHELLE-1, DETOURER-1 à 3.
* **Les sabotages écrits, pas joués** : `outils/sabotages/detourage-121.json` (25, chacun filtré sur
  son épreuve) et `echelle-121.json` (5) — joués par la fiche 123.

## 12. Ce qui reste

* **Rien n'a été regardé dans le vrai Glucose** : ni l'échelle (« Agrandir l'interface » deux
  fois), ni le détourage hors du banc (`~/.cache/glucose-bancs/detourage-121/pilote/essai.sh`).
* **La 2.0.4-beta.3 en brouillon** porte `15beb94` : ni le détourage, ni l'échelle, ni le travail
  du 19/11 — à refaire avant son clic.
* **Les tiroirs** : la page attendait sa parole ; elle est venue le 19/11 (*« exactement comme
  proposé, et go l'allure plate »*), et TIROIR-4 est écrit par la fiche 122.

## 13. La leçon

* **Une page qu'il ne peut pas ouvrir ne reçoit aucune réponse** : sa fiche de questions (un doc
  Claude) ne s'ouvrait pas depuis cette session, et ses réponses n'ont pas été lues. Depuis le
  19/11, ses réponses vivent dans son Glucose, lues par sa porte (fiche 122).
