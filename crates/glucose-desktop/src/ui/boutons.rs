//! Les boutons de la barre supérieure : leur **place**, puis leur dessin.
//!
//! # Une seule liste, lue deux fois (loi L4)
//!
//! [`layout_topbar`] produit la liste des boutons — chacun avec son rectangle, son icône et
//! son action ; le dessin et le test de clic lisent **cette** liste. Il devient impossible
//! qu'un bouton ne clique pas là où il est dessiné.
//!
//! Les largeurs sont **mesurées** sur le libellé, jamais des littéraux calibrés à l'œil :
//! c'est le premier changement de police qui l'a imposé (R-51), quand « Trans-domaines » a
//! débordé de son cadre.

use super::{ActiveTool, UiAction, UiState};
use crate::icons::{draw_icon_scaled, IconType};
use crate::params::{ButtonState, ScaledRect};
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Stroke, Transform};

/// Corps du libellé d'un bouton d'action de la barre d'outils.
pub(crate) const ACTION_LABEL_FONT: f32 = 12.0;
/// Abscisse du libellé dans un bouton d'action : la marge de l'icône, l'icône, son écart.
pub(crate) const ACTION_LABEL_X: f32 = 26.0;
/// Marge entre la fin du libellé et le bord droit d'un bouton d'action.
pub(crate) const ACTION_LABEL_PAD_RIGHT: f32 = 10.0;

/// Largeur d'un bouton d'action pour `label`, à l'échelle `s`.
///
/// Le libellé est **mesuré**, pas supposé : les largeurs étaient des littéraux calibrés à
/// l'œil sur une police donnée, et le premier changement de police (R-51) a fait déborder
/// « Trans-domaines » de son cadre. Il est mesuré en gras — la graisse du bouton actif, la
/// plus large — pour qu'un bouton ne change pas de taille quand on le bascule.
fn action_button_width(typo: &Typography, label: &str, s: f32) -> f32 {
    let (text_w, _) = typo.measure_text(label, ACTION_LABEL_FONT * s, Face::Bold);
    (ACTION_LABEL_X + ACTION_LABEL_PAD_RIGHT) * s + text_w
}

#[derive(Debug, Clone)]
pub struct TopbarButtonDef {
    pub action: UiAction,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub icon: IconType,
    pub label: &'static str,
    pub active: bool,
    pub is_tool: bool,
}

pub struct TopbarLayout {
    pub buttons: Vec<TopbarButtonDef>,
    pub separators: Vec<f32>,
    pub img_badge: Option<(f32, String)>,
}

/// La règle graduée sur laquelle les boutons se posent : elle avance, et retient ce qu'elle
/// a posé.
///
/// # Pourquoi ce type existe
///
/// Poser un bouton demandait dix lignes dont six ne disaient rien — `y`, `h`, `is_tool`,
/// `active: false`, `label: ""` — répétées quinze fois, et le curseur avançait à la main
/// entre chacune. C'est la répétition structurelle que la fiche 05 § 1 nomme comme la
/// première cause de laideur : on ne voyait plus la barre, seulement ses champs.
///
/// Ici la règle connaît ce qui ne change pas — l'échelle, les hauteurs, la largeur de la
/// fenêtre — et chaque groupe de la barre tient en quelques lignes qui disent **ce qu'il
/// contient**.
struct Regle<'a> {
    boutons: Vec<TopbarButtonDef>,
    separateurs: Vec<f32>,
    /// Où le prochain bouton se pose.
    x: f32,
    s: f32,
    /// Le côté d'un outil carré et son ordonnée.
    outil: (f32, f32),
    /// La hauteur d'un bouton d'action et son ordonnée.
    action: (f32, f32),
    typo: &'a Typography,
    /// Sous ces largeurs de fenêtre, les libellés cèdent la place : d'abord ceux de droite
    /// (compact), puis tous (ultra-compact).
    compact: bool,
    ultra: bool,
}

