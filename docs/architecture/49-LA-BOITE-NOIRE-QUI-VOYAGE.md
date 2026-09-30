# 49 — La boîte noire qui voyage

> **Rôle de ce document.** La partie C du plan de la fiche [`46`](46-LA-SUITE-DANS-L-ORDRE.md),
> commencée le 30/09 : que la boîte noire de la fiche [`45`](45-VOIR-SANS-POSSEDER-ET-LA-BOITE-NOIRE.md)
> dise **pourquoi** une session s'est arrêtée, puis qu'elle **voyage** jusqu'à lui — avec l'accord
> de chacun. **Écrite au fil de l'eau.**
>
> **Date** : 2026-09-30.

---

## 0. En une page

| | ce qui change pour lui |
|---|---|
| **Les plantages hors de Rust** | une session qui s'arrêtait « sans rien dire » se dit désormais, sous Windows, **« plantée dans nvoglv64.dll (violation d'accès à la mémoire) »** ou **« gelée, puis fermée par le système »** — lu dans ce que Windows a noté, sans caisse, sans code qui tourne pendant le plantage, sans jamais lire la mémoire du programme (§ 1) |
| **Ce qui voyagera** | conçu, pas écrit : éteint par défaut, « voir ce qui part », des lots chiffrés, un identifiant qu'on renouvelle (§ 2) |
| **Le serveur** | **sa décision** — sa box NixOS derrière un nom gratuit, ou un hébergement gratuit ; mon avis : sa box (§ 3) |
| **Une trouvaille pour la partie F** | Windows a noté quatre plantages de Glucose sur sa machine depuis le 21/09 : tous des **épreuves**, trois dans `vulkan-1.dll` au même octet (§ 4) |

---

## 1. Les plantages hors de Rust : ce que le système a vu

La boîte noire savait trois fins : fermée proprement, tombée sur une panique, **arrêtée sans rien
dire** — tuée, plantée dans un pilote, gelée puis fermée, ou l'appareil éteint. Ce dernier cas est
celui qui comptera le plus chez ses testeurs, dont les cartes graphiques et les pilotes sont
inconnus.

**Le plan prévoyait un processus témoin** (`crash-handler`, `minidumper`). **Windows fait déjà ce
travail** : son rapporteur d'erreurs note chaque plantage dans le journal Application — le module,
le code, le décalage, le numéro du processus et l'heure de sa création — et chaque gel qu'il a
fermé. La boîte noire le relit au lancement suivant, pour la session d'avant et elle seule. Le
détail et la comparaison sont dans la note [`decisions/06`](decisions/06-CE-QUE-LE-SYSTEME-A-VU.md) ;
l'essentiel :

* **aucune caisse**, **aucun code qui tourne dans un programme déjà abîmé**, **aucune mémoire lue**
  — un minidump en contient, donc ce qu'il écrivait ;
* la session se reconnaît **sans seuil** : même numéro de processus, créé au plus tard au début de
  la session, tombé au plus tôt à ce début ;
* ce qui s'écrit : le module et sa version — **un nom de fichier, jamais un chemin** —, le code, le
  décalage. Au lancement, la console dit par exemple : `[Glucose] la session précédente : plantée
  dans nvoglv64.dll 32.0.16.1074 (violation d'accès à la mémoire, c0000005), après 12 min 3 s`.

**Ce qui le tient** : quatre épreuves de la reconnaissance, quatre de la boîte noire (plantée,
muette sans rien du système, gelée, la ligne en JSON), la conversion des heures de Windows vérifiée
sur un vrai événement ; **huit sabotages tombent**. Et **sur les machines de GitHub**, de bout en
bout : un processus d'épreuve tombe pour de vrai — une exception à notre code, `0xE0474C55`, le
rapporteur en silence — et le lecteur doit retrouver ce processus et ce code dans le journal.

**Ailleurs qu'à Windows**, rien encore : une session reste « arrêtée sans rien dire ». Chaque
système a son registre (le journal du noyau sous Linux, les rapports de diagnostic du Mac,
`ApplicationExitInfo` sous Android) ; la même porte, une voie par système.

## 2. Ce qui voyagera — la conception, avant le code

Les règles sont celles de la fiche 44 § 2 ; voici comment elles se tiennent :

