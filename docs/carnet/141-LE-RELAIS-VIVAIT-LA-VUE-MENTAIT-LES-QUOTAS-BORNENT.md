# 141 — Le relais vivait : la vue mentait, les quotas bornent

> Le 24/11/2026 vers 9 h, compte b, dans son VS Code. Sa parole : *« assure-toi que le système de
> script H24 de Claude fonctionne, car je pense pas qu'il fonctionne actuellement, et depuis hier soir
> — il faut réparer ça et permettre à Claude de coder H24 sur Glucose »* ; puis, pendant le
> diagnostic : *« peut-être que tout fonctionnait, j'avais juste pas visuellement sur VS Code l'update
> graphique »*. Les deux sont vrais, chacun à moitié.

## 1. La nuit, lue au journal (`~/.local/state/relais/journal.txt`)

| De → à | Qui | Pourquoi |
|---|---|---|
| 21 h 51 → 22 h 28 | personne | en retrait devant sa session à lui |
| 22 h 28 → 23 h 01 | compte b | jusqu'au mur de ses 5 h |
| 23 h 01 → 23 h 25 | compte a | jusqu'au mur de **sa semaine** (pleine jusqu'au 25/11 à 0 h) |
| 23 h 25 → 2 h 00 | **personne** | les deux comptes au mur ; b se remet à 2 h |
| 2 h 00 → 3 h 31 | compte b | jusqu'au mur de ses 5 h |
| 3 h 31 → 7 h 00 | **personne** | les deux comptes au mur ; b se remet à 7 h |
| 7 h 00 → 8 h 59 | compte b | 13 commits (ARBITRE-6, DEPLACER-1, IDENTITE-2, MELER-1…), puis sa passation : il venait d'écrire à sa session à lui |

**Le relais n'a rien raté** : chaque trou tombe sur deux comptes au mur, et chaque remise relance une
session dans la minute. Mais **2 h 28 de travail sur 9 h 09** de nuit.

## 2. La vue mentait — réparée

Chaque VS Code (`.vscode/tasks.json`, local à ce poste) ne suivait que la session de **son** compte
(`relais-$RELAIS_COMPTE`). Le VS Code du compte a est resté vide de 23 h 25 à 9 h, pendant que le
compte b travaillait. Il a donc cru le relais en panne.

**`outils/relais/voir.sh`** : la session du relais qui vit, quel que soit son compte (il n'y en a
qu'une à la fois) ; sans elle, ce que dit le ❤️ (pourquoi, et jusqu'à quand). La tâche de VS Code le
relit toutes les 20 secondes : un changement de la vue vaut désormais sans recharger VS Code. Trois
épreuves (`test_relais.py`, `Vue`), trois sabotages (`relais-132.json`, « RELAIS-VUE »).
**Prend effet au prochain démarrage de ses deux VS Code** (ou « Developer: Reload Window ») :
l'ancienne boucle tourne encore dans leurs terminaux.

## 3. Ce que permettent deux comptes **Pro**

Les deux comptes sont des abonnements **Pro** (`subscriptionType`), le plus petit des abonnements
payants. Mesures tirées des conversations du relais (les jetons de chaque appel) :

* **Une fenêtre de 5 h ≈ 1 h 30 de travail** continu d'Opus 5.5 : de 2 h 00 à 3 h 31, 338 appels
  ont rempli 100 % de la fenêtre ; de 7 h 00 à 8 h 49, 294 appels en ont rempli 73 %.
* **Une semaine ≈ 7 à 8 fenêtres** (le compte b : 49 % de sa semaine pour environ 3,7 fenêtres
  pleines depuis le 23/11 à 16 h) — soit **11 à 12 h de travail par compte et par semaine**,
  **22 à 25 h pour les deux, sur 168**. Le H24 demande environ **sept fois** plus.
