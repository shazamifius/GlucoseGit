"""**Construire les polices d'émojis de Glucose** (EMOJI-2, fiche 125, `decisions/18`).

    python3 -I construire.py <travail> <noto> <couleur.ttf> <reste.ttf>

* `<travail>` : un dossier à soi (la source téléchargée, la construction de nanoemoji) ;
* `<noto>` : `outils/emojis/NotoEmoji-Regular.ttf`, la police monochrome d'avant (EMOJI-1, `decisions/10`) —
  ses métriques sont la mesure, et ce qu'elle couvrait doit rester couvert ;
* `<couleur.ttf>` : l'écrite, Twemoji 17 en **COLRv0** — chaque émoji, une pile de glyphes ordinaires
  chacun d'une couleur de la palette ;
* `<reste.ttf>` : l'écrite, **ce que Twemoji n'a pas**, tiré de Noto (le cadre de touche U+20E3, les
  joints et sélecteurs invisibles) — monochrome, à l'encre du texte, comme aujourd'hui.

Rien n'est choisi à la main : la source est épinglée par son empreinte ; l'em, l'avance, la montante
et la descendante sont celles de Noto ; l'échelle et le centre sont ceux du 😀 de Noto, pour qu'un
texte déjà posé ne se remette pas en page et qu'un émoji garde sa taille. La fin vérifie que chaque
caractère que Noto dessinait (hors zone d'usage privé) reste dessiné, à la même avance.
"""
import hashlib
import math
import os
import re
import shutil
import subprocess
import sys
import tarfile
import urllib.request

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.ttLib.tables.C_P_A_L_ import Color
from picosvg.svg import SVG

# **La source** : les dessins de Twemoji 17 (jdecked/twemoji, sous CC-BY 4.0), l'archive du tag, dont
# l'empreinte a été vérifiée deux fois le 19/11/2026 (fiches 124 et 125).
VERSION = "17.0.3"
ADRESSE = f"https://github.com/jdecked/twemoji/archive/refs/tags/v{VERSION}.tar.gz"
EMPREINTE = "a0855654b633045ae2337537e77f1bb4361162f7fcd910e613eaab1d6d9c5fca"
SOURIRE = 0x1F600  # 😀 : la mesure commune aux deux polices
SELECTEUR = 0xFE0F  # le sélecteur « en émoji », que Twemoji garde dans certains noms

# **Le papier et l'encre d'un texte de Glucose** (`theme.rs` : `card_paper`, `card_body`) : la palette
# du fond sombre se construit pour eux (EMOJI-3, fiche 126), et l'épreuve EMOJI-3 du bureau vérifie
# que la police porte la loi pour ceux du thème — un thème qui change fait tomber l'épreuve.
PAPIER, ENCRE = (0x0D, 0x0D, 0x0D), (0xDC, 0xE1, 0xEB)
# APCA (0.0.98G-4g, la mesure de LIEN-BLEU-1) : sous Lc 15, « designers should treat anything below
# this level as invisible » ; Lc 45, « the minimum for pictograms with fine details » (APCA in a
# Nutshell). Deux niveaux publiés, pas choisis.
INVISIBLE, PICTOGRAMME = 15, 45
CLAIRE, SOMBRE = 0x0001, 0x0002  # CPAL 1 : USABLE_WITH_LIGHT_BACKGROUND, USABLE_WITH_DARK_BACKGROUND
ENCRE_DU_TEXTE = 0xFFFF  # COLR : la couche « de la couleur du texte »


