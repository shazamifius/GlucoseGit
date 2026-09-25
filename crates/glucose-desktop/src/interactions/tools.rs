//! Les outils de création : ce qui naît sous le curseur quand on clique avec un outil.
//!
//! # Une seule fabrique par nœud
//!
//! Chaque nœud a **une** fonction qui le fait naître, et elle passe par le constructeur du
//! modèle. Le clic d'outil, la carte d'accueil, un test : tous appellent la même, donc tous
//! obtiennent la même chose. Avant, le clic recopiait quatorze champs par nœud et faisait naître
//! un pense-bête de 180 × 130 en corps 12 là où la fiche 06 § 5.2 en veut un de 160 × 120 en
//! corps 13 — et où le rendu, la sélection et le magnétisme en attendaient un de 160 × 120.
//!
//! Ce qu'une fabrique pose, c'est ce que le document doit **connaître** : une carte de texte
//! porte sa largeur de naissance et la hauteur de son texte (TEXT-FIT-1), parce que l'arbitre
//! de clic, l'index spatial et les poignées lisent cette boîte dans le document. Tout le reste
//! — couleur, corps, taille d'un pense-bête — reste `None` : c'est le rendu qui connaît les
//! valeurs de la référence, et un document n'a pas à les recopier.

use crate::app::GlucoseApp;
use crate::renderer::card::text_card_fit_height;
use crate::renderer::math::MathRenderer;
use crate::typography::Typography;
use crate::ui::ActiveTool;
use glucose_core::text::Selection;
use glucose_core::types::{Annotation, CanvasFolder, DEFAULT_TEXT_CARD_WIDTH};

/// Ce qu'une carte de texte fraîche dit.
pub const NEW_TEXT: &str = "Nouveau texte";
/// Ce qu'un pense-bête frais dit.
pub const NEW_STICKY: &str = "Nouvelle note";
/// Le nom d'une membrane fraîche.
pub const NEW_MEMBRANE: &str = "Groupe";
/// Le nom d'un dossier frais.
pub const NEW_FOLDER: &str = "Dossier";

/// Le vecteur d'une flèche posée d'un clic, en unités monde — en attendant qu'une flèche se
/// dessine par glisser (fiche 12, 2.B.1).
pub const NEW_ARROW_VECTOR: (f64, f64) = (120.0, 80.0);

/// La taille d'une membrane et d'un dossier posés d'un clic, en unités monde.
///
/// La même pour les deux : ce sont les deux conteneurs du canevas, et rien ne justifierait
/// qu'ils naissent de tailles différentes. Le minimum de la fiche 06 § 8.1 est 180 × 120 ;
/// celle-ci laisse de quoi poser quelque chose dedans — en attendant le dessin par glisser.
pub const NEW_CONTAINER_SIZE: (f64, f64) = (320.0, 240.0);

/// En deçà de cette longueur, en unités monde, une flèche tracée est tenue pour un simple
/// clic et reprend son vecteur de naissance.
///
/// La valeur n'est pas choisie : c'est la demi-bande qui la désigne au curseur
/// ([`glucose_core::arrow::BAND_PX`]). Plus courte qu'elle, une flèche tient entièrement
/// sous le point qui sert à l'attraper — elle ne serait qu'une cible sans trait.
pub const MIN_ARROW_LENGTH: f64 = glucose_core::arrow::BAND_PX / 2.0;

/// Une carte de texte à sa largeur de naissance, haute comme son texte (TEXT-FIT-1).
pub fn text_card(
    typography: &Typography,
    math: &MathRenderer,
    id: impl Into<String>,
    x: f64,
    y: f64,
    text: impl Into<String>,
) -> Annotation {
    let text = text.into();
    let height = text_card_fit_height(typography, math, &text, DEFAULT_TEXT_CARD_WIDTH);
    let mut card = Annotation::text(id, x, y, text);
    if let Annotation::Text {
        width: w,
        height: h,
        ..
    } = &mut card
    {
        *w = Some(DEFAULT_TEXT_CARD_WIDTH);
        *h = Some(height);
    }
    card
}

