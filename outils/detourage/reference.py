"""L'étalon du moteur de détourage (DETOURER-1, fiche 121) : lit le fichier que `convertir.py`
écrit et le fait tourner en numpy, opération par opération, à la lettre du format.

Usage : python3 -I outils/detourage/reference.py <modèle.bin> <image> <masque.png> [modèle.onnx]
        python3 -I outils/detourage/reference.py etalon <modèle.bin> <sortie>   (l'étalon des épreuves)
        Avec le modèle ONNX d'origine, compare au masque d'ONNX Runtime (l'écart doit être nul à
        l'arrondi des flottants près) : c'est la preuve que la conversion n'a rien changé.

Le prétraitement est celui des auteurs d'U²-Net (`ToTensorLab`) : l'image divisée par sa plus
grande valeur, puis centrée et réduite ; le masque, étiré de son minimum à son maximum (`normPRED`).
"""
import struct
import sys

import numpy as np
from PIL import Image


def lire(chemin):
    d = open(chemin, "rb").read()
    assert d[:8] == b"GLUCDET1"
    taille, ntens, nops, sortie = struct.unpack_from("<4I", d, 8)
    norm = struct.unpack_from("<6f", d, 24)
    p, ops = 48, []
    for _ in range(nops):
        sorte, relu, nent, out = struct.unpack_from("<BBHI", d, p)
        p += 8
        ent = list(struct.unpack_from(f"<{nent}I", d, p))
        p += 4 * nent
        corps = {}
        if sorte == 1:
            cin, cout, kh, kw, pas, dil, pad = struct.unpack_from("<7I", d, p)
            p += 28
            n = cout * cin * kh * kw
            w = np.frombuffer(d, np.float32, n, p).reshape(cout, cin, kh, kw)
            p += 4 * n
            b = np.frombuffer(d, np.float32, cout, p)
            p += 4 * cout
            corps = dict(w=w, b=b, pas=pas, dil=dil, pad=pad)
        elif sorte == 2:
            k, s, plafond = struct.unpack_from("<3I", d, p)
            p += 12
            corps = dict(k=k, s=s, plafond=plafond)
        elif sorte == 3:
            h, w_, rep = struct.unpack_from("<2IB", d, p)
            p += 9
            corps = dict(h=h, w=w_, rep=rep)
        ops.append((sorte, relu, ent, out, corps))
    assert p == len(d), "octets en trop"
    return taille, ntens, sortie, norm, ops


def conv(x, w, b, pas, dil, pad):
    cin, h, l = x.shape
    cout, _, kh, kw = w.shape
    xp = np.pad(x, ((0, 0), (pad, pad), (pad, pad)))
    ho = (h + 2 * pad - dil * (kh - 1) - 1) // pas + 1
    lo = (l + 2 * pad - dil * (kw - 1) - 1) // pas + 1
    y = np.zeros((cout, ho, lo), np.float64)
    for i in range(kh):
        for j in range(kw):
            tranche = xp[:, i * dil: i * dil + pas * (ho - 1) + 1: pas, j * dil: j * dil + pas * (lo - 1) + 1: pas]
            y += np.tensordot(w[:, :, i, j].astype(np.float64), tranche, axes=([1], [0]))
    return (y + b[:, None, None]).astype(np.float32)


