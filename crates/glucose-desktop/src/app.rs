//! Application Glucose Desktop — Event Loop Winit 0.30 et Framebuffer Softbuffer 0.4.

mod accueil;
mod evenements;
mod fenetre;
mod mouvement;
mod peinture;
mod presentation;
pub mod reveil;
mod terrain;

use crate::dock::{apply_organize_layout, DockCache, DockManager, OrganizeState};
use crate::interactions::resize::ResizeSession;
use crate::interactions::tools::text_card;
use crate::renderer::{Renderer, TextEditSession};
use crate::ui::UiState;
use glucose_core::hit_priority::CycleState;
use glucose_core::smart_align::{AlignRect, AlignTarget, SnapGuides};
use glucose_core::store::Store;
use std::num::NonZeroU32;
use std::sync::Arc;
use tiny_skia::Pixmap;
use winit::event::ElementState;
use winit::keyboard::ModifiersState;
use winit::window::Window;

/// Ce que dit la carte d'accueil d'un document neuf.
const WELCOME_TEXT: &str = "# Bienvenue dans Glucose !\n- 100% Rust ultra-rapide\n- Teintes symbiotiques dynamiques\n- Double-cliquez pour éditer";

pub struct LastClickInfo {
    /// L'instant du clic, en millisecondes depuis [`GlucoseApp::click_epoch`].
    ///
    /// Une date entière et non un `Instant` : le compte des clics rapprochés et le cycle de
    /// profondeur posent la **même** question — « ces deux clics se suivent-ils ? » — et la
    /// posaient à deux horloges différentes, l'une réelle, l'autre en millisecondes. Deux
    /// horloges pour une question, c'est une divergence qui attend son bug.
    pub at_ms: i64,
    pub pos: (f64, f64),
    pub id: String,
    /// Le nombre de clics rapprochés sur ce même nœud : 1, puis 2, puis 3. C'est lui qui
    /// décide de la maille d'une sélection de texte (MOUSE-1), et le compter ici plutôt que
    /// dans le texte évite qu'un double-clic sur une carte et un double-clic sur un dossier
    /// aient deux définitions de « rapproché ».
    pub count: u32,
}

