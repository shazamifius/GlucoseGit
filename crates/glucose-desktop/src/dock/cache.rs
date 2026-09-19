//! Le tampon d'un panneau, et ce qui décide qu'il est périmé (DOCK-CACHE-1).
//!
//! # Ce qui a justifié ce module
//!
//! `bench_chrome` a mesuré la chose qui décide : sur cent images sans rien changer, les six
//! panneaux rendaient cent fois **exactement** les mêmes octets. Un coût élevé appelle un code
//! plus rapide ; c'est la répétition, et elle seule, qui appelle un cache.
//!
//! # Ce que la mesure a ensuite démenti
//!
//! Le gain est **bien plus faible qu'attendu** : environ 1,1× sur le dock, et un panneau y
//! perd. Composer un tampon de trois cent mille pixels en « source-over » coûte à peu près ce
//! que coûte le dessin qu'il remplace — les panneaux sont faits de formes simples et de
//! glyphes déjà mis en cache par la typographie.
//!
//! Le mesurer a servi à autre chose, et c'est là que le chantier a payé : il a mis au jour
//! deux défauts réels que personne ne cherchait — un second pinceau qui recopiait le premier
//! (soixante-treize lignes), et une phase sous-pixel qui basculait sur une soustraction en
//! virgule flottante (`Typography::draw_text_offset`).

use super::paint::Brush;
use super::render::draw_panel;
use super::{
    domains, plugins, DockManager, DockPass, OrganizeState, PanelLayoutBox, PomodoroState,
    PresetsState, StoryboardState, TabId,
};
use crate::composition::poser;
use glucose_core::report::Melange;
use glucose_core::store::Store;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use tiny_skia::{Pixmap, PixmapMut, Transform};

/// L'état d'interaction du panneau concerné, et de lui seul.
///
/// Le dock entier serait trop cher à cloner six fois par image, et surtout il mêlerait des
/// panneaux qui n'ont rien à voir : changer le minuteur referait le panneau des domaines.
#[derive(Clone, PartialEq)]
enum EtatPanneau {
    Organize(OrganizeState),
    Pomodoro(PomodoroState),
    Storyboard(StoryboardState),
    Plugins(plugins::PluginsState),
    Preset(PresetsState),
    Domains(domains::DomainsUi),
}

impl EtatPanneau {
    fn de(dock: &DockManager, tab: TabId) -> Self {
        match tab {
            TabId::Organize => Self::Organize(dock.organize.clone()),
            TabId::Pomodoro => Self::Pomodoro(dock.pomodoro.clone()),
            TabId::Storyboard => Self::Storyboard(dock.storyboard.clone()),
            TabId::Plugins => Self::Plugins(dock.plugins.clone()),
            TabId::Preset => Self::Preset(dock.presets.clone()),
            TabId::Domains => Self::Domains(dock.domains.clone()),
        }
    }
}

/// Ce dont l'image d'un panneau dépend — et rien d'autre (DOCK-CACHE-1).
///
/// # Pourquoi une comparaison et non une empreinte
///
/// Une empreinte condensée serait plus courte à garder, mais elle demanderait d'énumérer à la
/// main les champs qui comptent — et un champ ajouté plus tard serait silencieusement absent
/// du calcul, donc jamais vu changer. Le défaut serait invisible : un panneau qui refuse de
/// se rafraîchir.
///
/// Comparer la clé par valeur ferme cette porte. `PartialEq` est **dérivé**, donc tout champ
/// ajouté à l'un de ces états entre de lui-même dans la comparaison ; c'est le compilateur
/// qui tient l'invariant, pas la vigilance. Et il n'y a pas de collision possible.
///
/// # Le pointeur n'y est que lorsqu'il compte
///
/// Le survol se lit à l'intérieur des panneaux, sur des rectangles qu'eux seuls connaissent :
/// savoir de l'extérieur *quel* bouton est survolé demanderait de refaire leur géométrie.
/// Mais la question se retourne — un pointeur **hors** du panneau n'en survole aucun bouton,
/// donc sa position ne change rien à l'image et n'a pas à figurer dans la clé.
///
/// Le cas courant est justement celui-là : la souris est sur le canevas, et les panneaux
/// tiennent. Quand elle entre dans un panneau, ce panneau-là se refait à chaque mouvement —
/// un seul, celui avec lequel on est en train d'interagir.
#[derive(Clone, PartialEq)]
struct ClePanneau {
    geometrie: PanelLayoutBox,
    echelle: u32,
    tampon: (i32, i32, u32, u32),
    pointeur: Option<(u32, u32)>,
    etat: EtatPanneau,
    document: u64,
}

