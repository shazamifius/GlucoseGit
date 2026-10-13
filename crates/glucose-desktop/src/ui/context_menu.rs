//! Le menu contextuel du clic droit.
//!
//! # Le conflit qu'il fallait résoudre d'abord
//!
//! Dans Glucose, le clic droit **déplace la vue** — c'est écrit dans la spécification
//! d'origine, et c'est un geste qu'on fait des centaines de fois par séance. La même
//! spécification annonce pourtant un menu contextuel au clic droit. Les deux ne peuvent pas
//! cohabiter naïvement : ouvrir un menu à chaque fin de pan serait insupportable.
//!
//! La résolution est celle de PICK-1 : **le geste se décide au relâchement**. Un clic droit
//! qui a déplacé le curseur était un pan, et rien ne s'ouvre. Un clic droit relâché sur
//! place — moins de quelques pixels — était une demande de menu. Aucun mode, aucun
//! modificateur, aucun réglage : l'intention se lit dans le geste lui-même.
//!
//! # Ce qu'il propose, et ce qu'il tait
//!
//! Seulement des gestes qui existent. La spécification d'origine décrit aussi « Couper »,
//! « Copier », « Grouper dans une nouvelle membrane » et « Créer un sous-dossier » ; aucun
//! n'est écrit en Rust, donc aucun n'est proposé. Un menu qui offre ce qui n'existe pas est
//! la même faute qu'un bouton qui affiche un toast sans rien faire (R-33).
//!
//! Chaque entrée porte son raccourci. C'est la seule façon qu'un menu a d'apprendre à s'en
//! passer.
//!
//! # Les dimensions sont assumées, pas relevées
//!
//! La référence n'a jamais eu ce composant — `ContextMenu.tsx` est décrit dans la
//! spécification d'origine et n'existe dans aucun commit du dépôt. Il n'y a donc aucune
//! capture à égaler : les valeurs ci-dessous sont choisies pour s'accorder à la barre
//! d'action, qui est la surface flottante voisine, et n'ont pas à être défendues comme une
//! parité.

use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use entrees::{entrees, Def};
use glucose_core::store::Store;
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Transform};

mod entrees;

/// Marges intérieures du menu.
const PAD: f32 = 4.0;
/// Hauteur d'une entrée.
const ROW: f32 = 24.0;
/// Marge horizontale d'une entrée.
const ROW_PAD_X: f32 = 10.0;
/// Espace minimal entre le libellé et son raccourci.
const SHORTCUT_GAP: f32 = 24.0;
/// Corps du texte.
const FONT: f32 = 12.0;
/// Rayon des coins, celui de la barre d'action.
const RADIUS: f32 = 6.0;
/// **Au doigt** (fiche 56), une entrée fait 48 points de haut — la cible tactile d'Android
/// et de Material, 44 sur iPad — et son texte 16, le corps des listes d'Android.
const ROW_AU_DOIGT: f32 = crate::theme::CIBLE_DU_DOIGT;
const FONT_AU_DOIGT: f32 = 16.0;
/// Hauteur de la bande d'un séparateur, filet compris.
const SEPARATOR: f32 = 7.0;

