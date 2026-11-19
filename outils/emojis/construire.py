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
from picosvg.svg import SVG

# **La source** : les dessins de Twemoji 17 (jdecked/twemoji, sous CC-BY 4.0), l'archive du tag, dont
# l'empreinte a été vérifiée deux fois le 19/11/2026 (fiches 124 et 125).
VERSION = "17.0.3"
ADRESSE = f"https://github.com/jdecked/twemoji/archive/refs/tags/v{VERSION}.tar.gz"
EMPREINTE = "a0855654b633045ae2337537e77f1bb4361162f7fcd910e613eaab1d6d9c5fca"
SOURIRE = 0x1F600  # 😀 : la mesure commune aux deux polices
SELECTEUR = 0xFE0F  # le sélecteur « en émoji », que Twemoji garde dans certains noms


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
    attribuer(os.path.join(construction, nom), sortie)


def attribuer(construite, sortie):
    """**L'attribution que la CC-BY 4.0 demande**, dans la police même — qui voyage avec le
    programme — : qui a dessiné, sous quelle licence, ce qui a changé (`decisions/18`)."""
    police = TTFont(construite)
    noms = police["name"]
    for n, texte in (
        (0, "Twemoji graphics Copyright 2019-2026 Twitter, Inc and other contributors "
            "(github.com/jdecked/twemoji). Converted to COLRv0 for Glucose."),
        (13, "Twemoji graphics licensed under CC-BY 4.0; converted to a COLRv0 font by "
             "nanoemoji, scaled to the metrics of Noto Emoji, one glyph per character."),
        (14, "https://creativecommons.org/licenses/by/4.0/"),
    ):
        noms.setName(texte, n, 3, 1, 0x409)
    police.save(sortie)


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
    print(f"couleur : {len(cc)} caractères, {len(c.getGlyphOrder())} glyphes, "
          f"{sum(len(v) for v in couches.values())} couches, {len(palette)} couleurs")
    print(f"reste   : {len(rc)} caractères — " + " ".join(f"U+{cp:04X}" for cp in sorted(rc)))
    for chemin in (couleur, reste):
        with open(chemin, "rb") as f:
            octets = f.read()
        print(f"{os.path.basename(chemin)} : {len(octets)} octets, sha256 {hashlib.sha256(octets).hexdigest()}")
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