pub struct GlucoseApp {
    pub store: Store,
    pub renderer: Renderer,
    /// Les animations en cours — pour l'instant, le vol de la caméra.
    pub animator: crate::animation::Animator,
    /// L'elan de la camera : ce que la main a demande et que l'image n'a pas encore montre,
    /// plus la vitesse qui lui survit quand la main lache (voir [`crate::interactions::elan`]).
    pub elan: crate::interactions::elan::Elan,
    /// Le vol de camera en cours : une destination decidee, rejointe en douceur plutot que
    /// par teleportation (voir [`crate::interactions::vol`]).
    pub vol: crate::interactions::vol::Vol,
    /// Le defilement en cours vient-il d'un pave tactile ?
    ///
    /// Observe, jamais suppose : un delta fractionnaire ou lateral est une chose qu'une
    /// molette ne peut pas produire. Le verdict vaut pour tout le geste -- une source ne
    /// change pas en son milieu -- et s'efface quand la vue s'immobilise, donc entre deux
    /// gestes. Aucune duree n'a eu a etre choisie.
    pub defilement_au_doigt: bool,
    /// Ce que l'oeil tolere de perdre, a la vitesse a laquelle la vue bouge en ce moment.
    ///
    /// Recalcule a chaque image depuis le deplacement REEL de la vue -- donc quelle qu'en
    /// soit la cause : un geste, l'elan qui s'eteint, un vol de camera.
    pub perception: crate::perception::Perception,
    /// La vue de l'image precedente, pour mesurer de combien elle a bouge.
    vue_precedente: Option<glucose_core::types::Viewport>,
    /// Ce que cette image montre du mouvement : le pas de temps sur lequel il a ete integre,
    /// et la vitesse apparente qui en resulte, en pixels par seconde.
    ///
    /// Les deux se lisent a la PRESENTATION, pas au calcul : c'est la seule facon de comparer
    /// le temps que le mouvement a parcouru au temps que l'ecran l'a montre (RYTHME-1).
    pas_et_vitesse: (std::time::Duration, f64),
    /// Ce que la derniere presentation a montre, tel que l'instantane le portera.
    rythme_de_l_image: crate::chronique::rythme::Mesure,
    /// Le tempo de soumission : combien de balayages par image, et quand soumettre pour que ce
    /// soit exactement cela (voir [`crate::tempo`]).
    pub tempo: crate::tempo::Tempo,
    /// La derniere image a-t-elle demande la suivante ?
    ///
    /// Vrai quand une raison de reveil etait active -- animation, elan, vol, decodage. Faux
    /// quand l'application s'est endormie faute de quoi que ce soit a faire : l'intervalle qui
    /// suit est alors du repos, pas un gel, et la trajectoire n'a rien a rattraper.
    image_attendue: bool,
    /// D'où vient chaque image : la main, une raison de réveil, un dépôt, ou le système.
    pub(crate) provenance: reveil::Provenance,
    /// Le bouton gauche tient-il la minimap ?
    ///
    /// Tant qu'il tient, la destination du vol **suit le curseur** : c'est le voyage continu
    /// que Glucose Tauri permet, par opposition au saut par clic.
    pub minimap_tenue: bool,
    /// L'horloge de la trajectoire : le temps tel que l'ECRAN le montre (voir
    /// [`crate::horloge`]).
    ///
    /// # Elle remplace deux horloges, et c'est le fond du sujet
    ///
    /// Il y en avait deux -- celle de l'elan, celle de la boucle -- et toutes deux mesuraient
    /// entre deux DEBUTS de rendu. Le contenu avancait donc du temps de calcul, pendant qu'il
    /// etait montre pendant l'intervalle de presentation : deux durees qui different
    /// exactement de la variation du cout d'une image, soit jusqu'a soixante millisecondes.
    pub horloge: crate::horloge::Horloge,
    /// De combien la scene est rendue plus petite que la fenetre pendant un geste, et le
    /// tampon ou elle se rend alors (voir [`crate::resolution`]).
    pub resolution: crate::resolution::Resolution,
    pub tampon_reduit: Option<Pixmap>,
    /// La couche qui passe **sur** les photos, quand la carte les pose elle-meme.
    ///
    /// Elle part transparente a chaque image : tout ce qui n'y est pas dessine laisse voir
    /// les photos, et c'est ce qui rend la composition juste sans calculer une seule region.
    pub tampon_dessus: Option<Pixmap>,
    /// Ou chaque photo se pose, pour la voie graphique (fiche 21, etape 1).
    pub confie: crate::renderer::Confie,
    pub pixmap: Option<Pixmap>,
    pub ui: UiState,
    pub dock_manager: DockManager,
    /// Les tampons des panneaux du dock (DOCK-CACHE-1).
    ///
    /// Mesuré avant qu'il existe : sur un plateau vide, les panneaux étaient le premier poste
    /// de l'application — plus cher que tout le contenu réuni — et rendaient cent fois de
    /// suite exactement les mêmes octets.
    pub dock_cache: DockCache,
    pub window: Option<Arc<Window>>,
    /// Ce qui met l'image à l'écran — la carte graphique, ou le processeur à défaut.
    ///
    /// Derrière un `dyn` parce que le choix se fait au démarrage, une fois, en essayant : un
    /// adaptateur graphique peut manquer, et sur une machine virtuelle ou un bureau distant
    /// il manque souvent. L'appel indirect est payé une fois par image, contre les quelques
    /// millisecondes que la présentation elle-même coûte.
    pub presenter: Option<Box<dyn crate::present::Presenter>>,
    pub scale_factor: f64,

