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
use crate::dock::WidgetRect;
use crate::params::Pointer;
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
    Temps(super::temps::TempsUi),
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
            TabId::Temps => Self::Temps(dock.temps.clone()),
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
///
/// # La sélection, que la version du document ne voit pas (DOCKS-1)
///
/// « Domaines » écrit combien de nœuds sont sélectionnés et grise ses boutons d'assignation
/// quand il n'y en a aucun ; « Ordonner » compte les images visées. Or sélectionner n'est pas
/// une commande — c'est de la navigation, et la fiche 05 § 3.5 interdit qu'elle touche
/// l'annulation — donc la **version** du document ne bouge pas. Le panneau restait sur son
/// ancien compte, boutons grisés, jusqu'à ce que la souris passe dessus : 27 198 pixels
/// faux, jusqu'à 197 niveaux, dans le test qui l'a montré.
///
/// Les deux panneaux ne lisent que le **nombre** de nœuds sélectionnés, et c'est ce que la clé
/// retient : comparer les identifiants eux-mêmes coûterait une copie de la sélection entière
/// par panneau et par image, pour un « tout sélectionner » sur un document de dix millions de
/// nœuds.
#[derive(Clone, PartialEq)]
struct ClePanneau {
    geometrie: PanelLayoutBox,
    echelle: u32,
    tampon: (i32, i32, u32, u32),
    etat: EtatPanneau,
    document: u64,
    selection: (usize, usize),
}

impl ClePanneau {
    /// **Ce qui a changé depuis le dernier rendu de ce panneau** : un bit par raison, ou zéro
    /// s'il est à jour.
    ///
    /// La question que la fiche 24 § 14.1 a posée et laissée ouverte : les panneaux se
    /// refont sur dix des douze images les plus lentes, et c'est *la clé qui est trop large* —
    /// mais **laquelle de ses parties** ? Aucune durée ne le dit, et la deviner a déjà fait
    /// annoncer un chantier qui n'aurait rien changé.
    fn ce_qui_differe(ancienne: Option<&Self>, neuve: &Self, survol_change: bool) -> u16 {
        let Some(a) = ancienne else {
            return Raison::PremiereFois.bit();
        };
        [
            (
                a.geometrie != neuve.geometrie || a.tampon != neuve.tampon,
                Raison::Place,
            ),
            (a.echelle != neuve.echelle, Raison::Echelle),
            (survol_change, Raison::Pointeur),
            (a.etat != neuve.etat, Raison::Etat),
            (a.document != neuve.document, Raison::Document),
            (a.selection != neuve.selection, Raison::Selection),
        ]
        .into_iter()
        .filter(|(differe, _)| *differe)
        .fold(0, |masque, (_, raison)| masque | raison.bit())
    }
}

/// **Pourquoi un panneau s'est redessiné** — une partie de sa clé, nommée pour la chronique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Raison {
    PremiereFois,
    Place,
    Echelle,
    Pointeur,
    Etat,
    Document,
    Selection,
}

impl Raison {
    /// Dans l'ordre des bits du masque.
    pub const TOUTES: [Self; 7] = [
        Self::PremiereFois,
        Self::Place,
        Self::Echelle,
        Self::Pointeur,
        Self::Etat,
        Self::Document,
        Self::Selection,
    ];

    /// Ce que le rapport en dit, sans jargon.
    pub fn nom(self) -> &'static str {
        match self {
            Self::PremiereFois => "premiere fois",
            Self::Place => "place ou taille",
            Self::Echelle => "echelle de l'interface",
            Self::Pointeur => "souris dans le panneau",
            Self::Etat => "reglage du panneau",
            Self::Document => "document modifie",
            Self::Selection => "selection",
        }
    }

    /// Le bit de cette raison dans le masque d'une image.
    pub fn bit(self) -> u16 {
        1 << Self::TOUTES.iter().position(|r| *r == self).unwrap_or(0)
    }
}

/// Le tampon d'un panneau, et la clé qui dit de quoi il est l'image.
struct Panneau {
    pixmap: Pixmap,
    cle: ClePanneau,
    /// Les questions de survol que son dessin a posées, et ce que le pointeur y répondait
    /// (SURVOL-2). Les mêmes réponses au pointeur suivant disent les mêmes pixels.
    survol: Vec<(WidgetRect, bool)>,
}

impl Panneau {
    /// Le pointeur a-t-il changé une réponse que le dessin a lue ?
    fn survol_change(&self, pointer: Pointer) -> bool {
        self.survol
            .iter()
            .any(|(r, oui)| r.contains(pointer.x, pointer.y) != *oui)
    }
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
    /// Pourquoi les panneaux se sont refaits depuis la dernière fois qu'on a demandé — un bit
    /// par [`Raison`] (DOCKS-1).
    raisons: Cell<u16>,
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

    /// **Pourquoi des panneaux se sont refaits** depuis le dernier appel, et on repart de zéro.
    pub fn prendre_les_raisons(&self) -> u16 {
        self.raisons.take()
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
        etat: EtatPanneau::de(dock, panel.tab),
        document: store.version,
        selection: (
            store.selected_image_ids.len(),
            store.selected_annotation_ids.len(),
        ),
    };

    let mut panneaux = cache.panneaux.borrow_mut();
    let garde = panneaux.get(&panel.tab);
    let raisons = ClePanneau::ce_qui_differe(
        garde.map(|p| &p.cle),
        &cle,
        garde.is_some_and(|p| p.survol_change(pointer)),
    );
    cache.raisons.set(cache.raisons.get() | raisons);
    if raisons != 0 {
        let Some(mut neuf) = Pixmap::new(taille.0, taille.1) else {
            // Une taille que le tampon refuse : plutôt que de ne rien dessiner, on retombe
            // sur le rendu direct. Un cache qui échoue effacerait le panneau, ce qui serait
            // pire que le coût qu'il évite.
            drop(panneaux);
            let brush = Brush::nouveau((typo, theme), s, pointer, (0.0, 0.0));
            cache.rendus.set(cache.rendus.get() + 1);
            draw_panel(pixmap, &brush, dock, store, panel, s);
            return;
        };
        let brush = Brush::nouveau((typo, theme), s, pointer, origin);
        cache.rendus.set(cache.rendus.get() + 1);
        draw_panel(&mut neuf.as_mut(), &brush, dock, store, panel, s);
        let survol = brush.prendre_le_survol();
        panneaux.insert(
            panel.tab,
            Panneau {
                pixmap: neuf,
                cle,
                survol,
            },
        );
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
