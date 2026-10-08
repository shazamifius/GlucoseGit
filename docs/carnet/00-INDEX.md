# Le carnet de bord de Glucose Rust

> **Ce qu'est ce dossier.** Les fiches écrites session après session, du 10/09 au 30/09/2026 :
> plans, mesures, décisions, et ce que la mesure a démenti. **Elles ne se réécrivent pas** : une
> correction s'ajoute plus loin et se date. **Le code les cite par leur numéro** (« fiche 22 § 5 ») :
> on ne renumérote jamais, on n'efface jamais.
>
> **Pour savoir où en est le projet, ne commencez pas ici** : lisez [ÉTAT](../ETAT.md),
> [SUITE](../SUITE.md), [ARCHITECTURE](../ARCHITECTURE.md) et [MÉTHODE](../METHODE.md). Le carnet
> sert quand le code cite une fiche, ou quand on veut le raisonnement d'un choix.

**Statut** : **foi** = fait encore autorité · *hist.* = historique, à lire pour comprendre, pas
pour décider · *→* = remplacée par le document indiqué.

---

## L'état des lieux et les règles (10/09)

| # | fiche | statut |
|---|---|---|
| 01 | [Audit du code Rust](01-AUDIT-CODE-RUST.md) — 33 constats R-xx au commit `7a13c86` | *hist.* |
| 02 | [Architecture cible](02-ARCHITECTURE-CIBLE.md) — crates, dix lois L1-L10, format v2 | *hist.* (les lois restent citées) → [ARCHITECTURE](../ARCHITECTURE.md) |
| 03 | [Parité fonctionnelle](03-PARITE-FONCTIONNELLE.md) — 279 fonctionnalités de Tauri inventoriées | *hist.* : l'inventaire sert, les états sont périmés |
| 04 | [Roadmap](04-ROADMAP.md) — treize phases | *hist.* → [SUITE](../SUITE.md) |
| 05 | [Standards de code](05-STANDARDS-DE-CODE.md) — R1, R2, § 1 à 9 | **foi**, sauf ses seuils : les cliquets (80/600) l'emportent |

## La cible : Glucose Tauri, relevé au pixel

| # | fiche | statut |
|---|---|---|
| 06 | [Rendu visuel et design system](06-SPEC-RENDU-VISUEL-DESIGN-SYSTEM.md) — couleurs, grille, poignées, flèches | **foi** |
| 07 | [Animations et interactions](07-SPEC-ANIMATIONS-ET-INTERACTIONS.md) — durées, courbes, PICK-1, SNAP-1 | **foi** |
| 08 | [Nœuds et fonctionnalités](08-SPEC-FONCTIONNALITES-CANVAS-ET-NOEUDS.md) — chaque sorte de nœud | **foi** |
| 09 | [Systèmes métier et persistance](09-SPEC-SYSTEMES-METIER-ET-PERSISTANCE.md) — undo, format, domaines, collaboration | **foi** |
| 10 | [Panneaux et interface](10-SPEC-VISUELLE-PANNEAUX-ET-UI.md) — chaque panneau, captures dans [`screens/`](screens/) | **foi** |

## Les plans successifs (septembre)