/// Un objet en train de naître sous la main.
///
/// # DRAW-1 — le geste se décide au relâchement
///
/// Un outil de création pose son objet dès l'appui, à sa taille de naissance, puis le suit
/// tant que la main glisse. Un **clic** garde donc la naissance, un **glisser** dessine —
/// et c'est la même différence, au même endroit, que celle qui sépare un pan d'un menu
/// contextuel au clic droit. Aucun mode, aucun modificateur : le curseur a bougé, ou non.
///
/// L'objet est créé tout de suite plutôt qu'au relâchement, pour qu'on le voie naître. Les
/// écritures qui suivent tiennent dans une seule entrée d'annulation.
#[derive(Debug, Clone)]
pub struct DrawSession {
    /// Le nœud posé à l'appui, celui que le glisser étire.
    pub id: String,
    /// Le point du monde où la main s'est posée.
    pub start: (f64, f64),
}

impl GlucoseApp {
    /// Pose sous `(wx, wy)` ce que l'outil actif crée, puis rend l'outil de sélection.
    ///
    /// Rend `false` pour les outils qui ne créent rien — sélection et main — : le clic n'a
    /// pas été consommé.
    pub fn place_with_tool(&mut self, wx: f64, wy: f64) -> bool {
        // PLACEMENT-1 : l'élément se pose là où son fantôme était — aimanté.
        let (wx, wy) = self
            .placement_aimante((wx, wy))
            .map_or((wx, wy), |f| (f.rect.left, f.rect.top));
        self.ui.placement.fantome = None;
        match self.ui.active_tool {
            ActiveTool::Select | ActiveTool::Pan => return false,
            ActiveTool::Text => self.place_text_card(wx, wy),
            ActiveTool::Sticky => self.place_sticky(wx, wy),
            ActiveTool::Arrow => self.place_arrow(wx, wy),
            ActiveTool::Membrane => self.place_membrane(wx, wy),
            ActiveTool::Folder => self.place_folder(wx, wy),
        }
        // L'outil annonce ce qu'il vient de poser, et il l'annonce **ici**. Chaque fabrique
        // portait son propre message : quatre sites pour un seul événement.
        if let Some(pose) = self.ui.active_tool.creation_label() {
            self.ui.show_toast(pose);
        }
        self.ui.active_tool = ActiveTool::Select;
        true
    }

    fn place_text_card(&mut self, wx: f64, wy: f64) {
        let id = self.store.generate_id("text");
        let card = text_card(
            &self.renderer.typography,
            &self.renderer.math,
            &id,
            wx,
            wy,
            NEW_TEXT,
        );
        self.add_and_edit(card, id, NEW_TEXT);
    }

    fn place_sticky(&mut self, wx: f64, wy: f64) {
        let id = self.store.generate_id("sticky");
        let sticky = Annotation::sticky(&id, wx, wy, NEW_STICKY);
        self.add_and_edit(sticky, id, NEW_STICKY);
    }

    /// Une carte ou un pense-bête naît en édition : le texte de départ est là pour être
    /// remplacé, donc il naît **sélectionné** — la première touche l'efface.
    ///
    /// Il naissait le curseur à la fin, et il fallait `Ctrl+A` avant d'écrire : le commentaire
    /// disait « là pour être remplacé », et le code le laissait être prolongé.
    fn add_and_edit(&mut self, ann: Annotation, id: String, text: &str) {
        let board = self.store.project.active_board_id.clone();
        self.store.add_annotation(&board, ann);
        self.start_text_edit_at(id, text.to_string(), Selection::all(text));
    }

    /// Pose une flèche et ouvre le geste qui l'étire (DRAW-1), **accrochée** à ce qu'elle
    /// touche (ARROW-2).
    fn place_arrow(&mut self, wx: f64, wy: f64) {
        let board = self.store.project.active_board_id.clone();
        let id = self.store.generate_id("arrow");
        let (dx, dy) = NEW_ARROW_VECTOR;
        let depart = self.snap_for_arrow((wx, wy), &[]);
        let (sx, sy) = depart.point;

        self.store.begin_live_edit();
        let mut fleche = Annotation::arrow(&id, sx, sy, sx + dx, sy + dy);
        if let Annotation::Arrow { source_id, .. } = &mut fleche {
            *source_id = depart.node.clone();
        }
        self.store.add_annotation(&board, fleche);
        self.draw_session = Some(DrawSession {
            id,
            start: (sx, sy),
        });
    }