def apca(texte, fond):
    """Le contraste APCA (0.0.98G-4g) d'une couleur sur un fond, en octets sRGB ; son signe dit le
    sens (négatif : clair sur sombre), sa grandeur la lisibilité."""
    def y(c):
        r, g, b = (v / 255 for v in c)
        y = 0.2126729 * r ** 2.4 + 0.7151522 * g ** 2.4 + 0.0721750 * b ** 2.4
        return y + (0.022 - y) ** 1.414 if y < 0.022 else y
    yt, yf = y(texte), y(fond)
    if abs(yf - yt) < 0.0005:
        return 0.0
    if yf > yt:
        s = (yf ** 0.56 - yt ** 0.57) * 1.14
        return 0.0 if s < 0.1 else (s - 0.027) * 100
    s = (yf ** 0.65 - yt ** 0.62) * 1.14
    return 0.0 if s > -0.1 else (s + 0.027) * 100


def _lineaire(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def _gamma(c):
    return 12.92 * c if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055


def oklch(c):
    """Une couleur en octets sRGB, en Oklch — les matrices d'Ottosson, celles du noyau
    (`glucose-core/src/couleur.rs`)."""
    r, g, b = (_lineaire(v / 255) for v in c)
    l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b) ** (1 / 3)
    m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b) ** (1 / 3)
    s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b) ** (1 / 3)
    L = 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s
    a = 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s
    bb = 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s
    return L, math.hypot(a, bb), math.atan2(bb, a)


def srgb(L, C, h):
    """Le retour d'Oklch en sRGB (`0..=1` par canal, hors de l'écran si un canal en sort)."""
    a, b = C * math.cos(h), C * math.sin(h)
    l_ = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3
    m_ = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3
    s_ = (L - 0.0894841775 * a - 1.2914855480 * b) ** 3
    return [_gamma(v) for v in (4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_,
                                -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_,
                                -0.0041960863 * l_ - 0.7034186147 * m_ + 1.7076147010 * s_)]


def chroma_maximale(L, h):
    """La plus forte chroma que l'écran montre à cette clarté et cette teinte — la loi du noyau
    (`Oklch::chroma_maximale`) : depuis le dehors, par pas d'un dixième de la différence à peine
    visible (0,02), puis cinquante bissections ; un canal à moins d'un demi-niveau reste à l'écran."""
    demi = 0.5 / 255
    def dedans(C):
        return all(-demi <= v <= 1 + demi for v in srgb(L, C, h))
    if not dedans(0.0):
        return 0.0
    pas = 0.002
    # La plus forte chroma de tout l'écran : celle des six couleurs pures du cube.
    purs = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (0, 255, 255), (255, 0, 255), (255, 255, 0)]
    dehors = max(oklch(c)[1] for c in purs) + pas
    ici = dehors - pas
    while ici > 0 and not dedans(ici):
        dehors, ici = ici, ici - pas
    ici = max(ici, 0.0)
    for _ in range(50):
        milieu = (ici + dehors) / 2
        ici, dehors = (milieu, dehors) if dedans(milieu) else (ici, milieu)
    return ici


def usage_prive(cp):
    """Un point de code d'usage privé ne dit rien d'une police à l'autre : ni gardé, ni compté."""
    return 0xE000 <= cp <= 0xF8FF or cp >= 0xF0000


def la_source(travail):
    """L'archive, téléchargée une fois, vérifiée à chaque fois ; ses dessins et leur licence."""
    archive = os.path.join(travail, f"twemoji-{VERSION}.tar.gz")
    if not os.path.exists(archive):
        urllib.request.urlretrieve(ADRESSE, archive)
    with open(archive, "rb") as f:
        lue = hashlib.sha256(f.read()).hexdigest()
    if lue != EMPREINTE:
        sys.exit(f"l'archive n'est pas celle qu'on attend : {lue}")
    racine = os.path.join(travail, f"twemoji-{VERSION}")
    with tarfile.open(archive) as t:
        # **Reproductible au bit près** : l'heure qu'une police porte (`head`) est celle de la
        # source, la plus récente de l'archive — pas celle de la construction. fontTools et
        # nanoemoji lisent `SOURCE_DATE_EPOCH` (reproducible-builds.org).
        os.environ["SOURCE_DATE_EPOCH"] = str(max(m.mtime for m in t.getmembers()))
        if not os.path.isdir(racine):
            t.extractall(travail, filter="data")
    return os.path.join(racine, "assets", "svg"), os.path.join(racine, "LICENSE-GRAPHICS")


