# Glucose — Langage visuel (style guide)

> Référence courte et précise, à consulter avant tout travail d'UI/canvas.
> **Glucose est BRUTALISTE.**

## Intention

Glucose = une **feuille de papier** (noire). L'interface n'existe presque pas : elle
est **monochrome, plate, brutale**, pour que **le contenu de l'utilisateur soit tout**.
La couleur n'appartient PAS à l'UI — elle appartient à l'utilisateur, qui *colorie sa
feuille* (ses images, ses domaines, ses membranes). Mantra : *poser, relier, zoomer,
explorer — rien d'autre.* Pas de modes.

Registre : **brutalisme** — fonctionnel, sans décor, haute lisibilité, géométrie nette.
Pas de glassmorphisme, pas de glow décoratif, pas d'ombres molles sur la chrome.

## Loi de la couleur (la plus importante)

- **Chrome / UI = monochrome STRICT** : noir, blancs, gris. Boutons, panneaux, sélection,
  poignées, icônes → **aucune couleur**.
- **Le JAUNE** (`~#eab308`) : unique accent, employé **très parcimonieusement**, uniquement
  pour **souligner une chose importante à savoir/retenir** (jalons, alerte douce). Jamais
  décoratif, jamais sur un bouton « normal ».
- **La couleur vient du CONTENU**, jamais de l'app : les images ont leurs couleurs ; les
  **domaines** (assignés par l'utilisateur) teintent membranes/blocs ; la **teinte par
  position** (`getSymbioticHue`) colore les blocs de contenu (défaut effaçable). C'est
  l'utilisateur qui colorie ; l'app ne choisit jamais une couleur de chrome.

## Palette (tokens)

| Token | Valeur | Emploi |
|---|---|---|
| `canvas` | `#0d0d0d` | La « feuille » (fond) |
| `surface` / `raised` | `#111` · `#161616` · `#1a1a1a` | Panneaux, docks, boutons |
| `hairline` | `#1c1c1c`–`#2a2a2a` | Séparateurs, bordures 1px |
| `text` | `#e6e6e6` → `#fff` | Texte / marques |
| `text-muted` | `#6f6f6f`–`#7c7c7c` | Légendes, méta |
| `emphasis` (rare) | jaune `~#eab308` | Souligner une info importante — usage minimal |
| *(contenu)* | images + teintes domaine/position | **Couleur = utilisateur**, hors chrome |

> Dette connue : certains panneaux (IA locale / plugins) emploient encore un vert/bleu
> accent — **à resserrer** vers le monochrome.

## Matérialité

- **Plat & net** : surfaces unies, bordures *hairline* 1px, coins peu ou pas arrondis sur
  la chrome. **Pas de glow décoratif, pas d'ombre portée molle.**
- **Hairline** partout : traits fins, **épaisseur CONSTANTE à l'écran** (espace-écran : on
  divise par le `scale` du monde) — jamais d'épaississement au zoom.
- **Membrane** (contenu, *pas* chrome) : zones de l'utilisateur, remplissage très translucide
  (`fillOpacity 0.02–0.05`), bordure + label discrets ; une zone se lit par sa **bordure**,
  pas par son fond.

## Typographie

- **`ui-monospace`** pour labels techniques / dimensions ; **prose** pour le contenu ;
  **KaTeX** pour les maths. Gras = concept-clé, *italique* = nom propre. Pas de souligné.
- Échelles : 10–11px (méta, `uppercase letter-spacing 1.2`), 12–13px (UI), corps variable.

## Mouvement

- Transitions **courtes et amorties** (`cubic ease-out`, **180–400 ms**). **Rendu à la
  demande** (0 GPU au repos). Sobriété — le mouvement doit valoir son coût.

## Chrome interactive — sélection & poignées (spec)

- **Cadre de sélection** : *hairline* **blanc** (`alpha ≈ 0.8`), **largeur constante à
  l'écran**, léger débord (~3px).
- **Poignées de coin** : petit **carré blanc** (~9px) à **fin liseré quasi-noir** (`#111`)
  — « découpe papier ». Le blanc le rend lisible sur le canvas sombre, le liseré sur une
  image claire. **Aucune couleur, aucun glow.**
- **Taille CONSTANTE à l'écran à TOUT zoom** : redessiner les poignées au changement de
  zoom (sinon elles enflent en zoom-in / rétrécissent en zoom-out).
- Curseurs directionnels (`nwse-resize` / `nesw-resize`).

## Loi de redimensionnement

- **Défaut** → ancrage au **coin opposé** à la poignée tirée (le coin opposé reste fixe),
  comme un logiciel d'image.
- **Ctrl** → ancrage au **centre** (croît symétriquement).
- **Ratio toujours verrouillé** ; taille minimale plancher.

## Anti-patterns

- ❌ **Couleur sur la chrome** (bleu/vert/dégradés sur boutons, sélection, poignées).
- ❌ Glow décoratif, glassmorphisme, ombres portées molles sur l'UI.
- ❌ Poignées / traits qui **grossissent avec le zoom** (toujours espace-écran constant).
- ❌ L'app qui **choisit une couleur de contenu** à la place de l'utilisateur (sauf la
  teinte par position, défaut effaçable).
- ❌ Modes UI distincts. Une seule surface, un seul outil contextuel.