    /// **Qui choisit la carte graphique, en la regardant travailler** (ARBITRE-1).
    ///
    /// `None` quand `GLUCOSE_CARTE` impose une carte : l'utilisateur a tranche, il n'y a plus
    /// rien a arbitrer.
    pub arbitre: Option<crate::present::arbitre::Arbitre>,
    /// **Ou l'arbitre retient la carte qu'il a choisie** (ARBITRE-4) -- le fichier de ce
    /// que l'application a appris de cette machine. Un champ plutot qu'un appel, pour que les
    /// tests ecrivent ailleurs que chez l'utilisateur.
    pub souvenir_de_la_carte: std::path::PathBuf,

    /// **La selection a reduire au relachement, si le geste n'etait qu'un clic** (SEL-MULTI-1).
    ///
    /// Presser sur un element deja selectionne ne doit PAS reduire la selection : c'est ce qui
    /// permet de deplacer plusieurs images d'un seul geste. Mais un clic simple sur l'un
    /// d'eux, lui, doit bien reduire -- sinon on ne pourrait plus jamais ramener la selection
    /// a un seul element sans passer par le vide.
    ///
    /// Les deux gestes commencent par la meme pression et ne se distinguent qu'a la fin :
    /// **c'est donc le relachement qui tranche**, et rien d'autre ne le peut.
    pub reduire_a_la_relache: Option<String>,

    /// De quoi réveiller la boucle depuis un autre fil — le veilleur du budget de la carte
    /// (ETAGES-2). Donné par `main`, qui seul tient la boucle avant qu'elle tourne.
    pub reveil: Option<winit::event_loop::EventLoopProxy<()>>,

    /// Le passe et l'avenir de la saisie en cours (TEXTE-UNDO-1).
    ///
    /// Il vit **a cote** de la session et non dedans : depuis COMPOSANT-2, l'empreinte de la
    /// texture d'une carte en saisie porte une copie de cette session, et un historique range
    /// la serait clone a chaque frappe.
    pub historique_du_texte: crate::interactions::text_edit::historique::Historique,

    /// Un `Ctrl+B` dont les originaux reviennent de chez le système (ETAGES-1).
    pub bordures_en_attente: Option<crate::interactions::recadrage::LotDeBordures>,

    /// **Le prochain clic ne sert qu'à revenir au premier plan** (REVEIL-1).
    ///
    /// Windows transmet à la fenêtre le clic qui l'active, et Glucose l'exécutait donc sur le
    /// canevas : revenir d'un navigateur pour coller une image désélectionnait ce qu'on avait
    /// choisi, déplaçait un nœud, ou traçait un trait — selon l'outil actif et l'endroit où le
    /// curseur se trouvait. C'est le comportement que macOS refuse depuis toujours, et pour
    /// une bonne raison : **un clic d'activation n'est pas un geste**, c'est une formalité du
    /// système d'exploitation.
    ///
    /// Le drapeau se lève à la reprise du focus et retombe au premier mouvement de souris :
    /// quelqu'un qui revient par Alt+Tab, bouge la souris puis clique veut vraiment cliquer.
    /// Aucune durée n'a eu à être choisie — un délai aurait été une constante arbitraire, et
    /// il aurait avalé des clics une demi-seconde après coup.
    pub clic_de_reveil: bool,
    /// Le relâchement du clic avalé se jette avec lui : sans cela, la moitié d'un geste
    /// arrive sans sa première moitié, et c'est exactement ce qui referme un tracé qui n'a
    /// jamais commencé.
    pub relachement_a_jeter: bool,

