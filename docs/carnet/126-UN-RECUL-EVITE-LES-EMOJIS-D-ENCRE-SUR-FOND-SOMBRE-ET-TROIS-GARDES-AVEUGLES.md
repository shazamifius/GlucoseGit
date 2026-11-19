# 126 — Un recul évité : les émojis d'encre sur un fond sombre ; trois gardes aveugles ; un outil de preuve qui nomme

> Session du 19/11/2026, depuis 17 h 27, après la fiche 125 (arrêtée à 16 h 20, à sa demande). La
> passation : lire la vérification de GitHub sur `78ed630` ; jouer les sabotages restants ; la
> beta.3 ; le compte Cloudflare et les ✔ foncés, à sa réponse.

## 1. En arrivant

* **GitHub sur `78ed630`** : neuf tâches sur dix vertes ; le Mac à puce Apple *« not acquired by
  Runner […] even after multiple attempts »* — relancé une fois, encore annulé après quinze minutes
  d'attente : la capacité de GitHub, pas notre code. Le flux de publication ne construit pas de Mac
  (Windows, Linux, Android) : il ne bloque pas la beta.3.
* **Son Glucose** (par sa porte, les deux onglets) : rien de neuf de sa part ; `text-19` (le compte
  Cloudflare) sans réponse. **La question des ✔ (`claude-code-coches`) était dans l'onglet
  « Textes optimisés », à x = 7 760**, quand tout son contenu tient entre 0 et 3 600 : ses
  coordonnées étaient celles de la place sous `text-19`, dans l'onglet principal. La porte pose
  dans l'onglet qu'il regarde quand on ne lui en nomme aucun — et il regardait celui-là. Il ne
  l'aurait sans doute jamais vue (§ 4).
* **La mémoire** : 3,6 Go disponibles, 1,8 Go de swap pris — il travaillait à côté.
* **La télémétrie** (lue à 18 h 30) : 208 sessions ; **son PC de l'école en `2.0.4-beta.3`**
  depuis l'après-midi — la beta.3 était publiée (§ 2).

## 2. Remis en question

* **« La 2.0.4-beta.3 est un brouillon vide, à retirer par `gh release delete` »** (la passation,
  ÉTAT, SUITE) : **faux**. Elle est **publiée depuis 16 h 20 — son clic**, l'instant même où la
  session d'avant s'arrêtait, qui ne l'a jamais su ; « Latest », son `latest.json` lu quatre fois,
  son PC de l'école mis à jour tout seul (deux sessions en beta.3). Elle porte le travail de
  `15beb94` : le texte net de l'interface, les trois modes de la fenêtre, la barre de Windows, `F2`.
  Le geste prévu aurait **supprimé une version publique, déjà installée**. Lu à temps par
  `gh release view --json isDraft`, avant tout geste. La suivante s'appelle donc **2.0.4-beta.4** :
  une machine en beta.3 ne se mettrait jamais à jour vers un autre programme du même numéro ; ses
  notes disent « depuis la 2.0.4-beta.3 » et ne répètent rien de ce qu'elle portait.
* **`saboter.py`, encore** : la fiche 125 l'avait fait exiger la preuve d'une épreuve tombée ;
  mais **le second essai, un fil à la fois** (pour les épreuves graphiques qui se sautent),
  comptait encore « TOMBE » pour n'importe quel code de sortie non nul — le mensonge même que la
  fiche 125 disait retiré. Et « TOMBE » ne disait pas *quelle* épreuve tombait : une épreuve
  fragile, étrangère au sabotage, aurait passé pour la garde qui tient. Corrigé (§ 3).
