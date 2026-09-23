# Politique de sécurité

## Signaler une vulnérabilité

**N'ouvrez pas d'issue publique** pour une faille de sécurité. Utilisez le signalement privé de
GitHub : onglet [Security](https://github.com/shazamifius/GlucoseGit/security), puis
« Report a vulnerability ».

Indiquez si possible :

- ce que permet la faille, et ce qu'elle expose ;
- comment la reproduire, avec le fichier en cause s'il y en a un ;
- la version concernée (le commit, ou la release) et votre système.

Un accusé de réception arrive en général sous quelques jours, puis des nouvelles de la
correction.

## Versions suivies

| Version | Correctifs de sécurité |
|---|---|
| **Glucose Rust**, branche `main` | oui |
| **Glucose Tauri**, branche `tauri-v1.0.1` et releases `v0.2.0` à `v1.0.2-beta.1` | **non** : cette version est figée |

## Ce qu'il faut surveiller

Glucose est une application de bureau, sans compte ni serveur. Ce qu'elle reçoit de
l'extérieur passe par quatre portes :

- **les fichiers `.glucose`**, qu'on peut recevoir de quelqu'un d'autre. Rien de ce qu'un
  document contient ne doit pouvoir faire exécuter quoi que ce soit : seuls les liens
  `http://` et `https://` s'ouvrent, et une tuile de fichier montre le fichier dans
  l'explorateur sans jamais le lancer ;
- **les images**, dont le décodage lit des octets qui peuvent être hostiles ;
- **le réseau** : sous Windows, une image glissée depuis un navigateur est téléchargée par
  Glucose lui-même, par la pile réseau de Windows (WinHTTP). C'est le seul moment où Glucose
  se connecte à Internet ;
- **le presse-papiers et le glisser-déposer**, qui apportent du contenu venu d'autres
  applications.
