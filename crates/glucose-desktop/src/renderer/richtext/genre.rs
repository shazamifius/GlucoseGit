//! **Du genre d'un bloc aux pixels.**
//!
//! Le genre est décidé par `glucose_core::text::block`, qui ne connaît ni police ni couleur.
//! Les cinq fonctions de ce module sont la seule frontière entre ce qu'un bloc *est* et ce
//! qu'il *devient* à l'écran — taille, graisse, encre, retrait, et la place de son ornement.
//! Rien d'autre n'a le droit de regarder un `BlockKind` pour en tirer une valeur.

use crate::theme::Theme;
use glucose_core::text::{BlockKind, Emphasis};

/// Grossissement d'un titre `# `.
const H1_FACTOR: f32 = 1.25;
/// Grossissement d'un sous-titre `## `.
const H2_FACTOR: f32 = 1.10;
/// Rapetissement d'un `-# `.
///
/// Il n'a pas de valeur à lui : le petit texte est au titre ce que le corps est au titre,
/// donc l'inverse du grossissement d'un `# `. Une constante de moins à choisir, et la
/// symétrie tient toute seule si le facteur des titres change un jour.
const SMALL_FACTOR: f32 = 1.0 / H1_FACTOR;
// Les titres sont plus grands que le corps, et un titre plus qu'un sous-titre : vérifié à la
// compilation, pas dans un test qu'on pourrait oublier de lancer.
const _: () = assert!(H1_FACTOR > H2_FACTOR && H2_FACTOR > 1.0);

/// Le corps d'un bloc. Les grossissements sont des **multiples du corps courant** : ils
/// héritent de l'unique transformation au lieu de redériver du zoom (CARD-1).
///
/// Au-delà du sous-titre, les niveaux gardent le corps et se distinguent par la graisse. La
/// référence ne donne de valeur que pour `#` et `##` ; en inventer quatre autres serait
/// affirmer des chiffres que personne n'a mesurés.
pub fn font_of(kind: BlockKind, body: f32) -> f32 {
    match kind {
        BlockKind::Heading(1) => body * H1_FACTOR,
        BlockKind::Heading(2) => body * H2_FACTOR,
        BlockKind::Small => body * SMALL_FACTOR,
        _ => body,
    }
}

/// L'emphase que le genre impose à toute la ligne.
///
/// Elle s'**unit** à celle des fragments, ce qui fait qu'un `*mot*` dans un titre est en gras
/// italique sans cas particulier — et qu'une ligne de bloc de code passe en chasse fixe par
/// le même chemin qu'un `` `code` `` en ligne, avec son fond et tout le reste.
pub fn emphasis_of(kind: BlockKind) -> Emphasis {
    match kind {
        BlockKind::Heading(_) => Emphasis::BOLD,
        BlockKind::Code => Emphasis::CODE,
        _ => Emphasis::NONE,
    }
}

/// L'encre d'une ligne.
///
/// Une citation garde celle du corps : ce qui la distingue est sa barre et son retrait, pas
/// une couleur de plus. La barre, elle, est grise — elle **est** le `>` que le repos efface,
/// donc elle prend l'encre des signes.
pub fn ink_of(kind: BlockKind, theme: &Theme) -> tiny_skia::Color {
    match kind {
        BlockKind::Heading(1) => theme.card_heading,
        BlockKind::Heading(_) => theme.card_subheading,
        _ => theme.card_body,
    }
}

/// Le retrait d'une ligne, en multiples de `unit`.
///
/// Une seule règle, un seul endroit : la mise en page l'applique pour savoir où couper, le
/// tracé pour savoir où poser la plume, le clic pour savoir où viser. Elle vivait en deux
/// exemplaires, et les deux devaient rester d'accord à la main.
pub fn indent_of(kind: BlockKind, unit: f32) -> f32 {
    match kind {
        // Un élément rangé sous un autre se décale d'un retrait de plus par niveau (LISTE-1).
        BlockKind::Bullet { niveau } | BlockKind::Ordered { niveau, .. } => {
            unit * (f32::from(niveau) + 1.0)
        }
        BlockKind::Quote => unit,
        _ => 0.0,
    }
}

/// **Où se pose l'ornement d'une ligne** — sa puce, son numéro, sa barre —, depuis la marge :
/// un retrait de moins que son texte. Déduit de [`indent_of`] plutôt que redit : la puce d'un
/// élément de niveau 1 reste devant son texte, et non à la marge (LISTE-1).
pub fn retrait_d_ornement(kind: BlockKind, unit: f32) -> f32 {
    (indent_of(kind, unit) - unit).max(0.0)
}