    // États d'interaction
    pub mouse_pos: (f64, f64),
    /// Le curseur a-t-il ete pose au moins une fois dans cette fenetre ?
    ///
    /// `(0, 0)` est une position parfaitement valide, donc indiscernable de « on ne sait pas
    /// encore » -- et c'est la valeur de depart. Sans ce drapeau, une fenetre qu'on vient
    /// d'ouvrir zoome autour de son coin superieur gauche.
    pub curseur_vu: bool,
    pub modifiers: ModifiersState,
    pub right_or_middle_down: bool,
    /// Où le clic droit s'est enfoncé, tant qu'il l'est.
    ///
    /// C'est ce qui permet de savoir, **au relâchement**, si le geste était un déplacement de
    /// la vue ou une demande de menu contextuel : la seule différence est que le curseur a
    /// bougé, ou non.
    pub right_down_at: Option<(f64, f64)>,
    pub is_panning: bool,
    pub is_dragging_item: bool,
    pub drag_start_world: (f64, f64),
    pub drag_selection_base: Option<AlignRect>,
    pub drag_snap_targets: Vec<AlignTarget>,
    pub drag_applied_delta: (f64, f64),
    /// Le redimensionnement en cours, s'il y en a un (RESIZE-1).
    pub resize_session: Option<ResizeSession>,
    /// L'objet en train de naître sous la main, s'il y en a un (DRAW-1).
    pub draw_session: Option<crate::interactions::tools::DrawSession>,
    /// Le coude de flèche tenu sous la main, s'il y en a un (ARROW-3).
    pub bend_session: Option<crate::interactions::arrow_edit::BendSession>,
    pub active_guides: SnapGuides,
    pub selection_box: Option<(f64, f64, f64, f64)>,
    pub always_on_top: bool,

    // Session d'édition de texte in-place (double-clic)
    pub editing_session: Option<TextEditSession>,
    /// Le glisser de sélection de texte en cours, s'il y en a un (MOUSE-1).
    pub text_drag: Option<crate::interactions::text_mouse::TextDrag>,
    pub last_click: Option<LastClickInfo>,
    /// **Ce qui arrive du système par glisser-déposer** : les fichiers de `winit`, le pont
    /// natif, et les images annoncées qui ne sont pas encore livrées.
    pub depot: crate::interactions::depot_web::Arrivees,
    /// Où en est le cycle de profondeur (PICK-1) : la pile visée au dernier clic, et le rang
    /// qu'on y a atteint. `None` quand le dernier clic n'a désigné aucun nœud, ou qu'il a
    /// fait autre chose que sélectionner — ouvrir, éditer, glisser.
    pub pick_cycle: Option<CycleState>,
    /// L'origine du temps des clics — celle que [`GlucoseApp::now_ms`] mesure.
    ///
    /// Un `Instant` ne se soustrait pas à un entier, et l'arbitre de clic raisonne en
    /// millisecondes. C'est aussi ce qui rend les gestes testables **sans dormir** : un test
    /// qui veut jouer un re-clic « une seconde plus tard » recule cette origine, au lieu
    /// d'attendre vraiment.
    pub click_epoch: std::time::Instant,
    pub last_blink_phase: bool,
    /// La cadence de l'écran, lue et non supposée (CADENCE-1).
    ///
    /// Tout le projet a longtemps raisonné sur soixante hertz. Sur un écran à 240 Hz, un banc
    /// qui annonce « tenu » à 9 ms ment de plus du double, et fait optimiser dans la mauvaise
    /// direction. C'est elle qui dit ce qui reste pour le travail de fond après une image.
    pub cadence: crate::cadence::Cadence,