* **L'audit des anciens comptes, que la passation demandait** : jouer le témoin seul des anciens
  fichiers ne pouvait rien trouver — GitHub est vert, donc toute épreuve passe sur le code
  d'aujourd'hui. Le vrai défaut à chercher est une garde qui ne tombe plus : seul le rejeu des
  sabotages eux-mêmes le dit (§ 3, l'échantillon).
* **« Les ✔ foncés laissés fidèles, en attendant sa réponse »** (fiche 125 § 5) : faux, et pire
  qu'un défaut d'allure. Mesuré, le ✔ de Twemoji vaut **Lc 0** en APCA sur le papier d'un nœud —
  invisible —, et **34 dessins** sont dans ce cas, que sa version installée dessine en clair (Noto
  monochrome, à l'encre du texte). **La version suivante les aurait effacés** : un recul, sur les signes que
  les IA écrivent le plus (✔ ✖ ➕ ➖). Corrigé avant elle (§ 4).
* **Mon premier plan pour les ✔ était faux, deux fois** — trouvé en mesurant, avant d'écrire :
  * un seuil de pictogramme (Lc 45) pour tous aurait éclairci **115** dessins, dont ❤ ❌ 🔴 :
    APCA ne mesure que la clarté, et un rouge (Lc 31 ici) se voit par sa teinte. La loi juste ne
    touche que l'**invisible** (sous Lc 15) ;
  * une palette « fond sombre » posée partout aurait éclairci le ✔ d'un **post-it jaune**, où il
    se lit dans sa couleur d'origine. La palette doit suivre le papier de chaque texte.
* **La loi de la veille (fiche 124 § 7)** : juste sous Windows, **fausse ailleurs**. L'`Instant`
  de Rust lit `CLOCK_MONOTONIC` sous Linux et Android (et une horloge qui s'arrête pendant la
  veille sous macOS) : une veille n'y fait aucune image géante, et retrancher « BOOTTIME moins
  MONOTONIC » d'une image mesurée par `Instant` y donnerait une durée **négative**. La loi juste :
  mesurer les images à une horloge qui ne compte pas la veille — sous Windows,
  `QueryUnbiasedInterruptTimePrecise` ; ailleurs, `Instant` le fait déjà — et compter la veille à
  part, pour la télémétrie. À écrire ainsi (SUITE).
* **L'index du carnet et le `CLAUDE.md`** disaient tous deux « la prochaine fiche porte le numéro
  125 », quand la 125 existait : corrigés (127).

## 3. Les sabotages, et l'outil qui les juge

**`outils/saboter.py`** : « TOMBE » nomme l'épreuve tombée (libtest, `node --test`, un essai
« NON ») ; le second essai, un fil à la fois, exige la même preuve ; un témoin qui ne joue **aucune**
épreuve (un filtre qui ne trouve rien) arrête ses sabotages — chacun aurait coûté une compilation
pour dire « aveugle » d'une épreuve qui n'a pas tourné ; `--temoins <fichiers>…` joue le code intact
seul, chaque commande une fois.

**Les fichiers en attente**, joués avec témoin dans l'arbre des sabotages, deux fils :

| fichier | tombent | aveugles |
|---|---|---|
| `temoin-123` | 2 sur 2 | — |
| `table-123` | 10 sur 10 | — |
| `tiroirs-122` | 4 sur 6 | « first fit devient last fit » ; « une colonne de trop sort de l'écran » |
| `porte-122` | 3 sur 3 | — |
| `applique-122` | 2 sur 2 | — |
| `echelle-121` | 4 sur 5 | « un souvenir abîmé est cru » |
| `emoji-126` (§ 4) | 9 sur 9 | — |

**Les trois épreuves qui manquaient** :

* **TIROIR-4, le « first fit »** : aucune scène n'avait deux colonnes capables de prendre le même
  tiroir — ranger dans la dernière ou la première revenait au même. L'épreuve construit la scène
  sur les hauteurs mêmes des tiroirs de gauche : les deux plus hauts ne tiennent pas l'un sous
  l'autre, le plus bas tient sous chacun ; il va sous le premier.
* **TIROIR-4, rien ne sort de l'écran** : les sept tiroirs sur un écran de 640 × 360, et l'épreuve
  vérifie qu'elle exerce son cas (les colonnes bout à bout sont plus larges que l'écran).
* **ECHELLE-1, le souvenir abîmé** : l'épreuve existait, mais **aucune de ses entrées ne passait
  par la garde** — « 99999 » ne tient pas dans un `u16` et la lecture le refusait avant elle ;
  « abc », « », « 125.5 » ne sont pas des nombres ; « 0 » tombait sur `> 0`. « 60000 » et « 137 »,
  lisibles et faux, sont l'interface géante que la garde empêche : ajoutés.

