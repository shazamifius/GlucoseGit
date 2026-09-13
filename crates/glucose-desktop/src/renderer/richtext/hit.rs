//! Où tombe un point dans un texte, et où tombe un texte sous un point.
//!
//! # HIT-1 — le clic et le curseur lisent la même mise en page
//!
//! Une sélection à la souris tient à une seule chose : que l'offset rendu par un clic soit
//! celui que le curseur dessine ensuite. Les deux conversions vivent donc ici, face à face,
//! et parcourent la **même** plage de fragments dans le **même** ordre —
//! [`offset_to_x`] et [`x_to_offset`] sont l'une l'inverse de l'autre, ce qu'un test tient
//! pour chaque frontière de chaque ligne.
//!
//! Les écrire séparément, l'une dans le dessin et l'autre dans les interactions, c'était
//! garantir qu'elles divergent : il aurait suffi qu'une seule oublie le visage d'un fragment
//! pour que le curseur se pose à côté du caractère cliqué, d'autant plus loin que la ligne
//! contient de gras.
//!
//! # Tout est en unités monde
//!
//! Comme la mise en page (CARD-1), ces conversions ignorent le zoom : l'appelant ramène son
//! point écran en unités monde, et reçoit un offset qui n'en dépend pas. C'est ce qui fait
//! qu'un clic tombe sur le même caractère à tous les zooms.

use super::{TextLayout, VisualLine};
use crate::typography::Typography;

/// L'indice de la ligne visuelle qui porte l'ordonnée `y`, comptée depuis le haut du texte.
///
/// Au-dessus de la première ligne on rend la première, en dessous de la dernière on rend la
/// dernière : cliquer dans la marge d'une carte pose le curseur au plus près, jamais nulle
/// part.
pub fn line_at(layout: &TextLayout, y: f32, line_height: f32) -> usize {
    if layout.lines.is_empty() || line_height <= 0.0 {
        return 0;
    }
    let index = (y / line_height).floor();
    (index.max(0.0) as usize).min(layout.lines.len() - 1)
}

/// L'abscisse de `offset` dans `line`, depuis le début du texte de cette ligne.
///
/// Un offset avant la ligne rend `0`, un offset après rend la largeur entière : c'est ce qui
/// permet au surlignage d'une sélection multi-lignes de couvrir chaque ligne intermédiaire
/// sans cas particulier.
pub fn offset_to_x(
    typography: &Typography,
    layout: &TextLayout,
    line: &VisualLine,
    source: &str,
    offset: usize,
    font: f32,
) -> f32 {
    let mut x = 0.0;
    for fragment in layout.fragments_of(line) {
        if offset <= fragment.start {
            break;
        }
        let end = offset.min(fragment.end);
        let (w, _) = typography.measure_text(&source[fragment.start..end], font, fragment.face());
        x += w;
        if offset <= fragment.end {
            break;
        }
    }
    x
}

/// L'offset le plus proche de l'abscisse `x` dans `line`.
///
/// La frontière retenue est celle dont le caractère est coupé en deux par `x` : cliquer sur la
/// moitié gauche d'une lettre pose le curseur devant elle, sur la moitié droite derrière. C'est
/// la règle de tous les éditeurs, et c'est la seule qui rende le clic prévisible sur les
/// caractères larges.
pub fn x_to_offset(
    typography: &Typography,
    layout: &TextLayout,
    line: &VisualLine,
    source: &str,
    x: f32,
    font: f32,
) -> usize {
    // À gauche de tout : le début du premier fragment **dessiné**, qui n'est pas toujours le
    // début de la ligne source. Sur `# Titre` au repos, le `# ` est effacé et occupe l'abscisse
    // zéro avec le « T » ; y renvoyer l'offset de la ligne poserait le curseur avant le `#`,
    // où l'auteur qui vise le début de son titre ne veut pas écrire — et le clic dériverait,
    // puisque recliquer au même endroit rendrait alors un autre offset.
    if x <= 0.0 {
        return layout
            .fragments_of(line)
            .first()
            .map_or(line.start, |f| f.start);
    }
    let mut at = 0.0;
    for fragment in layout.fragments_of(line) {
        let face = fragment.face();
        for (i, ch) in source[fragment.start..fragment.end].char_indices() {
            let w = typography.advance(ch, font, face);
            if x < at + w / 2.0 {
                return fragment.start + i;
            }
            at += w;
        }
    }
    // À droite de tout ce qui est dessiné : la fin de la ligne **source**, signes compris.
    // Sans cela, entrer en édition en cliquant après `**gras**` poserait le curseur entre le
    // mot et son signe fermant, là où l'auteur voyait la fin de sa ligne.
    line.end
}

/// L'offset le plus proche du point `at`, en unités monde depuis le coin haut-gauche du texte.
pub fn offset_at(
    typography: &Typography,
    layout: &TextLayout,
    source: &str,
    at: (f32, f32),
    bx: &super::TextBox,
) -> usize {
    if layout.lines.is_empty() {
        return 0;
    }
    let index = line_at(layout, at.1, bx.line_height);
    let line = &layout.lines[index];
    let indent = if line.kind == super::LineKind::Bullet {
        bx.bullet_indent
    } else {
        0.0
    };
    x_to_offset(
        typography,
        layout,
        line,
        source,
        at.0 - indent,
        line.kind.font(bx.body),
    )
}

/// L'indice de la ligne visuelle qui porte `offset`.
///
/// Les lignes se suivent sans trou, donc la première dont la fin dépasse `offset` est la
/// bonne. Une sélection posée à la toute fin du texte appartient à la dernière ligne.
pub fn line_of_offset(layout: &TextLayout, offset: usize) -> usize {
    layout
        .lines
        .iter()
        .position(|l| offset <= l.end)
        .unwrap_or(layout.lines.len().saturating_sub(1))
}

#[cfg(test)]
mod tests;
