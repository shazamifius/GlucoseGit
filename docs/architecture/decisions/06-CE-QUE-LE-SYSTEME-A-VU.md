# 06 — Les plantages hors de Rust : lire ce que le système a vu

> La fiche [`44`](../44-LA-ROUTE-VERS-TOUTES-LES-MACHINES.md) § 1.1 prévoyait, pour les
> plantages qu'aucune panique ne voit — un pilote de carte graphique qui tombe —, un **processus
> témoin** : `crash-handler` et `minidumper`. Cette note en décide autrement, pour Windows d'abord.
> Elle n'ajoute **aucune caisse** : deux recoins de `windows`, déjà présente.
>
> **Date** : 2026-09-30 · **Portée** : `crates/glucose-desktop`, `plateforme::journal`,
> `boite_noire::plantage` · **Fiche** : [`49`](../49-LA-BOITE-NOIRE-QUI-VOYAGE.md).

---

## Ce que le système sait déjà

Windows note chaque plantage d'une application dans son journal Application — le rapporteur
d'erreurs y écrit **« Application Error », événement 1000** : le module fautif et sa version, le
code de l'exception, le décalage dans le module, **le numéro du processus et l'heure de sa
création**. Et chaque gel qu'il a fermé : **« Application Hang », 1002**. Tout utilisateur
interactif peut lire ce journal (`wevtutil gl Application` : `(A;;0x3;;;IU)`).

Relu sur sa machine le 30/09, sans rien y écrire : **quatre plantages de Glucose** notés depuis
le 21/09 — tous des programmes d'épreuve, trois dans `vulkan-1.dll` au même décalage exact, un
dans le pilote OpenGL de NVIDIA (`nvoglv64.dll`). Le format est celui que cette note lit.

## Pourquoi pas le processus témoin

| | un processus témoin (`minidumper`) | lire le journal du système |
|---|---|---|
| caisses | `crash-handler`, `minidumper`, `minidump-writer` et leurs dépendances | **aucune** |
| code qui tourne pendant le plantage | oui — dans un programme déjà abîmé | **aucun** |
| ce qu'il garde | la mémoire du programme — donc ce que l'utilisateur écrivait | **des nombres et deux noms** (module, version) |
| un second processus à chaque lancement | oui | non |
| ce qu'on apprend | tout, pile comprise | **où** (module, décalage) et **quoi** (le code) |

Le témoin en dit plus, mais pour des utilisateurs dont la vie privée compte avant tout (fiche 44
§ 2) — et un rapport qui contient de la mémoire se demande, il ne part pas de lui-même —, **où et
quoi** est ce qui manque aujourd'hui : la boîte noire ne dit que « arrêtée sans rien dire ». Si
un jour « où et quoi » ne suffit plus, le témoin reste possible, en plus.

## Reconnaître la session, sans seuil

Le nom d'une session porte son début et son processus (`session-<début>-<processus>.jsonl`). Un
numéro de processus se réutilise, mais jamais par deux processus vivants à la fois : un événement
de ce numéro, d'un processus **créé au plus tard au début** de la session et **tombé au plus tôt à
ce début**, ne peut être que le sien.

## Ce qu'on en garde

Le module et sa version — **un nom de fichier, jamais un chemin** : n'est gardé qu'un nom fait de
lettres, de chiffres et de `.-_+` —, le code, le décalage, et s'il s'agissait d'un gel. La session
d'avant se dit « plantée dans nvoglv64.dll 32.0.16.1074 (violation d'accès à la mémoire) » ou
« gelée, puis fermée par le système », et la session suivante l'écrit.

## Ce qu'on prend

| Ce qu'on prend | Pourquoi |
|---|---|
| `Win32_System_EventLog` | `EvtQuery`, `EvtNext`, `EvtCreateRenderContext`, `EvtRender`, `EvtClose` |
| `Win32_Security` | exigée par le type des valeurs rendues (`EVT_VARIANT`) ; déjà compilée pour `wgpu` |
| pour les épreuves seulement : `Win32_System_Diagnostics_Debug` | faire tomber un vrai processus (`RaiseException`) |

## Ailleurs qu'à Windows

Rien encore : une session y reste « arrêtée sans rien dire ». Chaque système a son registre —
Linux le journal du noyau ou `systemd-coredump` selon la distribution, le Mac ses rapports de
diagnostic, Android `ApplicationExitInfo` — : la même porte (`plateforme::journal`), une voie par
système, quand on s'y mettra.

## Ce qui a été vérifié, et comment