**L'échantillon des anciens comptes** (ceux que l'outil d'avant la fiche 125 avait faits) : 24
sabotages tirés au hasard (graine 126) parmi les 1 183 encore applicables des fichiers d'avant la
fiche 121, dans 18 fichiers — **24 sur 24 tombent**, chacun sur une épreuve de son chantier, les
22 témoins tiennent. Aucun mensonge trouvé ; vingt-quatre tirages sans échec bornent la part des
comptes faux à 12 % environ (à 95 %) : les anciens comptes sont **probablement** sains.

**Le nombre de sabotages qui s'appliquent encore** : 1 380 sur 1 527 (90 %) visent un passage qui
existe une fois dans le code d'aujourd'hui ; les 147 autres gardaient du code réécrit depuis — ils
disent « INTROUVABLE », ils ne mentent pas, mais ils ne gardent plus rien.

## 4. EMOJI-3 — un émoji se pose dans la palette de son papier

La décision, ses sources et sa licence : `decisions/18`, la section « La palette du fond sombre ».

* **Le constat, mesuré** (APCA 0.0.98G-4g, la mesure de LIEN-BLEU-1 — vérifiée d'abord sur ses
  trois valeurs de la fiche 113 : 52, 75, 88) : sur `#0d0d0d`, `#31373D` vaut Lc 0. Les 35 dessins
  dont toutes les couleurs sont sous Lc 15 n'ont **chacun qu'une couleur** : le « noir » de Twemoji
  (`#31373D`, `#292F33`) pour 32 d'entre eux, le violet de 👾, le brun de 👣 🐾 🪾. Inter dessine
  lui-même ™ © ® ▪ ; dans la police, 🪮 a une seconde couleur qui se voit (Lc 37) : **34** changent.
* **Ce que font les autres** : Discord dessine avec Twemoji ; ses utilisateurs s'en plaignent
  depuis des années, sans réponse. Twemoji a refusé (*« chose a colour that tries to be visible
  across as many dark backgrounds as possible »*, *« wontfix »*, 2020). Apple et Slack : aucune
  source trouvée.
* **Le format le prévoit** : `CPAL` version 1 déclare chaque palette pour un fond clair, sombre,
  ou les deux ; WebKit la branche sur `font-palette: dark`. **La construction**
  (`outils/emojis/construire.py`) ajoute la palette du fond sombre : un émoji invisible monte d'un
  même pas de clarté (Oklch) jusqu'à celle de l'encre du corps, teinte gardée, chroma au bord de
  l'écran (la loi du noyau, `chroma_maximale`, recopiée) — dans **des entrées neuves**, car
  `#31373D` est aussi le détail sombre de 81 dessins qui se voient. Cinq couleurs neuves :
  `#31373D → #DAE2EA`, `#292F33 → #DAE2E7`, `#553986 → #E4DAFF`, `#662113 → #FFD6CC`,
  `#5E2618 → #FFD6CB` (Lc 0-11 → 87-88). 853 476 octets (+2 304). La vérification de l'outil relit
  la loi, et que ✔ change, 🔴 et ❤ non.
* **Le moteur** (`typography/trace/couleur.rs`) : une couche garde ses deux couleurs ; **l'encre
  du texte dit le papier** — une encre plus claire que le milieu de l'échelle perçue (clarté Oklab
  0,5) est posée sur un papier sombre. Les trois dessins passent tous par `Couche::sur` : aucun
  appelant n'a à dire son papier. La palette sombre est celle que la police **déclare** (le bit
  `USABLE_WITH_DARK_BACKGROUND`, lu dans l'en-tête de `CPAL` : `ttf-parser` lit les palettes, pas
  leurs types), la palette 0 sinon.
