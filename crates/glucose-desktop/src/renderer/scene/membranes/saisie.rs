//! **Le titre d'une membrane qu'on écrit** (fiche 99) : ce qu'on tape, sa sélection, son curseur.
//!
//! Le titre se dessinait toujours depuis le document : la saisie remplissait son tampon, et rien
//! ne le montrait — ni lettre, ni curseur — jusqu'à ce qu'un clic ailleurs l'enregistre. C'était
//! le défaut qu'il a vu à son écran le 9/11/2026, sur le geste même qui fait une région
//! (`Ctrl+G`, puis un double-clic pour la nommer). Le texte et le post-it recevaient la saisie ;
//! la membrane, seule, avait été oubliée.

use crate::renderer::scale::WorldScale;
use crate::renderer::{curseur, PaintKit, TextEditSession};
use crate::typography::Face;
use tiny_skia::{Color, PixmapMut, Rect};

/// Un titre tel qu'il se pose : son texte, son coin haut-gauche à l'écran, son corps.
#[derive(Clone, Copy)]
pub(super) struct Titre<'a> {
    pub texte: &'a str,
    pub at: (f32, f32),
    pub corps: f32,
}

impl Titre<'_> {
    /// L'abscisse de ce rang du texte, mesurée **par la fonction même du dessin**, sur le début
    /// du texte : la barre tombe exactement entre deux lettres.
    fn x(&self, kit: PaintKit<'_>, rang: usize) -> f32 {
        let avant = self.texte.get(..rang).unwrap_or(self.texte);
        self.at.0 + kit.typography.measure_text(avant, self.corps, Face::Bold).0
    }
}

/// **La sélection, sous le titre** — dessinée après, elle recouvrirait ses lettres.
pub(super) fn surligner(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    titre: Titre<'_>,
    saisie: &TextEditSession,
) {
    let (debut, fin) = saisie.selection.range();
    if debut == fin {
        return;
    }
    let (gauche, droite) = (titre.x(kit, debut), titre.x(kit, fin));
    let hauteur = titre.corps * curseur::HAUTEUR;
    if let Some(boite) = Rect::from_xywh(gauche, titre.at.1, droite - gauche, hauteur) {
        crate::renderer::scale::fill_crisp(pixmap, boite, kit.theme.text_selection);
    }
}

/// **Le curseur, sur le titre**, quand la session clignote « allumé » (BLINK-1 : la phase se lit).
pub(super) fn poser_le_curseur(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    titre: Titre<'_>,
    (saisie, scale, encre): (&TextEditSession, WorldScale, Color),
) {
    if saisie.curseur_visible {
        let x = titre.x(kit, saisie.selection.head);
        curseur::poser(pixmap, (x, titre.at.1), titre.corps, scale, encre);
    }
}

#[cfg(test)]
mod tests {
    use crate::app::GlucoseApp;
    use crate::interactions::text_edit::keys::Command;
    use glucose_core::types::{Annotation, Viewport};
    use std::time::{Duration, Instant};

    const TAILLE: (u32, u32) = (800, 600);
    /// Le bord haut de la membrane, à l'écran : le titre se pose au-dessus.
    const HAUT: f64 = 300.0;

    /// Une application sans fenêtre, vue à 100 % : la membrane « m » de ce titre, et sa voisine
    /// « Feu », qu'on n'écrit pas — l'accueil éteint : sa lueur et son message changent d'une
    /// image à l'autre.
    fn membrane(titre: Option<&str>) -> GlucoseApp {
        let mut app = GlucoseApp::new();
        app.ui.current_toast = None;
        if let Some(b) = app.store.active_board_mut() {
            b.annotations.clear();
        }
        let board = app.store.project.active_board_id.clone();
        for (id, x, largeur, titre) in [("m", 0.0, 480.0, titre), ("n", 540.0, 200.0, Some("Feu"))]
        {
            let mut m = Annotation::membrane(id, x, 0.0, largeur, 240.0);
            if let Annotation::Membrane { text, .. } = &mut m {
                *text = titre.map(str::to_string);
            }
            app.store.add_annotation(&board, m);
        }
        app.store.clear_selection();
        let vue = Viewport {
            x: 40.0,
            y: HAUT,
            scale: 1.0,
        };
        app.store.set_viewport(&board, vue);
        app
    }

    /// Ouvre la saisie du titre et y joue ces commandes, par le chemin du clavier ; le curseur
    /// reste ensuite « allumé » (son horloge part dans le futur : la phase lue est la première).
    fn ecrire(app: &mut GlucoseApp, commandes: Vec<Command>) {
        let texte = app.editable_text_of("m").expect("une membrane");
        app.start_text_edit("m".into(), texte);
        for c in commandes {
            app.apply_text_command(c, false);
        }
        let session = app.editing_session.as_mut().expect("en saisie");
        session.blink_timer = Instant::now() + Duration::from_secs(3600);
    }

