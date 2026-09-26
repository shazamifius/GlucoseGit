//! La passe sur les annotations : ce qu'une frame garde constant d'un nœud à l'autre, et le
//! tri qui envoie chaque nœud à son dessin — la carte, le pense-bête, la flèche.

use super::arrow::{draw_arrow, Fleche};
use super::card::{draw_card_contenu, draw_card_ornements, draw_text_card, Contenants, TextCard};
use super::domain::{draw_domain_gauge, DomainTints};
use super::hue::SymbioticHueCache;
use super::math::MathRenderer;
use super::note::draw_sticky;
use super::scale::WorldScale;
use super::{PaintKit, TextEditSession};
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::arrow;
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Board, DomainAssignment, Viewport};
use tiny_skia::PixmapMut;

/// Épaisseur de l'anneau de sélection, **en pixels écran**.
///
/// Exception SCALE-1 : c'est une affordance, pas du contenu. Mis à l'échelle, il
/// disparaîtrait en dézoomant au moment précis où l'on cherche ce qu'on a sélectionné.
pub(super) const SELECTION_RING: f32 = 2.0;

// ── Ce qu'une passe d'annotations garde constant ────────────────────────────

/// Le bord de l'écran utile. Tout ce qui en sort est écarté avant d'être dessiné (loi L1).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Clip {
    pub width: f32,
    pub height: f32,
    pub top: f32,
}

impl Clip {
    /// La boîte écran `(x, y, w, h)` est-elle entièrement hors champ ou trop petite ?
    pub(super) fn rejects(self, x: f32, y: f32, w: f32, h: f32) -> bool {
        x + w < 0.0 || x > self.width || y + h < self.top || y > self.height || (w < 3.0 && h < 3.0)
    }
}

/// Ce qui ne change pas d'une annotation à l'autre pendant une frame.
pub(crate) struct Pass<'a> {
    pub typography: &'a Typography,
    /// Le moteur de formules — il ne mute rien de visible, son cache est interne.
    pub math: &'a MathRenderer,
    /// `domain_id → teinte`, déjà résolue pour cette version du document (DOMAIN-TINT-1).
    pub tints: &'a DomainTints,
    pub theme: &'a Theme,
    pub vp: Viewport,
    pub scale: WorldScale,
    pub clip: Clip,
}

/// **Ce que la souris fait briller ou effacer** sur les annotations, à cette image : les
/// passages qu'une flèche survolée désigne (FLECHE-4), et l'effacement des pastilles de
/// relation qu'elle survole (BADGE-1), de 0 à 1 par flèche.
#[derive(Clone, Copy, Default)]
pub(crate) struct Survols<'a> {
    pub eclairages: &'a [crate::params::Eclairage],
    pub badges: &'a [(String, f32)],
}

impl Survols<'_> {
    /// De combien la pastille de cette flèche s'est effacée.
    fn effacement(&self, fleche: &str) -> f32 {
        self.badges
            .iter()
            .find(|(id, _)| id == fleche)
            .map_or(0.0, |(_, e)| *e)
    }
}

/// Dessine les annotations visibles du tableau actif.
///
/// `cartes_par_la_carte` dit que les cartes de texte sont des **textures** que la carte
/// graphique pose elle-même (COMPOSANT-1) : cette passe ne dessine alors que leurs
/// ornements — poignées, réglette — et la carte en édition, qui reste au processeur.
pub(super) fn draw_annotations(
    hue_cache: &mut SymbioticHueCache,
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    (editing_session, cartes_par_la_carte, survols): (Option<&TextEditSession>, bool, Survols<'_>),
    pass: ViewPass<'_>,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        tints: kit.tints,
        theme: kit.theme,
        vp: pass.vp,
        scale: pass.echelle(),
        clip: Clip {
            width: pixmap.width() as f32,
            height: pixmap.height() as f32,
            top: pass.header_h,
        },
    };
    // Ce qu'une carte ne cache pas : le fond, et les membranes qui la contiennent (LUEUR-3).
    let formes = super::scene::formes_des_membranes(store, pass, (pixmap.width(), pixmap.height()));
    let contenants = Contenants::nouveaux(kit.theme, &formes);

    // **Deux passes, et c'est ce qui rend les deux voies identiques par construction**
    // (ORNEMENTS-1). Le contenu des cartes de texte d'abord, a son rang ; puis tout ce qui
    // passe au-dessus -- pense-betes, fleches, poignees, reglettes, la carte qu'on edite.
    //
    // Sur la voie graphique, la premiere passe est VIDE : les cartes sont des textures que la
    // carte pose avant la couche du dessus (COMPOSANT-1). Sur la voie processeur, elle
    // dessine ce que ces textures porteraient, dans le meme ordre. Une affordance -- une
    // poignee, un cadre de selection -- n'est donc jamais cachee par une carte voisine, ce
    // qui est aussi la seule facon de pouvoir l'attraper.
    if !cartes_par_la_carte {
        for ann in Visibles::nouvelles(pass.visibles, board).annotations() {
            let editing = editing_session.filter(|s| s.ann_id.as_str() == ann.id());
            if editing.is_none() {
                let autour = (&contenants, survols.eclairages);
                if let Some(carte) = carte_de(hue_cache, (ann, store, pass), autour, None) {
                    draw_card_contenu(&ctx, pixmap, carte);
                }
            }
        }
    }

    let entieres = dessiner_ce_qui_passe_au_dessus(
        hue_cache,
        (&ctx, pixmap, (&contenants, survols)),
        (store, board, pass),
        (editing_session, cartes_par_la_carte),
    );
    crate::perf::compteur("cartes_entieres", entieres);
}

