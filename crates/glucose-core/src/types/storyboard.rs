//! **Le storyboard d'un tableau** : ses cases, et la grille qui les range (STORYBOARD-1,
//! fiche 64 ; STORYBOARD-2, fiche 65).
//!
//! # La grille est une donnée du tableau
//!
//! Glucose Tauri la gardait avec le tableau (`board.storyboard` : `aspectRatio`, `panelWidth`,
//! `cols`, `gap`). STORYBOARD-1 l'avait laissée au panneau : un rectangle tracé à six colonnes
//! retombait à quatre au « + Ajouter » suivant, et un document rouvert ne savait plus la sienne.
//! Elle revient au tableau (schéma 3), et **une seule loi** ([`Grille::coin`]) dit où va chaque
//! case : la case de rang `i` est à la colonne `i mod c`, à la rangée `i div c`.

//!
//! # Ce qu'une case dit en plus de son image (schéma 4, fiche 67)
//!
//! Sa **durée** pour la lecture, ses **champs** de texte, son **plan**. La durée d'une case
//! est la sienne si on l'a réglée, sinon celle du storyboard ([`Storyboard::duree`]) — la loi
//! de Storyboarder (`boardDuration` : `board.duration`, sinon `scene.defaultBoardTiming`), en
//! une seule fonction : la sienne se contredit (sa frise prend `0` pour « par défaut », son
//! modèle non). Les champs sont **ceux du storyboard**, nommés une fois pour toutes ses cases,
//! comme les légendes de Toon Boom et les commentaires de Krita.
//!
//! # Plusieurs storyboards par tableau (schéma 5, fiche 68)
//!
//! Un tableau porte autant de storyboards que de séquences. **Chaque case nomme le sien**
//! ([`StoryboardPanel::storyboard`]) ; le tableau garde la liste plate de ses cases et celle de
//! ses storyboards. Storyboarder fait l'inverse — une scène contient ses cases —, mais ici
//! l'arène des 10⁷ nœuds est plate, et le journal défait une case en ne touchant qu'elle.
//!
//! **SEQUENCE-1** : toute case nomme un storyboard de son tableau, et tout storyboard a au
//! moins une case. Le magasin le tient à chaque geste ; un document d'avant le schéma 5, ou
//! venu de Glucose Tauri, y est ramené par une seule règle ([`rattacher`]).

use super::Id;

/// La hauteur de la description sous chaque case, en unités du monde — celle de Glucose Tauri
/// (`StoryboardLayer.ts`, `descH`). Le dessin, le fantôme et la prise d'une case la lisent.
pub const DESCRIPTION: f64 = 40.0;

#[derive(Debug, Clone, PartialEq)]
pub struct StoryboardPanel {
    pub id: Id,
    /// **Le storyboard de la case** — son identifiant dans le tableau (SEQUENCE-1).
    pub storyboard: Id,
    /// Son rang dans son storyboard.
    pub order: i32,
    pub description: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// **Sa durée à la lecture**, en millisecondes — `None` : celle du storyboard.
    pub duree_ms: Option<u32>,
    /// **Son plan**, s'il est dit.
    pub plan: Option<Plan>,
    /// **Le texte de chaque champ du storyboard**, au rang du champ ([`Storyboard::champs`]).
    /// Jamais plus long que les champs, et sans texte vide au bout : deux cases qui disent la
    /// même chose sont égales ([`StoryboardPanel::ecrire`]).
    pub textes: Vec<String>,
}

impl StoryboardPanel {
    /// Une case de ce storyboard, à ce rang, à ce cadre.
    pub fn new(
        id: impl Into<String>,
        storyboard: impl Into<String>,
        order: i32,
        (x, y): (f64, f64),
        (width, height): (f64, f64),
    ) -> Self {
        Self {
            id: id.into(),
            storyboard: storyboard.into(),
            order,
            description: String::new(),
            x,
            y,
            width,
            height,
            duree_ms: None,
            plan: None,
            textes: Vec::new(),
        }
    }

    /// Le cadre contient-il ce point ? La bande de description, sous lui, n'en est pas.
    pub fn contient(&self, (x, y): (f64, f64)) -> bool {
        (self.x..=self.x + self.width).contains(&x) && (self.y..=self.y + self.height).contains(&y)
    }

    /// **Ce que la case occupe sur le canevas** : son cadre, et la bande de description sous
    /// lui — ce que le cadrage du contenu et un export doivent couvrir (BORNES-2, fiche 70).
    pub fn emprise(&self) -> crate::geometry::Rect {
        crate::geometry::Rect::new(self.x, self.y, self.width, self.height + DESCRIPTION)
    }

    /// Le texte du champ de rang `champ` — vide s'il n'en a pas.
    pub fn texte(&self, champ: usize) -> &str {
        self.textes.get(champ).map_or("", String::as_str)
    }

    /// **Écrit le texte du champ de rang `champ`**, et garde la forme unique : les rangs d'avant
    /// comblés de vide, aucun vide au bout.
    pub fn ecrire(&mut self, champ: usize, texte: &str) {
        if self.textes.len() <= champ {
            self.textes.resize(champ + 1, String::new());
        }
        self.textes[champ] = texte.to_string();
        while self.textes.last().is_some_and(String::is_empty) {
            self.textes.pop();
        }
    }

