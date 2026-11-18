# 17 — Le détourage : u2netp embarqué, lu par un moteur à nous

> Sa parole du 18/11/2026 (VISION § 3) : *« à voir quel est le meilleur algorithme, le plus adapté
> pour notre projet actuellement ; attention aux licences »*. Cette note dit le modèle embarqué —
> **une donnée, pas une caisse** : aucune dépendance n'entre dans le programme —, pourquoi celui-là,
> et ce que sa licence engage.
>
> **Date** : 2026-11-18 · **Portée** : `crates/glucose-core/donnees/detourage-u2netp.bin`,
> `crates/glucose-core/src/detourage*`, `outils/detourage/` · **Fiche** : 121.

---

## Ce qu'il faut

* La **forme principale** d'une image, seule, en transparence — sa boule sur un fond multicolore ;
* **sur toutes les machines** (*« aucune machine exclue »*) : le téléphone, le vieux portable, sans
  carte graphique, sans réseau ;
* une licence qui permette à Glucose (MIT) d'être forké, vendu, embarqué : **aucune clause non
  commerciale**.

## Ce qu'on prend

**u2netp** — la petite U²-Net (Qin et al., *Pattern Recognition* 106, 2020), du dépôt
[`xuebinqin/U-2-Net`](https://github.com/xuebinqin/U-2-Net), **licence Apache 2.0** (son texte est
dans `donnees/`). 1,13 M de paramètres, **4,5 Mo** en f32, entrée 320 × 320, **19,8 milliards de
multiplications** par image (comptées sur son graphe).

**Écrit ici, sans caisse** : le moteur qui le fait tourner (`glucose-core/src/detourage/`). Le
modèle n'emploie que sept opérations — convolution (dilatée 1, 2, 4, 8), ReLU, regroupement par
maximum, agrandissement bilinéaire, concaténation, addition, sigmoïde —, quelques centaines de
lignes. `outils/detourage/convertir.py` fige une fois le graphe ONNX en une liste d'opérations ;
`outils/detourage/reference.py` l'exécute en numpy, et rend le masque d'ONNX Runtime à 2·10⁻⁵
près ; le moteur rend l'étalon de numpy à 4·10⁻⁵ près (l'épreuve `le_moteur_rend_l_etalon`).

## Pourquoi celui-là, mesuré (fiche 121 § 3)

| | ce que ça donne | pourquoi non, ou pas encore |
|---|---|---|
| **u2netp** | sa boule sur fond multicolore : 0,29 % d'erreur moyenne d'opacité ; un anneau sur blanc, son trou compris : 0,47 % ; 1,0 s le masque sur son VivoBook (7 fils), 1,2 à 1,5 s avec la couleur du premier plan | **pris** |
| ISNet (`isnet-general-use`, Apache 2.0) | même famille, mêmes sept opérations : **le même moteur le fait tourner** ; 44 M de paramètres, 1024², 160 milliards de multiplications | 176 Mo ; **entraîné sur DIS5K, dont les conditions interdisent l'usage commercial « même après traitement »** (le PDF de ses auteurs, lu). Le modèle plus fort selon la machine, s'il passe la question de sa licence |
| BiRefNet, BiRefNet lite (MIT) | le meilleur des modèles ouverts | un transformeur Swin et des convolutions déformables à 1024² : lent sans carte graphique ; entraîné sur DIS5K lui aussi |
| BEN2 base (MIT sur sa carte) | bon sur les cheveux | 219 Mo en fp16 ; le modèle complet de son éditeur est commercial |
| RMBG-1.4, RMBG-2.0 (BRIA) | | **CC BY-NC 4.0, non commercial** — et le défaut de `rembg` depuis sa 2.0.80 : le piège |
| `ort` (ONNX Runtime), `tract`, `candle`, `burn` | n'importe quel modèle | une caisse lourde, du C++ précompilé (`ort`) qu'un paquet Nix ne télécharge pas ; pour sept opérations |
| les poids en demi-précision | 2,3 Mo de moins | le masque bougeait jusqu'à 0,1 sur des bords : non |
| la voie du système (Vision sur macOS, ML Kit sous Android) | rien à embarquer | pas sous Windows ni Linux, et un résultat différent par machine |

## Ce que sa licence engage, et ce qu'on accepte en le sachant

* **Les poids** sont sous Apache 2.0, donnés par leurs auteurs avec leur code : Glucose peut les
  embarquer, à condition de garder leur licence et de dire leur provenance (`donnees/LISEZ-MOI.md`).
* **Ses données d'entraînement** (DUTS-TR) n'ont aucune licence publiée par leurs auteurs ; les
  miroirs disent « académique » ou « inconnue ». **Aucun jeu de données de détourage n'a de licence
  commerciale claire** : c'est l'état du métier, et `rembg` (MIT) sert ces mêmes poids partout. On
  l'accepte, en le disant ici, parce que Glucose est gratuit et que les poids, eux, sont donnés sous
  une licence permise. Le jour où un modèle à données libres existera — ou le nôtre (sa parole :
  *« un jour, on créera notre propre algo »*) —, il prendra la place, et cette note le dira.

## Ce qui reste

* **Le dessin au trait** n'est pas pour le modèle (il garde le blanc entre les traits, 32 %
  d'erreur) : c'est « Retirer le fond uni », exact, sans IA (DETOURER-3).
* **De faibles fantômes** du fond (0,5 % des pixels du fond au-dessus de 10 % d'opacité, sur la
  boule) : la limite de u2netp à 320² ; aucun seuil ne les cache — ce serait une constante.
* **La voie de la carte graphique** : la même convolution en nuanceur (`wgpu`), pour ISNet sur une
  bonne machine ; et un modèle selon la machine (ses « trois IA », VISION § 8 bis).
* **Le choix de l'objet** : sur un vélo devant une façade couverte de graffitis, u2netp garde la
  façade entière — la « forme principale » y est ambiguë ; un clic pour désigner l'objet viendra.

## Sources (consultées le 18/11/2026)

* U²-Net : <https://github.com/xuebinqin/U-2-Net> ; les poids en ONNX : <https://github.com/danielgatis/rembg/releases/tag/v0.0.0>
* DIS (ISNet) et les conditions de DIS5K : <https://github.com/xuebinqin/DIS>
* BiRefNet : <https://github.com/ZhengPeng7/BiRefNet> ; BEN2 : <https://huggingface.co/PramaLLC/BEN2>
* RMBG-2.0 : <https://huggingface.co/briaai/RMBG-2.0>
* Le sondage des modèles ouverts de nukebg (2026-Q4) : <https://github.com/yocreoquesi/nukebg/issues/431>
* La couleur du premier plan : Marco Forte, *Approximate Fast Foreground Colour Estimation*, ICIP
  2021, doi:10.1109/ICIP42928.2021.9506164
* La « couleur vers alpha » de GIMP : <https://docs.gimp.org/2.10/en/gimp-filter-color-to-alpha.html>
