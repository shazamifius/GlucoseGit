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
use crate::icons::IconType;
use crate::typography::{Face, Typography};

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
    /// Le libellé montré : vide quand la barre ne garde que les icônes.
    pub label: &'static str,
    /// **Le nom du bouton, toujours**, quelle que soit la densité : le rail l'écrit à côté
    /// de l'icône (fiche 58) — trois icônes de Tauri se ressemblent trop pour se passer de lui.
    pub nom: &'static str,
    pub active: bool,
    pub is_tool: bool,
}

/// **Un libellé mesuré** : la largeur du bouton, ce qu'il montre, et son nom.
#[derive(Clone, Copy)]
struct Libelle {
    largeur: f32,
    montre: &'static str,
    nom: &'static str,
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
    /// Les libellés cèdent la place quand la barre ne tient pas : d'abord ceux de droite
    /// (compact), puis tous (ultra-compact).
    compact: bool,
    ultra: bool,
}

/// Combien de libellés la barre garde, du plus riche au plus sobre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Densite {
    Complete,
    Compacte,
    Icones,
}

impl<'a> Regle<'a> {
    fn nouvelle(ui: &UiState, typo: &'a Typography, densite: Densite) -> Self {
        let s = ui.scale();
        // Les boutons se centrent sur la barre sous la barre d'état (BORD-1).
        let marge = ui.marge_du_haut();
        let topbar_h = ui.topbar_height() - marge;
        let cote = 30.0 * s;
        let haut_action = 28.0 * s;
        Self {
            boutons: Vec::new(),
            separateurs: Vec::new(),
            x: 100.0 * s,
            s,
            outil: (cote, marge + (topbar_h - cote) / 2.0),
            action: (haut_action, marge + (topbar_h - haut_action) / 2.0),
            typo,
            compact: densite != Densite::Complete,
            ultra: densite == Densite::Icones,
        }
    }

    /// La largeur d'un bouton d'action et le libellé qu'il gardera : un bouton sans libellé
    /// est carré comme un outil.
    fn mesurer(&self, nom: &'static str, efface: bool) -> Libelle {
        let (largeur, montre) = if efface {
            (self.outil.0, "")
        } else {
            (action_button_width(self.typo, nom, self.s), nom)
        };
        Libelle {
            largeur,
            montre,
            nom,
        }
    }

    /// Un outil carré : il n'a qu'une icône, et il est actif ou non.
    fn outil(&mut self, action: UiAction, icon: IconType, nom: &'static str, actif: bool) {
        let (cote, y) = self.outil;
        self.boutons.push(TopbarButtonDef {
            action,
            x: self.x,
            y,
            w: cote,
            h: cote,
            icon,
            label: "",
            nom,
            active: actif,
            is_tool: true,
        });
        self.x += cote + 2.0 * self.s;
    }

    /// Un bouton d'action, avec son libellé tant que la fenêtre le permet.
    fn action(&mut self, action: UiAction, icon: IconType, label: &'static str, actif: bool) {
        let libelle = self.mesurer(label, self.ultra);
        self.poser(action, icon, libelle, actif, 4.0);
    }