/// Ce qu'une entrée du menu déclenche.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    /// Des images de l'appareil : le sélecteur de fichiers du bureau, celui des photos du
    /// téléphone (fiche 56).
    AjouterDesImages,
    /// La sélection entière vers le presse-papiers (fiche 51 § 2).
    Copier,
    /// Un document vierge, à la place de celui-ci (fiche 51 § 4).
    NouveauDocument,
    /// Un autre document : le sélecteur de fichiers, ou la liste du téléphone (fiche 56).
    OuvrirUnDocument,
    /// Entrer dans le mode référence, ou en sortir (fiche 51 § 5).
    ModeReference,
    /// La même, puis retirée une fois copiée.
    Couper,
    Duplicate,
    /// Ses octets d'origine, dans un fichier.
    EnregistrerLImage,
    /// Les liens choisis repartent chercher leur image, et lui laissent leur place (DEPOT-WEB-6).
    RemplacerParLImage,
    ToggleLock,
    /// Retirer les bandes unies des images sélectionnées (BORDURES-1).
    TrimBorders,
    ToFront,
    ToBack,
    Delete,
    Paste,
    SelectAll,
    /// Sur un onglet : l'ouvrir au renommage (BOARDS-1).
    RenommerOnglet,
    /// Sur un onglet : le supprimer, avec ses dossiers.
    SupprimerOnglet,
    /// Sur un onglet : un board de plus.
    NouvelOnglet,
    /// Sur un onglet : un autre document, dans des onglets neufs (BOARDS-2).
    AjouterUnDocument,
    /// Envoyer le journal technique, ou cesser de l'envoyer (fiche 54).
    JournalTechnique,
    /// Ouvrir le dossier de la boîte noire : exactement ce qui part (fiche 49 § 2.2).
    VoirCeQuiPart,
    /// Le dernier geste défait, puis refait — `Ctrl+Z`, `Ctrl+Y` (ACCES-1).
    Annuler,
    Retablir,
    /// Le vol qui montre tout le tableau, ou la sélection (`Alt+F`, `F`).
    ToutCadrer,
    CadrerLaSelection,
    /// Les signets de vue : y voler, en poser un (fiche 53 ; ACCES-1).
    Signets,
    /// Le document : un jalon, ou un autre nom (`Ctrl+S`, `Ctrl+Maj+S`).
    Enregistrer,
    EnregistrerSous,
    /// Le tableau dans un autre format (`Ctrl+E`).
    Exporter,
}

/// Une ligne du menu : soit une entrée, soit un filet.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuRow {
    Item {
        action: MenuAction,
        label: &'static str,
        shortcut: &'static str,
        rect: (f32, f32, f32, f32),
    },
    /// Un filet de séparation, à cette ordonnée, sur la largeur de sa colonne.
    Separator { y: f32, x: f32, largeur: f32 },
}

/// Le menu, une fois placé.
#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenu {
    pub rect: (f32, f32, f32, f32),
    pub rows: Vec<MenuRow>,
    /// Le corps du texte, en pixels : celui de la souris, ou celui du doigt.
    pub font: f32,
}

/// Ce qu'un menu proposerait à cet endroit, ouvert en `(ax, ay)` — sur le canevas, ou sur
/// l'onglet qu'on nomme.
///
/// Fonction pure : elle ne lit que le store et la typographie, et ne dessine rien. Rend `None`
/// quand il n'y aurait rien à proposer.
///
/// **Ouvert au doigt**, ses entrées prennent la taille d'un doigt et taisent leurs raccourcis :
/// il n'y a pas de clavier sous la main. La mesure vient de la main qui l'ouvre, jamais d'un
/// « mode téléphone » — une tablette Windows au doigt reçoit la même.
pub fn layout_context_menu(
    store: &Store,
    typography: &Typography,
    (at, onglet): ((f32, f32), Option<&str>),
    screen: (f32, f32),
    scale: f32,
    doigt: bool,
) -> Option<ContextMenu> {
    let s = crate::theme::clamp_ui_scale(scale);
    let (rangee, corps) = if doigt {
        (ROW_AU_DOIGT, FONT_AU_DOIGT)
    } else {
        (ROW, FONT)
    };
    let (rangee, font) = (rangee * s, corps * s);
    let mut defs = entrees(store, onglet)?;
    if doigt {
        for (_, _, raccourci) in defs.iter_mut().flatten() {
            *raccourci = "";
        }
    }

    let largeur_ligne = |label: &str, shortcut: &str| {
        let (a, _) = typography.measure_text(label, font, Face::Regular);
        let (b, _) = typography.measure_text(shortcut, font, Face::Regular);
        a + SHORTCUT_GAP * s + b + ROW_PAD_X * s * 2.0
    };
    let (pad, filet) = (PAD * s, SEPARATOR * s);
    let colonnes = en_colonnes(defs, screen.1 - pad * 2.0, (rangee, filet));
    let largeurs: Vec<f32> = colonnes
        .iter()
        .map(|c| {
            c.iter()
                .flatten()
                .map(|(_, l, k)| largeur_ligne(l, k))
                .fold(0.0f32, f32::max)
        })
        .collect();
    let largeur = largeurs.iter().sum::<f32>();
    let hauteur = pad * 2.0
        + colonnes
            .iter()
            .map(|c| hauteur_de(c, (rangee, filet)))
            .fold(0.0f32, f32::max);

    // Le menu reste dans la fenêtre : s'il déborde, il bascule de l'autre côté du curseur,
    // comme n'importe quel menu système. Sans cela, un clic droit près du bord ouvrirait un
    // menu dont la moitié est hors de l'écran.
    let x = if at.0 + largeur <= screen.0 {
        at.0
    } else {
        (at.0 - largeur).max(0.0)
    };
    let y = if at.1 + hauteur <= screen.1 {
        at.1
    } else {
        (at.1 - hauteur).max(0.0)
    };

    Some(ContextMenu {
        rect: (x, y, largeur, hauteur),
        rows: poser(colonnes, &largeurs, (x, y), (pad, rangee, filet)),
        font,
    })
}

