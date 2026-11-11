#!/usr/bin/env python3
# **Le modèle du sens, pour Glucose** (fiche 106) : convertit `static-similarity-mrl-multilingual-v1`
# (sentence-transformers, Apache 2.0) en un seul fichier que le noyau lit sans dépendance.
#
#   python3 -I convertir.py <dossier du modèle> <sortie.bin> [dimensions=128]
#
# Le dossier du modèle : `0_StaticEmbedding/model.safetensors` et `tokenizer.json`, tels que Hugging Face
# les donne. Le modèle a été entraîné « en poupées russes » (Matryoshka) : ses 128 premières
# dimensions portent l'essentiel — mesuré sur la carte de Wikipédia, 128 dimensions en octets valent
# les 1 024 en flottants (fiche 106).
#
# Le format, en petit-boutiste :
#   « GSENS1\0\0 », u32 nombre de jetons, u32 dimensions,
#   puis pour chaque jeton : u16 longueur, ses octets UTF-8 (le vocabulaire, dans l'ordre des rangs),
#   puis une échelle f32 par jeton, puis la table : dimensions octets signés par jeton.
# Un jeton se lit `échelle × octet` : chaque ligne est ramenée à [−127, 127] par son plus grand
# module, la quantification que l'évaluation a mesurée sans perte.
import json, struct, sys
import numpy as np
from safetensors.numpy import load_file
dossier, sortie = sys.argv[1], sys.argv[2]
d = int(sys.argv[3]) if len(sys.argv) > 3 else 128
E = next(iter(load_file(f'{dossier}/model.safetensors').values()))[:, :d].astype(np.float32)
vocab = json.load(open(f'{dossier}/tokenizer.json', encoding='utf-8'))['model']['vocab']
mots = [None] * len(vocab)
for mot, rang in vocab.items():
    mots[rang] = mot
assert all(m is not None for m in mots) and len(mots) == E.shape[0], 'un rang sans jeton'
echelle = np.abs(E).max(axis=1) / 127.0
echelle[echelle == 0] = 1.0
octets = np.clip(np.round(E / echelle[:, None]), -127, 127).astype(np.int8)
with open(sortie, 'wb') as f:
    f.write(b'GSENS1\0\0' + struct.pack('<II', len(mots), d))
    for m in mots:
        b = m.encode('utf-8')
        f.write(struct.pack('<H', len(b)) + b)
    f.write(echelle.astype('<f4').tobytes())
    f.write(octets.tobytes())
print(f'{len(mots)} jetons × {d} dimensions → {sortie}')
