"""Tirer d'un modèle de détourage ONNX le fichier que Glucose lit (DETOURER-1, fiche 121).

Usage : python3 -I outils/detourage/convertir.py <modèle.onnx> <sortie.bin> <taille> <normalisation>
        <normalisation> : « imagenet » (U²-Net, u2netp) ou « demi » (ISNet)

Glucose ne lit jamais ONNX et ne dépend d'aucun moteur d'inférence : ce script fige, une fois pour
toutes, le graphe du modèle à sa taille d'entrée — une liste d'opérations aux tailles connues. Les
calculs de forme du graphe (Shape, Gather, Slice, Cast, Unsqueeze, Concat de l'axe 0, Constant) n'y
entrent pas : chaque agrandissement y garde la taille qu'il vise. Ce qui ne mène pas au masque (les
sorties de côté d'U²-Net) est retiré. Une convolution suivie de son seul ReLU les porte toutes deux.

Le format, en petit-boutiste :

    "GLUCDET1"                                  8 octets
    u32 taille                                  l'image d'entrée est taille × taille
    u32 tenseurs, u32 operations, u32 sortie    le tenseur 0 est l'entrée ; `sortie` est le masque
    f32 × 3 moyenne, f32 × 3 ecart              x = (pixel / max(image) − moyenne) / écart
    puis chaque opération :
      u8 sorte, u8 relu, u16 entrees, u32 sortie, u32 × entrees
      sorte 1, Conv     : u32 cin, cout, kh, kw, pas, dilatation, remplissage ;
                          f32 × cout·cin·kh·kw (poids), f32 × cout (biais)
      sorte 2, MaxPool  : u32 noyau, pas, plafond (1 : la taille arrondie vers le haut)
      sorte 3, Resize   : u32 hauteur, largeur ; u8 repere (0 « half_pixel », 1 « pytorch_half_pixel »)
      sorte 4, Concat   : (les canaux, dans l'ordre des entrées)
      sorte 5, Add      : —
      sorte 6, Sigmoid  : —

Les poids restent en f32 : en demi-précision, le masque de u2netp bougeait jusqu'à 0,1 sur des bords
(fiche 121 § 3).
"""
import struct
import sys

import numpy as np
import onnx
from onnx import helper, numpy_helper

NORMALISATIONS = {
    "imagenet": ((0.485, 0.456, 0.406), (0.229, 0.224, 0.225)),
    "demi": ((0.5, 0.5, 0.5), (1.0, 1.0, 1.0)),
}
CALCULS_DE_FORME = {"Shape", "Gather", "Slice", "Cast", "Unsqueeze", "Constant"}


def attributs(n):
    return {a.name: helper.get_attribute_value(a) for a in n.attribute}


def formes_statiques(m, taille):
    """La forme réelle de chaque agrandissement, l'entrée fixée à 1 × 3 × taille × taille.

    Le calcul de forme d'ONNX ne la connaît pas toujours (elle vient d'un `Shape` du graphe) : on la
    demande à ONNX Runtime, qui exécute le graphe une fois sur une entrée nulle."""
    import onnxruntime as ort
    resizes = [n.output[0] for n in m.graph.node if n.op_type == "Resize"]
    sonde = onnx.ModelProto()
    sonde.CopyFrom(m)
    connues = {o.name for o in sonde.graph.output}
    for nom in resizes:
        if nom not in connues:
            sonde.graph.output.append(onnx.ValueInfoProto(name=nom))
    s = ort.InferenceSession(sonde.SerializeToString(), providers=["CPUExecutionProvider"])
    x = np.zeros((1, 3, taille, taille), np.float32)
    noms = [o.name for o in s.get_outputs()]
    valeurs = s.run(noms, {s.get_inputs()[0].name: x})
    return m, {n: list(v.shape) for n, v in zip(noms, valeurs) if n in resizes}


