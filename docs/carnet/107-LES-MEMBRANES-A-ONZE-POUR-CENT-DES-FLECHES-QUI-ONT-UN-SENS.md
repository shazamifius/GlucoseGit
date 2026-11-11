# 107 — Les membranes à 11 %, des flèches qui ont un sens

> Session du 11/11/2026, à partir de 21 h 35. La passation (fiche 106) laissait quatre sabotages
> aveugles à rejouer et la fiche 106 en brouillon ; ses mots du soir, en réponse à la page du halo :
> les flèches grises acceptées seulement si c'est temporaire, les membranes pas assez transparentes,
> et l'IA qui aide, que l'humain corrige, qui apprend. Envoyé : `e6cbb9d`, `906510f` (la fiche 106),
> `864909d` (les membranes), `5e48642` ; son Glucose mis à jour (`5e48642`).

## 1. La fiche 106 finie

L'épreuve réécrite du halo a tourné (verte), puis toute la suite : **3 122 épreuves**, clippy et
`cargo fmt` à zéro. Les quatre sabotages aveugles, rejoués dans un arbre détaché, tombent : **36 sur
36**. La fiche 106 est posée depuis son brouillon, ses trous remplis ; l'index cite la note de
décision 15, qu'il oubliait.

## 2. Ses mots du soir (VISION § 5, § 6, § 8 bis)

* Les flèches grises : *« si c'est temporaire qu'elles soient grises, ça me va ; seulement si c'est
  temporaire »*.
* *« Il faut que le maximum de choses fasse sens sans que l'humain doive trop y réfléchir ; l'IA doit
  pouvoir aider l'humain, et l'humain doit corriger l'IA ; et l'IA doit enregistrer, apprendre, et
  ensuite un re-fine-tuning, ou la création d'une vraie IA from scratch, plus tard »*.
* *« Non, je trouve que les membranes ne sont pas encore assez transparentes ; il faudrait les rendre
  plus transparentes »* — puis, devant les trois forces : *« c'est vraiment vraiment parfait, 11 % !
  on garde »*.

## 3. Les membranes à 11 % (REGION-1)

Son « non » répondait à la question des familles (« Eau, de près » : se lisent-elles assez ?). Mesuré
sur la capture : la lueur cyan d'une famille posée sur le sol orangé de sa région était à peine plus
vive que lui — une grande surface veut une couleur sourde, une petite marque une couleur vive.

* **Essayé sur ses quatre vues**, par la carte graphique, à 21,3, 11,3, 8 et 6 % (une force d'essai
  lue dans l'environnement, retirée avant le commit). À 11,3 %, les régions se reconnaissent encore
  de loin, la famille se détache 3,8 fois de son sol au lieu de 2, et le bord de chaque région, que
  ses halos éclairent, se dessine mieux que son corps (le motif « territoire » des cartographes). À
  8 et 6 %, de loin, les régions s'éteignent et la toile des flèches grises domine.
* **La loi** : le corps prend **le voile d'une membrane seule chez Tauri** (ses trois couches
  translucides, 8, 8 et 14 sur 255) — le « posé deux fois » arbitraire disparaît.
* **Une affirmation corrigée avant d'être écrite** : je voulais dire dans le code qu'au voile simple
  un halo sûr reste toujours plus vif que sa région. Vérifié sur tout le tour des teintes : faux pour
  une membrane d'une couleur poussée au bord de l'écran. Le commentaire dit la mesure faite sur sa
  carte, et la limite.
* **Prouvé** : l'épreuve neuve refait le ton de Tauri sans la fonction gardée (trois couches posées
  l'une après l'autre) et voit le voile double ; 3 123 épreuves vertes ; les trois empreintes
  témoins regardées avant/après (113 723 à 123 066 pixels, tous dans la boîte de la membrane) ;
  **3 sabotages sur 3** (`outils/sabotages/membranes-107.json`). La page :
  <https://claude.ai/artifact/6cUix7pewFDikQumRqjnLZ>.

## 4. Des flèches qui ont un sens (PARENTE-4, prototype)

Le gris d'une flèche n'est accepté que temporairement : il faut qu'elle dise quelque chose. Première
mesure, en Python sur le modèle entier : l'IA juge les liens de Wikipédia comme un lecteur (« Lune →
Système solaire » sûr, « Soleil → Vêtement » nul).

