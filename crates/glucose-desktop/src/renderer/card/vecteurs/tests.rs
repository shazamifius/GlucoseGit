//! Ce qu'une ligne peut dessiner hors de sa boîte (LIGNE-1, fiche 73 § 7).

use super::carte_en_vecteurs;
use crate::renderer::card::TextCard;
use crate::renderer::pass::{Clip, Pass};
use crate::renderer::richtext::draw::CODE_PAD_Y;
use crate::renderer::richtext::{font_of, LINE_FACTOR};
use crate::renderer::scale::WorldScale;
use crate::renderer::vectoriel::{Releve, Vectoriel};
use crate::typography::{toutes_les_polices, Face, Police, Typography};
use glucose_core::membrane_forme::Arrondi;
use glucose_core::text::BlockKind;
use glucose_math::Forme;
use tiny_skia::Color;

/// **LIGNE-1 — rien de ce qu'une ligne dessine ne sort de plus d'une hauteur de ligne de sa
/// boîte**, vérifié sur les tables de toutes les polices embarquées, à toutes les tailles de
/// ligne.
///
/// Le corps vaut 1 et la ligne `[0, LINE_FACTOR]` ; une lettre de taille `f` a sa ligne de base
/// en `f`. Son encre va de `f − yMax·f` à `f − yMin·f` (la boîte de la police, sa table `head`,
/// en em) : elle doit rester dans `[−LINE_FACTOR, 2·LINE_FACTOR]`. La plaque d'un `code` en
/// ligne va de `f − (montante + marge)·f` à `f − (descente − marge)·f` : de même.
#[test]
fn test_ligne_1_rien_ne_sort_d_une_ligne_de_plus_d_une_ligne() {
    let (haut, bas) = (-LINE_FACTOR, 2.0 * LINE_FACTOR);
    let tailles: Vec<f32> = [
        BlockKind::Heading(1),
        BlockKind::Heading(2),
        BlockKind::Small,
        BlockKind::Quote,
    ]
    .into_iter()
    .map(|genre| font_of(genre, 1.0))
    .collect();
    assert!(
        tailles.iter().any(|&f| f > 1.0),
        "l'épreuve doit voir un titre"
    );
    for police in toutes_les_polices() {
        let face = ttf_parser::Face::parse(police.octets, 0).expect("une police embarquée");
        let (b, u) = (face.global_bounding_box(), f32::from(face.units_per_em()));
        let (y_min, y_max) = (f32::from(b.y_min) / u, f32::from(b.y_max) / u);
        for &f in &tailles {
            let encre = (f - y_max * f, f - y_min * f);
            assert!(
                encre.0 >= haut && encre.1 <= bas,
                "{} à {f} : son encre va de {} à {} (ligne de 0 à {LINE_FACTOR})",
                police.nom,
                encre.0,
                encre.1
            );
        }
    }
    let typo = Typography::new();
    for face in Face::ALL {
        let m = typo
            .font(face)
            .horizontal_line_metrics(1.0)
            .expect("les métriques d'une police embarquée");
        for &f in &tailles {
            let plaque = (
                f - (m.ascent + CODE_PAD_Y) * f,
                f - (m.descent - CODE_PAD_Y) * f,
            );
            assert!(
                plaque.0 >= haut && plaque.1 <= bas,
                "{face:?} à {f} : la plaque d'un code va de {} à {}",
                plaque.0,
                plaque.1
            );
        }
    }
}

/// Le relevé, sans LIGNE-1 : chaque ligne se parcourt, et seul le relevé écarte ce qui ne touche
/// pas l'écran — la référence.
struct SansLigne1(Releve);

impl Vectoriel for SansLigne1 {
    fn rectangle(&mut self, boite: (f32, f32, f32, f32), rayon: f32, c: Color) {
        self.0.rectangle(boite, rayon, c);
    }
    fn rectangle_net(&mut self, boite: (f32, f32, f32, f32), c: Color) {
        self.0.rectangle_net(boite, c);
    }
    fn anneau(&mut self, milieu: Arrondi, demi: f32, c: Color) {
        self.0.anneau(milieu, demi, c);
    }
    fn disque(&mut self, centre: (f32, f32), rayon: f32, c: Color) {
        self.0.disque(centre, rayon, c);
    }
    fn groupe(&mut self, mots: &str, encre: Color) {
        self.0.groupe(mots, encre);
    }
    fn fin_du_groupe(&mut self) {
        self.0.fin_du_groupe();
    }
    fn glyphe(&mut self, glyphe: (Police, u16), plume: (f32, f32), corps: f32) {
        self.0.glyphe(glyphe, plume, corps);
    }
    fn forme(&mut self, f: &Forme, boite: (f32, f32, f32, f32), t: Option<f32>, c: Color) {
        self.0.forme(f, boite, t, c);
    }
}