    /// Où un bout de flèche se pose ici, et à quoi il s'accroche (ARROW-2).
    fn snap_for_arrow(&self, point: (f64, f64), exclude: &[&str]) -> glucose_core::arrow::Snap {
        self.store.active_board().map_or_else(
            || glucose_core::arrow::Snap::free(point),
            |board| glucose_core::arrow::snap_to_nearest(board, point, exclude),
        )
    }

    /// Le glisser en cours amène la pointe de la flèche sous le curseur — ou sur le nœud
    /// qu'elle vise, s'il est assez près (ARROW-2).
    pub fn update_draw(&mut self, wx: f64, wy: f64) {
        let Some(session) = self.draw_session.clone() else {
            return;
        };
        let board = self.store.project.active_board_id.clone();
        // La flèche elle-même et le nœud dont elle part sont écartés par `snap_for_tip` :
        // sans quoi elle se refermerait sur son origine dès le premier pixel de glisser.
        let cible = self.store.active_board().map_or_else(
            || glucose_core::arrow::Snap::free((wx, wy)),
            |b| glucose_core::arrow::snap_for_tip(b, &session.id, (wx, wy)),
        );

        self.store.update_annotation(&board, &session.id, |ann| {
            if let Annotation::Arrow {
                x2, y2, target_id, ..
            } = ann
            {
                *x2 = cible.point.0;
                *y2 = cible.point.1;
                *target_id = cible.node.clone();
            }
        });
        self.mark_dirty();
    }

    /// Ferme le geste de création : une seule entrée d'annulation pour tout le tracé.
    ///
    /// Une flèche **dégénérée** — celle d'un clic dont la main n'a pas bougé, ou d'un glisser
    /// revenu à son point de départ — reprend son vecteur de naissance : un trait de longueur
    /// nulle ne se voit pas, ne se clique pas, et ne s'annule donc plus qu'à l'aveugle.
    pub fn finish_draw(&mut self) {
        let Some(session) = self.draw_session.take() else {
            return;
        };
        let board = self.store.project.active_board_id.clone();
        let (dx, dy) = NEW_ARROW_VECTOR;
        self.store.update_annotation(&board, &session.id, |ann| {
            if let Annotation::Arrow { x, y, x2, y2, .. } = ann {
                if (*x2 - *x).hypot(*y2 - *y) < MIN_ARROW_LENGTH {
                    *x2 = *x + dx;
                    *y2 = *y + dy;
                }
            }
        });
        self.store.end_live_edit();
        self.mark_dirty();
    }

    fn place_membrane(&mut self, wx: f64, wy: f64) {
        let board = self.store.project.active_board_id.clone();
        let id = self.store.generate_id("membrane");
        let (w, h) = NEW_CONTAINER_SIZE;
        let mut membrane = Annotation::membrane(id, wx, wy, w, h);
        if let Annotation::Membrane { text, .. } = &mut membrane {
            *text = Some(NEW_MEMBRANE.to_string());
        }
        self.store.add_annotation(&board, membrane);
    }