/// **Ce qui passe au-dessus des cartes** : pense-bêtes, flèches, poignées, la carte qu'on
/// édite — et le compte de ce que le processeur a dessiné **entier**.
///
/// Extraite de [`draw_annotations`], qui portait les deux passes d'ORNEMENTS-1 et le compteur
/// dans la même fonction : le cliquet des quatre-vingts lignes a raison, et les deux passes
/// ne changent pas pour les mêmes raisons — la première est le contenu à son rang, celle-ci
/// est ce qui doit rester attrapable.
///
/// # Ce que le compte tranche
///
/// Sur la voie graphique il ne devrait y avoir **aucune** carte dessinée entière : elles sont
/// des textures que la carte pose. Il en reste deux sortes — celle qu'on édite, et les
/// pense-bêtes, qui ne sont pas encore des composants. Le terrain du 22/09 donne
/// `annotations` à 9,74 ms au p99 du zoom avec **zéro** texture rendue, ce qui ne s'explique
/// que par un dessin direct ; ce compteur dit lequel, au lieu de le supposer.
fn dessiner_ce_qui_passe_au_dessus(
    hue_cache: &mut SymbioticHueCache,
    (ctx, pixmap, (contenants, survols)): (&Pass, &mut PixmapMut, (&Contenants<'_>, Survols<'_>)),
    (store, board, pass): (&Store, &glucose_core::types::Board, ViewPass<'_>),
    (editing_session, cartes_par_la_carte): (Option<&TextEditSession>, bool),
) -> f64 {
    let mut entieres = 0.0_f64;
    for ann in Visibles::nouvelles(pass.visibles, board).annotations() {
        let selected = store.selected_annotation_ids.iter().any(|s| s == ann.id());
        let editing = editing_session.filter(|s| s.ann_id.as_str() == ann.id());
        match ann {
            Annotation::Text { x, y, .. } => {
                let autour = (contenants, survols.eclairages);
                if let Some(carte) = carte_de(hue_cache, (ann, store, pass), autour, editing) {
                    // **La carte qu'on edite est un composant comme les autres** depuis
                    // COMPOSANT-2, et seuls ses ornements restent ici -- poignees, curseur,
                    // previsualisation de formule (COMPOSANT-3). Elle ne se dessine entiere
                    // que sur la voie processeur.
                    let au_processeur = editing.is_some() && !cartes_par_la_carte;
                    if au_processeur {
                        entieres += 1.0;
                        draw_text_card(ctx, pixmap, carte);
                    } else {
                        draw_card_ornements(ctx, pixmap, carte);
                    }
                }
                draw_node_gauge(ctx, pixmap, (*x, *y), ann.domains());
            }
            Annotation::Sticky { x, y, .. } => {
                // Un pense-bete n'est pas encore un composant : il se dessine entier a chaque
                // image, sur les deux voies. C'est le meme mecanisme que COMPOSANT-1 et il
                // n'a jamais ete applique -- ce compteur dira ce qu'il coute ici.
                entieres += 1.0;
                draw_sticky(ctx, pixmap, ann, selected, editing);
                draw_node_gauge(ctx, pixmap, (*x, *y), ann.domains());
            }
            Annotation::Arrow { .. } => {
                draw_arrow_node(
                    ctx,
                    pixmap,
                    hue_cache,
                    (ann, board, pass),
                    (selected, editing, !cartes_par_la_carte),
                    survols.effacement(ann.id()),
                );
            }
            _ => {}
        }
    }
    entieres
}

/// Ce qu'une carte de texte montre, tel que la passe le dessine — ou `None` si ce nœud n'en
/// est pas une.
///
/// La teinte symbiotique se calcule ici, et une seule fois par carte et par passe : c'est ce
/// que `draw_annotations` faisait déjà, au même endroit du même parcours.
fn carte_de<'a>(
    hue_cache: &mut SymbioticHueCache,
    (ann, store, pass): (&'a Annotation, &Store, ViewPass<'_>),
    (contenants, eclairages): (&Contenants<'_>, &'a [crate::params::Eclairage]),
    editing: Option<&'a TextEditSession>,
) -> Option<TextCard<'a>> {
    let Annotation::Text { x, y, text, .. } = ann else {
        return None;
    };
    let board = store.active_board()?;
    let tint = super::card::teinte_de_carte(hue_cache, ann, (pass.index, board));
    let body = editing.map(|e| e.buffer.as_str()).unwrap_or(text.as_str());
    let (w, h) = ann.size()?;
    let (cx, cy) = world_to_screen(x + w / 2.0, y + h / 2.0, &pass.vp);
    Some(TextCard {
        origin: (*x, *y),
        size: (w as f32, h as f32),
        body,
        tint,
        fond: contenants.fond_en((cx as f32, cy as f32)),
        eclaires: super::passages::eclaires_de(eclairages, ann.id(), tint),
        selected: store.selected_annotation_ids.iter().any(|s| s == ann.id()),
        editing,
    })
}