    /// Ce que le document a sur le disque : son fichier, son histoire en train de s'écrire,
    /// et où sont les octets de ses images (HISTOIRE-1).
    pub disque: crate::persist::disque::Disque,
    /// Chemin du `.glucose` courant. `None` tant que le projet n'a jamais été enregistré :
    /// c'est ce qui fait que `Ctrl+S` ouvre un dialogue la première fois seulement.
    pub project_path: Option<std::path::PathBuf>,
    /// `store.version` au moment du dernier enregistrement ou de la dernière ouverture.
    ///
    /// INVARIANT SAVE-2 — « modifié » se lit `store.version != saved_version`. Aucun drapeau
    /// à lever dans chaque mutation, donc aucune mutation ne peut oublier de le lever : la
    /// pile d'undo fait déjà avancer la version, et elle seule (la navigation ne la touche pas).
    pub saved_version: u64,
    /// Dernier titre posé sur la fenêtre, pour ne pas repayer un appel système par frame.
    pub window_title_cache: String,
    /// Ce que la session a observe : ce qui coute, et ce qui a gele (CHRONIQUE-1).
    ///
    /// Enregistre en permanence, chez l'utilisateur, pendant l'usage reel. C'est la seule
    /// mesure qui dise ce qu'il vit -- un banc ne mesure que ce qu'on lui demande.
    pub chronique: crate::chronique::Chronique,
    /// L'echelle de la vue a l'image precedente, qui suffit a reconnaitre un zoom.
    ///
    /// Le zoom n'a pas de session : c'est un evenement de molette. Le deduire du document
    /// evite d'inventer une duree d'attente apres laquelle on cesserait de "zoomer".
    echelle_precedente: f64,
    /// Ce qui a changé depuis la dernière image, et doit donc être redessiné (A.1).
    ///
    /// Dans une `Cell` pour que [`GlucoseApp::mark_dirty`] reste en `&self` : soixante-deux
    /// appelants la prennent ainsi, et leur imposer `&mut` pour noter une salissure aurait
    /// remonté l'emprunt à travers tout l'arbre des gestes. `Salissure` est `Copy`, donc la
    /// cellule ne coûte rien.
    salissure: std::cell::Cell<crate::salissure::Salissure>,
    /// **L'instant où la prochaine image est devenue nécessaire** (GEL-1) — le plus ancien
    /// depuis la dernière image rendue.
    ///
    /// Posé par qui demande une image : un geste qui salit la vue, un dépôt, une animation qui
    /// réclame la suivante, un réveil programmé. Le rendu le prend à son début. Un gel se
    /// mesure depuis lui, et non depuis l'image précédente : entre les deux, l'application
    /// n'avait peut-être rien à montrer, et ce repos n'est pas un gel.
    ///
    /// En `Cell` pour la même raison que la salissure : `mark_dirty` ne prend que `&self`.
    image_due: std::cell::Cell<Option<std::time::Instant>>,
}

/// `new` ne prend aucun argument : `Default` est donc exactement le même constructeur.
/// Le déclarer évite qu'un appelant générique ait à connaître le nom `new`.
impl Default for GlucoseApp {
    fn default() -> Self {
        Self::new()
    }
}