* **Le sens d'une région.** Les flèches de la carte vont du texte d'un article à la **membrane**
  d'un autre (`outils/wikipedia/squelette.py`, étape 7) : la première loi, qui ne jugeait que les
  flèches entre deux textes, passait ses épreuves et ne changeait **aucun pixel** de la carte. Une
  membrane se juge par ce qu'elle porte (ses textes, ceux de ses filles) et par son nom.
* **Le hasard de la cible, hors d'elle.** Contre le hasard du document, une grande région, somme de
  beaucoup de textes, ressemble à tout (la direction commune de la langue). Contre le hasard de la
  cible lu sur tout le document, « Océan → Eau » tombait à 0 : la région porte des centaines de
  paragraphes qui lui ressemblent. Lu hors d'elle : « Atmosphère terrestre → Eau » 0,93, « Océan →
  Eau » 0,72, « Lune → Eau » 0 ; sur 1 746 liens, 39 % sûrs, 36 % nuls.
* **Deux lois, regardées.** (1) `d + (1 − d) s` : un lien de sens garde sa force même loin de la
  souris. À « Philosophie », l'encre des flèches **doublait** — plus chargé qu'avant. (2) **La
  densité ne change pas ; la souris ne rapproche que les liens de sens** (`max(d, proximité × s)`) :
  près de « Religion », seules deux longues flèches de sens s'allument, les flèches d'habitude
  restent discrètes ; survoler le bloc allume tout ; loin, rien ne change.
* **La question qui reste à sa parole** : Glucose ne sait pas qui a posé une flèche. Juger aussi
  celles qu'il tire lui-même ferait, sur un document où l'IA hésite, que ses propres flèches cessent
  de s'allumer près de la souris — les épreuves de LIEN-4, dont le document planté n'a que des
  textes identiques (l'IA n'y voit aucun écart, chaque lien à 0), l'ont montré. Mon avis : retenir
  qui a posé chaque flèche, ne juger que celles qu'il n'a pas tirées, et noter comme une correction
  le geste qui reprend une flèche écartée — la première ligne du journal des rectifications.
  La page : <https://claude.ai/artifact/M5YZbYFWmwH3jScdcKwWws>.