    /// **La case porte-t-elle quelque chose qu'on a écrit ou choisi** — une description, un
    /// texte, un plan, une durée à elle ? Les images se comptent à part : elles sont au tableau.
    pub fn porte_quelque_chose(&self) -> bool {
        !self.description.trim().is_empty()
            || self.textes.iter().any(|t| !t.trim().is_empty())
            || self.plan.is_some()
            || self.duree_ms.is_some()
    }
}

/// **L'échelle des plans**, du plus large au plus serré — celle du cinéma français.
/// Storyboarder en compte onze, de l'« extrêmement près » au plan de situation
/// (`shot-generator/utils/cameraUtils.js`) ; l'échelle française en nomme huit, chacune par ce
/// que le cadre garde du corps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    General,
    Ensemble,
    Moyen,
    Americain,
    RapprocheTaille,
    RapprochePoitrine,
    Gros,
    TresGros,
}

impl Plan {
    /// Dans l'ordre de l'échelle. **Le rang d'un plan ici est son numéro dans le fichier**
    /// (PERSIST-4) : on peut en ajouter au bout, jamais en déplacer un.
    pub const TOUS: [Plan; 8] = [
        Plan::General,
        Plan::Ensemble,
        Plan::Moyen,
        Plan::Americain,
        Plan::RapprocheTaille,
        Plan::RapprochePoitrine,
        Plan::Gros,
        Plan::TresGros,
    ];

    /// Son nom, et son sigle — celui qu'on écrit sous une case.
    pub fn noms(self) -> (&'static str, &'static str) {
        match self {
            Self::General => ("Plan général", "PG"),
            Self::Ensemble => ("Plan d'ensemble", "PE"),
            Self::Moyen => ("Plan moyen", "PM"),
            Self::Americain => ("Plan américain", "PA"),
            Self::RapprocheTaille => ("Plan rapproché taille", "PRT"),
            Self::RapprochePoitrine => ("Plan rapproché poitrine", "PRP"),
            Self::Gros => ("Gros plan", "GP"),
            Self::TresGros => ("Très gros plan", "TGP"),
        }
    }

    /// **Le plan d'après, en tournant** : aucun, puis chaque plan, puis aucun — le sélecteur
    /// qui tourne, comme le format.
    pub fn suivant(plan: Option<Plan>) -> Option<Plan> {
        match plan {
            None => Some(Self::TOUS[0]),
            Some(p) => {
                let rang = Self::TOUS.iter().position(|q| *q == p).unwrap_or(0);
                Self::TOUS.get(rang + 1).copied()
            }
        }
    }
}

/// **Un storyboard** (schéma 4) : la grille qui range ses cases, la durée d'une case qui n'a pas
/// la sienne, et les champs que chaque case remplit.
#[derive(Debug, Clone, PartialEq)]
pub struct Storyboard {
    /// Son identifiant, que chacune de ses cases nomme (schéma 5, fiche 68).
    pub id: Id,
    pub grille: Grille,
    /// La durée d'une case, en millisecondes, quand elle n'a pas la sienne.
    pub duree_ms: u32,
    /// Le nom de chaque champ, dans l'ordre où les cases les montrent.
    pub champs: Vec<String>,
}

/// La durée d'une case qu'on n'a pas réglée, à la naissance d'un storyboard : deux secondes,
/// celle de Storyboarder (`defaultBoardTiming: 2000`, `src/js/prefs.js`). Réglable ensuite,
/// pour tout le storyboard.
pub const DUREE_DE_NAISSANCE_MS: u32 = 2000;

/// Les champs d'un storyboard à sa naissance : ceux de Storyboarder (`dialogue`, `action`,
/// `notes`). Toon Boom y ajoute « Slugging », le minutage écrit : ici, la durée est un nombre.
pub fn champs_de_naissance() -> Vec<String> {
    ["Dialogue", "Action", "Notes"].map(String::from).to_vec()
}

/// **L'identifiant du storyboard d'avant le schéma 5** : un tableau n'en portait qu'un, sans
/// nom. Ses cases le nomment ainsi à la lecture ([`rattacher`]) — un document relu, ou un
/// geste de son histoire, désigne toujours le même.
pub const STORYBOARD_D_AVANT: &str = "storyboard";

/// **Ramène des cases et des storyboards à SEQUENCE-1** — la seule règle, pour un document
/// d'avant le schéma 5 et pour Glucose Tauri : une case qui ne nomme aucun storyboard présent
/// va au [`STORYBOARD_D_AVANT`], né à la grille de Tauri s'il n'existe pas ; un storyboard sans
/// case disparaît — il ne montrerait rien, et rien ne le désignerait.
pub fn rattacher(cases: &mut [StoryboardPanel], storyboards: &mut Vec<Storyboard>) {
    for case in cases.iter_mut() {
        if !storyboards.iter().any(|s| s.id == case.storyboard) {
            case.storyboard = STORYBOARD_D_AVANT.to_string();
            if !storyboards.iter().any(|s| s.id == STORYBOARD_D_AVANT) {
                storyboards.push(Storyboard::neuf(STORYBOARD_D_AVANT, Grille::default()));
            }
        }
    }
    storyboards.retain(|s| cases.iter().any(|c| c.storyboard == s.id));
}