def les_dessins_seuls(svg, travail):
    """**Un émoji par caractère** : Glucose dessine caractère par caractère, sans moteur de mise en
    forme (`decisions/10`) — une séquence (famille, drapeau, ton de peau) ne serait jamais appelée."""
    choisis = os.path.join(travail, "choisis")
    os.makedirs(choisis, exist_ok=True)
    pris = {}
    for nom in sorted(os.listdir(svg)):
        parts = [int(p, 16) for p in nom[:-4].split("-") if int(p, 16) != SELECTEUR]
        if len(parts) == 1 and not usage_prive(parts[0]):
            cible = os.path.join(choisis, f"{parts[0]:x}.svg")
            with open(os.path.join(svg, nom), encoding="utf-8") as f:
                dessin = au_carre(f.read())
            # Réécrit seulement s'il change : ninja garde alors ce qu'il a déjà fait.
            if not os.path.exists(cible) or open(cible, encoding="utf-8").read() != dessin:
                with open(cible, "w", encoding="utf-8") as f:
                    f.write(dessin)
            pris[parts[0]] = cible
    for nom in os.listdir(choisis):
        if os.path.join(choisis, nom) not in pris.values():
            os.remove(os.path.join(choisis, nom))
    return pris


def au_carre(dessin):
    """**Un dessin tient dans le carré de l'em**, centré : nanoemoji cale la hauteur de la vue sur
    la montante et la descendante, et une vue plus large que haute — 🍉, 36 × 25,22 — débordait
    l'avance (3 426 au lieu de 2 600) et aurait remis en page chaque texte qui le porte. La vue
    s'agrandit sur son petit côté, autour de son centre : le dessin garde l'échelle des autres."""
    m = re.search(r'viewBox="([^"]+)"', dessin)
    x, y, l, h = (float(v) for v in m.group(1).replace(",", " ").split())
    if l == h:
        return dessin
    c = max(l, h)
    vue = f'viewBox="{x - (c - l) / 2:g} {y - (c - h) / 2:g} {c:g} {c:g}"'
    return dessin[:m.start()] + vue + dessin[m.end():]


def la_mesure(noto, sourire_svg):
    """Les métriques de Noto, et la transformation qui pose le 😀 de Twemoji sur celui de Noto.

    nanoemoji pose la vue d'un dessin sur la hauteur montante − descendante, centrée dans l'avance,
    l'axe des y retourné (`color_glyph.map_viewbox_to_font_space`) ; on y compose une homothétie de
    centre celui du 😀 de Noto, de rapport celui des deux boîtes (à surface égale).
    """
    f = TTFont(noto)
    em, montante, descendante = f["head"].unitsPerEm, f["hhea"].ascent, f["hhea"].descent
    g = f.getBestCmap()[SOURIRE]
    avance = f["hmtx"][g][0]
    glyphe = f["glyf"][g]
    glyphe.recalcBounds(f["glyf"])
    n = (glyphe.xMin, glyphe.yMin, glyphe.xMax, glyphe.yMax)

    dessin = SVG.parse(sourire_svg)
    vue, boite = dessin.view_box(), dessin.bounding_box()
    echelle = (montante - descendante) / vue.h
    dx = (avance - echelle * vue.w) / 2
    # La boîte du 😀 de Twemoji, telle que nanoemoji la pose sans transformation.
    tx0, tx1 = dx + echelle * (boite.x - vue.x), dx + echelle * (boite.x + boite.w - vue.x)
    ty0, ty1 = montante - echelle * (boite.y + boite.h - vue.y), montante - echelle * (boite.y - vue.y)
    k = math.sqrt((n[2] - n[0]) * (n[3] - n[1])) / math.sqrt((tx1 - tx0) * (ty1 - ty0))
    cx, cy = (n[0] + n[2]) / 2, (n[1] + n[3]) / 2
    ux, uy = (tx0 + tx1) / 2, (ty0 + ty1) / 2
    transformation = f"matrix({k:.6f} 0 0 {k:.6f} {cx - k * ux:.3f} {cy - k * uy:.3f})"
    return (em, avance, montante, descendante), transformation, n


