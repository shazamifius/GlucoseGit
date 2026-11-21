# 128 — Les IA gratuites dans Collaborer : une clé, un essai réel, et la liste qui ment

**Date** : 21/11/2026 · **Chantier** : « L'API publique » (demande du 21/11, autonomie accordée).

## 1. Ce qui est fait

- **Le bouton « L'API publique… »** dans Collaborer ([api.rs](../../crates/glucose-desktop/src/app/collaborer/api.rs)) : tester les modèles gratuits, donner la clé Google AI Studio, donner la clé GitHub.
- **Les clés** ([ia/cles.rs](../../crates/glucose-desktop/src/ia/cles.rs)) : un fichier `ia-cles.json` dans le dossier de Glucose, créé à **0600** sous Unix. Jamais dans le dépôt, jamais dans un journal.
- **Le classement** ([ia/modeles.rs](../../crates/glucose-desktop/src/ia/modeles.rs)) : la version d'abord (2.5 < 3.1 < 3.5), la gamme ensuite (pro > flash > flash-lite). Seuls les noms `gemini-` qui génèrent du texte comptent.
- **L'essai** ([ia/essai.rs](../../crates/glucose-desktop/src/ia/essai.rs)) : liste les modèles, puis en **génère une réponse** pour les huit premiers, un à un. Sur un fil à part ; la boucle reprend le bilan au tour suivant.

## 2. Ce que la vraie clé a appris (le point capital)

La **liste** de Google ne dit pas ce qui marche. Sur la clé de l'auteur, le 21/11 :

| Modèle | Réponse réelle |
|---|---|
| `gemini-2.5-flash`, `2.5-pro`, `2.5-flash-lite` | **404** : « plus disponible pour les nouveaux comptes » |
| `gemini-3.1-pro-preview`, `gemini-4-argon`, `gemini-pro-latest` | **429** : limite atteinte (temporaire) |
| `gemini-3.8-flash`, `gemini-4-hydrogen`, `gemini-flash-latest` | **503** : surcharge (temporaire) |
| `gemini-3-flash-preview`, `gemini-3.1-flash-lite`, `gemini-3.5-flash-lite` | **200** |

Conséquences :
- **Un modèle listé n'est pas un modèle utilisable.** Il faut générer une réponse pour le savoir ; c'est ce que fait l'essai.
- **Le classement seul ne suffit pas** : il faut parcourir la liste jusqu'à en trouver qui répondent (huit essais, pas trois ; trois ne trouvaient rien le 21/11).
- **Les 429 et 503 sont passagers** : un modèle qui échoue aujourd'hui peut répondre demain.

## 3. Un défaut trouvé en chemin

Le classement prenait `antigravity-preview-09-2026` pour la version « 9.2026 » et le plaçait en tête. Corrigé : seuls les noms `gemini-` entrent. Une épreuve le verrouille (`les_autres_familles_ne_faussent_pas_le_rang`).

## 4. Vérifié, et ce qui ne l'est pas

**Vérifié** :
- compilation (`cargo build`), clippy `-D warnings`, `cargo fmt --check` : verts ;
- 16 cliquets : verts ; bibliothèque complète : **2186 épreuves vertes** ;
- l'essai réel contre Google (`essai_reel_google`, lancé à la main, la clé en variable d'environnement) : **passe**, un modèle répond ;
- un sabotage (permission 0644 sur le fichier des clés) : l'épreuve **tombe**, puis le code est rétabli.

**Pas encore vérifié** :
- le menu **à l'écran** : il n'a pas été regardé dans le Glucose en marche ;
- l'essai **sous Windows** : il répond « pas encore branché » ;
- le test GitHub (il attend un jeton que l'utilisateur donnera dans le champ prévu).

## 5. La suite (non faite)

1. Le **basculement** : quand un modèle atteint sa limite, passer au suivant avec un avertissement ; revenir au meilleur dès qu'il répond.
2. La **fenêtre de chat** et son branchement au **MCP** de Glucose (le point d'entrée déjà en place par la porte publique).
3. L'essai sous **Windows** (WinHTTP).
4. Le test **GitHub Models**.

## 6. Choix pris seul, à contester

- **La version d'abord** : 3.1-pro (1013) rang sous 3.5-flash-lite (3051). Un modèle plus récent et plus léger passe avant un pro plus ancien. Discutable.
- **Huit essais** au plus : un chiffre, pas une mesure.
- **Le fichier en clair à 0600**, plutôt qu'un coffre : Glucose n'a pas de coffre sans nouvelle dépendance.
- **Un champ de saisie pour la clé**, dans la question : la clé ne passe par aucun journal.
