**Glucose {version}** — Glucose, réécrit en Rust.

**Ce qui change depuis la 2.0.3-beta.3**

* **De près, le texte redevient net bien plus vite** : une grande lettre se dessine désormais par
  son contour, au lieu d'une image d'elle entière qu'il fallait refaire à chaque zoom. Vu de
  près, l'écran entier se refait jusqu'à trois fois plus vite — le texte, les titres des
  membranes, les formules. Au-delà de ×32, les lettres se dessinaient aussi trop petites : c'est
  réparé.
* **On zoome bien plus près** : jusqu'à ×256 (c'était ×20).
* **Au téléphone, quand Glucose se ferme sans prévenir**, le journal technique — si tu l'as
  accepté — dit désormais **pourquoi** Android l'a fermé (la mémoire, un plantage, un gel) et
  ce que le lancement suivant a retrouvé. Les longues sessions partent enfin : elles étaient
  refusées pour leur longueur, et c'étaient celles qui comptaient le plus.
* **Au téléphone, si Android relance Glucose pendant qu'il vivait encore en arrière-plan**,
  Glucose repart proprement au lieu de se refermer aussitôt.
* **Si Glucose est relancé pendant que l'ancien finissait de se fermer**, le travail sans nom
  que l'ancien tenait encore n'est plus sauté : il revient de lui-même dès qu'il est libre.
* **La palette de commandes** : `Ctrl+Maj+P`, ou au clic droit sur le vide (deux touchers au
  téléphone). Chaque commande de Glucose, cherchée par son nom — sans se soucier des accents —,
  avec son raccourci ; la liste défile à la molette, au pavé, aux flèches et au doigt.
* **Les raccourcis sont désormais une donnée** : un fichier texte, une ligne par raccourci —
  la fondation des presets à venir. Rien n'a changé pour toi.

**À savoir avant d'installer**

* **Au téléphone, une prudence** : sur deux téléphones, un tableau de plusieurs milliers
  d'images s'est fermé tout seul, et le travail n'est pas revenu au lancement suivant. **La
  cause n'est pas encore connue** : cette version la fera dire. Pour l'instant, au téléphone,
  préfère plusieurs documents de quelques centaines d'images à un seul très grand, et ne
  désinstalle pas Glucose si cela t'arrive : ce qui a été écrit est peut-être encore là.

| système | fichier |
|---|---|
| Windows 10 et 11 | `Glucose_{version}_x64-setup.exe` — installé pour l'utilisateur seul, sans droits d'administrateur |
| Linux, toute distribution | `Glucose_{version}_amd64.AppImage` |
| Debian, Ubuntu | `Glucose_{version}_amd64.deb` |
| Fedora | `Glucose-{version}-1.x86_64.rpm` |
| NixOS | `nix run github:shazamifius/GlucoseGit` |
| Android 5.0 et après | `Glucose_{version}_android.apk` — au doigt : deux touchers sur le vide ouvrent le menu, un appui long fait le clic droit, deux doigts touchés annulent, trois rétablissent |

**Sous Windows**, un avertissement peut paraître (« Windows a protégé votre ordinateur ») :
Glucose n'est pas signé par un certificat payant. « Informations complémentaires », puis
« Exécuter quand même ».

**Sous Android**, ouvrir l'APK depuis le téléphone : il demande d'autoriser l'installation
d'applications inconnues pour l'application qui l'ouvre (Fichiers, Drive, le navigateur), puis
peut dire qu'elle n'est pas vérifiée — elle n'est pas sur le Play Store. « Installer quand même ».
Les versions suivantes arrivent d'elles-mêmes.

**Avec Glucose Tauri ou une version précédente déjà installée**, la mise à jour arrive
d'elle-même, par la fenêtre habituelle. Les documents et les images restent à leur place.

Les fichiers `.sig` et `latest.json` servent la mise à jour automatique.