impl<'a> Regle<'a> {
    fn nouvelle(width: f32, ui: &UiState, typo: &'a Typography) -> Self {
        let s = ui.scale();
        let topbar_h = ui.topbar_height();
        let cote = 30.0 * s;
        let haut_action = 28.0 * s;
        Self {
            boutons: Vec::new(),
            separateurs: Vec::new(),
            x: 100.0 * s,
            s,
            outil: (cote, (topbar_h - cote) / 2.0),
            action: (haut_action, (topbar_h - haut_action) / 2.0),
            typo,
            // Responsive design :
            // - Mode complet : width >= 1320px * scale
            // - Mode compact : 1050px * scale <= width < 1320px * scale
            // - Mode ultra-compact : width < 1050px * scale
            compact: width < 1320.0 * s,
            ultra: width < 1050.0 * s,
        }
    }

    /// La largeur d'un bouton d'action et le libellé qu'il gardera : un bouton sans libellé
    /// est carré comme un outil.
    fn mesurer(&self, label: &'static str, efface: bool) -> (f32, &'static str) {
        if efface {
            (self.outil.0, "")
        } else {
            (action_button_width(self.typo, label, self.s), label)
        }
    }

    /// Un outil carré : il n'a qu'une icône, et il est actif ou non.
    fn outil(&mut self, action: UiAction, icon: IconType, actif: bool) {
        let (cote, y) = self.outil;
        self.boutons.push(TopbarButtonDef {
            action,
            x: self.x,
            y,
            w: cote,
            h: cote,
            icon,
            label: "",
            active: actif,
            is_tool: true,
        });
        self.x += cote + 2.0 * self.s;
    }

    /// Un bouton d'action, avec son libellé tant que la fenêtre le permet.
    fn action(&mut self, action: UiAction, icon: IconType, label: &'static str, actif: bool) {
        let (largeur, label) = self.mesurer(label, self.ultra);
        self.poser(action, icon, label, largeur, actif, 4.0);
    }

    /// Le même, avec l'écart qui le suit : les groupes ne respirent pas tous pareil.
    fn poser(
        &mut self,
        action: UiAction,
        icon: IconType,
        label: &'static str,
        largeur: f32,
        actif: bool,
        ecart: f32,
    ) {
        let (hauteur, y) = self.action;
        self.boutons.push(TopbarButtonDef {
            action,
            x: self.x,
            y,
            w: largeur,
            h: hauteur,
            icon,
            label,
            active: actif,
            is_tool: false,
        });
        self.x += largeur + ecart * self.s;
    }

    /// Un filet vertical entre deux groupes : où il tombe, et ce qu'il laisse après lui.
    fn separateur(&mut self, avant: f32, apres: f32) {
        self.separateurs.push(self.x + avant * self.s);
        self.x += apres * self.s;
    }
}

pub fn layout_topbar(
    width: f32,
    ui: &UiState,
    typo: &Typography,
    board_img_count: usize,
) -> TopbarLayout {
    let mut regle = Regle::nouvelle(width, ui, typo);
    groupe_des_outils(&mut regle, ui);
    groupe_des_images(&mut regle);
    groupe_des_panneaux(&mut regle, ui);
    let img_badge = groupe_de_droite(&mut regle, width, board_img_count);
    TopbarLayout {
        buttons: regle.boutons,
        separators: regle.separateurs,
        img_badge,
    }
}

/// Les outils : la sélection et la main, puis les cinq qui posent quelque chose.
fn groupe_des_outils(regle: &mut Regle<'_>, ui: &UiState) {
    for outil in [ActiveTool::Select, ActiveTool::Pan] {
        regle.outil(
            UiAction::SelectTool(outil),
            outil.icone(),
            ui.active_tool == outil,
        );
    }
    regle.separateur(3.0, 9.0);
    for outil in [
        ActiveTool::Text,
        ActiveTool::Sticky,
        ActiveTool::Arrow,
        ActiveTool::Folder,
        ActiveTool::Membrane,
    ] {
        regle.outil(
            UiAction::SelectTool(outil),
            outil.icone(),
            ui.active_tool == outil,
        );
    }
    regle.separateur(3.0, 9.0);
}