* **La suite probable** : le compte a reprend le 25/11 à 0 h ; le b n'a plus qu'environ quatre
  fenêtres avant le 30/11 à 16 h. Vers le 27/11, les deux semaines risquent d'être pleines : environ
  trois jours sans aucune session, jusqu'au 30/11.

Sa parole du 23/11 le disait déjà : *« du code H24 dans les limites des sessions »*.

## 4. Où part la consommation : la relecture

À chaque geste, Claude relit toute sa conversation. Les sessions du relais relisent **380 000 à
470 000 jetons par appel** en moyenne, et jusqu'à 784 000, parce qu'une conversation n'est renouvelée
qu'à 90 % de la mémoire du modèle (sa règle du 23/11, fiche 134). La session de 2 h a relu
158 millions de jetons, contre 0,76 million de jetons neufs écrits en cache et 0,36 million de
réponse.

Anthropic ne publie pas comment un abonnement compte la relecture. Deux estimations :

* **Au barème public** (une relecture coûte un dixième d'un jeton neuf, une réponse cinq fois), la
  relecture fait environ **85 %** du coût de cette session.
* **Selon la mesure d'un utilisateur** ([limitcast](https://limitcast.dev/blog/claude-code-usage-limits/) :
  1 % de sa fenêtre de 5 h ≈ 166 000 jetons neufs — entrée, écriture en cache et réponse, sans poids
  — ou 11,2 millions relus, soit une relecture à 1/67 ; un seul compte, plan et modèle non dits),
  elle en ferait environ **les deux tiers**.

**Le renouvellement qui coûte le moins** : une conversation neuve paie son départ D (lire les
documents : environ 100 000 à 200 000 jetons écrits en cache, au poids w), puis chaque appel relit
un contexte qui grandit d'environ g ≈ 2 000 jetons (au poids r de la relecture). Le coût moyen par
appel est le plus bas après n* = √(2·w·D / (r·g)) appels. Selon les deux estimations, cela fait
renouveler la conversation vers **200 000 à 450 000 jetons**, au lieu de 900 000. Gain estimé, une
fois retranchés les départs plus fréquents : **de +25 % à deux fois plus de travail** par fenêtre.
C'est loin du facteur sept, et c'est **à mesurer** : le pourcentage de fenêtre par heure, avant et
après.

## 5. Ce qui attend sa parole

1. **Renouveler la conversation plus tôt**, contre sa règle du 23/11 (« 90 % partout »). Argument :
   pour les 5 h et la semaine, aller à 90 % ne gaspille rien ; pour la mémoire du modèle, 90 % est
   l'endroit où chaque geste coûte le plus cher. Contrepartie : plus de passations, donc le fil
   perdu plus souvent.
2. **Sonnet pour le relais** : environ trois fois moins cher qu'Opus selon une source tierce (non
   officielle), mais moins sûr sur un code aussi exigeant (au bit près, la charte).
3. **Un abonnement plus grand** : Max 5x ou 20x, soit 5 ou 20 fois Pro par fenêtre de 5 h selon
   Anthropic. La semaine n'est pas publiée, et une refonte des offres fin octobre 2026 reste à
   vérifier sur claude.com/pricing. C'est son choix, c'est son argent.

Sources : [Anthropic, le plan Max](https://support.claude.com/en/articles/11049741-what-is-the-max-plan) ·
[limitcast, la relecture mesurée](https://limitcast.dev/blog/claude-code-usage-limits/) (copiée sur
[DEV](https://dev.to/samstull9007/claude-code-usage-limit-reached-how-the-5-hour-and-weekly-limits-work-3mn1)) ·
[heyuan110, Opus contre Sonnet](https://www.heyuan110.com/posts/ai/2026-02-28-claude-rate-limits/) ·
[ITNEXT, la semaine de Max 20x](https://itnext.io/if-you-thought-claude-max-20x-meant-20x-weekly-usage-you-were-wrong-042f70e0035e) ·
[tech-insider, la refonte d'octobre](https://tech-insider.org/?p=53620).