impl GlucoseApp {
    pub fn new() -> Self {
        let mut renderer = Renderer::new();
        let store = accueil::document_d_accueil(&renderer);
        let saved_version = store.version;
        let disque = accueil::brancher_le_disque(&mut renderer, &store);

        Self {
            store,
            renderer,
            animator: crate::animation::Animator::new(),
            elan: crate::interactions::elan::Elan::default(),
            vol: crate::interactions::vol::Vol::default(),
            defilement_au_doigt: false,
            perception: crate::perception::Perception::nette(),
            vue_precedente: None,
            pas_et_vitesse: (std::time::Duration::ZERO, 0.0),
            rythme_de_l_image: crate::chronique::rythme::Mesure::default(),
            image_attendue: false,
            provenance: reveil::Provenance::default(),
            tempo: crate::tempo::Tempo::nouveau(),
            minimap_tenue: false,
            horloge: crate::horloge::Horloge::nouvelle(),
            resolution: crate::resolution::Resolution::nette(),
            tampon_reduit: None,
            tampon_dessus: None,
            confie: crate::renderer::Confie::default(),
            pixmap: None,
            ui: accueil::interface_d_accueil(),
            dock_manager: DockManager::new(),
            dock_cache: DockCache::new(),
            arbitre: None,
            souvenir_de_la_carte: crate::present::souvenir::chemin(),
            reduire_a_la_relache: None,
            reveil: None,
            historique_du_texte: Default::default(),
            bordures_en_attente: None,
            clic_de_reveil: false,
            relachement_a_jeter: false,
            window: None,
            presenter: None,
            scale_factor: 1.0,
            mouse_pos: (0.0, 0.0),
            curseur_vu: false,
            modifiers: ModifiersState::empty(),
            right_or_middle_down: false,
            right_down_at: None,
            is_panning: false,
            is_dragging_item: false,
            drag_start_world: (0.0, 0.0),
            drag_selection_base: None,
            drag_snap_targets: Vec::new(),
            drag_applied_delta: (0.0, 0.0),
            resize_session: None,
            draw_session: None,
            bend_session: None,
            active_guides: SnapGuides::default(),
            selection_box: None,
            always_on_top: false,
            editing_session: None,
            text_drag: None,
            last_click: None,
            depot: Default::default(),
            pick_cycle: None,
            click_epoch: std::time::Instant::now(),
            last_blink_phase: true,
            cadence: crate::cadence::Cadence::inconnue(),
            disque,
            project_path: None,
            saved_version,
            window_title_cache: String::new(),
            chronique: crate::chronique::Chronique::nouvelle(),
            echelle_precedente: 1.0,
            // Tout, et non rien : la première image doit se dessiner entièrement.
            salissure: std::cell::Cell::new(crate::salissure::Salissure::Tout),
            image_due: std::cell::Cell::new(None),
        }
    }

    pub fn redraw(&mut self) {
        // Ce que le dernier événement a changé dans le document s'écrit (JRN-5) — avant le
        // titre, qu'un document enregistré au fil de l'eau ne marque plus « modifié ».
        self.consigner();
        // Le marqueur « modifié » du titre suit l'état réel du document (INVARIANT SAVE-2).
        // Le poser ici plutôt que dans chaque mutation garantit qu'aucune ne l'oublie ;
        // `sync_window_title` ne touche la fenêtre que lorsque le titre change vraiment.
        self.sync_window_title();
        if let Some(window) = self.window.clone() {
            crate::perf::frame_begin();
            let frame_started = std::time::Instant::now();
            // L'echeance de CETTE image, prise avant tout dessin : ce que le rendu demandera
            // ensuite -- une surface perdue, un composant manquant -- est pour la suivante.
            let due = self.image_due.take();
            let size = window.inner_size();
            let width = size.width.max(1);
            let height = size.height.max(1);

            if let (Some(w), Some(h), Some(presenter)) = (
                NonZeroU32::new(width),
                NonZeroU32::new(height),
                &mut self.presenter,
            ) {
                if let Err(e) = presenter.resize(w, h) {
                    eprintln!("[GlucoseDesktop] redimensionnement de la surface : {e}");
                }
            }

            let need_new_pixmap = match &self.pixmap {
                Some(p) => p.width() != width || p.height() != height,
                None => true,
            };
            if need_new_pixmap {
                self.pixmap = Pixmap::new(width, height);
            }

            // La camera bouge ICI, une seule fois par image, et jamais dans l'evenement :
            // c'est ce qui fait qu'une diagonale est une diagonale et non un escalier.
            self.appliquer_l_elan(width, height);

            let tampon_neuf = need_new_pixmap | self.accorder_le_tampon_reduit(width, height);
            // Un `Ctrl+B` qui attend ses originaux les redemande, ou s'applique s'ils sont
            // revenus (ETAGES-1) — avant la peinture, pour que ce qu'il change se voie ici.
            self.poursuivre_les_bordures();
            self.preparer_la_carte((width, height));
            self.peindre_ce_qui_a_change((width, height), tampon_neuf);

            // TEMPO-1 : l'image ne part pas quand elle est prete, elle part quand c'est
            // l'heure -- un nombre entier et CONSTANT de balayages apres la precedente. Ce
            // qui reste d'ici la sert au travail de fond.
            self.attendre_l_heure_de_soumettre();
            self.presenter_et_noter_le_rythme(frame_started, due);
            self.clore_l_image(frame_started, (width, height));
        }
    }