/// **Les rangées, posées colonne après colonne** depuis le coin `(x, y)` du menu. Chaque
/// colonne a la largeur de sa plus longue entrée ; ses entrées, cette largeur moins les marges
/// — la mesure d'une seule colonne, inchangée.
fn poser(
    colonnes: Vec<Vec<Def>>,
    largeurs: &[f32],
    (x, y): (f32, f32),
    (pad, rangee, filet): (f32, f32, f32),
) -> Vec<MenuRow> {
    let mut rows = Vec::new();
    let mut gauche = x;
    for (colonne, &l) in colonnes.into_iter().zip(largeurs) {
        let mut curseur = y + pad;
        for def in colonne {
            match def {
                Some((action, label, shortcut)) => {
                    rows.push(MenuRow::Item {
                        action,
                        label,
                        shortcut,
                        rect: (gauche + pad, curseur, l - pad * 2.0, rangee),
                    });
                    curseur += rangee;
                }
                None => {
                    rows.push(MenuRow::Separator {
                        y: curseur + filet / 2.0,
                        x: gauche + pad,
                        largeur: l - pad * 2.0,
                    });
                    curseur += filet;
                }
            }
        }
        gauche += l;
    }
    rows
}

/// La hauteur d'une colonne d'entrées : une rangée par entrée, un filet par séparation.
fn hauteur_de(colonne: &[Def], (rangee, filet): (f32, f32)) -> f32 {
    colonne
        .iter()
        .map(|d| if d.is_some() { rangee } else { filet })
        .sum()
}

/// **Les entrées, en colonnes qui tiennent dans `hauteur`** (ACCES-1) — la règle du rail :
/// colonne après colonne, quand la place manque. Un menu plus haut que l'écran en perdait le
/// bas : au doigt, sur un téléphone de cinq pouces, il n'en tient qu'une douzaine.
///
/// Une colonne se ferme **à une séparation** quand le groupe qui suit ne tiendrait pas entier
/// sous elle : les gestes d'un même groupe restent ensemble. Seul un groupe plus haut qu'une
/// colonne se coupe entre deux entrées. Une séparation ne commence ni ne finit une colonne.
fn en_colonnes(defs: Vec<Def>, hauteur: f32, (rangee, filet): (f32, f32)) -> Vec<Vec<Def>> {
    let groupes: Vec<&[Def]> = defs.split(Option::is_none).collect();
    let mut colonnes: Vec<Vec<Def>> = vec![Vec::new()];
    for groupe in groupes.into_iter().filter(|g| !g.is_empty()) {
        let colonne = colonnes.last_mut().expect("au moins une colonne");
        let sous = if colonne.is_empty() { 0.0 } else { filet };
        let deja = hauteur_de(colonne, (rangee, filet));
        if !colonne.is_empty() && deja + sous + hauteur_de(groupe, (rangee, filet)) > hauteur {
            colonnes.push(Vec::new());
        } else if !colonne.is_empty() {
            colonne.push(None);
        }
        for def in groupe {
            let colonne = colonnes.last_mut().expect("au moins une colonne");
            if !colonne.is_empty() && hauteur_de(colonne, (rangee, filet)) + rangee > hauteur {
                colonnes.push(Vec::new());
            }
            colonnes
                .last_mut()
                .expect("au moins une colonne")
                .push(*def);
        }
    }
    colonnes
}

/// L'entrée visée en `(px, py)`, s'il y en a une.
pub fn hit_context_menu(menu: &ContextMenu, px: f32, py: f32) -> Option<MenuAction> {
    menu.rows.iter().find_map(|row| match row {
        MenuRow::Item { action, rect, .. } if dans(*rect, px, py) => Some(*action),
        _ => None,
    })
}

