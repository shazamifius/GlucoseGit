# Politique de sécurité

## Signaler une vulnérabilité

La sécurité de Glucose est prise au sérieux. **Merci de ne pas ouvrir d'issue
publique** pour une faille de sécurité.

À la place, utilise le signalement privé de GitHub :
**onglet [Security](../../security) → « Report a vulnerability »**
(GitHub Private Vulnerability Reporting).

Merci d'inclure si possible :
- une description de la faille et de son impact ;
- les étapes pour la reproduire ;
- la version / plateforme concernée.

On s'efforce d'accuser réception sous quelques jours et de te tenir informé de la
correction.

## Périmètre

Glucose est une application **desktop offline**. Les points d'attention principaux :
- l'accès au système de fichiers via le backend Rust/Tauri (scan de dossiers,
  lancement de fichiers) ;
- le traitement de contenus importés (fichiers, images, vidéos).

## Versions supportées

Le projet est en développement actif : seule la **dernière version** reçoit les
correctifs de sécurité.
