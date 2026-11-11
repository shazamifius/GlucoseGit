#!/usr/bin/env python3
# **La référence du sens** (fiche 106) : pour des textes choisis pour piéger un découpeur, les rangs
# exacts que donne le découpeur de Hugging Face, et la ressemblance de quelques paires, calculée sur le
# fichier converti. Les épreuves du noyau s'y comparent.
#
#   python3 -I reference.py <tokenizer.json> <sens-multilingue-128.bin> <sortie.txt>
#
# Le format, ligne par ligne (le noyau n'a pas de lecteur JSON, et une épreuve n'en vaut pas un) :
#   « T <octets UTF-8 du texte, en hexadécimal> », puis « R <rangs séparés par des espaces> » ;
#   enfin « P <i> <j> <ressemblance> » pour chaque paire.
import json, struct, sys
import numpy as np
from tokenizers import Tokenizer
tok = Tokenizer.from_file(sys.argv[1])
b = open(sys.argv[2], 'rb').read()
n, d = struct.unpack_from('<II', b, 8)
p = 16
for _ in range(n):
    (l,) = struct.unpack_from('<H', b, p); p += 2 + l
echelles = np.frombuffer(b, '<f4', n, p); p += 4 * n
table = np.frombuffer(b, np.int8, n * d, p).reshape(n, d).astype(np.float32) * echelles[:, None]
textes = [
    "L'eau est une substance chimique constituée de molécules H₂O.",
    "Hydrogène — l'élément le plus léger de l'univers.",
    "ÉCOLE, Château, Noël, cœur, Œuvre, æther, déjà-vu !",
    "« Guillemets » et — tirets — … points de suspension",
    "Un texte\twith\ttabs\nand newlines\r\nmixed",
    "Espace insécable : 12 000 € et 3 %",
    "Zéro​width‍joiner﻿BOM et trait­dunion",
    "東京は日本の首都です。北京是中国的首都。",
    "서울은 한국의 수도입니다.",
    "Москва — столица России. Ελληνικά γράμματα ΣΟΦΟΣ.",
    "مرحبا بالعالم",
    "Emoji 😀🎉 et symboles ∑ ∫ √ → ≠ ©",
    "anticonstitutionnellement" * 5,
    "x" * 101,
    "un_mot-composé, avec/des:séparateurs;variés?oui!",
    "Les modèles de langage (LLM) transforment l'IA en 2026.",
    "La photosynthèse convertit la lumière en énergie chimique.",
    "Le Moyen Âge s'étend du Ve au XVe siècle en Europe.",
    "L'ÉTÉ ÉTAIT ÉTOUFFANT À SÈTE",
    "İstanbul, ǅemal, ﬁnance, ½, ²",
    "",
    "   ",
    "123 456,78 3.14159 1e-9",
    "C'est l'heure : 14h30 ; demain 9h.",
    "mot́ avec accent combinant été",
    "Ⅻ ⅓ ℃ ℝ ﬀ",
    "tabulation\x0bverticale\x0cet\x1cséparateurs",
    "privéet\U000F0000",
]
paires = [(0, 1), (0, 16), (1, 16), (15, 16), (17, 1), (0, 0), (2, 18), (7, 8), (9, 0)]
def plonger(t):
    ids = tok.encode(t, add_special_tokens=False).ids
    if not ids: return None
    v = table[ids].mean(axis=0)
    return v / np.linalg.norm(v)
V = [plonger(t) for t in textes]
with open(sys.argv[3], 'w', encoding='ascii') as f:
    for t in textes:
        f.write('T ' + t.encode('utf-8').hex() + '\n')
        f.write('R ' + ' '.join(map(str, tok.encode(t, add_special_tokens=False).ids)) + '\n')
    for i, j in paires:
        if V[i] is not None and V[j] is not None:
            f.write(f'P {i} {j} {float(V[i] @ V[j]):.7f}\n')
print(len(textes), 'textes')