    fn pixels(app: &mut GlucoseApp) -> Vec<u8> {
        app.une_image_sans_fenetre(TAILLE);
        app.pixmap.as_ref().expect("une image").data().to_vec()
    }

    /// La boîte `(x0, y0, x1, y1)` des pixels qui diffèrent entre deux images, bornes comprises.
    fn ecart(a: &[u8], b: &[u8]) -> Option<(usize, usize, usize, usize)> {
        let l = TAILLE.0 as usize;
        let mut boite: Option<(usize, usize, usize, usize)> = None;
        for (i, _) in a
            .chunks(4)
            .zip(b.chunks(4))
            .enumerate()
            .filter(|(_, (p, q))| p != q)
        {
            let (x, y) = (i % l, i / l);
            boite = Some(boite.map_or((x, y, x, y), |(x0, y0, x1, y1)| {
                (x0.min(x), y0.min(y), x1.max(x), y1.max(y))
            }));
        }
        boite
    }

    /// **Une barre qui part du haut du titre et mesure une ligne de ses lettres** : ici le corps
    /// vaut 16 × √1,5 ≈ 19,6 et l'élévation 18 × √1,5 ≈ 22 (TITRE-1) ; la barre en fait 1,2 fois le
    /// corps, jambages compris — comme un « g » du titre, elle mord d'un pixel sur le bord.
    fn barre_du_titre(y0: usize, y1: usize) -> bool {
        let haut_du_titre = HAUT - 18.0 * 1.5_f64.sqrt();
        (y0 as f64 - haut_du_titre).abs() <= 1.0 && (22..=25).contains(&(y1 - y0 + 1))
    }

    /// **Ce qu'on tape dans un titre se voit, tel qu'il sera enregistré** (sa parole du 9/11/2026 :
    /// « on tape, les lettres n'apparaissent pas »). Écrire « salée » après « Eau » ne diffère du
    /// titre « Eau salée » au repos que d'**une barre de curseur**, au-dessus du bord, à droite —
    /// et la voisine « Feu » ne change pas.
    #[test]
    fn saisie_1_ce_qu_on_tape_dans_un_titre_se_voit_comme_il_sera_enregistre() {
        let enregistre = pixels(&mut membrane(Some("Eau salée")));
        let mut app = membrane(Some("Eau"));
        let avant = pixels(&mut app);
        assert!(
            ecart(&avant, &enregistre).is_some(),
            "les deux titres diffèrent"
        );
        ecrire(&mut app, vec![Command::Insert(" salée".into())]);
        let pendant = pixels(&mut app);
        let (x0, y0, x1, y1) = ecart(&pendant, &enregistre).expect("le curseur se voit");
        assert!(
            x1 - x0 <= 3,
            "en saisie, le titre diffère de l'enregistré sur {x0}..={x1} : plus qu'un curseur"
        );
        assert!(
            barre_du_titre(y0, y1),
            "une barre de la hauteur du titre : {y0}..={y1}"
        );
        assert!(
            x0 > 40 + 80,
            "le curseur suit la dernière lettre, pas la première : x = {x0}"
        );
    }

    /// **Un titre vide qu'on écrit montre son curseur** : c'est le titre d'une membrane neuve,
    /// celle que `Ctrl+G` vient de faire et qu'un double-clic va nommer.
    #[test]
    fn saisie_1_un_titre_vide_qu_on_ecrit_montre_son_curseur() {
        let mut app = membrane(None);
        let repos = pixels(&mut app);
        ecrire(&mut app, Vec::new());
        let pendant = pixels(&mut app);
        let (x0, y0, x1, y1) = ecart(&repos, &pendant).expect("un curseur, avant toute lettre");
        assert!(
            x1 - x0 <= 3 && barre_du_titre(y0, y1),
            "une barre de la hauteur du titre : {x0}..={x1}, {y0}..={y1}"
        );
    }

    /// **Une sélection se voit** : `Ctrl+A` dans un titre surligne ce que la frappe suivante va
    /// remplacer — sans quoi on efface un mot qu'on ne voyait pas choisi.
    #[test]
    fn saisie_1_la_selection_d_un_titre_se_voit() {
        let enregistre = pixels(&mut membrane(Some("Eau salée")));
        let mut app = membrane(Some("Eau salée"));
        ecrire(&mut app, vec![Command::SelectAll]);
        let pendant = pixels(&mut app);
        let (x0, _, x1, _) = ecart(&pendant, &enregistre).expect("la sélection se voit");
        assert!(x1 - x0 >= 60, "le surlignage couvre le titre : {x0}..={x1}");
    }
}