* **Les épreuves** : ✔ se lit sur le papier d'un nœud (Lc 45), et sa couleur d'origine y est
  invisible (la preuve à l'envers) ; sur un post-it, il garde `#31373D` et s'y lit ; **la loi sur
  toute la police, contre le thème** — ce qui se voit ne change pas d'un octet, ce qui ne se voit
  pas prend la clarté de l'encre, teinte gardée à moins d'une différence à peine visible : un thème
  qui change sans reconstruire la police fait tomber l'épreuve ; la palette déclarée est lue.
  **9 sabotages sur 9** (`outils/sabotages/emoji-126.json`), dont le papier et l'encre du thème.

### Regardé dans le vrai Glucose

Au pilote (sway sans écran), le programme de ce commit sur une copie de son document
(`~/.cache/glucose-bancs/pilote-126/`) : dans « Textes optimisés », *« ✔ Rien ne se chevauche, ✔
rien ne sort de l'écran »* — les ✔ en `#DAE2EA`, à côté du texte en `#DCE1EB` : ils se lisent comme
la phrase ; 🟢 🔵 📜 ✍️ 🔗 💬 dans leurs couleurs. Un post-it posé et tapé au pilote, *« ✔ fait ➕ 👣
🔴 »* : ✔ et ➕ en `#31373D` (mesuré au pixel), 👣 brun, 🔴 rouge, sur le jaune.

## 4 bis. Le détourage, regardé

« Jamais regardé dans le vrai Glucose hors du banc » (ÉTAT, SUITE) : faux — l'essai de la fiche 121
avait tourné le 18/11 sur la boule (`~/.cache/glucose-bancs/detourage-121/pilote/boule/`). Rejoué le
19/11 sur le programme de ce jour, avec une vraie photo (une pomme sur fond blanc, 1 280 × 1 160,
`~/.cache/glucose-bancs/pilote-126/detourage-pomme/`) : la pomme sort entière, sa queue comprise,
son ombre portée sur le fond partie ; `Ctrl+Z` puis `Ctrl+Y` la rendent et la reprennent.

**Ce que l'image montre, et que personne n'avait noté** : un **liseré clair** au bord — le long d'une
rangée du bord droit, du rouge de la pomme (109, 30, 35) à un rose grisé (152, 109, 111 ; 145, 111,
119) avant le noir. Un bord seulement transparent serait plus sombre que la pomme sur le noir : plus
clair, il garde du blanc du fond, sur huit à dix pixels de l'image. Et sur la boule, de fines traces
des rayures du fond restent au-dessus d'elle. À mesurer (l'erreur du premier plan sur les images de
vérité du banc) puis corriger — SUITE, le détourage.

## 5. VEILLE-1 — une image qui contient une veille garde ce qu'elle a coûté

`plateforme/veille.rs`. **Le temps endormi que l'horloge des images compte** : sous Windows,
`QueryInterruptTimePrecise` (qui compte la veille) moins `QueryUnbiasedInterruptTimePrecise` (qui
ne la compte pas), en unités de 100 ns — une fonctionnalité de plus de la caisse `windows` déjà là
(`Win32_System_WindowsProgramming`), aucune caisse neuve ; ailleurs, **zéro par construction**. La
boucle le lit au début et à la fin de l'image (`app.rs`), et `clore_l_image` retranche ce qui a
grandi entre les deux, comme elle retranche déjà l'attente du tempo. Aucune ligne neuve ne part
au serveur : rien à déployer avant.

* **Épreuves** : une image de trois secondes dont 2,99 endormies ne coûte pas trois secondes à la
  chronique (par le vrai `clore_l_image`) ; la preuve à l'envers, sans veille dite, en vaut trois ;
  le temps endormi ne recule pas, et vaut zéro hors de Windows. **Pas éprouvé ici** : la lecture de
  Windows (seule la CI la compile) ; `depuis` et le branchement dans `redraw` ne se gardent pas sous
  Linux — le temps endormi y est nul, et `redraw` demande une fenêtre.
* **Pas encore envoyé** au moment du brouillon de la beta.4 : il partira dans la version d'après,
  vérifié par GitHub (§ 8).

## 6. La recherche (consultée le 19/11/2026)