/// Le bouton qui ajoute des photos, seul de son groupe.
fn groupe_des_images(regle: &mut Regle<'_>) {
    let (largeur, label) = regle.mesurer("Images", regle.ultra);
    regle.poser(
        UiAction::AddImages,
        IconType::Plus,
        label,
        largeur,
        false,
        6.0,
    );
    regle.separateur(1.0, 7.0);
}

/// Les panneaux et les deux bascules : ce qui change l'état de l'application.
fn groupe_des_panneaux(regle: &mut Regle<'_>, ui: &UiState) {
    regle.action(UiAction::Organize, IconType::Organize, "Ordonner", false);
    regle.action(UiAction::ToggleTimer, IconType::Timer, "Timer", false);
    regle.action(
        UiAction::ToggleStoryboard,
        IconType::Storyboard,
        "Storyboard",
        false,
    );
    regle.separateur(2.0, 8.0);
    regle.action(
        UiAction::ToggleMagnet,
        IconType::Magnet,
        "Aimant",
        ui.smart_align,
    );
    regle.action(
        UiAction::ToggleTransDomain,
        IconType::TransDomain,
        "Trans-domaines",
        ui.trans_domain,
    );
}

/// Le groupe de droite, **posé depuis le bord droit** : sa largeur se calcule avant de
/// commencer, sinon il ne saurait pas où démarrer.
///
/// Rend le badge du nombre d'images, qui se pose après le dernier bouton.
fn groupe_de_droite(
    regle: &mut Regle<'_>,
    width: f32,
    board_img_count: usize,
) -> Option<(f32, String)> {
    let fin_de_gauche = regle.x;
    let s = regle.s;
    // Les deux premiers gardent leur libellé plus longtemps que les trois derniers.
    let collab = regle.mesurer("Collaborer", regle.ultra);
    let export = regle.mesurer("Exporter", regle.ultra);
    let droite = [
        (
            UiAction::TogglePlugins,
            IconType::Plugins,
            regle.mesurer("Plugins", regle.compact),
        ),
        (
            UiAction::TogglePreset,
            IconType::Preset,
            regle.mesurer("Preset", regle.compact),
        ),
        (
            UiAction::ToggleDomains,
            IconType::Domains,
            regle.mesurer("Domaines", regle.compact),
        ),
    ];

    let badge_w = if board_img_count > 0 { 42.0 * s } else { 0.0 };
    let total = collab.0
        + 8.0 * s
        + export.0
        + 8.0 * s
        + droite.iter().map(|(_, _, (w, _))| *w).sum::<f32>()
        + 8.0 * s
        + badge_w
        + 16.0 * s;
    regle.x = (width - total - 12.0 * s).max(fin_de_gauche + 16.0 * s);

    regle.poser(
        UiAction::ToggleCollab,
        IconType::Collab,
        collab.1,
        collab.0,
        false,
        5.0,
    );
    regle.separateur(1.0, 7.0);
    regle.poser(
        UiAction::ExportMenu,
        IconType::Export,
        export.1,
        export.0,
        false,
        5.0,
    );
    regle.separateur(1.0, 7.0);
    for (action, icone, (largeur, label)) in droite {
        regle.poser(action, icone, label, largeur, false, 4.0);
    }

    (board_img_count > 0).then(|| (regle.x + 4.0 * s, format!("{board_img_count}img")))
}

fn push_ui_rounded_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.min(w / 2.0).min(h / 2.0);
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
}

pub fn draw_tool_button(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    rect: ScaledRect,
    icon: IconType,
    state: ButtonState,
) {
    let ScaledRect { x, y, w, h, scale } = rect;
    let ButtonState { active, hover } = state;
    let bg_color = if active {
        theme.bg_active
    } else if hover {
        theme.bg_hover
    } else {
        Color::TRANSPARENT
    };

    if bg_color != Color::TRANSPARENT {
        let mut p = Paint::default();
        p.set_color(bg_color);
        p.anti_alias = true;
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(&mut pb, x, y, w, h, 4.0 * scale);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(
                &path,
                &p,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    if active {
        let mut sp = Paint::default();
        sp.set_color(theme.border_accent);
        sp.anti_alias = true;
        let stroke = Stroke {
            width: 1.0 * scale,
            ..Default::default()
        };
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(
            &mut pb,
            x + 0.5 * scale,
            y + 0.5 * scale,
            w - 1.0 * scale,
            h - 1.0 * scale,
            4.0 * scale,
        );
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);
        }
    }

    let icon_color = if active {
        theme.text_accent
    } else if hover {
        theme.text_primary
    } else {
        theme.text_muted
    };

    let icon_size = 14.0 * scale;
    let icon_x = x + (w - icon_size) / 2.0;
    let icon_y = y + (h - icon_size) / 2.0;
    draw_icon_scaled(
        pixmap,
        icon,
        icon_x,
        icon_y,
        icon_size,
        icon_color,
        1.4 * scale,
    );
}

