# 137 — La sélection en équerres, et le reste du canevas flouté (Q12)

> Le 24/11/2026, à partir de 2 h, session du relais (compte b, sa semaine à 27 %). Sa consigne au
> relais (VISION § 9) ; la passation du compte a (23 h 21) : Q12 dans `main`, puis TEXT-FIT-3, puis
> l'archive. Sa réponse finale à Q12 (VISION § 8, sous l'image D) : *« les carrés avec le flou, mais
> alors au moins 60 % moins que ce qu'il y a actuellement »*.

## 1. Le dernier fichier de sabotages jamais joué : `detourage-121`, 25 sur 25

Le compte a l'avait lancé à 23 h 24 et fut coupé au mur après dix sabotages (tous tombés), **sans
restaurer** : l'arbre détaché gardait le onzième (`modele.rs`, « un tenseur se lit avant d'être
écrit »). Remis au propre (`git checkout`), les quinze restants rejoués d'un fichier tiré du premier
(`detourage-121.json`, rangs 10 à 24), sous `CARGO_PROFILE_DEV_OPT_LEVEL=1` : **15 sur 15**, donc
**25 sur 25**, chacun nommant son épreuve tombée. La cible optimisée est retirée.

**Le piège payé** : une session coupée au mur pendant `saboter.py` laisse son sabotage dans l'arbre.
La session suivante regarde `git -C ~/.cache/glucose-bancs/sabotage-arbre status` avant tout.

Et deux `pointeur.py` du pilote vivaient depuis trois et quatre heures, orphelins : leur compositeur
était mort sans que `piloter.sh arreter` soit appelé (des sessions coupées, elles aussi). Arrêtés.