    /// Le dossier capture ce qui se trouve sous lui : `create_folder` le fait, crée le tableau
    /// enfant, et enregistre le tout comme **un** geste annulable.
    fn place_folder(&mut self, wx: f64, wy: f64) {
        let board = self.store.project.active_board_id.clone();
        let mut folder = CanvasFolder::new(self.store.generate_id("folder"), NEW_FOLDER, "");
        folder.x = wx;
        folder.y = wy;
        (folder.width, folder.height) = NEW_CONTAINER_SIZE;
        self.store.create_folder(&board, folder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glucose_core::types::{DEFAULT_STICKY_HEIGHT, DEFAULT_STICKY_WIDTH};

    fn app_with_tool(tool: ActiveTool) -> GlucoseApp {
        let mut app = GlucoseApp::new();
        app.ui.active_tool = tool;
        app
    }

    fn last_annotation(app: &GlucoseApp) -> &Annotation {
        app.store
            .active_board()
            .expect("un tableau")
            .annotations
            .last()
            .expect("une annotation")
    }

    /// Fiche 06 § 5.2 — un pense-bête naît de la taille de naissance du modèle, sans rien
    /// recopier dans le document : le rendu, la sélection et le magnétisme lisent la même boîte.
    #[test]
    fn test_a_sticky_is_born_with_the_model_size_and_nothing_hardcoded() {
        let mut app = app_with_tool(ActiveTool::Sticky);
        assert!(app.place_with_tool(10.0, 20.0));

        let sticky = last_annotation(&app);
        assert_eq!(
            sticky.size(),
            Some((DEFAULT_STICKY_WIDTH, DEFAULT_STICKY_HEIGHT))
        );
        let Annotation::Sticky {
            width,
            height,
            font_size,
            color,
            bg_color,
            ..
        } = sticky
        else {
            panic!("un pense-bête");
        };
        assert!(
            width.is_none() && height.is_none(),
            "la taille est celle du modèle"
        );
        assert!(font_size.is_none() && color.is_none() && bg_color.is_none());
        assert_eq!(
            app.ui.active_tool,
            ActiveTool::Select,
            "l'outil rend la main"
        );
        assert!(
            app.editing_session.is_some(),
            "le pense-bête naît en édition"
        );
    }

    /// TEXT-FIT-1 — une carte naît à sa largeur de naissance et haute comme son texte, et le
    /// document le sait : c'est ce que l'arbitre de clic et les poignées liront.
    #[test]
    fn test_a_text_card_is_born_at_its_birth_width_and_fitted_height() {
        let mut app = app_with_tool(ActiveTool::Text);
        assert!(app.place_with_tool(0.0, 0.0));

        let card = last_annotation(&app);
        let expected = text_card_fit_height(
            &app.renderer.typography,
            &app.renderer.math,
            NEW_TEXT,
            DEFAULT_TEXT_CARD_WIDTH,
        );
        assert_eq!(card.size(), Some((DEFAULT_TEXT_CARD_WIDTH, expected)));
    }

    /// **Une carte et un pense-bête naissent leur texte sélectionné** : la première lettre
    /// tapée remplace « Nouveau texte » au lieu de s'y ajouter.
    #[test]
    fn test_le_texte_de_naissance_se_remplace_a_la_premiere_touche() {
        use crate::interactions::text_edit::keys::Command;
        for (outil, provisoire) in [
            (ActiveTool::Text, NEW_TEXT),
            (ActiveTool::Sticky, NEW_STICKY),
        ] {
            let mut app = app_with_tool(outil);
            assert!(app.place_with_tool(0.0, 0.0));
            let session = app.editing_session.as_ref().expect("en edition");
            assert_eq!(session.selection, Selection::all(provisoire));
            app.apply_text_command(Command::Insert("A".into()), false);
            assert_eq!(
                app.editing_session.as_ref().expect("en edition").buffer,
                "A"
            );
        }
    }

    /// Les outils qui ne créent rien ne consomment pas le clic.
    #[test]
    fn test_select_and_pan_create_nothing() {
        for tool in [ActiveTool::Select, ActiveTool::Pan] {
            let mut app = app_with_tool(tool);
            let before = app
                .store
                .active_board()
                .expect("un tableau")
                .annotations
                .len();
            assert!(!app.place_with_tool(0.0, 0.0));
            let after = app
                .store
                .active_board()
                .expect("un tableau")
                .annotations
                .len();
            assert_eq!(before, after);
            assert_eq!(app.ui.active_tool, tool);
        }
    }

    /// Un dossier et une membrane naissent de la même taille : ce sont les deux conteneurs du
    /// canevas.
    #[test]
    fn test_membrane_and_folder_are_born_the_same_size() {
        let mut app = app_with_tool(ActiveTool::Membrane);
        app.place_with_tool(0.0, 0.0);
        let membrane_size = last_annotation(&app).size();

        app.ui.active_tool = ActiveTool::Folder;
        app.place_with_tool(500.0, 500.0);
        let folder = &app.store.active_board().expect("un tableau").folders[0];

        assert_eq!(membrane_size, Some((folder.width, folder.height)));
        assert_eq!(membrane_size, Some(NEW_CONTAINER_SIZE));
    }
}
