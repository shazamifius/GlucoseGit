# 47 — La sûreté de ses données

> **Rôle de ce document.** La partie A du plan de la fiche [`46`](46-LA-SUITE-DANS-L-ORDRE.md),
> faite le 29/09 au soir : **chaque chemin qui écrit ses données passé au crible de COLLER-2**,
> diagnostiqué sur des **copies** de ses documents ; **plus rien dans le dossier temporaire du
> système** ; et **le dossier `%LOCALAPPDATA%\Glucose`**, que Glucose Tauri partage, vérifié sur
> le binaire de son désinstalleur. **Écrite au fil de l'eau.**
>
> **Date** : 2026-09-29 · commits `a5e67e8` (le crible) et `c604d9c` (le dossier temporaire).
> **État à la fin** : sous Windows, **1 954 épreuves vertes** (25 de plus), clippy strict à
> zéro, l'application se construit ; après chaque suite d'épreuves, `%LOCALAPPDATA%\Glucose`
> est **identique**, fichier pour fichier, à sa photographie d'avant.

---

## 0. En une page

| | ce qui change pour lui |
|---|---|
| **Ses documents, lus sur copies** | ses 27 documents retrouvés et copiés : ceux de Glucose Rust sont **intacts** — chaque image scellée, et elle se décode ; ceux de Tauri se lisent tous, identiques à la bibliothèque de référence (§ 1) |
| **Le désinstalleur de Tauri** | lu sur son binaire : il ne vide **jamais** `%LOCALAPPDATA%\Glucose` ; ses brouillons ne risquent rien de lui (§ 2) |
| **FIN-1** | une entrée abîmée au milieu d'un document ne détruit plus tout ce qui la suit : c'est mis de côté avant d'être recouvert (§ 4) |
| **SAUVER-1** | enregistrer par-dessus un fichier ne le vide plus avant que le nouveau soit entier ; « Enregistrer sous » ne renomme plus sa copie avant de l'avoir poussée sur le disque (§ 5) |
| **FRAPPE-1** | un texte en cours de frappe dont le document est introuvable au lancement n'est plus effacé : il revient dans une carte neuve (§ 6) |
| **BROUILLON-1** | un travail sans nom ne peut plus se cacher : `Ctrl+O` pose la question, et un brouillon passe avant tout document au lancement (§ 7) |
| **AJOUT-1** | ajouter dans un onglet un document à l'image scellée vide ne reprend plus le vide (§ 8) |
| **COLLER-3** | une image collée n'a plus de fichier : ses octets entrent dans le document **avant** le geste qui la pose (§ 9) |
| **DEPOT-4** | ce qu'une page livre reste en mémoire ; un PDF glissé va dans les Téléchargements (§ 10) |
| **APERCU-5** | un aperçu se nomme par l'empreinte de ses octets : une image collée s'ouvre déjà montrée ; un aperçu abîmé se tait et se refait (§ 11) |

---

## 1. Ses documents, lus sur des copies

**Tous ses documents `.glucose`** ont été cherchés sur tout le disque `C:` (le seul) : **27**,
tous dans son dossier personnel. Chacun a été **copié** dans mon dossier de travail — avec le
magasin d'images de Tauri (`%APPDATA%\com.glucose.app\assets`, 197 fichiers, 188 Mo) et les
dossiers `objects/` de ses trois documents portables — et seuls ces copies ont été lues, par
des outils qui n'écrivent jamais (`verifier_images`, `lire_histoire`, `oracle_tauri`).

**Ses documents de Glucose Rust** — `fuser`, les deux `random photo !!!!`, `testtranslate`,
`testtttttt`, `proteo` : **chaque image est scellée dans son document, et se décode** (254, 481,
429, 386, 386 et 4). Aucune ne dépend plus d'un fichier extérieur. Les trois images de `proteo`
que COLLER-2 avait scellées vides sont revenues, octet pour octet identiques à leurs fichiers.