* **Reconnaître** : le sien, celui d'avant, celui d'après, un autre numéro, le plus tardif de
  deux, un gel, un nom qui porte un chemin — par des épreuves sur des événements écrits comme le
  journal les rend ; **huit sabotages tombent**.
* **La boîte noire** : une session lâchée sans être close, rouverte avec un témoin qui a vu son
  plantage, se dit plantée, et la suivante l'écrit ; sans rien du système, elle reste muette ; un
  gel se dit gel.
* **Le lecteur, de bout en bout, sur les machines de GitHub** : un processus d'épreuve lève une
  exception que personne n'attrape (le code `0xE0474C55`, à nous seuls) ; le lecteur doit
  retrouver **ce** processus et **ce** code. Seulement là : un plantage fait écrire à Windows un
  rapport dans ses propres dossiers, et ce n'est pas sur sa machine qu'on le provoque.
* **Ce que le premier essai a appris** (`7f043d0`) : le processus est bien tombé sur notre code,
  mais la machine de GitHub **n'a rien noté** en deux minutes. Plutôt que de le supposer éteint,
  la vérification **affiche** le rapporteur de la machine, puis le règle comme sur un poste
  ordinaire — allumé, qui note (`Disabled` et `LoggingDisabled` à 0, leurs valeurs par défaut) —,
  sans fenêtre (`DontShowUI`) : `outils/rapporteur_ordinaire.ps1`. L'épreuve ne règle plus rien
  elle-même, et dit ce qu'elle a trouvé si elle tombe encore.
* **Ce que le deuxième essai a appris** (`197b596`) : **mon hypothèse était fausse** — le
  rapporteur de la machine était allumé et journalisait. La cause est dans le processus :
  *« a child process inherits the error mode of its parent »*, et avec `SEM_NOGPFAULTERRORBOX`,
  *« the system does not invoke Windows Error Reporting »* (SetErrorMode). Le programme qui pilote
  la machine de GitHub pose ce mode (une autre intégration continue, Cirrus, a eu le même défaut) ;
  l'épreuve lance désormais son processus **avec le mode par défaut** (`CREATE_DEFAULT_ERROR_MODE`).
  **Chez ses utilisateurs, rien de tel** : lancé par l'Explorateur, l'updater de Tauri ou son
  installeur, Glucose a le mode par défaut — NSIS ne pose que `SEM_NOOPENFILEERRORBOX |
  SEM_FAILCRITICALERRORS` (lu dans sa source, `exehead/Main.c`).
* **Le troisième essai** (`a3174e5`, run `36687729226`) : **vert**, les neuf tâches — sur la
  machine Windows de GitHub, un vrai processus tombe, Windows le note, le lecteur retrouve ce
  processus et ce code, et la boîte noire le reconnaît. Les deux échecs d'avant, sur la même
  machine, prouvent que l'épreuve y tourne au lieu de se sauter.

## Les sources

* Microsoft, [*Consuming Events*](https://learn.microsoft.com/en-us/windows/win32/wes/consuming-events)
  — le sous-ensemble de XPath, `timediff` ; [`EvtQuery`](https://learn.microsoft.com/en-us/windows/win32/api/winevt/nf-winevt-evtquery),
  [`EvtRender`](https://learn.microsoft.com/en-us/windows/win32/api/winevt/nf-winevt-evtrender).
* Les gabarits des deux événements, lus sur la machine : `(Get-WinEvent -ListProvider 'Application
  Error').Events` et `'Application Hang'` — `ProcessId` (`win:HexInt32`), `ProcessCreationTime` et
  `StartTime` (`win:HexInt64`, un `FILETIME`).
* Microsoft, [*WER Settings*](https://learn.microsoft.com/en-us/windows/win32/wer/wer-settings) —
  `Disabled`, `LoggingDisabled`, `DontShowUI`, et leurs valeurs par défaut ;
  [`SetErrorMode`](https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-seterrormode)
  — `SEM_NOGPFAULTERRORBOX` et l'héritage du mode par les processus enfants.
* Cirrus CI, [*Don't start tasks with SEM_NOGPFAULTERRORBOX set*](https://github.com/cirruslabs/cirrus-ci-docs/issues/1068) ;
  NSIS, [`Source/exehead/Main.c`](https://github.com/kichik/nsis/blob/master/Source/exehead/Main.c).
* Embark Studios, [`crash-handling`](https://github.com/EmbarkStudios/crash-handling) — le témoin,
  écarté pour l'instant.

---

**Retour** : [`00-INDEX.md`](../00-INDEX.md)
