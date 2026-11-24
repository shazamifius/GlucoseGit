# 147 — Ce que la jauge compte ; les témoins de la beta.6 (24/11, le relais, compte b)

> Sa réponse URGENTE sur `relais-q-h24-quotas` : *« la perte de token es ma prioriter actuel !!! […] je ne
> shouaite pas switch sur sonnet »*. Ma réponse dans son Glucose : `relais-rep-jauge`.

## 1. La relecture ne compte probablement pas

La fiche 142 a pesé les jetons au prix de l'API (relire le cache 0,1 ; l'écrire 2 ; écrire 5) : 78 % pour la
relecture, d'où une conversation neuve à 300 000 jetons. **Mais la jauge d'un abonnement n'est pas le prix
de l'API**, et Anthropic ne publie pas ce qu'elle compte.

**Ce soir, un intervalle tranche** : deux sondes `claude -p` (17 h 30 - 17 h 32) ont presque tout ajouté et
presque rien relu — 14 requêtes, 0,95 M relus, 78 000 ajoutés, 13 000 écrits — et la jauge de 5 h a pris
**6 points**. Un point de jauge, en équivalents (ajouté × 2 + écrit × 5 + relu × w) :

| intervalle | w = 0 | w = 0,1 |
|---|---|---|
| six intervalles de la journée (12 h - 17 h 25, relevés de la fiche 142 et du journal) | 33 à 57 k, 43 en moyenne | 129 à 171 k, 154 en moyenne |
| les sondes (17 h 30 - 17 h 32) | **37 k** | **53 k** |

Avec w = 0, les sondes coûtent ce que coûte le travail ; avec w = 0,1, trois fois moins — ce n'est pas
l'arrondi au point près (± 17 % sur 6 points). **Probablement : la relecture ne compte pas, ou presque** ;
ce qui compte est ce qui entre de neuf (le cache écrit) et ce que le modèle écrit, pensée comprise. Un relevé
public (anthropics/claude-code#45756, avril 2026 : 1 500 requêtes, six fenêtres) trouve la même chose : w = 0
disperse le moins (34 %, contre 102 % à 0,1). **À confirmer** : un seul intervalle décisif ; les six autres,
trop semblables entre eux, ne départagent pas (dispersion 16 % à w = 0, 10 % à w ≥ 0,05).

**Ce que ça changerait** : si la relecture est gratuite, chaque conversation neuve coûte son démarrage —
~20 000 jetons de socle écrits, puis ~70 000 de ses notes et du code relus (la session de 17 h : de 47 000 à
120 000 jetons en 28 requêtes) — soit **4 à 5 % de la fenêtre de 5 h**, payés toutes les 25 minutes à
300 000 jetons. Le seuil devrait alors remonter (vers 700 000), à l'inverse de la fiche 142. **Pas changé ce
soir** : une décision qui défait une autre attend sa preuve.

**Les heures de pointe** (annoncées le 26/03/2026 : en semaine, 5 h - 11 h à San Francisco, **14 h - 20 h à
Paris**, la fenêtre de 5 h se vide plus vite ; la semaine n'est pas touchée) : aucun écart visible dans les
intervalles de 14 h 00 - 14 h 36 et 17 h 00 - 17 h 25. Le H24 bute sur la semaine, pas sur les 5 h.

## 2. Fait

* **L'histoire des jauges** (`relais.garder_l_histoire`) : la barre d'état écrit une ligne dans
  `~/.local/state/relais/jauges-histoire.jsonl` chaque fois qu'une jauge bouge — l'heure, le compte, la
  conversation, les deux pourcentages, la remise des 5 h. Rien de plus à payer : la barre d'état tourne déjà.
* **`outils/relais/mesurer_jetons.py [pas]`** : croise cette histoire avec les journaux des conversations ;
  par intervalle d'au moins `pas` points, les requêtes, le relu, l'ajouté, l'écrit, l'heure de pointe ; puis,
  pour chaque poids de la relecture, le coût d'un point et sa dispersion. **Le poids qui disperse le moins
  est ce que l'abonnement compte.**
* **Les outils qu'une session du relais ne sert jamais** (`SANS_OUTILS`, `--disallowedTools`) : Artifact (le
  plus long mode d'emploi de tous), la question en menu, le réveil, l'avis, le rapport de revue. Agent reste
  (lire une page web). Le socle relu à chaque requête était de 46 000 à 48 000 jetons (requête 0 des
  sessions de 13 h 10, 17 h 00, 17 h 25) : **à relever sur la prochaine session.**
* Épreuves : 77 sur 77 ; `outils/sabotages/relais-147.json`, 5 sabotages sur 5 tombent. Veilleur relancé à
  17 h 33.

## 3. Les témoins de la beta.6

La vérification GitHub de `b4d257b` (run 45436174373) est tombée sur Linux, Windows et Mac : **deux
empreintes de rendu** (la scène sélectionnée, le menu contextuel) — le bouton « Écrire sur la flèche » de
GESTE-19 au bout de la barre d'une flèche. La session de la fiche 146 n'avait éprouvé que ses épreuves, pas
la suite entière. Capturées ici, mêmes empreintes que GitHub sur les trois systèmes, regardées : c'est le
bouton voulu. **Doute** : avec plusieurs flèches choisies, le bouton écrit sur la première
(`selected_arrows().first()`) — à trancher avec la parité (19).

## 4. Pièges payés

* **Une sonde `claude -p` lancée d'une session du relais hérite `RELAIS=1`** : le crochet de fin de tour l'a
  relancée trois fois (le journal du relais le montre, 17 h 31). Sans `RELAIS`, lancée dans le dépôt, elle
  prendrait la main (un bail de 35 minutes, `parole`). Une sonde : `env -u RELAIS`, hors du dépôt — et elle
  coûte (les deux de ce soir : 6 % de la fenêtre de 5 h). L'histoire des jauges les remplace.
* **Avant de lancer la vérification GitHub d'une publication, la suite entière** : une épreuve d'empreinte
  tombe sur un geste visible.

## 5. Ce qui reste

1. **Trancher le poids de la relecture** (`mesurer_jetons.py`, quand l'histoire tient une dizaine
   d'intervalles, dont des débuts et des fins de conversation) ; s'il est sous 0,03, **remonter le seuil**
   (`loi.SEUIL_JETONS`) en rejouant la simulation de la fiche 142 avec ce poids.
2. **Ce que le modèle écrit** pèse alors autant que ce qui entre (la session de 17 h : 47 % pour l'écrit, la
   pensée surtout) : l'effort de pensée (`--effort`) est le levier suivant — sa parole « Opus reste » ne dit
   rien de l'effort, à lui proposer chiffré.