/// **LIGNE-1 ne change jamais le relevé** : qu'une ligne se parcoure ou non, les primitives qui
/// partent à la carte sont les mêmes — à toutes les places d'un nœud qui entre par le bas de
/// l'écran et sort par le haut, par pas de treize pixels : un titre dont l'encre sort de sa
/// ligne par en haut (« Ǻ », le plus haut glyphe d'Inter SemiBold, 1,105 em : 37 pixels ici) et
/// par en bas (« ① », −0,32 em : 70 pixels), une formule haute qui se dessine depuis sa première
/// ligne sur celles qu'elle réserve, une puce, du code, un emoji.
#[test]
fn test_ligne_1_ne_change_jamais_le_releve() {
    let renderer = crate::renderer::Renderer::new();
    let kit = renderer.kit();
    let texte =
        "# Gggyyy jpq \u{1FA}\u{2460}gy\n\n$$\\sum_{i=1}^{n} \\frac{a_i}{b_i} \\int_0^1 f$$\n\n\
                 - une puce gy\n\n`code` et \u{1F600} fin";
    let ecran = (800.0f32, 600.0f32);
    let echelle = 20.0;
    let mut comparees = 0;
    let mut dernier_releve = 0;
    for pas in 0..380 {
        let vp = glucose_core::types::Viewport {
            x: -100.0,
            y: 600.0 - f64::from(pas) * 13.0,
            scale: echelle,
        };
        let ctx = Pass {
            typography: kit.typography,
            math: kit.math,
            mises_en_page: kit.mises_en_page,
            tints: kit.tints,
            theme: kit.theme,
            vp,
            scale: WorldScale::new(echelle, 1.0),
            clip: Clip {
                width: ecran.0,
                height: ecran.1,
                top: 0.0,
            },
            poignees: true,
        };
        let carte = || TextCard {
            origin: (0.0, 0.0),
            size: (240.0, 0.0),
            body: texte,
            tint: (120, 160, 220),
            fond: (20, 16, 16),
            eclaires: None,
            selected: false,
            editing: None,
        };
        let mut avec = Releve::nouveau(ecran);
        carte_en_vecteurs(&ctx, carte(), &mut avec);
        let mut sans = SansLigne1(Releve::nouveau(ecran));
        carte_en_vecteurs(&ctx, carte(), &mut sans);
        let lire = |r: &Releve| format!("{:?}", r.primitives);
        assert_eq!(
            lire(&avec),
            lire(&sans.0),
            "a la place {pas}, LIGNE-1 change le releve"
        );
        comparees += usize::from(!avec.primitives.is_empty());
        dernier_releve = avec.primitives.len();
    }
    // L'épreuve a vu le nœud entrer par le bas de l'écran, le traverser, et sortir tout entier
    // par le haut — là où LIGNE-1 écarte toutes ses lignes.
    assert!(
        comparees > 200,
        "le nœud n'a presque jamais été vu ({comparees})"
    );
    assert_eq!(
        dernier_releve, 0,
        "le balayage s'arrête avant que le nœud sorte"
    );
}

/// **Le relevé voit une bande dès qu'elle touche l'écran** — en haut, au milieu, en bas, à
/// cheval sur un bord ; et ne voit pas celle qui passe tout entière au-dessus ou au-dessous. À
/// une échelle où les lignes sont grandes, la marge de LIGNE-1 cachait une règle qui ne voyait
/// que la moitié de l'écran ; à une autre, elle aurait effacé des lignes.
#[test]
fn test_le_releve_voit_toute_bande_qui_touche_l_ecran() {
    let releve = Releve::nouveau((800.0, 600.0));
    for (haut, bas) in [
        (-50.0, 10.0),
        (100.0, 120.0),
        (290.0, 310.0),
        (550.0, 599.0),
        (590.0, 700.0),
        (-10.0, 700.0),
    ] {
        assert!(
            releve.voit_la_bande(haut, bas),
            "[{haut}, {bas}] touche l'écran"
        );
    }
    for (haut, bas) in [(-80.0, -1.0), (601.0, 900.0)] {
        assert!(
            !releve.voit_la_bande(haut, bas),
            "[{haut}, {bas}] est dehors"
        );
    }
}
