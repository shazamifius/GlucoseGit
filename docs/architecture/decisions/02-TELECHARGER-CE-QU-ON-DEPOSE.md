# 02 — Télécharger ce qu'on dépose : WinHTTP, et aucune caisse de plus

> La fiche [`05`](../05-STANDARDS-DE-CODE.md) § 8.2 exige une note par dépendance. Celle-ci
> n'ajoute **aucune caisse** : elle nomme un recoin de plus de `windows`, déjà présente
> ([`decisions/01`](01-WINDOWS-POUR-LE-DEPOT-NATIF.md)). Mais elle engage autre chose qu'une
> dépendance, et c'est pour cela qu'elle s'écrit : **Glucose se connecte à Internet.**
>
> **Date** : 2026-09-23 · **Portée** : `crates/glucose-desktop`, cible `cfg(windows)`.

---

## La décision, et elle est de l'utilisateur

La note 01 se félicitait d'avoir **évité** le réseau : le navigateur télécharge, Glucose lit
un flux. C'était vrai pour les pages qui livrent l'image. La session du 23/09 a montré que
Pinterest n'en livre **jamais** : dix dépôts sur dix portaient des adresses, un fragment de
page, ou des données à lui — aucun ne portait l'image.

La question a été posée, et tranchée en ces termes : *« l'objectif ultime et final de Glucose,
c'est qu'il puisse complètement télécharger depuis Internet de lui-même »* — et *« dans sa
qualité optimale »*.

## Ce que Glucose télécharge, et rien d'autre

* **Ce que l'utilisateur a glissé**, et seulement au moment où il le glisse. Aucune requête
  de fond, aucune télémétrie, aucune adresse que le dépôt ne portait pas — ou que la page
  déposée ne déclarait pas comme son image (`og:image`).
* **`http` et `https` seulement** — `sources::decouper` refuse `file:`, `javascript:` et les
  adresses qui portent un identifiant.
* **Des octets d'image seulement** : une réponse qui ne commence pas comme un PNG, un JPEG, un
  GIF, un WebP, un BMP ou un HEIF ne s'écrit pas. Au plus 256 Mo — la borne du dépôt.
* **Hors de la boucle d'images**, sur un fil à part : la charte interdit qu'une image attende.

## Ce qu'on prend

| Ce qu'on prend | Pourquoi |
|---|---|
| `Win32_Networking_WinHttp` | `WinHttpOpen`, `Connect`, `OpenRequest`, `SendRequest`, `ReceiveResponse`, `QueryHeaders`, `QueryDataAvailable`, `ReadData`, `SetTimeouts`, `CloseHandle` |

## Combien de lignes pour le remplacer

Un client HTTP est cent lignes ; **TLS ne se réécrit pas.** Les alternatives étaient `ureq`
avec `rustls` et ses certificats — une vingtaine de caisses —, ou `reqwest` — une centaine.
WinHTTP est la pile de tous les programmes de Windows : ses certificats sont ceux du système,
son proxy celui de l'utilisateur, ses correctifs de sécurité arrivent par Windows Update.
Écrire la nôtre ne serait pas du contrôle, ce serait une faille de plus à tenir.

## Ce qu'il tire transitivement

**Rien** : `winhttp.dll` est une bibliothèque du système, liée par `windows` comme les autres.

## Comment on s'en défait

Tout ce qui décide est dans `plateforme/sources.rs`, pur et testé partout ; l'orchestration
dans `plateforme/rapatrier.rs`, sans une ligne de Windows ; et WinHTTP dans
`plateforme/telechargement_windows.rs`, derrière `plateforme::telecharger(url, limite)`.
Sur une autre plateforme, cette fonction rend une erreur et le dépôt retombe sur son lien.
Porter Glucose sur macOS demandera `NSURLSession` au même endroit, et zéro appelant ne
changera.

## Ce qui a été vérifié, et comment

* les décisions — l'ordre des candidats, l'image qu'une page déclare, la ponctuation d'une
  adresse, la signature d'une image — par des tests qui ne touchent pas au réseau ;
* le téléchargement lui-même, **sur la machine**, par `examples/essai_rapatriement.rs`, sur
  quatre épingles que l'utilisateur avait réellement glissées : quatre originaux rapatriés, de
  512 × 512 à 2048 × 2048, en 0,8 à 1,5 s. Le premier essai a rapporté une **icône** : la page
  en porte des dizaines, et le code prenait la première. C'est en regardant l'image, pas le
  journal, qu'on l'a vu.

---

**Retour** : [`00-INDEX.md`](../00-INDEX.md)