/// Le point tombe-t-il sur le menu ? Ce qui tombe dessus ne descend jamais au canevas.
pub fn covers(menu: &ContextMenu, px: f32, py: f32) -> bool {
    dans(menu.rect, px, py)
}

fn dans((x, y, w, h): (f32, f32, f32, f32), px: f32, py: f32) -> bool {
    px >= x && px <= x + w && py >= y && py <= y + h
}

/// Dessine le menu, avec l'entrée survolée en surbrillance.
pub fn draw_context_menu(
    pixmap: &mut PixmapMut,
    menu: &ContextMenu,
    typography: &Typography,
    theme: &Theme,
    pointer: (f32, f32),
    scale: f32,
) {
    let s = crate::theme::clamp_ui_scale(scale);
    let font = menu.font;

    let mut pb = PathBuilder::new();
    let (x, y, w, h) = menu.rect;
    crate::renderer::push_rounded_rect(&mut pb, x, y, w, h, RADIUS * s);
    if let Some(path) = pb.finish() {
        let mut paint = Paint {
            anti_alias: true,
            ..Default::default()
        };
        paint.set_color(theme.btn_bg);
        pixmap.fill_path(
            &path,
            &paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );
        paint.set_color(theme.btn_border);
        pixmap.stroke_path(
            &path,
            &paint,
            &tiny_skia::Stroke {
                width: 1.0,
                ..Default::default()
            },
            Transform::identity(),
            None,
        );
    }

    for row in &menu.rows {
        match row {
            MenuRow::Separator { y, x, largeur } => filet(pixmap, *x, *y, *largeur, theme),
            MenuRow::Item { .. } => draw_row(pixmap, row, typography, theme, pointer, font),
        }
    }
}

/// Une entrée : sa surbrillance si le curseur est dessus, son libellé, son raccourci.
///
/// Le rouge de la suppression n'apparaît qu'au survol, comme dans la barre d'action : au
/// repos, une liste d'entrées se lit d'un seul ton.
fn draw_row(
    pixmap: &mut PixmapMut,
    row: &MenuRow,
    typography: &Typography,
    theme: &Theme,
    pointer: (f32, f32),
    font: f32,
) {
    let MenuRow::Item {
        action,
        label,
        shortcut,
        rect,
    } = row
    else {
        return;
    };
    let survol = dans(*rect, pointer.0, pointer.1);
    if survol {
        remplir_arrondi(pixmap, *rect, RADIUS / 2.0, theme.bg_hover);
    }
    let ink = match (survol, action) {
        (true, MenuAction::Delete) => theme.alert,
        (true, _) => theme.text_primary,
        (false, _) => theme.text_secondary,
    };
    let base = rect.1 + (rect.3 - font) / 2.0;
    let pad = ROW_PAD_X * font / FONT;
    typography.draw_text(
        pixmap,
        label,
        rect.0 + pad,
        base,
        TextStyle {
            size: font,
            color: ink,
            face: Face::Regular,
        },
    );
    let (kw, _) = typography.measure_text(shortcut, font, Face::Regular);
    typography.draw_text(
        pixmap,
        shortcut,
        rect.0 + rect.2 - pad - kw,
        base,
        TextStyle {
            size: font,
            color: theme.text_muted,
            face: Face::Regular,
        },
    );
}

/// Un filet horizontal d'un pixel, posé sur la grille de pixels et sans anti-aliasage.
fn filet(pixmap: &mut PixmapMut, x: f32, y: f32, largeur: f32, theme: &Theme) {
    let Some(rect) = Rect::from_xywh(x.round(), y.round(), largeur.round().max(1.0), 1.0) else {
        return;
    };
    let mut paint = Paint {
        anti_alias: false,
        ..Default::default()
    };
    paint.set_color(theme.border_medium);
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}

fn remplir_arrondi(pixmap: &mut PixmapMut, (x, y, w, h): (f32, f32, f32, f32), r: f32, c: Color) {
    let mut pb = PathBuilder::new();
    crate::renderer::push_rounded_rect(&mut pb, x, y, w, h, r);
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(c);
    pixmap.fill_path(
        &path,
        &paint,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );
}

#[cfg(test)]
mod tests;