| # | fiche | statut |
|---|---|---|
| 11 | [Plan de marche](11-PLAN-DE-MARCHE.md) — la charte confrontée aux fiches 02 et 04 | *hist.* |
| 12 | [Plan d'exécution](12-PLAN-D-EXECUTION.md) — cinq vagues, la règle S | *hist.* |
| 13 | [Plan de correction](13-PLAN-DE-CORRECTION.md) — quatorze défauts vus à la main | *hist.* |
| 14 | [L'état réel, et tout ce qui reste](14-ETAT-ET-RESTE.md) (16/09) | *hist.* → [ÉTAT](../ETAT.md) |
| 15 | [Plan de performance](15-PLAN-DE-PERFORMANCE.md) | *hist.* |
| 16 | [Adaptativité](16-ADAPTATIVITE.md) — voies mesurées, bornes qui suivent la machine | *hist.* (principes dans les mémoires) |
| 18 | [Cent quarante images par seconde](18-PLAN-R-ET-D.md) — le plan de R&D du rendu | *hist.* |
| 21 | [Les voies](21-LES-VOIES.md) (21/09) — deux moteurs, un arbitre, la place qui reste | *hist.*, la fondation des deux voies |

## Les sessions de la fluidité (18-24/09)

| # | fiche | ce qu'elle a trouvé |
|---|---|---|
| 17 | [18/09](17-SESSION-DU-18-09.md) | REPORT-1, CASCADE-1, COUT-1 ; « un compteur jamais lu vaut zéro » |
| 19 | [19/09](19-SESSION-DU-19-09.md) | le judder venait de l'horloge ; pas de synchronisation verticale ; le tempo |
| 20 | [20/09](20-SESSION-DU-20-09.md) | la salissure ; deux cercles vicieux ; un seul cœur sur seize composait |
| 22 | [21/09](22-SESSION-DU-21-09.md) | le fond et les lueurs sur la carte graphique ; le texte comme composant |
| 23 | [21/09 soir](23-SESSION-DU-21-09-SOIR.md) | les deux cartes graphiques ; trois corrections refusées par la mesure |
| 24 | [22/09](24-SESSION-DU-22-09.md) | l'instrument était aveugle (tri par médiane) ; COMPOSANT-2 |
| 25 | [22/09 soir](25-SESSION-DU-22-09-SOIR.md) | dépôt depuis un navigateur, recadrage, `Ctrl+B` |
| 26 | [23/09](26-SESSION-DU-23-09.md) | le gel avait un nom : la bascule de carte |
| 27 | [L'image et sa provenance](27-L-IMAGE-ET-SA-PROVENANCE.md) | Pinterest en pleine résolution ; SauceNAO, pHash — **le chantier de la provenance reste ouvert** |
| 28 | [23/09 soir](28-SESSION-DU-23-09-SOIR.md) | le liseré de `Ctrl+B` ; le seuil de pHash se calcule |
| 29 | [La session suivante](29-LA-SESSION-SUIVANTE.md) | dossier de pistes |
| 30 | [23/09 nuit](30-SESSION-DU-23-09-NUIT.md) | de près, au repos, en vagues ; Trans-domaines retiré |
| 31 | [La longue session lue](31-LA-LONGUE-SESSION-LUE.md) | ORNEMENTS-2, VRAM-1 |
| 32 | [La mémoire par étages](32-LA-MEMOIRE-PAR-ETAGES.md) | ETAGES-1 à 4 |
| 33 | [Le filet, et l'histoire](33-LE-FILET-ET-L-HISTOIRE.md) | BORDURES-5 ; l'enregistrement à la git, conçu |
| 34 | [La tache sur la marge](34-LA-TACHE-SUR-LA-MARGE.md) | BORDURES-6 ; TEMPO-2 |
| 35 | [La frange et le vol](35-LA-FRANGE-ET-LE-VOL.md) | BORDURES-7 ; VOL-2 (van Wijk et Nuij) |

## Vers la V2 (24-30/09)

| # | fiche | ce qu'elle a fait |
|---|---|---|
| 36 | [La route vers la V1](36-LA-ROUTE-VERS-LA-V1.md) | neuf phases → [SUITE](../SUITE.md) |
| 37 | [Le document, et le fil qui dessine](37-LE-DOCUMENT-ET-LE-FIL-QUI-DESSINE.md) | le journal, l'histoire en ajout seul, la Time Machine, l'import de Tauri, MEMB-1 et 2 |
| 38 | [La forme, et les onglets](38-LA-FORME-ET-LES-ONGLETS.md) | les membranes par une loi ; les onglets ; **§ 8 : les membranes à discuter** |
| 39 | [Le registre de Tauri](39-LE-REGISTRE-DE-TAURI.md) | **foi** : ses dix-huit relevés, à tenir à jour |
| 40 | [Les flèches](40-LES-FLECHES.md) | l'aspect de Tauri, la barre d'options, les ancres de texte |
| 41 | [La flèche qui se voit, et la frappe](41-LA-FLECHE-QUI-SE-VOIT-ET-LA-FRAPPE.md) | PICK-2, PLACEMENT-1 ; **§ 14 : références pour les rideaux** |
| 42 | [Son compte rendu](42-SON-COMPTE-RENDU-ET-LES-FLECHES-QUI-CONTOURNENT.md) | LaTeX fidèle ; les flèches qui contournent |
| 43 | [La version, et le repos](43-LA-VERSION-ET-LE-REPOS.md) | VERSION-1, ENVOI-1, GLISSER-1, SNAP-4 |
| 44 | [La route vers toutes les machines](44-LA-ROUTE-VERS-TOUTES-LES-MACHINES.md) | la boîte noire, la vie privée, la bascule → [SUITE](../SUITE.md) |
| 45 | [Voir sans posséder, et la boîte noire](45-VOIR-SANS-POSSEDER-ET-LA-BOITE-NOIRE.md) | la CI sur six systèmes ; la boîte noire ; NixOS ; COLLER-2 |
| 46 | [La suite, dans l'ordre](46-LA-SUITE-DANS-L-ORDRE.md) | la passation du 29/09, sa contrainte d'argent → [SUITE](../SUITE.md) |
| 47 | [La sûreté de ses données](47-LA-SURETE-DE-SES-DONNEES.md) | le crible de chaque chemin qui écrit ; plus rien dans le dossier temporaire |
| 48 | [La mise à jour et la bascule](48-LA-MISE-A-JOUR-ET-LA-BASCULE.md) | l'installeur, Linux, la répétition, la publication, sa clé — **§ 15.4 : publier** |
| 49 | [La boîte noire qui voyage](49-LA-BOITE-NOIRE-QUI-VOYAGE.md) | les plantages vus par Windows ; l'envoi conçu ; **§ 3 : le serveur, sa décision** |
| 50 | [Le toucher : Android et l'iPad](50-LE-TOUCHER-ANDROID-ET-L-IPAD.md) | **le plan du chantier 3** de la suite |
| 51 | [Ses retours sur la V2](51-SES-RETOURS-SUR-LA-V2.md) | la souris instantanée, le lot de nœuds, l'image au clic droit, Ctrl+N, le mode référence, le repos — **§ 9 : ses essais** |
| 52 | [La carte de l'écran](52-LA-CARTE-DE-L-ECRAN.md) | ECRAN-1 (le lag : un balancier entre deux cartes), POIGNEE-1, le glisser qui se tait, le pincement — **§ 7 : ses essais** |
| 53 | [Pinterest et le pavé](53-PINTEREST-ET-LE-PAVE.md) | DEPOT-WEB-6 (la page demandée compressée, un repli qui le dit, « Remplacer par l'image ») ; le pavé par *Direct Manipulation* ; la sélection en cinq directions ; `F` ; Pinterest la copie d'abord ; **transformer une sélection entière (§ 10)** — **§ 6 : ses essais** |
| 54 | [La nuit du pavé et d'Android](54-LA-NUIT-DU-PAVE-ET-ANDROID.md) | la similitude entière des doigts, sans bascule ; GROUPE-1 ; la boîte noire suit le pavé ; la télémétrie des deux côtés (Cloudflare) ; Glucose compile pour Android, avec les doigts — **§ 9 : ce qui attend sa parole** |
| 55 | [Le téléphone et la 2.0.2](55-LE-TELEPHONE-ET-LA-2-0-2.md) | Glucose sur son Redmi 9 ; le serveur de la boîte noire en ligne ; la clé Android ; la 2.0.2-beta.1 en brouillon ; la skill du téléphone ; le rail — **§ 4 : ce qui reste de sa liste** |
| 56 | [Le partage et la vie de l'application](56-LE-PARTAGE-ET-LA-VIE-DE-L-APPLICATION.md) | VIE-1 (la surface qu'Android reprend) ; PARTAGE-1 (Glucose dans « Partager ») ; le double toucher et « Ajouter des images… » ; QUESTION-1 (la question dessinée) et VUE-1 (le refus donné à sa place) ; DOCUMENTS-1 (les documents du téléphone) — **§ 7 : le clavier d'abord** |
| 57 | [Le clavier, l'appui long et les documents](57-LE-CLAVIER-L-APPUI-LONG-ET-LES-DOCUMENTS.md) | CLAVIER-1 à 3 (le clavier du téléphone, miroir de la saisie ; l'état d'avant l'envoi ; la ligne en vue) ; APPUI-1 (l'appui long, le délai du système, la question qui répond au relâchement) ; DOCUMENTS-2 (renommer, dupliquer, supprimer) ; ce que son téléphone a dit par la télémétrie |
| 58 | [Les questions dessinées, le rail, les emojis, la mise à jour d'Android](58-LES-QUESTIONS-DESSINEES-LE-RAIL-ET-LES-EMOJIS.md) | POPUP-1 (toutes les questions dessinées, au clavier aussi ; une suite au lieu d'un retour ; DIAL-5, la seule boîte avant la fenêtre) ; les noms du rail, colonne après colonne ; GESTES-1 proposé ; EMOJI-1 (Noto Emoji en repli d'Inter) ; MAJ-ANDROID-1 (l'APK confié à `PackageInstaller`) ; le journal technique lisible (`journal-technique`, un lisez-moi en colonnes, la clé d'une session) ; GESTES-1 écrit |

## Les décisions — **foi**

Une note par dépendance ou par choix structurant, dans [`decisions/`](decisions/) :
[01](decisions/01-WINDOWS-POUR-LE-DEPOT-NATIF.md) la caisse `windows` pour le dépôt ·
[02](decisions/02-TELECHARGER-CE-QU-ON-DEPOSE.md) télécharger ce qu'on dépose ·
[03](decisions/03-LIRE-LES-DOCUMENTS-DE-TAURI.md) lire les documents de Tauri ·
[04](decisions/04-CHERCHER-LES-MISES-A-JOUR.md) chercher les mises à jour ·
[05](decisions/05-LE-RESEAU-HORS-DE-WINDOWS.md) le réseau hors de Windows ·
[06](decisions/06-CE-QUE-LE-SYSTEME-A-VU.md) ce que le système a vu des plantages ·
[07](decisions/07-LE-PAVE-PAR-DIRECT-MANIPULATION.md) le pavé par *Direct Manipulation* ·
[08](decisions/08-ANDROID-PAR-GAMEACTIVITY.md) Android par `GameActivity` ·
[09](decisions/09-LE-PARTAGE-PAR-JNI.md) le partage vers Glucose, par `jni` ·
[10](decisions/10-LES-EMOJIS-PAR-NOTO-EMOJI.md) les emojis, par Noto Emoji.

La prochaine fiche porte le numéro **59**.
