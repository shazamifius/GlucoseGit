# 148 — REVENIR-2 : un document revient dans sa salle (24/11, relais, compte b)

Suite de la fiche 147 § 6 (le noyau, `Store::revenir_dans_la_salle`) et du plan de la fiche 146 § 4 (« un
document, une salle »). Sa note : *« objectif es de ne JAMAIS avoir de doublon […] je perds la sync, j'ai
bien le fichier mais seul sans mise à jour »* (`text-c3kr-2251`).

## 1. Ce qui change pour lui

* **« Quitter » garde le code** : le document sort de la salle, mais s'en souvient (dehors). Son prochain
  **« Partager » y revient, sous les mêmes codes**, au lieu d'ouvrir une seconde salle — la cause des
  doublons.
* **Un arrêt net revient aussi** : avant, le lancement disait « arrêté net » et ne rejoignait rien — *le
  fichier, mais seul, sans mise à jour*. Maintenant, tout document resté dedans revient au lancement.
* **La fermeture n'attend plus** le retour de ses gestes (jusqu'à 30 s, RETOUR-1, fiche 144 § 2) : partis
  ou non, ils sont dans le fichier, et le retour les rejoint.

## 2. Comment

* **Le souvenir** (`collaborer/reprise.rs`, `canevas-partages.json`) : le code, **une borne basse** (un
  numéro de la salle que le fichier contient à coup sûr) et **dedans**. Écrit sous de nouvelles clés
  (`retenu`, `dedans`) : un Glucose d'avant y lirait `confirme` comme un numéro exact et rejouerait deux
  fois un déplacement ; l'ancien souvenir se lit encore (`confirme`, `null` → 0).
* **La borne ne dépasse jamais le disque** : si elle était plus haute que le fichier, la fusion prendrait
  pour « changé ici » ce que la salle a changé, et le défairait. Elle monte ainsi (`tenir_la_borne`, à
  chaque tour du partage) : quand la salle a confirmé plus loin, ce qui attend se consigne, une
  synchronisation part **sans qu'on l'attende** (`Ecriture::synchroniser_plus_tard`, le même ordre que la
  synchronisation bloquante) ; revenue, le numéro qu'elle couvrait se retient. Une seule en vol : le
  disque donne son propre rythme, aucune constante. Une borne trop basse ne perd rien : au pire, un
  élément que la salle a changé deux fois est dit en conflit (la salle l'emporte, c'est la bonne valeur).
* **L'entrée `Revient`** (`collaborer/revenir.rs`) : le fil part de 0 ; la base et chaque geste sont
  gardés (`Retour`) ; à « à jour », `terminer_les_gestes_en_cours`, puis `store.revenir_dans_la_salle`,
  et l'on participe. Sans base (une salle vide), on le dit et l'on attend encore. `Entree::attend()`
  réunit `AttendLaBase` et `Revient` partout où l'on n'est pas encore dedans (le fil qu'on ne coupe pas,
  le curseur, les photos).
* **La version d'ici d'un conflit, gardée à côté** (`Store::revenir_dans_la_salle`, `poser_a_cote`) :
  pour chaque photo ou note que la salle a emportée, une copie d'ici, sous un nom neuf de ma marque, à
  droite du nœud de la salle, de l'écart d'un nœud dupliqué (`ECART_D_UNE_COPIE`, la constante de
  Ctrl+D, désormais partagée) — dans le même geste que la fusion (un seul Ctrl+Z). Pas les flèches (une
  flèche tient à ses deux bouts) ni un nom de tableau : dits seulement. Épreuves : REVENIR-1 (la photo
  d'Alice à 41 gardée à 161) et `revenir_2_…` (la note, pas la flèche) ; 7 sabotages sur 7
  (`revenir-148-noyau.json`).
* **Supprimés** : `rejoindre_au_numero` (le numéro exact), `ARRETE_NET`, `attendre_le_retour_de_mes_gestes`.

Épreuves (`collaborer/reprise/tests.rs`, 6 neuves) : fermé proprement, il revient depuis sa borne ; un geste
en attente n'empêche rien ; un arrêt net revient ; « Quitter » garde le code, dehors, et « Partager » y
revient ; **jusqu'au bout** (la salle arrive par le fil d'épreuve : base, geste de Bob, à jour — le texte
de Bob et celui écrit pendant le retour sont là, le sien part comme son geste) ; **la borne attend le
disque** (scribe retenu : le souvenir reste à 0 ; relâché, il monte à 2). Sabotages :
`outils/sabotages/revenir-148.json`.

**L'essai réel** (`outils/porte/essai/essai_revenir_pilote.sh`, deux vrais Glucose hors écran, la salle
locale de `wrangler dev`, 24/11 à 18 h 50) : Alice pose, partage, Bob rejoint ; Alice quitte ; Bob pose
« Bob », Alice, dehors, « ici » ; Alice repartage — *« revient dans sa salle (au moins au n° 1) »*, puis
*« revenu au n° 3, 0 conflit »*, aucun code neuf ; Alice arrêtée, Bob pose « B3 », Alice relancée — *« au
moins au n° 4 »* (la borne était montée), *« revenu au n° 6 »*. Les deux canevas finissent identiques
(Alice, ici, Bob, B3), le curseur d'Alice relancée chez Bob. Piège : le premier « Partager » pose d'abord
la question du nom (IDENTITE-2) — l'essai la valide. L'arrêt du pilote ferme la fenêtre proprement : le
vrai arrêt net (la borne seule) n'est éprouvé que par `revenir_la_borne_attend_le_disque`.

## 3. Ce qui reste

1. Dans Collaborer, un document dehors qui a sa salle pourrait dire « Revenir dans sa salle » plutôt que
   « Partager ».
2. « Changer les codes » (la salle qui se ferme) attend la connexion Cloudflare ; la liste des accès
   (Collaborer en trois portes).
3. **À voir chez lui**, une fois son Glucose recompilé : relancer son Glucose pendant un partage — le
   canevas de loin suit encore.

## 4. Les jetons (sa priorité)

**La requête 0 d'une session du relais : 26 940 jetons** (cette session, le journal de sa conversation),
contre 46 000 à 48 000 avant `SANS_OUTILS` (fiche 147) : le socle a perdu 42 %. L'histoire des jauges
tenait 5 intervalles à 18 h, tous en heure de pointe : la relecture de poids 0 y disperse le moins
(15 %), mais il en faut une dizaine, dont des heures creuses, avant de remonter le seuil.

**Ce que la session écrit, décomposé** (18 h 56, 90 requêtes) : ~80 000 jetons écrits, dont ~40 000 pour
les outils (le code, les fiches, les commandes : 114 000 caractères), ~1 000 de texte, et donc ~40 000 de
pensée (le journal de la conversation ne garde pas la pensée : déduite). Avec les poids 2 (ajouté) et
5 (écrit) de `mesurer_jetons.py`, la pensée pèse **~25 %** de la jauge de cette session, l'ajouté ~49 %.
Un effort de pensée plus bas en gagnerait au plus une dizaine de pour cent : un levier secondaire, à lui
proposer seulement si l'histoire des jauges confirme les poids.