def utiles(g, sortie):
    """Les nœuds dont le masque dépend, dans l'ordre du graphe."""
    prod = {o: n for n in g.node for o in n.output}
    vus, pile = set(), [sortie]
    while pile:
        t = pile.pop()
        n = prod.get(t)
        if n is None or id(n) in vus:
            continue
        vus.add(id(n))
        if n.op_type == "Resize":
            pile.append(n.input[0])  # la taille visée est figée : son calcul ne sert plus
        else:
            pile.extend(n.input)
    return [n for n in g.node if id(n) in vus and n.op_type not in CALCULS_DE_FORME
            and not (n.op_type == "Concat" and attributs(n).get("axis") == 0)]


def convertir(chemin, sortie_bin, taille, normalisation):
    m, formes = formes_statiques(onnx.load(chemin), taille)
    g = m.graph
    poids = {i.name: numpy_helper.to_array(i) for i in g.initializer}
    masque = g.output[0].name
    noeuds = utiles(g, masque)
    consommateurs = {}
    for n in noeuds:
        for i in n.input:
            consommateurs[i] = consommateurs.get(i, 0) + 1
    numero = {g.input[0].name: 0}

    def num(nom):
        if nom not in numero:
            numero[nom] = len(numero)
        return numero[nom]

    ops = []
    absorbes = set()
    for n in noeuds:
        if id(n) in absorbes:
            continue
        a = attributs(n)
        sorte, relu, corps = None, 0, b""
        entrees = [n.input[0]]
        sortie = n.output[0]
        if n.op_type == "Conv":
            w, b = poids[n.input[1]].astype(np.float32), poids[n.input[2]].astype(np.float32)
            assert a.get("group", 1) == 1
            dil, pas, pads = a["dilations"], a["strides"], a["pads"]
            assert dil[0] == dil[1] and pas[0] == pas[1] and len(set(pads)) == 1, n.name
            cout, cin, kh, kw = w.shape
            # Le ReLU qui suit, s'il est le seul à lire la convolution, entre dans l'opération.
            suivants = [r for r in noeuds if r.op_type == "Relu" and r.input[0] == sortie]
            if suivants and consommateurs.get(sortie) == 1:
                relu, sortie = 1, suivants[0].output[0]
                absorbes.add(id(suivants[0]))
            sorte = 1
            corps = struct.pack("<7I", cin, cout, kh, kw, pas[0], dil[0], pads[0])
            corps += w.tobytes() + b.tobytes()
        elif n.op_type == "Relu":
            raise SystemExit(f"un ReLU seul n'est pas prévu : {n.name}")
        elif n.op_type == "MaxPool":
            assert a["kernel_shape"][0] == a["kernel_shape"][1] and set(a.get("pads", [0])) == {0}
            sorte = 2
            corps = struct.pack("<3I", a["kernel_shape"][0], a["strides"][0], a.get("ceil_mode", 0))
        elif n.op_type == "Resize":
            assert a["mode"] == b"linear", n.name
            repere = {b"half_pixel": 0, b"pytorch_half_pixel": 1}[a["coordinate_transformation_mode"]]
            h, w_ = formes[n.output[0]][2:]
            sorte = 3
            corps = struct.pack("<2IB", h, w_, repere)
        elif n.op_type == "Concat":
            assert a["axis"] == 1, n.name
            sorte, entrees = 4, list(n.input)
        elif n.op_type == "Add":
            sorte, entrees = 5, list(n.input)
        elif n.op_type == "Sigmoid":
            sorte = 6
        else:
            raise SystemExit(f"opération inconnue : {n.op_type} ({n.name})")
        tete = struct.pack("<BBHI", sorte, relu, len(entrees), num(sortie))
        tete += struct.pack(f"<{len(entrees)}I", *[num(e) for e in entrees])
        ops.append(tete + corps)
    moy, ec = NORMALISATIONS[normalisation]
    with open(sortie_bin, "wb") as f:
        f.write(b"GLUCDET1")
        f.write(struct.pack("<4I", taille, len(numero), len(ops), numero[masque]))
        f.write(struct.pack("<6f", *moy, *ec))
        for o in ops:
            f.write(o)
    sortes = {}
    for o in ops:
        sortes[o[0]] = sortes.get(o[0], 0) + 1
    print(f"{sortie_bin} : {len(ops)} opérations {sortes}, {len(numero)} tenseurs")


if __name__ == "__main__":
    convertir(sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4])
