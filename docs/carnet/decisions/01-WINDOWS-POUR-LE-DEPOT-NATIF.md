# 01 — `windows` et `windows-core`, pour le dépôt natif

> La fiche [`05`](../05-STANDARDS-DE-CODE.md) § 8.2 exige une note de décision par dépendance,
> répondant à quatre questions. Les voici, pour les deux caisses que DEPOT-WEB-1 ajoute.
>
> **Date** : 2026-09-22 · **Portée** : `crates/glucose-desktop`, cible `cfg(windows)` seulement.

---

## Ce qu'elles font

`windows` est la liaison officielle de Microsoft vers l'API Win32. On en emploie six recoins,
et pas un de plus :

| Ce qu'on prend | Pourquoi |
|---|---|
| `Win32_System_Ole` | `IDropTarget`, `RegisterDragDrop`, `RevokeDragDrop` — la cible de dépôt |
| `Win32_System_Com` | `IDataObject`, `FORMATETC`, `IStream` — lire ce qu'un dépôt porte |
| `Win32_System_Com_StructuredStorage` | `STGMEDIUM`, `ReleaseStgMedium` |
| `Win32_System_DataExchange` | `RegisterClipboardFormatW` — les formats du shell n'ont pas de numéro fixe |
| `Win32_System_Memory` | `GlobalLock` / `GlobalUnlock` |
| `Win32_UI_Shell` | `DragQueryFileW`, `FILEGROUPDESCRIPTORW` |

`windows-core` ne s'ajoute que pour une raison : la macro `#[implement(IDropTarget)]` écrit du
code qui la nomme par son nom de caisse. Elle est déjà l'intérieur de `windows` ; la déclarer
ne fait que la rendre nommable.

## Combien de lignes pour les remplacer

**Les déclarations, une centaine, et elles ne sont pas le sujet.** Ce qui coûterait, c'est
l'implémentation d'une interface COM à la main : une table de fonctions virtuelles,
`QueryInterface`, `AddRef`, `Release` et leur compte de références, chacun `unsafe`, chacun
faux d'une façon qui ne se voit qu'à l'exécution et seulement parfois. `#[implement]` écrit
exactement cela, et il est relu par tout un écosystème.

C'est le cas que la charte prévoit : *« si on ne peut pas faire sans, alors on fera
intelligemment avec »*. Écrire nous-mêmes un `IUnknown` ne serait pas du contrôle, ce serait un
compte de références de plus à déboguer.

## Ce qu'elles tirent transitivement

**Rien.** `windows` 0.62.2 était déjà dans l'arbre, tiré par `wgpu-hal` via `gpu-allocator` ;
`windows-core` 0.62.2 est sa propre dépendance. L'arbre de la session compte donc **le même
nombre de caisses qu'avant** : ces deux lignes de `Cargo.toml` ne rendent nommable que ce qui
était déjà compilé.

> **Un piège rencontré, et il vaut d'être écrit.** `cargo add windows-core` prend la dernière
> version, `0.100`, qui n'est pas celle de `windows` 0.62 : deux versions de la même caisse
> coexistent alors, et `IDropTarget` cesse d'implémenter `Interface` avec un message qui ne dit
> pas pourquoi. La version se **pin** sur celle de `windows`, et les deux montent ensemble.

## Comment on s'en défait

Par le haut, et c'est déjà le cas : tout ce qui les nomme est dans
`plateforme/depot_windows.rs`, derrière `plateforme::installer`, qui rend `Option<Depots>`.
Le reste de l'application ne connaît qu'un canal et une [`Moisson`](../../../crates/glucose-desktop/src/plateforme/moisson.rs).
Sur toute autre plateforme, la fonction rend `None` et rien de tout cela n'est compilé.

Le jour où une couche portable ferait le même travail, c'est un fichier qui change et zéro
appelant.

---

## Et la dépendance qu'on a **évitée**

Le plan de la session posait l'arbitrage ainsi : ou bien un pont Windows natif, ou bien
**télécharger** l'image depuis son adresse — ce qui impose HTTPS, donc TLS, donc une poignée de
caisses, et une décision qui engage la charte.

**La recherche a démenti la prémisse.** Windows a, depuis 2009, un format pour les fichiers qui
n'existent pas encore — `CFSTR_FILEDESCRIPTORW` et `CFSTR_FILECONTENTS` — et les navigateurs
l'offrent pour toute image glissée hors d'une page. **C'est le navigateur qui télécharge**, avec
ses connexions, son cache et ses cookies ; nous lisons un flux.

Le pont natif est donc *à la fois* le chemin sans dépendance réseau et le seul qui fonctionne
sur une page demandant une authentification. L'arbitrage n'avait pas lieu d'être, et c'est une
mesure — pas une préférence — qui l'a montré.

---

**Retour** : [`00-INDEX.md`](../00-INDEX.md)