/// Le fond arrondi d'un bouton, et son liseré quand il est actif.
///
/// Les deux vont ensemble : ce sont les deux couches sous le contenu, et elles partagent le
/// même rectangle arrondi.
fn fond_du_bouton(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    rect: ScaledRect,
    state: ButtonState,
    rayon: f32,
) {
    let ScaledRect { x, y, w, h, scale } = rect;
    let ButtonState { active, hover } = state;
    let bg_color = if active {
        theme.bg_active
    } else if hover {
        theme.bg_hover
    } else {
        Color::TRANSPARENT
    };
    if bg_color != Color::TRANSPARENT {
        let mut p = Paint {
            anti_alias: true,
            ..Default::default()
        };
        p.set_color(bg_color);
        let mut pb = PathBuilder::new();
        push_ui_rounded_rect(&mut pb, x, y, w, h, rayon * scale);
        if let Some(path) = pb.finish() {
            pixmap.fill_path(
                &path,
                &p,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }
    if !active {
        return;
    }
    let mut sp = Paint {
        anti_alias: true,
        ..Default::default()
    };
    sp.set_color(theme.border_accent);
    let stroke = Stroke {
        width: 1.0 * scale,
        ..Default::default()
    };
    // Le liseré se pose sur le demi-pixel : un trait d'un pixel centré sur la frontière
    // s'étalerait en deux demi-teintes.
    let mut pb = PathBuilder::new();
    push_ui_rounded_rect(
        &mut pb,
        x + 0.5 * scale,
        y + 0.5 * scale,
        w - 1.0 * scale,
        h - 1.0 * scale,
        rayon * scale,
    );
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &sp, &stroke, Transform::identity(), None);
    }
}

/// La couleur du contenu d'un bouton, selon ce qu'il vit.
fn teinte_du_contenu(theme: &Theme, state: ButtonState) -> Color {
    if state.active {
        theme.text_accent
    } else if state.hover {
        theme.text_primary
    } else {
        theme.text_secondary
    }
}

pub fn draw_action_button(
    pixmap: &mut PixmapMut,
    typo: &Typography,
    theme: &Theme,
    rect: ScaledRect,
    icon: IconType,
    label: &str,
    state: ButtonState,
) {
    let ScaledRect { x, y, w, h, scale } = rect;
    fond_du_bouton(pixmap, theme, rect, state, 4.0);
    let color = teinte_du_contenu(theme, state);
    let icon_size = 14.0 * scale;
    let icon_y = y + (h - icon_size) / 2.0;
    // Sans libellé, le bouton est carré et son icône est centrée ; avec, elle se range à
    // gauche et le texte suit à une abscisse que la mesure du bouton a déjà réservée.
    if label.is_empty() {
        let icon_x = x + (w - icon_size) / 2.0;
        draw_icon_scaled(pixmap, icon, icon_x, icon_y, icon_size, color, 1.3 * scale);
        return;
    }
    draw_icon_scaled(
        pixmap,
        icon,
        x + 8.0 * scale,
        icon_y,
        icon_size,
        color,
        1.3 * scale,
    );
    let font = ACTION_LABEL_FONT * scale;
    typo.draw_text(
        pixmap,
        label,
        x + ACTION_LABEL_X * scale,
        y + (h - font) / 2.0,
        TextStyle {
            size: font,
            color,
            face: if state.active {
                Face::Bold
            } else {
                Face::Regular
            },
        },
    );
}
