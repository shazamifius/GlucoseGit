# 162 — Claude se met à jour seul (25/11, compte a)

> Sa réponse du 24/11 dans `relais-q-h24-quotas`, la note que sa consigne désigne : *« ce serait
> bien aussi que tu arrives à faire fonctionner la mise à jour claude car il y a une nouvelle mise à
> jour présente sur le plugin vscode, le problème c'est que j'ai pas accès à mon pc »*.

## 1. Ce qui se passait

L'extension Claude de VS Code ne se met à jour que dans un VS Code ouvert ; le relais tourne sans lui
(aucun VS Code ne vivait ce matin). Le 24/11, la fiche 142 § 3 avait trouvé la 2.1.331 déjà installée.
Le 25/11 à 5 h 30, le marché de VS Code avait la **2.1.332** (publiée à 2 h 46 pour Linux) ; la
machine, la 2.1.331. Personne ne l'aurait installée avant son retour.

## 2. L'installation, à la main d'abord

`code --install-extension anthropic.claude-code --force` : **refusé** (« Signature verification
failed with 'PackageIntegrityCheckFailed' »). Le même, avec la version nommée
(`anthropic.claude-code@2.1.332`) : **installé**. Probablement un téléchargement abîmé, pas une
règle : la cause reste à confirmer — d'où deux essais dans l'automate.

**L'essai sans jeton** : la nouvelle version lancée dans tmux comme le relais la lance (bypass,
`--no-chrome`), **sans lui écrire** — sans un message, Claude ne dépense rien. Dans le dépôt, elle
arrive à son invite en 2 s, sans question. Dans un dossier neuf (le scratchpad), elle pose celle de
la confiance dans le dossier (« Enter to confirm ») : c'est ce que l'essai doit savoir voir.

## 3. MAJ-1 : le veilleur le fait seul

`outils/relais/mise_a_jour.py`, appelé par la boucle du veilleur après chaque tour, agit **toutes
les six heures** : la dernière version pour Linux (une requête de 1,7 Ko au marché, « la dernière
seulement ») ; plus neuve que celle que VS Code dit installée, elle s'installe (deux essais), puis
**s'essaie** (`prete` : « bypass permissions on » à l'écran, et aucun « Enter to confirm »). Prête,
elle reste ; sinon elle est **refusée** — la précédente réinstallée, la refusée jamais retentée. Un
seul nœud de sa membrane le raconte, récrit à chaque fois : `relais-maj-claude`.

**Le relais lance celle que VS Code dit installée** (`machine.claude_binaire`, par son registre
`extensions.json`), plus le plus grand dossier : une version refusée garde son dossier jusqu'à ce
que VS Code s'ouvre et fasse son ménage (`.obsolete`). Sans registre, le plus grand dossier, comme
avant.

**Ce qui ne change pas** : la session qui vit garde son Claude jusqu'à sa fin ; la suivante prend
le neuf. Aucune session n'est relancée pour une mise à jour.

**Épreuves** : 8 (`outils/relais/test_mise_a_jour.py` : l'écran prêt ou questionnant, installer
puis essayer, refuser et revenir, ne pas retenter, comparer des nombres et non des mots — « 2.1.99 »
avant « 2.1.332 » —, six heures entre deux regards, le registre plutôt que le plus grand dossier) ;
**7 sabotages sur 7** (`outils/sabotages/maj-1-162.json`) ; les 85 épreuves du relais vertes.
**Vérifié pour de vrai** : `essayer` sur la 2.1.332, prête en 2 s ; le veilleur relancé à 5 h 36,
son premier regard n'a rien trouvé de plus neuf (`~/.local/state/relais/mise-a-jour.json`).
**Pas encore vu** : une vraie version neuve installée par le veilleur lui-même — à la prochaine.

## 4. Ce qui reste

* Un compte déconnecté ferait paraître l'écran de connexion à l'essai : la version serait refusée à
  tort (une seule ; la suivante se réessaie). Le compte a est connecté.
* La provenance des images (fiche 27) attend sa clé SauceNAO : sans clé, SauceNAO répond désormais
  par un défi de Cloudflare (403), d'ici, avec ou sans en-tête de navigateur (`relais-q-saucenao`).
