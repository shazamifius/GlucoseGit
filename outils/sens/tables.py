#!/usr/bin/env python3
# **Les tables Unicode du découpeur** (fiche 106), tirées de la base Unicode de Python : les
# caractères qui perdent un accent (la décomposition canonique, sans ses marques), les marques
# seules, la ponctuation hors ASCII, les caractères de mise en forme. Écrit le fichier Rust du noyau.
#
#   python3 -I tables.py > crates/glucose-core/src/sens/unicode.rs
import unicodedata
def plages(predicat, fin=0x30000):
    out, debut = [], None
    for o in range(fin):
        p = predicat(chr(o))
        if p and debut is None: debut = o
        if not p and debut is not None: out.append((debut, o - 1)); debut = None
    if debut is not None: out.append((debut, fin - 1))
    return out
sans_accent = []
for o in range(0x30000):
    if 0xAC00 <= o <= 0xD7A3:
        continue  # les syllabes coréennes : leur décomposition se calcule (`decomposer_un_hangul`)
    c = chr(o)
    d = unicodedata.normalize('NFD', c)
    r = ''.join(x for x in d if unicodedata.category(x) != 'Mn')
    if d != c and r != c:
        sans_accent.append((o, r))
marques = plages(lambda c: unicodedata.category(c) == 'Mn')
ponctuation = plages(lambda c: ord(c) > 127 and unicodedata.category(c).startswith('P'))
forme = plages(lambda c: unicodedata.category(c) == 'Cf')
def lignes(elements, largeur=96):
    ligne = '   '
    for e in elements:
        if len(ligne) + 1 + len(e) > largeur:
            print(ligne); ligne = '   '
        ligne += ' ' + e
    if ligne.strip(): print(ligne)
def ecrire_plages(nom, doc, p):
    print(f'/// {doc}')
    print('#[rustfmt::skip]')
    print(f'pub(super) const {nom}: &[(u32, u32)] = &[')
    lignes([f'(0x{a:X}, 0x{b:X}),' for a, b in p])
    print('];\n')
print('//! **Les tables Unicode du découpeur** (fiche 106) — écrit par `outils/sens/tables.py`, depuis la')
print(f'//! base Unicode {unicodedata.unidata_version} de Python. Ne pas modifier à la main : relancer l\'outil. Les')
print('//! tables sont écrites serrées (`rustfmt::skip`) : une entrée par ligne en ferait des milliers.\n')
print('/// Les caractères qui perdent leurs marques à la décomposition canonique : le caractère, ce qui reste.')
print('#[rustfmt::skip]')
print('pub(super) const SANS_ACCENT: &[(u32, &str)] = &[')
lignes([f'(0x{o:X}, "' + r.replace(chr(92), chr(92) * 2).replace('"', chr(92) + '"') + '"),' for o, r in sans_accent])
print('];\n')
ecrire_plages('MARQUES', 'Les marques sans chasse (Mn) : un accent posé seul disparaît.', marques)
ecrire_plages('PONCTUATION', 'La ponctuation hors ASCII (P*) : un signe à part.', ponctuation)
ecrire_plages('MISE_EN_FORME', 'Les caractères de mise en forme (Cf) : effacés au nettoyage.', forme)