def la_palette_du_fond_sombre(police):
    """**EMOJI-3 — un émoji qu'on ne voit pas sur un fond sombre s'y éclaircit** (fiche 126).

    Twemoji dessine son « noir » d'un gris très sombre (`#31373D`, `#292F33`), pensé pour un fond
    clair : sur le papier d'un texte de Glucose, presque noir, ✔ ✖ ➕ ♠ ⚫ 🖤 🕷️ valent Lc 0 — APCA
    les dit invisibles. Le format le prévoit : une police peut porter une seconde palette, qu'elle
    déclare faite pour un fond sombre (OpenType, `CPAL` version 1, `USABLE_WITH_DARK_BACKGROUND`).

    * **Qui change** : un émoji dont **aucune** couleur ne se voit sur le papier (sous Lc 15). Une
      seule qui se voit suffit : le ❤ de Twemoji vaut Lc 31 et se lit — APCA ne mesure que la
      clarté, et un rouge se voit aussi par sa teinte. Ce qui se voit ne bouge pas.
    * **Comment** : toutes ses couleurs montent d'un même pas de clarté (Oklch), le plus petit qui
      pose la plus claire à la clarté de l'encre du texte ; la teinte reste, la chroma ne descend
      qu'au bord de l'écran si sRGB ne la tient plus. Un même pas garde les écarts entre les couches.
    * **Ses propres entrées** : une entrée de palette sert à tous les émojis de sa couleur —
      `#31373D` est le détail sombre de 81 dessins (un chapeau, une boule de billard) qui, eux, se
      voient. Un émoji qui change reçoit donc des entrées neuves, de sa couleur d'origine dans la
      palette 0 : rien ne bouge pour qui ne lit que la palette 0.

    Rend les glyphes de base qui ont changé.
    """
    cpal, couches = police["CPAL"], police["COLR"].ColorLayers
    claire = list(cpal.palettes[0])
    sombre = list(claire)

    def rvb(c):
        return c.red, c.green, c.blue

    neuves, touches = {}, []
    for base in sorted(couches):
        couleurs = [claire[l.colorID] for l in couches[base] if l.colorID != ENCRE_DU_TEXTE]
        if not couleurs or any(abs(apca(rvb(c), PAPIER)) >= INVISIBLE for c in couleurs):
            continue
        touches.append(base)
        pas = oklch(ENCRE)[0] - max(oklch(rvb(c))[0] for c in couleurs)
        for couche in couches[base]:
            if couche.colorID == ENCRE_DU_TEXTE:
                continue
            origine = claire[couche.colorID]
            L, C, h = oklch(rvb(origine))
            L = min(L + pas, 1.0)
            r, g, b = (round(min(max(v, 0.0), 1.0) * 255)
                       for v in srgb(L, min(C, chroma_maximale(L, h)), h))
            eclaircie = Color(blue=b, green=g, red=r, alpha=origine.alpha)
            cle = (tuple(origine), tuple(eclaircie))
            if cle not in neuves:
                neuves[cle] = len(claire)
                claire.append(origine)
                sombre.append(eclaircie)
            couche.colorID = neuves[cle]
    cpal.version = 1
    cpal.palettes = [claire, sombre]
    cpal.numPaletteEntries = len(claire)
    cpal.paletteTypes = [CLAIRE, SOMBRE]
    cpal.paletteLabels = [cpal.NO_NAME_ID] * 2
    cpal.paletteEntryLabels = [cpal.NO_NAME_ID] * len(claire)
    return touches