def maxpool(x, k, s, plafond):
    c, h, l = x.shape
    arr = (lambda v: -(-v // s)) if plafond else (lambda v: v // s)
    ho, lo = arr(h - k) + 1, arr(l - k) + 1
    y = np.full((c, ho, lo), -np.inf, np.float32)
    for i in range(k):
        for j in range(k):
            t = x[:, i::s, j::s][:, :ho, :lo]
            y[:, :t.shape[1], :t.shape[2]] = np.maximum(y[:, :t.shape[1], :t.shape[2]], t)
    return y


def axe(n_in, n_out, rep):
    if rep == 1 and n_out == 1:
        src = np.zeros(1)
    else:
        src = (np.arange(n_out) + 0.5) * (n_in / n_out) - 0.5
    src = np.clip(src, 0, n_in - 1)
    i0 = np.floor(src).astype(int)
    i1 = np.minimum(i0 + 1, n_in - 1)
    return i0, i1, (src - i0).astype(np.float32)


def agrandir(x, h, l, rep):
    y0, y1, fy = axe(x.shape[1], h, rep)
    x0, x1, fx = axe(x.shape[2], l, rep)
    haut = x[:, y0][:, :, x0] * (1 - fx) + x[:, y0][:, :, x1] * fx
    bas = x[:, y1][:, :, x0] * (1 - fx) + x[:, y1][:, :, x1] * fx
    return (haut * (1 - fy)[None, :, None] + bas * fy[None, :, None]).astype(np.float32)


def entree(image, taille, norm):
    im = np.asarray(Image.open(image).convert("RGB").resize((taille, taille), Image.Resampling.LANCZOS),
                    np.float32)
    im = im / max(im.max(), 1e-6)
    im = (im - np.array(norm[:3], np.float32)) / np.array(norm[3:], np.float32)
    return im.transpose(2, 0, 1).astype(np.float32)


def executer(modele, x):
    taille, ntens, sortie, norm, ops = modele
    t = [None] * ntens
    t[0] = x
    for sorte, relu, ent, out, c in ops:
        a = t[ent[0]]
        if sorte == 1:
            y = conv(a, **c)
        elif sorte == 2:
            y = maxpool(a, **c)
        elif sorte == 3:
            y = agrandir(a, c["h"], c["w"], c["rep"])
        elif sorte == 4:
            y = np.concatenate([t[e] for e in ent], 0)
        elif sorte == 5:
            y = t[ent[0]] + t[ent[1]]
        elif sorte == 6:
            y = 1 / (1 + np.exp(-a))
        if relu:
            y = np.maximum(y, 0)
        t[out] = y.astype(np.float32)
    return t[sortie][0]


def principal():
    if sys.argv[1] == "etalon":
        ecrire_l_etalon(sys.argv[2], sys.argv[3])
        return
    modele = lire(sys.argv[1])
    x = entree(sys.argv[2], modele[0], modele[3])
    brut = executer(modele, x)
    m = (brut - brut.min()) / max(brut.max() - brut.min(), 1e-6)
    Image.fromarray((m * 255 + 0.5).astype(np.uint8)).save(sys.argv[3])
    if len(sys.argv) > 4:
        import onnxruntime as ort
        s = ort.InferenceSession(sys.argv[4], providers=["CPUExecutionProvider"])
        r = s.run(None, {s.get_inputs()[0].name: x[None]})[0][0, 0]
        print(f"écart à ONNX Runtime : max {np.abs(r - brut).max():.2e}, moyen {np.abs(r - brut).mean():.2e}")


def entree_d_epreuve(taille):
    """L'entrée des épreuves du moteur, exacte en binaire et la même en Rust (`entree_d_epreuve`) :
    un disque uni sur des rayures — une forme principale que le modèle doit voir."""
    c, y, x = np.meshgrid(np.arange(3), np.arange(taille), np.arange(taille), indexing="ij")
    rayures = (((c * 31 + x * 13) % 97) - 48) / 32.0
    centre, rayon = taille // 2, taille * 9 // 32
    dedans = (x - centre) ** 2 + (y - centre) ** 2 < rayon * rayon
    disque = np.array([1.5, 0.5, -1.0])[c]
    return np.where(dedans, disque, rayures).astype(np.float32)


def ecrire_l_etalon(modele_bin, sortie):
    """L'étalon des épreuves : le masque brut de l'entrée d'épreuve, un pixel sur huit dans chaque
    sens, puis sa moyenne et son maximum sur toute l'image (f32, petit-boutiste)."""
    modele = lire(modele_bin)
    brut = executer(modele, entree_d_epreuve(modele[0]))
    ech = brut[::8, ::8].astype("<f4")
    with open(sortie, "wb") as f:
        f.write(struct.pack("<2I", *ech.shape))
        f.write(ech.tobytes())
        f.write(struct.pack("<2f", float(brut.mean()), float(brut.max())))
    print(f"{sortie} : {ech.shape}, moyenne {brut.mean():.6f}, max {brut.max():.6f}")


if __name__ == "__main__":
    principal()