1. **Éteint par défaut.** Au premier lancement, une question claire, en français simple : ce qui
   est recueilli (les coûts des images, la machine, la façon dont une session a fini), pourquoi
   (voir Glucose sur des machines qu'il ne possède pas), pour combien de temps. « Non » ne change
   rien au programme. Le choix se reprend à tout moment.
2. **Voir ce qui part.** La boîte noire est déjà un fichier lisible par session
   (`%LOCALAPPDATA%\Glucose\boite-noire\`) ; un bouton l'ouvre, et ce qui partira est
   **exactement** ces lignes.
3. **Ce qui part ne peut pas porter un mot de l'utilisateur** — c'est déjà vrai par le type des
   lignes (fiche 45 § 2.4), et vrai des deux seules lignes qui portent un texte : le fichier source
   d'une panique, le nom d'un module (§ 1).
4. **Des lots**, une session close à la fois, au lancement suivant — jamais pendant qu'on dessine.
   **HTTPS**, par la même porte que la mise à jour (`plateforme::telecharger`, qui apprendra
   l'envoi).
5. **Un identifiant tiré au hasard**, que la personne peut renouveler, et l'effacement sur demande.
   **Le serveur ne garde pas l'adresse IP.**
6. Une **page publique** dans le dépôt, qui dit exactement tout cela ; le code étant public, chacun
   peut le vérifier.

**La télémétrie de Glucose Tauri** partait en clair (`http`) vers l'adresse IP brute de sa box
(fiche 44 § 2). Glucose Rust ne la reprend pas : la bascule y met fin.

## 3. Le serveur — sa décision

Rien ne s'écrit de l'envoi tant que l'adresse n'est pas décidée : une adresse gravée dans des
programmes installés doit durer. **Aucun argent** (fiche 46 § 2) :

| | **sa box NixOS**, un nom gratuit | **Cloudflare Workers**, gratuit |
|---|---|---|
| le nom | un sous-domaine gratuit de **deSEC** (`…dedyn.io` — une association européenne, DNSSEC) | `…workers.dev`, fourni |
| le HTTPS | **Caddy** et Let's Encrypt, par le défi DNS de deSEC — aucun port 80 à ouvrir | fourni |
| chez lui | ouvrir le port 443 de sa box vers la machine NixOS ; quelques lignes de configuration NixOS | rien |
| qui voit les données | **lui seul** | Cloudflare (société américaine), qui termine le chiffrement |
| l'adresse de sa box | publique, comme aujourd'hui avec la télémétrie de Tauri | cachée |
| la limite | sa connexion | 100 000 requêtes par jour, largement assez pour vingt testeurs |

**Mon avis : sa box.** Il a dit de la vie privée qu'elle lui *« importe énormément »* : c'est la
seule voie où personne d'autre que lui ne voit ce que ses testeurs envoient, et elle ne dépend du
bon vouloir d'aucune entreprise. Le prix est une ouverture de port et une configuration que
j'écrirai entièrement — lui n'aura qu'à l'appliquer. Cloudflare est la voie sans entretien ; elle
reste possible plus tard sans rien changer côté Glucose, si l'adresse est un nom à lui (deSEC)
qu'on fait pointer où l'on veut.

## 4. Une trouvaille pour la partie F

Windows a noté **quatre plantages de Glucose** sur sa machine, les 21, 23, 24 et 25/09 — tous des
**programmes d'épreuve** (`glucose_desktop-….exe`, ceux de `cargo test`), jamais l'application,
chacun environ cinq secondes après son lancement : trois dans **`vulkan-1.dll` au même décalage
exact** (`0x45a34`), un dans le pilote OpenGL de NVIDIA. C'est très probablement « l'épreuve de la
carte qui tombe au hasard » de la fiche 35 § 4 — **à confirmer** : une épreuve graphique qui fait
tomber le processus entier, sans doute en refermant la carte. Le même octet trois fois en fait un
défaut reproductible, pas du hasard.

## 5. Les sources

Celles de la note [`decisions/06`](decisions/06-CE-QUE-LE-SYSTEME-A-VU.md) ; et :

* deSEC, [*Free Secure DNS*](https://desec.io/) et [la configuration d'un client dynDNS](https://desec.readthedocs.io/en/latest/dyndns/configure.html)
  — les sous-domaines gratuits `dedyn.io`.
* Caddy, [`dns.providers.desec`](https://caddyserver.com/docs/modules/dns.providers.desec) — le
  certificat Let's Encrypt par le défi DNS, sans port entrant pour lui.
* Cloudflare, [*Workers — pricing*](https://developers.cloudflare.com/workers/platform/pricing/) —
  100 000 requêtes par jour au plan gratuit.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