* Les émojis sombres : [`twitter/twemoji#420`](https://github.com/twitter/twemoji/issues/420) ;
  [Discord, *The "heavy" emojis are almost non-visible*](https://support.discord.com/hc/nl/community/posts/360052083493-The-heavy-emojis-are-almost-non-visible),
  [*Make the "™" Symbol (and others) Lighter when Using Dark Mode*](https://support.discord.com/hc/en-us/community/posts/360042680611-Make-the-Symbol-and-others-Lighter-when-Using-Dark-Mode) ;
  [OpenType, `CPAL`](https://learn.microsoft.com/en-us/typography/opentype/spec/cpal) ;
  [WebKit, *Customizing Color Fonts on the Web*](https://www.webkit.org/?p=12662) ;
  [APCA in a Nutshell](https://git.apcacontrast.com/documentation/APCA_in_a_Nutshell.html) (Lc 15
  *« treat as invisible »*, Lc 45 *« pictograms with fine details »*) ;
  [ARC, le mode bronze](https://a11yreadtech.github.io/ARC/tests/bronze-simple-mode) (ne couvre que
  le texte).
* Les tiroirs qui défilent : Blender — [*the panel tabs should never shrink, scroll
  instead*](https://devtalk.blender.org/t/the-panel-tabs-for-addons-should-never-shrink-scroll-instead/10196?page=5),
  [la molette sur les onglets de la barre latérale](https://projects.blender.org/blender/blender/issues/110503)
  (la molette défile, `Ctrl`+molette change d'onglet ; défiler seulement quand la hauteur manque).
* La veille : [Fuchsia, RFC-0259](https://fuchsia.dev/docs/contribute/governance/rfcs/0259_monotonic_clock_suspension_and_the_boot_timeline)
  (`uptimeMillis` est la ligne monotone, `elapsedRealtime` celle du démarrage) ;
  [l'horloge du démarrage d'Android en Rust](https://git.aicp-rom.com/packages_modules_DnsResolver.git/tree/doh/boot_time.rs)
  (un type à part : l'`Instant` de Rust ne compte pas la veille) ; [Linux, l'unification de
  MONOTONIC et BOOTTIME](https://lore.kernel.org/lkml/20180425065015.GA6222@amd/T/) (essayée pour
  4.17, puis retirée).
* La « server card » MCP : [SEP-2127](https://modelcontextprotocol.io/seps/2127-mcp-server-cards.md)
  — passée dans un dépôt *experimental* ; les sources donnent au moins cinq chemins différents
  (`/.well-known/mcp/server-card.json`, `<point MCP>/server-card`…). **Pas mûre** : attendre que
  le chemin se fixe.

## 7. Les leçons

* **Mesurer un défaut d'allure avant de le dire d'allure** : « gris foncé, peu lisible » était
  « Lc 0, invisible », et la version suivante l'aurait montré à tous. Une couleur se juge sur le
  papier où elle se pose, par la mesure que le dépôt emploie déjà.
* **Un seuil de lisibilité qui ne mesure que la clarté ne juge pas les couleurs** : 115 dessins au
  lieu de 34, les rouges compris. La loi juste touchait l'invisible, pas le moins lisible.
* **Une règle posée « partout » se vérifie sur chaque papier** : le post-it jaune démentait la
  palette constante, avant qu'une ligne du moteur soit écrite — en relisant les épreuves, qui
  dessinent sur du blanc.
* **Une épreuve dont aucune entrée ne passe par la garde est aveugle, quoi que dise son nom** :
  « un souvenir abîmé vaut cent » ne donnait que des souvenirs que la lecture refusait avant la
  garde.
* **Un outil de preuve se relit après chaque correction** : celui de la fiche 125 gardait le défaut
  qu'elle avait corrigé, dans sa seconde branche.
* **Ce que la porte pose sans onglet nommé va dans celui qu'il regarde** : nommer l'onglet, toujours.
* **Un état publié se lit sur GitHub, pas dans nos documents** : la beta.3 « brouillon » était
  publique depuis une heure. Avant tout geste sur une version : `gh release view --json isDraft` ;
  et la télémétrie, lue en début de session comme le `CLAUDE.md` le demande, l'aurait dit plus tôt.
* **Un tirage au hasard juge un compte que personne ne peut rejouer en entier** : 24 sabotages,
  une vingtaine de minutes, une borne chiffrée — plutôt que de croire, ou de tout rejouer.
