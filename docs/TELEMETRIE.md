# Le journal technique de Glucose

Glucose peut envoyer, **si tu le veux**, le journal technique de tes sessions. Ce journal permet
de voir comment Glucose se comporte sur des machines que son auteur ne possède pas : un vieux
téléphone, une tablette, un autre système. Cette page dit exactement ce qui part, et ce qui ne
part jamais. Le code est public : tout ce qui est écrit ici se vérifie.

## Rien ne part sans ton accord

Au premier lancement, Glucose te pose la question, une seule fois. Si tu réponds « non », rien ne
change dans le programme. Tu peux changer d'avis à tout moment : clic droit sur le canevas, puis
**Journal technique…**.

## Ce qui part

À chaque lancement, et seulement si tu as dit oui, Glucose envoie le journal des sessions
précédentes. On y trouve :

* le temps que chaque image a pris à dessiner, geste par geste (déplacer la vue, zoomer, glisser
  un nœud…) ;
* la version de Glucose, le système (Windows, Linux, macOS, Android) et le type de processeur ;
* l'état de la batterie, quand l'appareil en a une ;
* comment la session a fini : fermée normalement, plantée (et dans quel module du système), ou
  gelée ;
* l'état du pavé tactile, pour comprendre une panne de navigation.

Un identifiant tiré au hasard relie les sessions d'une même installation. Il ne dit rien de
toi.

## Ce qui ne part jamais

Le contenu de tes documents, leurs noms, tes images, tes textes, tes fichiers, ton nom, ton
adresse électronique. Les lignes du journal ne peuvent porter que des nombres et des mots
choisis à l'avance dans le code de Glucose. Le serveur refuse tout envoi qui contient autre
chose.

**Ton adresse IP n'est pas gardée** : le serveur ne la lit pas, et sa base n'a aucune place pour
elle. Seul le jour de réception est noté, pas l'heure.

## Voir ce qui part

Clic droit sur le canevas, puis **Voir ce qui part** : le dossier du journal s'ouvre. Chaque
fichier y est une session, une ligne par évènement. Ce sont **exactement** ces lignes qui partent,
sans rien ajouter.

Le dossier se trouve ici :

* sous Windows : `%LOCALAPPDATA%\Glucose\boite-noire\` ;
* sous Linux et macOS : `~/.local/state/glucose/boite-noire/`.

## Tout effacer

Réponds « non » à **Journal technique…**. Tout ce qui est parti de ton installation est effacé
du serveur, et un nouvel identifiant est tiré : ce qui partirait un jour ne serait plus relié à
ce qui a été effacé.

## Où vont les données

Sur un serveur Cloudflare Workers qui appartient à l'auteur de Glucose. Personne d'autre que lui
n'y a accès, et rien n'est revendu ni partagé. Le code du serveur est dans
[`outils/telemetrie/`](../outils/telemetrie/).
