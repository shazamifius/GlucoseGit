# Lit le programme compile d'un installeur ou desinstalleur NSIS 3 (Unicode, LZMA solide)
# et liste chaque ordre qui supprime : Delete (EW_DELETEFILE = 21) et RMDir (EW_RMDIR = 23),
# avec leurs drapeaux. Lecture seule : le binaire n'est jamais modifie (fiches 47 § 2, 48).
#
# Ecrit pour lire le desinstalleur de Glucose Tauri sur la machine de l'utilisateur : il ne vide
# jamais %LOCALAPPDATA%\Glucose. Garde aussi l'installeur de Glucose Rust : avec
# --exiger-aucune-recursive, la verification tombe si un RMDir /r y apparait un jour -- dans
# l'installeur comme dans le desinstalleur qu'il embarque.
#
# Usage : python lire_les_suppressions.py [--exiger-aucune-recursive] <exe>...
import lzma
import struct
import sys

SIG = struct.pack("<I", 0xDEADBEEF) + b"NullsoftInst"

# Variables internes de NSIS (index -> nom), dans l'ordre de fileform.h
VARS = [f"${i}" for i in range(10)] + [f"$R{i}" for i in range(10)] + [
    "$CMDLINE", "$INSTDIR", "$OUTDIR", "$EXEDIR", "$LANGUAGE", "$TEMP",
    "$PLUGINSDIR", "$EXEPATH", "$EXEFILE", "$HWNDPARENT", "$_CLICK", "$_OUTDIR",
]
CSIDL = {0x1A: "$APPDATA", 0x1C: "$LOCALAPPDATA", 0x10: "$DESKTOP", 0x02: "$SMPROGRAMS",
         0x0B: "$SMSTARTUP?", 0x17: "$SMPROGRAMS(all)", 0x19: "$DESKTOP(all)",
         0x05: "$DOCUMENTS", 0x26: "$PROGRAMFILES", 0x2B: "$COMMONFILES", 0x24: "$WINDIR",
         0x25: "$SYSDIR", 0x23: "$APPDATA(all)", 0x1D: "$ALTSTARTUP?"}


def trouver_en_tete(d):
    i = d.find(SIG)
    if i < 4:
        raise SystemExit("signature NSIS introuvable")
    debut = i - 4
    flags, sig, a, b, c, len_hdr, len_all = struct.unpack_from("<7i", d, debut)
    return debut + 28, len_hdr


def decompresser(d, pos):
    props = d[pos:pos + 5]
    lc_lp_pb = props[0]
    dict_size = struct.unpack_from("<I", props, 1)[0]
    lc = lc_lp_pb % 9
    rest = lc_lp_pb // 9
    lp = rest % 5
    pb = rest // 5
    filtres = [{"id": lzma.FILTER_LZMA1, "dict_size": dict_size, "lc": lc, "lp": lp, "pb": pb}]
    dec = lzma.LZMADecompressor(format=lzma.FORMAT_RAW, filters=filtres)
    return dec.decompress(d[pos + 5:], max_length=64 * 1024 * 1024)


def lire_chaine(strings, off):
    # Unicode : offset en caracteres UTF-16
    out = []
    i = off * 2
    while i + 1 < len(strings):
        w = struct.unpack_from("<H", strings, i)[0]
        i += 2
        if w == 0:
            break
        # NSIS 3 Unicode : 1 = langue, 2 = dossier du shell, 3 = variable, 4 = saut
        if w in (1, 2, 3, 4):
            arg = struct.unpack_from("<H", strings, i)[0]
            i += 2
            if w == 3:  # variable
                idx = (arg & 0x7F) | ((arg & 0x7F00) >> 1)
                out.append(VARS[idx] if idx < len(VARS) else f"$var{idx}")
            elif w == 2:  # dossier du shell
                lo, hi = arg & 0xFF, arg >> 8
                out.append(CSIDL.get(lo, f"$shell({lo:#x},{hi:#x})"))
            elif w == 1:  # chaine de langue
                idx = (arg & 0x7F) | ((arg & 0x7F00) >> 1)
                out.append(f"$(lang{idx})")
            else:  # saut
                out.append(chr(arg))
        else:
            out.append(chr(w))
    return "".join(out)


def lister(nom, hdr):
    blocs = [struct.unpack_from("<2i", hdr, 4 + 8 * k) for k in range(8)]
    off_entries, n_entries = blocs[2]
    off_strings = blocs[3][0]
    off_lang = blocs[4][0]
    strings = hdr[off_strings:off_lang]
    print(f"{nom}\n  en-tete {len(hdr)} octets, {n_entries} ordres")
    avant = None
    for k in range(n_entries):
        which, *p = struct.unpack_from("<7i", hdr, off_entries + 28 * k)
        if which == 21:
            # GetTempFileName (19) dans une variable, puis Delete simple (DEL_SIMPLE = 8) de cette
            # variable : NSIS cree un fichier temporaire et le remplace par le dossier de ses
            # greffons. (Le premier parametre est un numero de variable pour l'un, une chaine
            # pour l'autre.)
            cible = lire_chaine(strings, p[0])
            temporaire = (avant is not None and avant[0] == 19 and p[1] & 8
                          and avant[1] < len(VARS) and VARS[avant[1]] == cible)
            note = "  (le fichier temporaire que NSIS remplace par $PLUGINSDIR)" if temporaire else ""
            print(f"  [{k:5}] Delete  flags={p[1]:#x}  {cible}{note}")
        elif which == 23:
            recursif = "RECURSIF" if p[1] & 2 else "simple"
            print(f"  [{k:5}] RMDir   flags={p[1]:#x} {recursif:8}  {lire_chaine(strings, p[0])}")
            if p[1] & 2:
                RECURSIVES.append(f"{nom} : {lire_chaine(strings, p[0])}")
        avant = (which, p[0])


def en_tete(brut):
    taille = struct.unpack_from("<I", brut, 0)[0] & 0x7FFFFFFF
    return brut[4:4 + taille]


def main(chemin):
    d = open(chemin, "rb").read()
    pos, _ = trouver_en_tete(d)
    brut = decompresser(d, pos)
    lister(chemin, en_tete(brut))
    # Un installeur porte le desinstalleur qu'il ecrira : un second flux LZMA, range dans ses
    # donnees. On le cherche par ses proprietes (lc=3 lp=0 pb=2, 8 Mio), et on garde ce qui se
    # decompresse en un en-tete qui parle de desinstaller.
    vus = 0
    i = 0
    motif = b"\x5d\x00\x00\x80\x00"
    while True:
        i = brut.find(motif, i + 1)
        if i < 0:
            break
        try:
            sous = decompresser(brut, i)
            h = en_tete(sous)
            if len(h) > 1000 and b"u\x00n\x00i\x00n\x00s\x00t\x00a\x00l\x00l\x00" in h:
                vus += 1
                lister(f"{chemin} -> desinstalleur embarque (a {i})", h)
        except Exception:
            pass
    if vus == 0:
        print("  (aucun desinstalleur embarque trouve)")


# Les suppressions recursives rencontrees, pour --exiger-aucune-recursive.
RECURSIVES = []

if __name__ == "__main__":
    exiger = "--exiger-aucune-recursive" in sys.argv
    for c in [a for a in sys.argv[1:] if not a.startswith("--")]:
        main(c)
    if exiger and RECURSIVES:
        sys.exit("suppression recursive interdite : " + " ; ".join(RECURSIVES))
