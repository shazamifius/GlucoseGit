# 167 — Jamais au mur (25/11, le relais, compte a)

## 1. Sa preuve

Sa note (`text-ztqw-2319`, 25/11) : *« un jour Claude ne s'est pas du tout arrêté à la limite et j'ai
dû intervenir en urgence »* ; sa réponse (`text-ztqw-2323`) : *« aucun extra usage […] 2 secondes après
avoir dit attention il s'est arrêté net, et ce n'est pas le seul ; c'était hier »*. Sa capture (dans
`membrane-ztqw-2312`) : le 24/11 à 19 h 37, la session du compte b, le crochet qui répète « Ne t'arrête
pas », son message *« attention tu es à la limite !!!!! de token »*, la session qui écrit en hâte
l'état non commité (*« un fichier a peut-être été laissé abîmé par un essai coupé en route »*), puis
« You've hit your 5-hour limit ».

## 2. La cause : une règle exprès

Le journal : `19:16:54 seuil passé, aucun compte pour reprendre : le compte b va jusqu'au mur`. Depuis le
23/11, un compte à 90 % sans autre compte pour reprendre **continuait jusqu'au mur**, « prévenu une
fois » (« mieux vaut finir sa jauge que dormir » : ce matin-là, trois heures sans personne à 91 %). Le
crochet de fin de tour, lui, ne demandait la passation que si un autre compte pouvait reprendre : il
disait « continue ». Au mur, le veilleur tue la session (`tmux kill-session`) : le travail en cours
part, la passation n'est pas écrite, et un sabotage en route reste dans le code (tué, `saboter.py` ne
remet pas le fichier).

## 3. Elle ne rapportait rien

**La semaine borne tout.** Une fenêtre de cinq heures pleine prend ~13 points de la semaine (fiche
166) : un compte remplit sa semaine en ~8 fenêtres, quand la semaine en contient 33 ; les deux
comptes la remplissent bien avant leur remise (b plein jusqu'au 30/11). Ce qu'une fenêtre ne dépense
pas **reste à la semaine** : s'arrêter à 90 % des cinq heures ne retire aucun travail à la semaine, il
le décale. Seule exception, déjà tenue par RESERVE-1 (fiche 163) : la dernière fenêtre avant la remise
de la semaine, où ce qui reste serait perdu.

**Ce que coûte une passation**, mesuré sur les quatorze du 24 au 25/11 (`jauges-histoire.jsonl` contre
le journal) : 1 à 4 minutes, **au plus un point** des cinq heures, rien de la semaine.

## 4. Fait

* **À 90 %, toujours la passation** : sans personne pour reprendre, le veilleur la demande (`garder` →
  « repos », `POURQUOI["repos"]`) et le crochet aussi ; le relais attend ensuite la remise (`choisir`).
  Le geste « prévenir » et sa raison disparaissent.
* **Le repli** (`repli`) : faute de compte sous ses seuils, le travail ne reprend jamais un compte
  au-delà du seuil de ses cinq heures ; sa semaine va **jusqu'au bord** dans sa seule dernière fenêtre ;
  **ses notes** partent jusqu'au bord. **Le bord** (`BORD`, 99 %) : le mur moins une passation.
* Le veilleur relancé (12 h 06) : il lit le code neuf ; le crochet le lit à chaque fin de tour.
* Épreuves : cinq revues, une renommée (`…_la_passation_puis_l_attente_jamais_le_mur`), le crochet au
  seuil ; sabotages `relais-167.json`, 4 sur 4.

**Limite, dite** : le relais dort davantage entre deux fenêtres (le H24 l'était déjà par la semaine,
fiche 141) ; une session pour ses notes, partie à 98 %, peut encore toucher le mur si sa réponse
coûte plus d'un point.