> `proteo.glucose` pèse 222 Mo pour 4 images : son histoire (329 gestes, 29 jalons) montre des
> onglets importés — « UwU », « tst », « random photo !!!! » — puis supprimés, et des images
> retirées une à une sur plusieurs lancements. Ce sont ses gestes. L'histoire s'écrit en ajout
> seul : ce qui a existé reste dans le fichier, et la Time Machine peut y revenir.

**Ses 16 documents de Glucose Tauri** : notre lecteur les lit **identiques, arbre pour arbre**, à
la bibliothèque de référence d'Automerge, et toutes leurs images se retrouvent — **sauf une
version de juin** de « glucose pour film » (`FILM CAZAL\glucose pour film.glucose.versions\`),
dont 117 images sur 118 manquent au magasin de Tauri ; le document lui-même n'existe plus. C'est
une perte de l'époque de Tauri, que rien ici ne peut rendre.

**Ce que le crible a aussi montré** : son dossier temporaire garde **1 638 images collées
(2,9 Go)** et **172 dépôts web (26 Mo)** que Glucose y avait écrits et n'effaçait jamais.
**Aucun de ses documents n'en dépend plus** — chaque image qu'ils montrent est scellée en eux ;
les autres fichiers viennent sans doute de travail abandonné. Je n'y ai pas touché. Depuis le
§ 9, **plus rien ne s'y ajoute**. Aucune tuile ni carte de ses documents ne pointe
vers un fichier (l'outil le dit désormais aussi).

---

## 2. `%LOCALAPPDATA%\Glucose`, que Glucose Tauri partage — vérifié sur le binaire

Glucose Tauri s'installe « par utilisateur » dans **`%LOCALAPPDATA%\Glucose`** (`glucose.exe`,
`uninstall.exe`), le dossier même où Glucose Rust range ses brouillons, sa boîte noire, ses
aperçus. La question était : **que fait son désinstalleur de ce dossier ?**

Plutôt que de croire le modèle de Tauri ou sa documentation, j'ai lu le **programme compilé** :
le désinstalleur de sa machine, et celui qu'écrit l'installeur de la 1.0.2-beta.1 — la release
« latest », celle que ses cinq utilisateurs ont reçue (son `latest.json` a été lu 37 fois). Un
petit décodeur (`nsis_suppressions.py`, dans mon dossier de travail) décompresse l'en-tête LZMA
de l'exécutable NSIS et liste chaque ordre qui supprime, avec son drapeau. **Les deux sont
identiques** (618 ordres) :

```text
Delete  $INSTDIR\glucose.exe
Delete  $INSTDIR\uninstall.exe
RMDir   $INSTDIR                       simple   ← ne retire le dossier que s'il est VIDE
Delete  raccourcis du menu Démarrer et du bureau
RMDir   $APPDATA\com.glucose.app       RÉCURSIF ← seulement si la case « supprimer les données » est cochée
RMDir   $LOCALAPPDATA\com.glucose.app  RÉCURSIF ← idem
```

**Ses données de Glucose Rust ne risquent rien de ce désinstalleur** : il retire deux fichiers,
puis tente un `RMDir` sans `/r`, qui échoue dès que le dossier contient autre chose (la
documentation de NSIS le dit, § 13). Le modèle actuel de Tauri 2.10.1 fait la même chose.

**Ce qui, en revanche, compte pour la bascule** — et c'est une trouvaille :

* les documents de Tauri gardent leurs images dans **le magasin de Tauri**,
  `%APPDATA%\com.glucose.app\assets` — exactement ce que la case « supprimer les données de
  l'application » efface. **Tant qu'un document de Tauri n'a pas été rouvert et enregistré dans
  Glucose Rust, ce magasin est la seule copie de ses images.** La bascule ne devra jamais le
  toucher, et ses utilisateurs devront l'entendre : ne jamais cocher cette case ;
* le programme de mise à jour de Tauri (lu dans sa source, `tauri-plugin-updater` 2.10.1)
  télécharge le fichier que `latest.json` désigne, **vérifie sa signature**, l'écrit dans le
  dossier temporaire, le lance avec `/P /R /UPDATE /ARGS …` et se ferme. Il n'exige qu'un
  exécutable Windows : l'installeur de Glucose Rust est libre de s'installer où il faut.

**La décision** : les données de Glucose Rust **restent** dans `%LOCALAPPDATA%\Glucose` — aucune
migration, donc aucune chance d'en perdre —, et le **programme** s'installera ailleurs : dans
`%LOCALAPPDATA%\Programs\Glucose`, le dossier que Windows réserve aux programmes installés pour
un seul utilisateur (`FOLDERID_UserProgramFiles`, depuis Windows 7). Le jour de la bascule,
l'installeur retirera de `%LOCALAPPDATA%\Glucose` les deux fichiers de Tauri — ceux, et seulement
ceux, que son propre désinstalleur retire — et laissera le reste. Le détail est la partie B.

---

## 3. Le crible : chaque chemin qui écrit

Tous les appels qui écrivent, renomment ou suppriment un fichier dans le code de l'application
ont été relevés (hors épreuves), puis chaque chemin lu en entier. Les questions de COLLER-2 :
un lecteur peut-il voir un fichier à moitié écrit ? un vide peut-il passer pour une donnée ? une
donnée n'existe-t-elle un moment qu'à un endroit que le système peut vider ? un état abîmé
peut-il déjà exister chez lui, et se répare-t-il ? l'épreuve tombe-t-elle quand on casse ce
qu'elle garde ?

| chemin | verdict | |
|---|---|---|
| geste, `Ctrl+S` (un jalon) | sain | |
| premier enregistrement, par-dessus un fichier | **défaut** | SAUVER-1, § 5 |
| « Enregistrer sous » | **défaut** | SAUVER-1, § 5 |
| ouvrir, et reprendre l'écriture | **défaut** | FIN-1, § 4 |
| brouillons | **défaut** | BROUILLON-1, § 7 |
| texte en cours de frappe | **défaut** | FRAPPE-1, § 6 |
| coller une image | fragile | COLLER-3, § 9 |
| déposer depuis l'explorateur, importer (`Ctrl+I`) | sain : scellé depuis son fichier | |
| déposer depuis un navigateur, rapatrier | fragile | DEPOT-4, § 10 |
| ajouter un document dans un onglet | **défaut** | AJOUT-1, § 8 |
| ouvrir un document de Tauri | sain ; le magasin de Tauri à protéger | § 2 |
| exporter, dernier document, Time Machine | sains | |
| aperçus (un cache) | fragile | APERCU-5, § 11 |

Aucun de ces défauts ne s'était produit chez lui : ses documents le disent (§ 1).

---

## 4. FIN-1 — rien ne se recouvre sans avoir été mis de côté

**Le défaut.** À l'ouverture, la lecture s'arrête à la première entrée qui ne suit pas la chaîne
des sommes ; l'écriture **tronquait** ensuite le fichier à cet endroit. Pour une fin déchirée par
un plantage — quelques kilo-octets —, c'est juste. Mais une seule entrée abîmée **au milieu**
d'un document de deux cents mégaoctets — un secteur du disque, un défaut d'écriture — rendait
« ignoré » tout ce qui la suit, et la troncature le détruisait, avec pour seul avertissement :
*« la fin d'un enregistrement interrompu ignorée »*.

**La règle.** Aucune constante ne sépare « une fin déchirée » d'« une histoire coupée » : on ne
tranche pas, on garde. Ce qu'on va recouvrir est **copié d'abord** dans
`brouillons\recuperation\` (`abime-81234-1790….fin`), poussé sur le disque, et alors seulement le
fichier se tronque. Le fichier d'avant **se reconstitue à l'octet** : ses premiers octets — que
l'ajout seul ne touche plus jamais —, puis ce qui a été mis de côté. Une copie qui échoue
**refuse** la troncature : le document s'ouvre sans être modifié, et ce qu'on y change part dans
un brouillon. L'ouverture le dit : *« la fin d'un enregistrement interrompu (4,0 Kio) mise de
côté dans … »*.

**Ce qui le tient** : un document dont un octet change au milieu de son deuxième geste se rouvre,
s'écrit, et le fichier d'avant se reconstitue exactement ; une mise de côté impossible laisse le
fichier intact ; une fin plus courte que promis ne se met pas de côté à moitié. **Six sabotages
tombent** — dont un d'abord **aveugle** : la vérification « la copie est complète » n'était
jamais éprouvée ; elle a reçu son épreuve, et une copie ratée ne laisse plus de morceau.

**Ce qui reste** : une vraie **récupération** — reprendre la chaîne après l'entrée abîmée, et
rejouer à partir de l'instantané suivant — n'est pas écrite. Elle le pourra : rien n'est détruit.

---

## 5. SAUVER-1 — un seul geste pose un fichier à la place d'un autre

**Les défauts.** Le premier enregistrement d'un document ouvrait le fichier choisi, **le vidait**,
puis y écrivait : choisir un document existant dans le dialogue — « oui, le remplacer » — le
détruisait avant que le nouveau soit sur le disque. « Enregistrer sous » copiait à côté, puis
**renommait avant de pousser la copie sur le disque** : une coupure de courant à cet instant
pouvait laisser sous le nom choisi un fichier incomplet.

**Le remède.** `persist::atomic` devient **le seul endroit** qui pose un fichier à la place d'un
autre : écrire un voisin, le pousser sur le disque, le renommer, puis pousser le dossier. Le
premier enregistrement, « Enregistrer sous », le texte en cours de frappe et les aperçus y
passent. **Le cliquet 11** interdit tout autre `rename` ou `copy` dans le code : c'est la seule
façon de garder une règle qu'aucune épreuve ne peut voir.

**Ce qui le tient** : un **témoin lié en dur** à l'ancien fichier garde l'ancien contenu — écrit
en place, le témoin, qui est le même fichier sous un autre nom, aurait été vidé avec lui ;
« Enregistrer sous » vers une place qu'on ne peut pas prendre ne touche à rien et ne laisse
aucun voisin. **Honnêtement** : l'ordre « pousser, puis renommer » ne se voit qu'en coupant le
courant ; son sabotage passe en silence. Il est tenu par construction, et par le cliquet.

**Trouvé en chemin** : une épreuve d'`atomic.rs` comptait les voisins restés dans son dossier
sans le vider d'abord — une exécution tombée la faisait tomber à toutes les suivantes, et **un
sabotage a paru vu alors qu'aucune épreuve ne l'avait vu**. Elle part désormais d'un dossier vide.

---

## 6. FRAPPE-1 — un texte tapé n'est jamais effacé sans avoir été rendu

**Le défaut.** Au lancement, une saisie — le texte d'une carte en édition, gardé à côté du
document jusqu'à sa validation — **s'effaçait** quand son document n'existait plus : *« il n'y a
plus rien où la rendre »*. C'est pourtant la seule copie d'un texte jamais validé : un document
renommé après un plantage, ou resté sur une clé USB qu'on n'a pas rebranchée, et dix minutes de
frappe disparaissaient. Une saisie illisible s'effaçait aussi.

**Le remède.** Le texte revient **dans une carte neuve** du document qui s'ouvre, la vue posée
dessus, et le compte rendu du lancement le dit : *« Le texte que tu tapais dans « notes.glucose »,
introuvable, est revenu dans une carte neuve — une copie reste dans … »*. Le fichier de la saisie
est **rangé** avec ce qui a été mis de côté — copié entier sur le disque avant que l'original ne
parte —, jamais effacé. Une saisie illisible est rangée aussi, au lancement comme à l'ouverture.

**Une remise en question de ma propre idée.** J'avais d'abord cru qu'une saisie « périmée » — un
geste l'a suivie — pouvait cacher un texte perdu. C'est faux : pendant qu'on tape, **seule la
validation** écrit dans le document ; une saisie suivie d'un geste a donc été validée, son texte
est dans l'histoire, et la rendre ressusciterait ce qu'on a remplacé ensuite. L'épreuve existante
le montrait ; cette règle ne change pas.

**Ce qui le tient** : trois épreuves (le document renommé, la saisie illisible au lancement, et à
l'ouverture) ; **cinq sabotages tombent** — dont un d'abord aveugle : l'ouverture d'un document à
la saisie illisible n'avait pas d'épreuve.

---

## 7. BROUILLON-1 — un travail sans nom ne se cache jamais

**Le défaut.** Quitter un document sans nom par `Ctrl+O` ne posait **aucune question** : son
brouillon restait sur le disque, sans que rien ne le dise. Et le lancement ne rouvrait que **le
plus récent** de ce qui attend : un brouillon plus ancien qu'un document nommé pouvait ne
**jamais** reparaître. Rien n'était effacé — mais il l'aurait cru perdu.

**Le remède.**

* `Ctrl+O` sur un document sans nom qui a du travail pose **la question de la fermeture** (« Oui —
  enregistrer / Non — fermer sans enregistrer / Annuler ») ; rien n'est fermé tant que
  l'ouverture n'a pas réussi. Un document nommé s'écrit geste après geste : il se quitte sans un
  mot, comme avant. Le fichier se choisit d'abord : renoncer au choix ne change rien.
* Au lancement, **le plus récent des brouillons passe avant tout document** — un document nommé
  est en sûreté, et `Ctrl+O` le rouvre ; un brouillon n'a que le lancement pour reparaître —, et
  le compte rendu dit combien d'autres attendent : chacun reparaît à son tour.

**Ce qui le tient** : trois épreuves ; **quatre sabotages tombent**. **Pas éprouvable hors
fenêtre** : le branchement de la question dans `Ctrl+O` — il faut un vrai dialogue.

---

## 8. AJOUT-1 — l'ajout suit la règle de l'ouverture

Ajouter un document dans un onglet reprenait **tel quel** un objet d'image scellé vide (le défaut
de COLLER-2), quand l'ouverture savait déjà le relire depuis son fichier. Deux chemins, deux
règles : l'ajout suit désormais l'ouverture — *un objet vide n'est jamais une image*. Une épreuve
(sur un document abîmé comme le sien), un sabotage qui tombe.

---

## 9. COLLER-3 — une image collée n'a plus de fichier

**Pourquoi.** Une image collée n'a pas de fichier : on lui en écrivait un dans
`%TEMP%\glucose_pasted`, que le scribe relisait pour la sceller. Deux fils se passaient ainsi un
fichier — c'est ce qui a scellé trois images vides le 28/09 —, et tant qu'elle n'était pas
scellée, ce fichier était sa **seule copie**, dans un dossier que Windows vide (l'Assistant
Stockage efface les fichiers temporaires « que les applications n'utilisent pas », § 13). Et
rien ne les effaçait jamais : 2,9 Go chez lui.

**Ce qui est fait.** Sa clé est désormais un nom (`collee:…`), et ses octets une **promesse** :
l'atelier, qui l'encode en PNG sur un fil de fond (COLLER-1), la tient ; le scribe l'**attend à sa
place dans sa file** — qui est aussi celle des gestes. L'image entre donc dans l'histoire **avant
le geste qui la pose**, jamais sans ses octets : un arrêt pendant l'encodage ne laisse ni l'image
ni ce qui la suit — un document cohérent, et l'image encore dans le presse-papiers. Une promesse
que personne ne tiendra est **abandonnée**, et ne retient pas le scribe. Les promesses passent
**devant** les décodages dans l'atelier : `Ctrl+S` peut les attendre.

L'attente d'un fichier qui « paraît après coup » disparaît du scribe : seul le collage en
produisait.

**`Ctrl+C` sur une image** copie sa clé ; le collage la reconnaît désormais, et **duplique**
l'image — qu'un fichier la porte encore ou non. Seul le chemin d'un fichier existant se recollait :
une image collée, une image de Tauri, une photo dont le fichier a disparu devenaient une carte de
texte portant leur clé.

**Ce qui le tient** : dans le fichier, les octets de l'image **précèdent** le geste (lu dans
l'histoire) ; scellée dès l'image suivante, quel que soit le temps de l'encodage ; une promesse
abandonnée ne retient personne — l'épreuve attend sur un fil, avec un délai, pour **tomber** au
lieu de geler si le garde cassait ; le copier-coller duplique. **Cinq sabotages tombent** — dont
un d'abord aveugle : rien ne vérifiait que les promesses passent devant les décodages ; le
rangement d'un travail dans sa file est devenu éprouvable seul.

---

## 10. DEPOT-4 — ce qu'une page livre ne passe plus par le dossier temporaire

Ce qu'un navigateur promet (`FileContents`), la vignette qu'une page pose, et l'image rapatriée
d'une adresse s'écrivaient dans `%TEMP%\glucose_depose` pour redevenir des fichiers. Ils restent
désormais **en mémoire**, dans un reçu :

* **une image** se pose directement depuis ses octets, et se scelle dans le document avant le
  geste qui la pose ;
* **le reste** — un PDF, une archive — devient une tuile qui **mène** à un fichier : il lui faut
  un vrai fichier, qui dure. Il va dans **les Téléchargements** (`FOLDERID_Downloads`), comme si le
  navigateur l'y avait mis, sans jamais écraser un fichier existant (`rapport (1).pdf`) ; le nom
  est réservé, puis le contenu y est posé d'un bloc. Dans le dossier temporaire, la tuile aurait
  fini par ne plus mener nulle part. Hors de Windows, le dossier se lit dans `user-dirs.dirs`, qui
  est traduit (« Téléchargements »).

**Seul le vrai lancement connaît ses Téléchargements** : les épreuves écrivent dans un dossier
temporaire qui n'appartient à personne. Ma première version les donnait dans `habiter`, que
`GlucoseApp::new` appelle aussi — **l'épreuve « une application d'épreuve n'écrit rien chez
l'utilisateur » l'a vu** : chaque épreuve aurait écrit dans ses vrais Téléchargements. Ils se
donnent désormais dans `main.rs`, et nulle part ailleurs.

**Ce qui le tient** : une image livrée se pose sans fichier, puis se scelle ; un PDF va dans les
Téléchargements et un second du même nom prend `(1)` ; un raccourci livré devient un lien.
**Quatre sabotages tombent.** **Pas éprouvable hors d'un vrai glisser** : le pont COM
(`depot_windows`) lui-même.

---

## 11. APERCU-5 — l'aperçu d'une image, nommé par ses octets

Les aperçus (ETAGES-4) se nommaient par **la taille et la date du fichier** que la clé désigne.
Une image collée, une image de Tauri n'en avaient aucun ; une photo dont le fichier avait quitté
le dossier temporaire perdait le sien — elle se redécodait **en entier à chaque ouverture**.

Un aperçu se nomme désormais par **l'empreinte des octets scellés** : une identité qui ne change
jamais, partagée par tous les nœuds et tous les documents qui montrent la même image. Une image
pas encore scellée garde l'ancien nom. Chaque aperçu porte un **sceau** : abîmé, il se tait — il
ne montre jamais de faux pixels — et **se refait** (un aperçu déjà là ne se réécrivait pas : il
serait resté muet pour toujours). Cette version 2 vit dans `apercus\v2` ; **ceux d'avant se
retirent** au lancement — un cache, dont les noms ne désignaient plus rien : 48 Mo chez lui.

**Ce qui le tient** : une image sans fichier s'ouvre déjà montrée à la session suivante ; un
octet changé fait taire l'aperçu ; il se refait ; les anciens se retirent, et rien d'autre.
**Cinq sabotages tombent** — dont un d'abord aveugle (la réécriture de l'aperçu abîmé).

**Ce qui reste** : aucun aperçu ne s'efface quand son document disparaît. Le dossier grandit
d'un écran de pixels par document ; un ménage selon l'usage reste à concevoir.

---

## 12. Ce qu'il faut regarder à l'écran

```text
cargo run --release > sortie-surete.txt 2>&1
```

1. **Rouvrir tes documents** (`proteo`, `testtttttt`…) : tout doit être là. La première
   ouverture refait les aperçus (le dossier d'avant s'est retiré) : c'est normal si les images
   arrivent une seconde plus lentement, **une fois**.
2. **Coller une image** (`Ctrl+V` d'une capture) : elle paraît tout de suite. **`Ctrl+C` sur elle,
   puis `Ctrl+V`** : une copie de l'image, et non une carte de texte.
3. **Glisser une image depuis une page web** (Pinterest compris) : elle se pose comme avant.
   **Glisser un PDF depuis une page** : une tuile ; un double-clic montre le fichier **dans tes
   Téléchargements**.
4. **Un document sans nom** : écrire quelque chose dans un nouveau document, puis `Ctrl+O` et
   choisir un autre document — **la question** doit paraître.
5. Après la session, **ton dossier temporaire** : `%TEMP%\glucose_pasted` ne doit rien avoir reçu
   d'aujourd'hui. Aucun de tes documents ne dépend plus de ses 2,9 Go ; je n'y touche pas.

---

## 13. Les sources

* NSIS, [`RMDir`](https://nsis.sourceforge.io/Reference/RMDir) — *« Without /r, the directory will
  only be removed if it is completely empty »* ; et
  [*Validating $INSTDIR before uninstall*](https://nsis.sourceforge.io/Validating_$INSTDIR_before_uninstall)
  — pourquoi `RMDir /r $INSTDIR` est dangereux.
* Tauri, le modèle [`installer.nsi`](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi)
  — lu dans le CLI 2.10.1 installé chez lui — et
  [`tauri-plugin-updater` 2.10.1](https://crates.io/crates/tauri-plugin-updater/2.10.1) (`updater.rs`,
  `install_inner`) : le fichier téléchargé, vérifié, lancé avec `/P /R /UPDATE /ARGS`.
* Microsoft, [`KNOWNFOLDERID`](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
  — `FOLDERID_UserProgramFiles` (`%LOCALAPPDATA%\Programs`, les programmes d'un seul utilisateur,
  depuis Windows 7) et `FOLDERID_Downloads` ;
  [*Installation Context*](https://learn.microsoft.com/en-us/windows/win32/msi/installation-context).
* Microsoft, [*Manage drive space with Storage Sense*](https://support.microsoft.com/en-us/windows/manage-drive-space-with-storage-sense-654f6ada-7bfc-45e5-966b-e24aded96ad5)
  — l'Assistant Stockage efface les fichiers temporaires que les applications n'utilisent pas ;
  [une discussion Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/321616d1-e6a7-413a-8246-28f88f5ecc4e/storage-sense-configuration-for-deleting-temporary?forum=windows-all)
  sur ses réglages.
* freedesktop.org, [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/) —
  `XDG_DOWNLOAD_DIR`, traduit selon la langue du bureau.
* SQLite, [*Atomic Commit*](https://www.sqlite.org/atomiccommit.html) — pousser sur le disque
  avant de publier ; la leçon que FIN-1 et SAUVER-1 appliquent.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