    /// Applique a la camera ce que l'elan a retenu pour cette image.
    ///
    /// Un seul deplacement et un seul changement d'echelle, quel que soit le nombre
    /// d'evenements recus depuis la derniere image. La diagonale de la fenetre sert de mesure
    /// commune aux deux : elle dit ce qu'un reste de zoom deplacerait a l'ecran, donc quand il
    /// devient invisible.
    /// Marque **toute** la vue comme sale et planifie un rafraîchissement (R-15).
    ///
    /// C'est la déclaration de celui qui ne sait pas ce qu'il a changé, et elle reste juste :
    /// redessiner l'écran entier coûte ce qu'il coûtait hier. Un geste qui sait désigner sa
    /// zone appelle [`GlucoseApp::salir`] et paie beaucoup moins.
    /// Ce que la prochaine image devra redessiner, pris comme la peinture le prend : une
    /// épreuve y lit si un geste a demandé une image, sans avoir à en peindre une.
    #[cfg(test)]
    pub(crate) fn prendre_la_salissure(&self) -> crate::salissure::Salissure {
        self.salissure.replace(crate::salissure::Salissure::Rien)
    }

    pub fn mark_dirty(&self) {
        self.provenance.noter_une_demande();
        self.noter_l_echeance(std::time::Instant::now());
        self.salissure.set(crate::salissure::Salissure::Tout);
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// Marque cette zone du **monde** comme sale, et elle seule (A.1).
    ///
    /// Une zone précise ne peut jamais réduire une salissure déjà posée : si `Tout` a été
    /// demandé par ailleurs pendant la même image, il l'emporte. C'est ce qui rend l'ordre des
    /// déclarations indifférent, et donc ce mécanisme sûr à adopter progressivement.
    pub fn salir(&self, zone: glucose_core::geometry::Rect) {
        self.noter_l_echeance(std::time::Instant::now());
        self.salissure.set(self.salissure.get().avec(zone));
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// **Une image est due à cet instant** (GEL-1) : la plus ancienne échéance l'emporte.
    ///
    /// La plus ancienne, parce que c'est elle que l'œil attend depuis le plus longtemps : une
    /// seconde demande pendant qu'une première attend encore ne raccourcit pas le gel.
    pub(crate) fn noter_l_echeance(&self, a: std::time::Instant) {
        let plus_tot = self.image_due.get().map_or(a, |deja| deja.min(a));
        self.image_due.set(Some(plus_tot));
    }

    /// **Le gel en cours au moment de fermer** — ce qu'aucune image suivante ne mesurera.
    ///
    /// La chronique ne mesure un gel qu'entre deux images présentées. Un canevas qui fige et
    /// ne repart jamais ne laisse donc **aucune** trace : c'est exactement le gel que
    /// l'utilisateur a décrit le 23/09 — *« la 2e a COMPLÈTEMENT freeze »* — et sa chronique
    /// finissait sur une image normale. Fermer alors qu'une image est due depuis longtemps est
    /// un gel comme un autre, et il se range avec les autres, décomposé.
    pub(crate) fn noter_le_gel_en_cours(&mut self) {
        let maintenant = std::time::Instant::now();
        let due = self.image_due.get();
        self.chronique.rythme.en_retard(maintenant, maintenant, due);
        self.chronique.entracte.fermer(maintenant, due);
        self.chronique
            .noter_la_fermeture(due.map(|d| maintenant.saturating_duration_since(d)));
    }

    /// Réorganise automatiquement les éléments en grille ordonnée
    #[allow(dead_code)]
    pub fn organize_layout(&mut self) {
        let board_id = self.store.project.active_board_id.clone();
        let vide = self
            .store
            .active_board()
            .is_some_and(|b| b.images.is_empty() && b.annotations.is_empty());
        if vide {
            return;
        }
        // `push_undo` etait appele APRES la mise en page : le cliche capturait l'etat deja
        // modifie, et Ctrl+Z ne defaisait rien. La consigne se fait desormais autour du
        // geste, pas apres lui.
        self.store.mutate_board_layout(&board_id, |board| {
            glucose_core::layout::organize_board_grid(board, 40.0);
        });
        self.ui.show_toast("Canvas ordonné");
        self.mark_dirty();
    }

    /// **Ce clic agit-il sur le canevas, ou rend-il seulement le premier plan ?** (REVEIL-1)
    ///
    /// Windows transmet a la fenetre le clic qui l'active. Revenir d'un navigateur pour coller
    /// une image executait donc ce clic sur le canevas : il deselectionnait ce qu'on avait
    /// choisi, deplacait un noeud ou tracait un trait, selon l'outil actif et l'endroit ou le
    /// curseur se trouvait. macOS refuse ce comportement depuis toujours, et pour une bonne
    /// raison : **un clic d'activation n'est pas un geste**, c'est une formalite du systeme.
    ///
    /// Le relachement part avec l'appui qu'il termine. Sans cela, la seconde moitie d'un geste
    /// arrive sans la premiere -- et c'est exactement ce qui referme un trace qui n'a jamais
    /// commence, ou relache un panneau que personne n'a saisi.
    pub(crate) fn ce_clic_agit(&mut self, state: ElementState) -> bool {
        match state {
            ElementState::Pressed if std::mem::take(&mut self.clic_de_reveil) => {
                self.relachement_a_jeter = true;
                false
            }
            ElementState::Released if std::mem::take(&mut self.relachement_a_jeter) => false,
            _ => true,
        }
    }

    /// Applique la réorganisation issue du panneau ORDONNER (Masonry, Grille, Même Hauteur, etc.)
    ///
    /// # ORDONNER-1 — on range **ce qui est sélectionné**, et rien d'autre
    ///
    /// Le panneau rangeait toutes les images du tableau, quoi qu'on ait sélectionné. Choisir
    /// douze images pour les aligner et voir les quatre cents autres se réarranger avec elles
    /// n'est pas une maladresse d'ergonomie : c'est une fonction qui détruit un travail qu'on
    /// ne lui avait pas confié, et l'annulation est le seul recours.
    ///
    /// Une sélection vide veut dire « tout le tableau » — sinon le bouton ne ferait rien du
    /// tout, ce qui serait pire, et c'est le geste qu'on attend d'un rangement global.
    ///
    /// Le toast dit lequel des deux a eu lieu : un rangement qui ne dit pas ce qu'il a touché
    /// laisse chercher.
    pub fn apply_dock_layout(&mut self, state: &OrganizeState) {
        let board_id = self.store.project.active_board_id.clone();
        if self
            .store
            .active_board()
            .is_some_and(|b| b.images.is_empty())
        {
            self.ui.show_toast("Aucune image sur le canvas");
            return;
        }
        // **Le Store dit ce qu'il faut ranger**, parce que « une selection vide veut dire tout
        // le tableau » est une regle metier et non une commodite d'affichage.
        let a_ranger = self.store.images_a_organiser();
        let combien = self.store.selected_image_ids.len();
        // Meme correction que `organize_layout` : le cliche etait pris apres coup.
        self.store.mutate_board_layout(&board_id, |board| {
            for res in apply_organize_layout(&a_ranger, state) {
                if let Some(img) = board.images.iter_mut().find(|i| i.id == res.id) {
                    img.x = res.x;
                    img.y = res.y;
                    img.width = res.width;
                    img.height = res.height;
                }
            }
        });
        let quoi = if combien == 0 {
            "tout le canvas".to_string()
        } else {
            format!("{combien} image(s)")
        };
        self.ui
            .show_toast(format!("{} : {quoi} rangé(es)", state.layout.title()));
        self.mark_dirty();
    }
}
