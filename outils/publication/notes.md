**Glucose {version}** — Glucose, réécrit en Rust.

**Ce qui change depuis la 2.0.3-beta.4**

* **De près, le texte est net dès la première image, à tout zoom jusqu'à ×256.** Un texte plus
  grand que l'écran n'est plus une image qu'il fallait refaire (et qu'on voyait floue en
  attendant) : la carte graphique dessine chaque lettre depuis sa forme, à chaque image — les
  lettres, les formules, l'italique, les emojis, un passage qu'une flèche fait briller, la
  sélection pendant qu'on écrit. Une image de près coûte environ une milliseconde, contre une
  vingtaine avant.
* **Un très long texte reste fluide de près** : un nœud de plusieurs milliers de mots se
  parcourait de plus en plus lentement à mesure qu'il s'allongeait ; il coûte désormais la même
  chose qu'un texte court. Et un tableau plein de textes travaille moins à chaque image.
* **Tes raccourcis se changent dans Glucose** : `Ctrl+Maj+P`, « Changer un raccourci… », choisis
  la commande, appuie sur le nouvel accord. Glucose dit ce que cet accord faisait avant. On peut
  aussi retirer un raccourci, rendre celui de Glucose, ou les rétablir tous. Ils sont gardés dans
  un fichier `raccourcis.txt`, dans le dossier de Glucose — lisible, et modifiable à la main.

**À savoir avant d'installer**

* **Au téléphone, une prudence** : sur deux téléphones, un tableau de plusieurs milliers
  d'images s'est fermé tout seul, et le travail n'est pas revenu au lancement suivant. **La
  cause n'est pas encore connue** : le journal technique de cette version et de la précédente la
  dira. Pour l'instant, au téléphone, préfère plusieurs documents de quelques centaines d'images
  à un seul très grand, et ne désinstalle pas Glucose si cela t'arrive : ce qui a été écrit est
  peut-être encore là.

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