def construire_la_couleur(pris, travail, metriques, transformation, sortie):
    """nanoemoji écrit son plan ; ninja l'exécute à deux tâches, pour ne pas étouffer la machine
    (la règle des compilations bornées, `outils/borne.sh`)."""
    em, avance, montante, descendante = metriques
    construction = os.path.join(travail, "construction")
    os.makedirs(construction, exist_ok=True)
    nom = os.path.basename(sortie)
    subprocess.run(
        ["nanoemoji", "--color_format", "glyf_colr_0", "--family", "Glucose Emoji",
         "--upem", str(em), "--width", str(avance), "--ascender", str(montante),
         "--descender", str(descendante), "--transform", transformation,
         "--output_file", nom, "--build_dir", construction, "--noexec_ninja",
         *sorted(pris.values())],
        check=True, cwd=travail)
    # La police s'écrit à chaque fois (son heure est celle de la source, que ninja ne voit pas).
    if os.path.exists(os.path.join(construction, nom)):
        os.remove(os.path.join(construction, nom))
    subprocess.run(["ninja", "-j", "2", "-C", construction], check=True, cwd=travail)
    police = TTFont(os.path.join(construction, nom))
    touches = la_palette_du_fond_sombre(police)
    attribuer(police, len(touches))
    police.save(sortie)


def attribuer(police, eclaircis):
    """**L'attribution que la CC-BY 4.0 demande**, dans la police même — qui voyage avec le
    programme — : qui a dessiné, sous quelle licence, ce qui a changé (`decisions/18`)."""
    noms = police["name"]
    for n, texte in (
        (0, "Twemoji graphics Copyright 2019-2026 Twitter, Inc and other contributors "
            "(github.com/jdecked/twemoji). Converted to COLRv0 for Glucose."),
        (13, "Twemoji graphics licensed under CC-BY 4.0; converted to a COLRv0 font by "
             "nanoemoji, scaled to the metrics of Noto Emoji, one glyph per character; "
             f"a second palette, for dark backgrounds, lightens the {eclaircis} glyphs "
             "that no colour would show there."),
        (14, "https://creativecommons.org/licenses/by/4.0/"),
    ):
        noms.setName(texte, n, 3, 1, 0x409)


def construire_le_reste(noto, couverts, sortie):
    """Ce que Noto dessinait et que Twemoji n'a pas, à l'identique (sa licence, OFL, la permet)."""
    restants = sorted(cp for cp in TTFont(noto).getBestCmap()
                      if cp not in couverts and not usage_prive(cp))
    options = subset.Options()
    options.layout_features = []
    options.name_IDs = ["*"]
    options.notdef_outline = True
    police = TTFont(noto, recalcTimestamp=False)
    sous = subset.Subsetter(options)
    sous.populate(unicodes=restants)
    sous.subset(police)
    police.save(sortie)
    return restants