    /// Le même, avec l'écart qui le suit : les groupes ne respirent pas tous pareil.
    fn poser(
        &mut self,
        action: UiAction,
        icon: IconType,
        libelle: Libelle,
        actif: bool,
        ecart: f32,
    ) {
        let Libelle {
            largeur,
            montre: label,
            nom,
        } = libelle;
        let (hauteur, y) = self.action;
        self.boutons.push(TopbarButtonDef {
            action,
            x: self.x,
            y,
            w: largeur,
            h: hauteur,
            icon,
            label,
            nom,
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

/// # La densité se mesure, elle ne se choisit pas
///
/// La barre cédait ses libellés à des largeurs de fenêtre écrites en dur (1 320, puis 1 050
/// pixels) : un bouton de plus — la Time Machine — et le groupe de droite sortait de l'écran
/// avant que le seuil ne tombe. Elle se pose maintenant du plus riche au plus sobre — tous
/// les libellés, puis sans ceux de droite, puis des icônes seules — et garde **la première
/// qui tient**. Les libellés sont mesurés ; la fenêtre décide.
pub fn layout_topbar(
    width: f32,
    ui: &UiState,
    typo: &Typography,
    board_img_count: usize,
) -> TopbarLayout {
    let mut barre = poser_la_barre(width, ui, typo, board_img_count, Densite::Icones).0;
    for densite in [Densite::Complete, Densite::Compacte] {
        let (essai, tient) = poser_la_barre(width, ui, typo, board_img_count, densite);
        if tient {
            barre = essai;
            break;
        }
    }
    barre
}

/// **La barre tient-elle en icônes seules** dans `width` ? Sinon, elle part sur le côté
/// ([`super::rail`]) : la dernière marche de la même mesure.
pub(crate) fn tient_en_icones(
    width: f32,
    ui: &UiState,
    typo: &Typography,
    board_img_count: usize,
) -> bool {
    poser_la_barre(width, ui, typo, board_img_count, Densite::Icones).1
}

/// **Les boutons de la barre, dans leur ordre** — ce que le rail repose en grille. La même
/// liste, produite par la même fonction : aucun bouton n'existe dans l'un sans l'autre.
pub(crate) fn les_boutons(ui: &UiState, typo: &Typography) -> Vec<TopbarButtonDef> {
    poser_la_barre(f32::MAX, ui, typo, 0, Densite::Icones)
        .0
        .buttons
}

/// **Ce qu'un bouton de la barre fait lui-même**, avant que l'application n'en reçoive
/// l'action : l'aimant bascule et le dit, la collaboration dit qu'elle n'existe pas encore.
/// Une seule fois, pour la barre et pour le rail.
pub(crate) fn effet_du_bouton(ui: &mut UiState, action: UiAction) -> UiAction {
    match action {
        UiAction::ToggleMagnet => {
            ui.smart_align = !ui.smart_align;
            ui.show_toast(if ui.smart_align {
                "Aimant activé"
            } else {
                "Aimant désactivé"
            });
        }
        UiAction::ToggleCollab => {
            // Fiche 09 § 9 : aucun réseau n'existe. Le bouton ne « connecte » rien, et ne
            // doit pas le prétendre.
            ui.show_toast(super::NOT_YET_COLLAB);
        }
        _ => {}
    }
    action
}

/// La barre à cette densité, et si elle tient dans la fenêtre.
fn poser_la_barre(
    width: f32,
    ui: &UiState,
    typo: &Typography,
    board_img_count: usize,
    densite: Densite,
) -> (TopbarLayout, bool) {
    let mut regle = Regle::nouvelle(ui, typo, densite);
    groupe_des_outils(&mut regle, ui);
    groupe_des_images(&mut regle);
    groupe_des_panneaux(&mut regle, ui);
    let (img_badge, tient) = groupe_de_droite(&mut regle, width, board_img_count);
    let barre = TopbarLayout {
        buttons: regle.boutons,
        separators: regle.separateurs,
        img_badge,
    };
    (barre, tient)
}

/// Les outils : la sélection et la main, puis les cinq qui posent quelque chose.
fn groupe_des_outils(regle: &mut Regle<'_>, ui: &UiState) {
    for outil in [ActiveTool::Select, ActiveTool::Pan] {
        regle.outil(
            UiAction::SelectTool(outil),
            outil.icone(),
            outil.nom(),
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
            outil.nom(),
            ui.active_tool == outil,
        );
    }
    regle.separateur(3.0, 9.0);
}

/// Le bouton qui ajoute des photos, seul de son groupe.
fn groupe_des_images(regle: &mut Regle<'_>) {
    let libelle = regle.mesurer("Images", regle.ultra);
    regle.poser(UiAction::AddImages, IconType::Plus, libelle, false, 6.0);
    regle.separateur(1.0, 7.0);
}

/// Les panneaux, l'aimant, et Trans-domaines.
fn groupe_des_panneaux(regle: &mut Regle<'_>, ui: &UiState) {
    regle.action(UiAction::Organize, IconType::Organize, "Ordonner", false);
    regle.action(UiAction::ToggleTimer, IconType::Timer, "Timer", false);
    regle.action(
        UiAction::ToggleTimeMachine,
        IconType::Histoire,
        "Time Machine",
        false,
    );
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
    // Trans-domaines est posé, et sa fonction n'est pas encore définie : il n'a donc pas d'état
    // à montrer, et ne s'allume jamais (fiche 29 § 3.1).
    regle.action(
        UiAction::TransDomain,
        IconType::TransDomain,
        "Trans-domaines",
        false,
    );
}

/// Le groupe de droite, **posé depuis le bord droit** : sa largeur se calcule avant de
/// commencer, sinon il ne saurait pas où démarrer.
///
/// Rend le badge du nombre d'images, qui se pose après le dernier bouton, et si le groupe
/// tient sans chevaucher celui de gauche.
fn groupe_de_droite(
    regle: &mut Regle<'_>,
    width: f32,
    board_img_count: usize,
) -> (Option<(f32, String)>, bool) {
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
    let total = collab.largeur
        + 8.0 * s
        + export.largeur
        + 8.0 * s
        + droite.iter().map(|(_, _, l)| l.largeur).sum::<f32>()
        + 8.0 * s
        + badge_w
        + 16.0 * s;
    let depart = width - total - 12.0 * s;
    let tient = depart >= fin_de_gauche + 16.0 * s;
    regle.x = depart.max(fin_de_gauche + 16.0 * s);

    regle.poser(UiAction::ToggleCollab, IconType::Collab, collab, false, 5.0);
    regle.separateur(1.0, 7.0);
    regle.poser(UiAction::ExportMenu, IconType::Export, export, false, 5.0);
    regle.separateur(1.0, 7.0);
    for (action, icone, libelle) in droite {
        regle.poser(action, icone, libelle, false, 4.0);
    }

    let badge = (board_img_count > 0).then(|| (regle.x + 4.0 * s, format!("{board_img_count}img")));
    (badge, tient)
}

mod dessin;
pub use dessin::{draw_action_button, draw_tool_button};
