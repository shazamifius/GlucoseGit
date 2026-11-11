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
