# 166 — Le relais attend qu'il ait fini d'écrire ; la passation sans récit (25/11, le relais, compte a)

## 1. Sa note

Le 25/11 au matin, dans son Glucose (`membrane-ztqw-2312`, « Gros problème de système script py
automatique pour Claude ») : *« Les sessions Claude s'arrêtent au bout de même pas 3 minutes et sont
inefficaces avec des 100 de crédit brûlés »* ; *« un jour Claude ne s'est pas du tout arrêté à la limite et
j'ai dû intervenir en urgence »* ; *« c'est nul le fait qu'il met les récaps de ce qui s'est passé, ça tu le
vois par toi-même grâce au MCP »*. Sa priorité reste les jetons (24/11).

## 2. Ce que dit le journal

* **8 h 15 et 8 h 18, deux démarrages pour une note** : le compte b, sa semaine à 99 % (la réserve de
  RESERVE-1, fiche 163), lancé sur le seul titre de sa membrane, puis de nouveau trois minutes plus tard sur
  son texte. Le second meurt à 8 h 21, la semaine pleine, sans avoir répondu. Un démarrage lisait ~80 000
  jetons avant son premier geste utile (la session de 8 h 18 : 82 000 en deux minutes) : la consigne, une
  passation de **1 500 mots** allongée de session en session, le mode d'emploi (1 700 mots), son nœud
  désigné, ses notes.
* **10 h 10, 26 « notes » dont 13 rangées** : son autre appareil avait collé dans son archive
  (`board-wiov-333`) treize nœuds anciens, d'un seul geste ; le relais les donnait à lire comme neufs.
* **« Ne s'est pas arrêté à la limite »** : aucune trace au journal — chaque fois, la session est close au
  mur. Question posée dans son Glucose (`relais-rep-gros-probleme`) : quel jour, et l'usage supplémentaire
  payant (« extra usage ») est-il activé sur un compte ? Il ferait continuer Claude au-delà de la limite, à
  ses frais.
* **Le rythme d'un compte Pro** (`jauges-histoire.jsonl`, 24-25/11) : une fenêtre de 5 h pleine prend
  **13 points** de la semaine, en ~2 h 20 de travail ; une semaine ≈ 7,7 fenêtres ≈ 18 h.

## 3. Fait

1. **Un tour sans geste de sa main avant de lancer** (`loi.il_ecrit`, `relais.attendre`) : la règle
   qu'`entendre` suivait déjà pour taper ses notes dans une session vivante vaut maintenant pour en lancer
   une — aucune durée neuve, le tour du veilleur (une minute) en est la mesure.
2. **Un collage n'est pas une note** (`loi.ses_paroles`) : ce qu'il écrit se pose un nœud à la fois ; un
   geste qui en pose plusieurs colle ce qui existait. Un nœud collé qu'il retouche redevient sa parole ; un
   « retire » du même geste s'applique toujours.
3. **La passation se remplace, sans récit** (`TEXTES[TEXTE_PASSATION]`, et son nœud
   `relais-texte-passation` mis à jour) : une ligne datée, puis seulement le non commité, la prochaine étape,
   un piège ; ni ce qui est fait (`git log`), ni ses questions (son Glucose) ; rien de changé : la ligne
   remise à l'heure, au lieu d'une ligne ajoutée en tête.
4. **Le mode d'emploi n'est plus lu à chaque démarrage** (`TEXTES[TEXTE_DEPART]`) : seulement pour un
   travail qui touche au relais ; l'essentiel est déjà dans `REGLES` (`--append-system-prompt`).

3 épreuves (dont 2 adaptées : la session part au tour d'après), **6 sabotages sur 6**
(`outils/sabotages/relais-166.json`) ; le sabotage `SES-NOTES` de `relais-132.json`, que la ligne déplacée
rendait introuvable, réancré (il tombe).

## 4. Pièges payés, ce qui reste

* **Neuf sabotages anciens sont introuvables** (`relais-131` : 1, `relais-132` : 1, `relais-134` : 7) : le
  code qu'ils cassaient a changé avant cette session ; leurs gardes ne sont plus prouvées. À réancrer.
* La réserve d'une fin de semaine ne vaut que si elle tient au moins un démarrage : à 99 %, elle n'en tenait
  pas deux. Avec les corrections 1 et 4, un seul démarrage, plus léger ; aucun seuil de plus.