/// Pose la réglette de domaines d'une annotation au-dessus de son bord haut.
fn draw_node_gauge(
    ctx: &Pass,
    pixmap: &mut PixmapMut,
    origin: (f64, f64),
    domains: &[DomainAssignment],
) {
    let (wx, wy) = world_to_screen(origin.0, origin.1, &ctx.vp);
    draw_domain_gauge(
        ctx.typography,
        ctx.tints,
        pixmap,
        ctx.scale,
        (wx as f32, wy as f32),
        domains,
    );
}

/// Une flèche entière : son tracé, puis ce qu'elle dit (FLECHE-1).
///
/// Sa description se calcule **une fois** par image, et tout la lit : le trait, l'étiquette,
/// le badge, les poignées. Le trait ne se peint ici que sur la voie processeur : sur la voie
/// graphique, la carte le peint (FLECHE-2), et seuls l'étiquette, le badge et les poignées
/// restent dans la couche du dessus.
fn draw_arrow_node(
    ctx: &Pass<'_>,
    pixmap: &mut PixmapMut,
    hue_cache: &mut SymbioticHueCache,
    (ann, board, pass): (&Annotation, &Board, ViewPass<'_>),
    (selected, editing, au_processeur): (bool, Option<&TextEditSession>, bool),
    efface: f32,
) {
    let Annotation::Arrow { predicate, .. } = ann else {
        return;
    };
    let noeuds = super::arrow::NoeudsDuRendu {
        board,
        index: Some(pass.index),
        typographie: ctx.typography,
        math: ctx.math,
    };
    let Some(fleche) = Fleche::de(hue_cache, ann, (pass, noeuds), selected) else {
        return;
    };
    if au_processeur {
        draw_arrow(ctx, pixmap, &fleche);
    }
    // Les poignées de coude n'apparaissent que sur une flèche sélectionnée (ARROW-3) : une
    // affordance appartient à ce qu'on manipule.
    if selected {
        let poignees = arrow::handles(ann, noeuds);
        super::handles::draw_arrow_handles(pixmap, ctx.theme, &poignees, (&ctx.vp, ctx.scale));
    }
    let Some(milieu) = arrow::milieu_du_trace(&fleche.morceaux) else {
        return;
    };
    // L'étiquette et le prédicat visent **le même** point du tracé. Quand les deux sont là,
    // l'une monte et l'autre descend : sinon ils se recouvriraient exactement.
    let etiquette = super::arrow_label::montre_une_etiquette(ann, editing);
    let porte_les_deux = etiquette && predicate.is_some();
    let (r, g, b) = fleche.teintes.milieu;
    let encre = tiny_skia::Color::from_rgba8(r, g, b, 255);
    super::arrow_label::draw_arrow_label(
        ctx,
        pixmap,
        (milieu, encre),
        ann,
        editing,
        porte_les_deux,
    );
    if let Some(centre) = super::predicate::centre_du_badge(ann, milieu, etiquette) {
        super::predicate::draw_arrow_predicate(ctx, pixmap, centre, ann, 1.0 - efface);
    }
}