* **Le prototype est sauvé**, entier, épreuves comprises : `~/.cache/glucose-bancs/fleches-107/
  prototype-parente-4.patch`, sur `5e48642` (`git apply` le pose) — le noyau
  (`sens/liens.rs` : `Region`, `Bout`, `surete_des_liens`, quatre épreuves dont deux preuves à
  l'envers), le fil (`parente/liens.rs`), la présence (`presence.rs`), un agent qui lit la sûreté
  d'une flèche. Les cinq épreuves de LIEN-4 tombées attendent la réponse : leur document doit avoir
  des textes distincts, ou des flèches posées par un script.

## 4 bis. Son lag — mesuré, profilé, commencé

Sa chronique copiée (`derniere-session.txt`, le 11/11 à 5 h 41) datait d'avant les fiches 105 et 106 :
81 % des images au-dessus de 10 ms, les membranes en tête. **Remesuré aujourd'hui** sur sa carte, par
la carte graphique, son Glucose installé (`5e48642`) contre lui-même puis en paires entrelacées
(`~/.cache/glucose-bancs/lag-107/` : `mesure.sh`, `lire.py`) :

| geste | image médiane | p99 |
|---|---|---|
| de loin, la souris | 14,9 à 16,8 ms | 20 à 30 ms |
| de loin, déplacer la vue | 16,1 à 17,9 ms | 18 à 23 ms |
| de loin, zoomer | 19,1 à 19,8 ms | 21 à 22 ms |
| au fond de « Eau », déplacer | 4,2 à 4,7 ms | 16 à 17 ms |

**Le profil** (déplacer de loin, `profiler-leger.sh` ; une version à lignes de code dans
`lag-107/cible`) : les **titres des membranes** redessinés lettre à lettre au processeur à chaque
image (`draw_text_with_outline`, dont `blend_glyph` 20 %) — **26 %** ; le **tracé des flèches calculé
deux fois** par image, pour la carte (`fleches_a_poser`, 15 %) puis pour la couche du processeur
(`draw_arrow_node`, 11 %) — **21 %** ; renvoyer la couche du processeur à la carte graphique — 9 % ;
des conversions de couleur à chaque image (`pow`, `cbrt`) — 5 % ; le hachage SipHash — 4 %.

**Commencé, pas commité** : `porte_des_ornements` (`renderer/pass.rs`) — sur la voie graphique, une
flèche sans poignée, sans étiquette ni badge ne recalcule plus son tracé ; son épreuve
(`fleche_1_seule_une_fleche_ornee_se_recalcule`) et les 125 épreuves des flèches passent. Mesuré en
paires : le poste « annotations » passe de 1,9 à moins de 0,7 ms ; les médianes de loin gagnent 1 à
1,5 ms, dans le bruit de la machine. **Reste** : toute la suite, clippy, ses sabotages, le commit.
Une copie : `~/.cache/glucose-bancs/lag-107/fleche-sans-ornement.patch`.

**Le gros morceau, à concevoir** : les titres des membranes portés par la carte graphique, comme
des composants (COMPOSANT-1 : rendus une fois, posés à chaque image) — attention à l'ordre des
couches, un titre passe aujourd'hui sous les photos ; puis les conversions de couleur refaites à
chaque image.

## 5. La recherche (consultée le 11/11/2026)

* La couleur d'une surface et celle d'une marque : Szafir, *Modeling Color Difference for
  Visualization Design* (IEEE VIS 2017) — une marque plus petite demande un plus grand écart de
  couleur (<https://cmci.colorado.edu/visualab/VisColors/>) ; « petite surface, forte saturation ;
  grande surface, faible saturation » (Stone ; Munzner, *Visualization Analysis and Design*) ; un fond
  sourd sous des données vives, et le « test du plissement d'yeux » (Axis Maps,
  <https://axismaps.com/guide/visual-hierarchy>).
* Le sens des liens : Dimitrov, Singer, Lemmerich et Strohmaier, *What Makes a Link Successful on
  Wikipedia?* (WWW 2017, <https://arxiv.org/abs/1611.02508>) ; West et al. (2009) et Singer et al.
  (2013), cités par <https://arxiv.org/pdf/1405.4364> — les liens posés par habitude, le bruit des
  mesures tirées des liens.
* La méta-compréhension : Kulesza et al., *Principles of Explanatory Debugging* (IUI 2015) ; FaLA
  (EMNLP 2023 Findings), une adaptation linéaire sans rétropropagation ; Model2Vec et Tokenlearn
  (<https://github.com/MinishLab/model2vec>). Rangé dans
  `~/.cache/glucose-bancs/membranes-107/recherche-meta.md` pour la page de conception.

## 6. Les leçons

* **Un `pkill -f` dont le motif figure dans sa propre commande se tue lui-même** (MÉTHODE § 5 le
  disait) : arrêter par numéro de processus.
* **Le profileur, léger** : `-F 400 --call-graph dwarf,8192` (31 Mo, le rapport des appelants en deux
  minutes) au lieu de `dwarf,16384` à pleine cadence (627 Mo, plus de vingt minutes) ; viser le
  processus par `pgrep -n -x glucose-desktop`, jamais par un motif qui attrape le script.

* **Mesurer l'encre de la vraie carte avant de croire une épreuve verte** : la première loi des
  flèches passait toutes ses épreuves et ne changeait aucun pixel — les flèches de la carte visent
  des membranes. Lire comment les données ont été posées avant de concevoir pour elles.
* **Une loi qui ajoute de la lumière à une vue chargée l'alourdit** : redistribuer ce que la densité
  accorde, ne jamais l'augmenter.
* **Le hasard d'une cible se lit hors d'elle** : ce qu'une région porte lui ressemble forcément.
* **Un monde planté doit ressembler au vrai** : des textes « sans sujet » ne portaient que la langue
  commune, et ressemblaient vraiment à la grande région faite d'eux ; sur une carte, chaque texte
  parle de quelque chose.
* **Une affirmation générale se vérifie sur tout son domaine** avant d'entrer dans un commentaire :
  la vivacité du halo sur sa région, vraie pour l'orange et le cyan, fausse au bord de l'écran.