/// Le tampon d'un panneau, et la clé qui dit de quoi il est l'image.
struct Panneau {
    pixmap: Pixmap,
    cle: ClePanneau,
}

/// Les tampons des panneaux du dock (DOCK-CACHE-1).
///
/// Mesuré par `bench_chrome` : sur cent images sans rien changer, les six panneaux rendaient
/// cent fois **exactement** les mêmes octets, pour 14,28 ms par image. Un coût élevé appelle
/// un code plus rapide ; c'est la répétition qui appelle un cache, et elle est ici prouvée.
///
/// Il est en `RefCell` pour la même raison que le cache de glyphes de la typographie : un
/// cache est une optimisation transparente, et exiger un `&mut` pour dessiner obligerait
/// l'appelant à emprunter son propre état en écriture au milieu d'un rendu qui ne fait que
/// le lire.
#[derive(Default)]
pub struct DockCache {
    panneaux: RefCell<HashMap<TabId, Panneau>>,
    /// Combien de panneaux ont réellement été dessinés depuis le début.
    ///
    /// C'est ce qui rend le cache **testable** : sans ce compteur, un cache qui ne servirait
    /// jamais rendrait la même image et passerait tous les tests d'aspect. On ne saurait
    /// qu'il est inutile qu'au chronomètre, c'est-à-dire jamais de façon déterministe.
    rendus: Cell<usize>,
}

impl DockCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Combien de panneaux sont gardés.
    pub fn len(&self) -> usize {
        self.panneaux.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Combien de panneaux ont été dessinés depuis le début — le travail réellement fait.
    pub fn rendus(&self) -> usize {
        self.rendus.get()
    }
}

/// Dessine un panneau en passant par son tampon, qu'il ne refait que s'il a changé.
pub(super) fn draw_panel_cached(
    pixmap: &mut PixmapMut,
    cache: &DockCache,
    dock: &DockManager,
    store: &Store,
    pass: &DockPass<'_>,
    panel: &PanelLayoutBox,
    s: f32,
) {
    let (typo, theme, pointer) = (pass.typo, pass.theme, pass.pointer);
    let extent = panel.extent(s);
    // Le coin est ramené à l'entier inférieur : la composition se fait alors à une position
    // entière, et la fraction sous-pixel reste dans les coordonnées du dessin. Le panneau
    // tombe donc exactement sur les mêmes pixels qu'en rendu direct.
    let origin = (extent.x.floor(), extent.y.floor());
    let taille = (
        (extent.x + extent.w - origin.0).ceil().max(1.0) as u32,
        (extent.y + extent.h - origin.1).ceil().max(1.0) as u32,
    );
    let cle = ClePanneau {
        geometrie: panel.clone(),
        echelle: s.to_bits(),
        tampon: (origin.0 as i32, origin.1 as i32, taille.0, taille.1),
        pointeur: extent
            .contains(pointer.x, pointer.y)
            .then(|| (pointer.x.to_bits(), pointer.y.to_bits())),
        etat: EtatPanneau::de(dock, panel.tab),
        document: store.version,
    };

    let mut panneaux = cache.panneaux.borrow_mut();
    let a_jour = panneaux.get(&panel.tab).is_some_and(|p| p.cle == cle);
    if !a_jour {
        let Some(mut neuf) = Pixmap::new(taille.0, taille.1) else {
            // Une taille que le tampon refuse : plutôt que de ne rien dessiner, on retombe
            // sur le rendu direct. Un cache qui échoue effacerait le panneau, ce qui serait
            // pire que le coût qu'il évite.
            drop(panneaux);
            let brush = Brush {
                typo,
                theme,
                s,
                pointer,
                origin: (0.0, 0.0),
            };
            cache.rendus.set(cache.rendus.get() + 1);
            draw_panel(pixmap, &brush, dock, store, panel, s);
            return;
        };
        let brush = Brush {
            typo,
            theme,
            s,
            pointer,
            origin,
        };
        cache.rendus.set(cache.rendus.get() + 1);
        draw_panel(&mut neuf.as_mut(), &brush, dock, store, panel, s);
        panneaux.insert(panel.tab, Panneau { pixmap: neuf, cle });
    }

    let garde = &panneaux[&panel.tab];
    if !poser(pixmap, &garde.pixmap, origin, Melange::Composer) {
        // Une vue que REPORT-1 refuse — une taille impossible — retombe sur le rasteriseur.
        // Ne rien dessiner effacerait le panneau, ce qui serait pire que le coût évité.
        pixmap.draw_pixmap(
            origin.0 as i32,
            origin.1 as i32,
            garde.pixmap.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            Transform::identity(),
            None,
        );
    }
}
