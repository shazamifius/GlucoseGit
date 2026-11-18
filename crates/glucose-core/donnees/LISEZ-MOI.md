# Le modèle du sens

`sens-multilingue-128.bin` est la petite IA que le halo d'un texte emploie pour dire sa parenté
(SENS-1, PARENTE-1 et PARENTE-2, fiche 106 ; la note de décision 15).

* **Le modèle** : [`sentence-transformers/static-similarity-mrl-multilingual-v1`](https://huggingface.co/sentence-transformers/static-similarity-mrl-multilingual-v1),
  de l'organisation sentence-transformers sur Hugging Face, sous **licence Apache 2.0** — son texte est
  `LICENSE-static-similarity-mrl-multilingual-v1.txt`, à côté. Son vocabulaire est celui de
  `bert-base-multilingual-uncased` (Google, Apache 2.0).
* **Ce que Glucose en garde** : les 128 premières dimensions de ses 1 024 (il a été entraîné « en
  poupées russes », Matryoshka), en octets signés avec une échelle par morceau de mot — 15 Mo au
  lieu de 434. Mesuré sur la carte de Wikipédia : aussi juste que le modèle entier (fiche 106).
* **La conversion** : `python3 -I outils/sens/convertir.py <dossier du modèle> <sortie.bin> 128`,
  le dossier tenant `model.safetensors` et `tokenizer.json` tels que Hugging Face les donne ; le
  format est décrit en tête du script.
* **Les empreintes** (SHA-256), pour refaire le fichier à l'identique :
  * `model.safetensors` (de `0_StaticEmbedding/`) : `8245ab78ee71dded845a82d2270fcb9e785b29dad0e1619f69d5390c47d9ba00`
  * `tokenizer.json` : `11aaf894a4ccf3d95e8830e27c0f8152791fbbff2b988e29a265580b86edd216`
  * `sens-multilingue-128.bin` : `3b801916d06960a2116223e9d5d12a6f97fe212601c8c3af0bec769126f7c003`
* **La référence des épreuves** (`src/sens/tests/reference.txt`) : `outils/sens/reference.py`.
  **Les tables Unicode du découpeur** (`src/sens/unicode.rs`) : `outils/sens/tables.py`.

# Le modèle du détourage

`detourage-u2netp.bin` est le modèle que le geste « Détourer » emploie (DETOURER-1, fiche 121 ; la
note de décision 17).

* **Le modèle** : **u2netp**, la petite version d'U²-Net (Xuebin Qin, Zichen Zhang, Chenyang Huang,
  Masood Dehghan, Osmar Zaiane, Martin Jagersand, *U²-Net: Going Deeper with Nested U-Structure for
  Salient Object Detection*, Pattern Recognition 106, 2020), du dépôt
  [`xuebinqin/U-2-Net`](https://github.com/xuebinqin/U-2-Net) sous **licence Apache 2.0** — son texte
  est `LICENSE-U-2-Net.txt`, à côté. Ses poids, mis en ONNX par le projet `rembg` (licence MIT) :
  `u2netp.onnx` de sa publication [`v0.0.0`](https://github.com/danielgatis/rembg/releases/tag/v0.0.0)
  (4 574 861 octets, déposé le 24/11/2022).
* **Ce que Glucose en garde** : le graphe figé à son entrée de 320 × 320, ses ReLU fondus dans leurs
  convolutions, ses sorties de côté retirées (253 opérations), les poids en f32 — en demi-précision,
  le masque bougeait jusqu'à 0,1 sur des bords (fiche 121 § 3).
* **La conversion** : `python3 -I outils/detourage/convertir.py u2netp.onnx detourage-u2netp.bin 320
  imagenet` (le format est décrit en tête du script) ; **l'étalon** : `outils/detourage/reference.py`,
  qui rend le masque d'ONNX Runtime à 2·10⁻⁵ près.
* **Les empreintes** (SHA-256) :
  * `u2netp.onnx` : `309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8`
  * `detourage-u2netp.bin` : `1de3fd6dcf5af7a0b84efab253c5225d9f5b92c3beea92835adb85859e1d394d`
* **Ses données d'entraînement** (DUTS-TR) n'ont pas de licence publiée par leurs auteurs ; la note de
  décision 17 le dit, et pourquoi on l'accepte.