def verifier(noto, couleur, reste):
    """**Rien de perdu** : chaque caractère de Noto (hors usage privé) est dans l'une des deux, à la
    même avance ; et 🟢 et 🔴 sont deux couleurs, la preuve que le chantier sert."""
    n, c, r = (TTFont(p) for p in (noto, couleur, reste))
    nc, cc, rc = n.getBestCmap(), c.getBestCmap(), r.getBestCmap()
    fautes = []
    for cp, g in nc.items():
        if usage_prive(cp):
            continue
        ou = (c, cc) if cp in cc else (r, rc) if cp in rc else None
        if ou is None:
            fautes.append(f"U+{cp:04X} perdu")
        elif ou[0]["hmtx"][ou[1][cp]][0] != n["hmtx"][g][0]:
            fautes.append(f"U+{cp:04X} avance {ou[0]['hmtx'][ou[1][cp]][0]} au lieu de {n['hmtx'][g][0]}")
    couches = c["COLR"].ColorLayers
    palette = c["CPAL"].palettes[0]

    def teinte(cp):
        return {tuple(palette[l.colorID])[:3] for l in couches[cc[cp]]}

    if teinte(0x1F7E2) == teinte(0x1F534):
        fautes.append("🟢 et 🔴 ont la même couleur")
    fautes += verifier_le_fond_sombre(c, cc)
    print(f"couleur : {len(cc)} caractères, {len(c.getGlyphOrder())} glyphes, "
          f"{sum(len(v) for v in couches.values())} couches, {len(palette)} couleurs")
    print(f"reste   : {len(rc)} caractères — " + " ".join(f"U+{cp:04X}" for cp in sorted(rc)))
    for chemin in (couleur, reste):
        with open(chemin, "rb") as f:
            octets = f.read()
        print(f"{os.path.basename(chemin)} : {len(octets)} octets, sha256 {hashlib.sha256(octets).hexdigest()}")
    return fautes


def verifier_le_fond_sombre(police, cmap):
    """**EMOJI-3, relu** : deux palettes, la claire puis la sombre, déclarées telles ; un émoji
    invisible sur le papier dans la première se lit dans la seconde comme un pictogramme (Lc 45) ;
    tout autre émoji y a exactement ses couleurs ; ✔ a changé, 🔴 et ❤ non."""
    cpal, couches = police["CPAL"], police["COLR"].ColorLayers
    if len(cpal.palettes) != 2 or list(getattr(cpal, "paletteTypes", [])) != [CLAIRE, SOMBRE]:
        return ["CPAL : il faut la palette claire puis la sombre, déclarées telles"]
    claire, sombre = cpal.palettes

    def rvb(c):
        return c.red, c.green, c.blue

    fautes, eclaircis = [], set()
    for base, pile in couches.items():
        ids = [l.colorID for l in pile if l.colorID != ENCRE_DU_TEXTE]
        if ids and all(abs(apca(rvb(claire[i]), PAPIER)) < INVISIBLE for i in ids):
            eclaircis.add(base)
            if max(abs(apca(rvb(sombre[i]), PAPIER)) for i in ids) < PICTOGRAMME:
                fautes.append(f"{base} reste illisible sur le fond sombre")
        elif any(sombre[i] != claire[i] for i in ids):
            fautes.append(f"{base} se voyait, et sa couleur a changé")
    for cp, attendu in ((0x2714, True), (0x1F534, False), (0x2764, False)):
        if (cmap[cp] in eclaircis) != attendu:
            fautes.append(f"U+{cp:04X} : {'devait' if attendu else 'ne devait pas'} s'éclaircir")
    print(f"fond sombre : {len(eclaircis)} émojis éclaircis — "
          + " ".join(f"U+{cp:04X}" for cp in sorted(cp for cp, g in cmap.items() if g in eclaircis)))
    return fautes


def main():
    travail, noto, couleur, reste = (os.path.abspath(a) for a in sys.argv[1:5])
    os.makedirs(travail, exist_ok=True)
    svg, licence = la_source(travail)
    pris = les_dessins_seuls(svg, travail)
    metriques, transformation, boite = la_mesure(noto, pris[SOURIRE])
    print(f"{len(pris)} dessins seuls ; em {metriques[0]}, avance {metriques[1]}, "
          f"montante {metriques[2]}, descendante {metriques[3]} ; 😀 de Noto {boite} ; {transformation}")
    construire_la_couleur(pris, travail, metriques, transformation, couleur)
    construire_le_reste(noto, set(pris), reste)
    shutil.copyfile(licence, os.path.join(os.path.dirname(couleur), "LICENSE-Twemoji-graphics.txt"))
    fautes = verifier(noto, couleur, reste)
    for faute in fautes:
        print("FAUTE :", faute)
    sys.exit(1 if fautes else 0)


main()