impl Storyboard {
    /// Un storyboard neuf rangé par cette grille.
    pub fn neuf(id: impl Into<String>, grille: Grille) -> Self {
        Self {
            id: id.into(),
            grille,
            duree_ms: DUREE_DE_NAISSANCE_MS,
            champs: champs_de_naissance(),
        }
    }

    /// **La durée de cette case** : la sienne, sinon celle du storyboard — la seule loi.
    pub fn duree(&self, case: &StoryboardPanel) -> u32 {
        case.duree_ms.unwrap_or(self.duree_ms)
    }
}

/// **Les cinq formats de Glucose Tauri**, dans l'ordre de son panneau.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Cinema,
    Classique,
    Scope,
    Carre,
    Vertical,
}

impl Format {
    pub const TOUS: [Format; 5] = [
        Format::Cinema,
        Format::Classique,
        Format::Scope,
        Format::Carre,
        Format::Vertical,
    ];

    /// La largeur sur la hauteur d'une case.
    pub fn rapport(self) -> f64 {
        match self {
            Self::Cinema => 16.0 / 9.0,
            Self::Classique => 4.0 / 3.0,
            Self::Scope => 2.35,
            Self::Carre => 1.0,
            Self::Vertical => 9.0 / 16.0,
        }
    }
}

/// **La grille d'un storyboard** : le format des cases, leur largeur, les colonnes, l'écart.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grille {
    pub format: Format,
    pub largeur: f64,
    pub colonnes: usize,
    pub ecart: f64,
}

/// **La grille de Glucose Tauri** quand rien n'est réglé : le cinéma, 280 de large, quatre
/// colonnes, 24 d'écart (`StoryboardControls.tsx`, ses `useState`).
impl Default for Grille {
    fn default() -> Self {
        Self {
            format: Format::Cinema,
            largeur: 280.0,
            colonnes: 4,
            ecart: 24.0,
        }
    }
}

impl Grille {
    /// La hauteur d'une case : sa largeur sur son rapport.
    pub fn hauteur(&self) -> f64 {
        self.largeur / self.format.rapport()
    }

    /// Combien de colonnes, au moins une.
    pub fn colonnes(&self) -> usize {
        self.colonnes.max(1)
    }

    /// Combien de cases une grille neuve porte : deux rangées, comme dans Tauri.
    pub fn nombre_initial(&self) -> usize {
        self.colonnes() * 2
    }

    /// D'une case à sa voisine, en largeur et en hauteur : la case, sa bande, l'écart.
    pub fn pas(&self) -> (f64, f64) {
        (
            self.largeur + self.ecart,
            self.hauteur() + DESCRIPTION + self.ecart,
        )
    }

    /// **Le coin de la case de rang `rang`**, la grille partant de `origine` — la seule loi
    /// qui place une case.
    pub fn coin(&self, origine: (f64, f64), rang: usize) -> (f64, f64) {
        let (pas_x, pas_y) = self.pas();
        let colonnes = self.colonnes();
        (
            origine.0 + (rang % colonnes) as f64 * pas_x,
            origine.1 + (rang / colonnes) as f64 * pas_y,
        )
    }

    /// **L'étendue d'une grille de `nombre` cases** : sa largeur et sa hauteur, bandes
    /// comprises, sans l'écart d'après la dernière.
    pub fn etendue(&self, nombre: usize) -> (f64, f64) {
        let (pas_x, pas_y) = self.pas();
        let colonnes = self.colonnes().min(nombre.max(1));
        let rangees = nombre.max(1).div_ceil(self.colonnes());
        (
            colonnes as f64 * pas_x - self.ecart,
            rangees as f64 * pas_y - self.ecart,
        )
    }

    /// **Combien de colonnes et de rangées pour `(largeur, hauteur)`** : le nombre de cases
    /// dont l'étendue est la plus proche — au moins une de chaque.
    ///
    /// `n` cases et leurs `n − 1` écarts s'étendent sur `n·p − e` (`p` le pas) ; la plus proche
    /// de `L` est `n = round((L + e) / p)`. Pas la partie entière — « celles qui tiennent » :
    /// sur une étendue exacte, `(n·p − e + e) / p` peut valoir `n − 1 ulp` (sept rangées au
    /// format Scope donnent 6,999999999999999), et saisir la poignée au coin aurait retiré une
    /// rangée sans que la main bouge. L'arrondi n'a de bord qu'à mi-case.
    pub fn tient_dans(&self, (largeur, hauteur): (f64, f64)) -> (usize, usize) {
        let (pas_x, pas_y) = self.pas();
        let nombre = |l: f64, pas: f64| ((l + self.ecart) / pas).round().max(1.0) as usize;
        (nombre(largeur, pas_x), nombre(hauteur, pas_y))
    }
}

#[cfg(test)]
mod tests;
