//! **Ce qu'une texture de composant coute, et de quoi ce cout est fait.**
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_composant
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! Les chroniques de terrain du 21/09 au soir et du 22/09 donnent toutes la meme chose sur
//! leurs images les plus lentes :
//!
//! ```text
//!     3.9s  29.99ms  zoomer  1 noeud   text=1  kpx=2332   dont textures 24.88ms
//!    28.7s  21.34ms  zoomer  482       text=1  kpx=1464   dont textures 15.09ms
//! ```
//!
//! **UNE seule texture, et elle coute plus qu'une image entiere.** Le budget de CASCADE-2
//! borne le *nombre* de textures rendues par image ; il ne peut rien contre une seule, que la
//! garantie « au moins une par image » laisse toujours passer.
//!
//! Avant de concevoir quoi que ce soit, il faut savoir **de quoi** ces vingt-cinq
//! millisecondes sont faites -- et la fiche 23 § 8 dit pourquoi : deux estimations de la
//! session precedente se sont trompees d'un facteur soixante et d'un facteur six.
//!
//! # Comment il separe les postes sans instrumenter le code
//!
//! **Trois corps** pour la meme carte, a la meme echelle : vide, court, long. Ce qui reste au
//! vide est le tampon et le cadre ; la difference entre le vide et le long est le texte. Et
//! `Pixmap::new` est chronometre a part, ce qui donne le plancher d'allocation.
//!
//! Aucune marque n'est posee dans le code de production : trois marques imbriquees auraient
//! le defaut que les fiches 19, 22 et 23 racontent trois fois -- une marque absorbe tout ce
//! qui la precede. Trois **contenus** differents ne peuvent pas s'absorber les uns les autres.
//!
//! # Ce que ce banc a etabli, et ce qu'il a refuse
//!
//! * **Le tampon ne coute rien** : 0,01 ms pour deux megapixels. L'estimation qui lui donnait
//!   une milliseconde -- le temps de mettre neuf mebioctets a zero -- se trompait d'un facteur
//!   cent.
//! * **Le cout est du remplissage pur**, autour de cinq millisecondes par megapixel, dont a
//!   peu pres un tiers pour le cadre et deux tiers pour le texte. Il n'y a pas de gaspillage
//!   cache a supprimer : la texture coute ce que coute de peindre sa surface.
//! * **Trancher ce rendu en bandes de lignes est refuse, et c'est la mesure qui l'a refuse.**
//!   Une version de ce banc peignait la meme carte en deux a seize bandes, dans des tranches
//!   du meme tampon, via un `Composant::peindre(cible, depart)` ajoute pour l'occasion. Le
//!   cout tenait -- le decoupage se perd dans le bruit. Mais les **octets changent** : jusqu'a
//!   quarante-huit niveaux sur les bords verticaux du cadre, des DEUX bandes sur une carte
//!   sans texte. Le rasterisateur de `tiny-skia` accumule la couverture d'un bord le long du
//!   chemin et n'est pas invariant par decoupe, exactement comme la fiche 22 § 11.5 le decrit
//!   pour une translation entiere. L'aspect d'une carte aurait donc dependu du budget, donc
//!   du temps : le meme etat rendrait deux images differentes, ce que BLINK-1 a deja corrige
//!   une fois. Tout a ete retire.

use glucose_core::store::Store;
use glucose_core::types::Viewport;
use glucose_desktop::interactions::tools::text_card;
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::renderer::composants::Composant;
use glucose_desktop::renderer::{Regard, Renderer};
use glucose_desktop::ui::UiState;
use std::time::Instant;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (2560, 1600);
/// Combien de fois on rejoue le rendu une fois le cache de glyphes chaud.
const REPRISES: usize = 31;

const VIDE: &str = "";
const COURT: &str = "Une note";
const LONG: &str = "Carte 42 — une note posee sur le mur, avec **du gras** et du texte qui \
revient a la ligne parce qu'il depasse la largeur de la carte.";

fn centile(v: &[f64], p: f64) -> f64 {
    let mut t = v.to_vec();
    t.sort_by(f64::total_cmp);
    t[((t.len() as f64 * p) as usize).min(t.len() - 1)]
}

/// Un document d'une seule carte, posee a l'origine.
fn une_carte(renderer: &Renderer, corps: &str) -> Store {
    let mut store = Store::new("composant");
    let board = store.project.active_board_id.clone();
    let carte = text_card(
        &renderer.typography,
        &renderer.math,
        "seule".to_string(),
        0.0,
        0.0,
        corps.to_string(),
    );
    store.add_annotation(&board, carte);
    store.clear_selection();
    store
}

/// Le composant que cette carte donne a cette echelle, ou `None` si elle n'en est pas un.
///
/// La vue place le coin de la carte a dix pixels du bord : elle doit toucher l'ecran pour en
/// etre un, et ne pas le deborder pour en rester un.
fn preparer(renderer: &mut Renderer, store: &mut Store, echelle: f64) -> Option<Composant> {
    let board = store.project.active_board_id.clone();
    store.set_viewport(
        &board,
        Viewport {
            scale: echelle,
            x: 10.0,
            y: 60.0,
        },
    );
    let mut dessous = Pixmap::new(ECRAN.0, ECRAN.1)?;
    let mut dessus = Pixmap::new(ECRAN.0, ECRAN.1)?;
    let mut ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    let confie = renderer.rendre_les_couches(
        &mut dessous.as_mut(),
        &mut dessus.as_mut(),
        store,
        (&mut ui, Pointer { x: 0.0, y: 0.0 }),
        SceneOverlay {
            arrivages: &[],
            eclairages: &[],
            guides: &guides,
            selection_box: None,
            editing: None,
        },
        // A l'arret : l'echelle de rendu est l'echelle exacte, donc celle qu'on demande.
        Regard {
            degradation_permise: false,
            en_mouvement: false,
        },
    );
    confie.composants.first().cloned()
}

/// Ce qu'une mesure a donne : la taille de la texture, le premier rendu, puis les suivants.
struct Mesure {
    pixels: (u32, u32),
    froid: f64,
    chaud: f64,
}

/// Rend la carte de ce document a cette echelle, et chronometre sa texture.
///
/// **Le premier rendu est a part**, et ce n'est pas decoratif : c'est celui de l'application
/// quand un zoom franchit une octave et que les glyphes changent de taille. Les suivants
/// disent ce que la meme texture couterait s'il ne restait que le remplissage.
fn mesurer(renderer: &mut Renderer, store: &mut Store, echelle: f64) -> Option<Mesure> {
    let composant = preparer(renderer, store, echelle)?;
    let debut = Instant::now();
    let rendu = composant.rendre(renderer.kit())?;
    let froid = debut.elapsed().as_secs_f64() * 1000.0;
    let pixels = (rendu.width(), rendu.height());
    drop(rendu);
    let mut chauds = Vec::with_capacity(REPRISES);
    for _ in 0..REPRISES {
        let debut = Instant::now();
        let rendu = composant.rendre(renderer.kit());
        chauds.push(debut.elapsed().as_secs_f64() * 1000.0);
        drop(rendu);
    }
    Some(Mesure {
        pixels,
        froid,
        chaud: centile(&chauds, 0.5),
    })
}

/// Ce que coute la seule allocation d'un tampon de cette taille, mise a zero comprise.
fn cout_du_tampon(pixels: (u32, u32)) -> f64 {
    let mut v = Vec::with_capacity(REPRISES);
    for _ in 0..REPRISES {
        let debut = Instant::now();
        let p = Pixmap::new(pixels.0, pixels.1);
        v.push(debut.elapsed().as_secs_f64() * 1000.0);
        drop(p);
    }
    centile(&v, 0.5)
}

/// Une ligne du tableau : une echelle, ses trois corps, et ce qu'ils separent.
fn une_echelle(echelle: f64) {
    let mut chauds = Vec::new();
    let mut froids = Vec::new();
    let mut pixels = (0, 0);
    for corps in [VIDE, COURT, LONG] {
        // Un renderer NEUF par corps et par echelle : le cache de glyphes doit etre froid
        // pour que la premiere mesure dise ce qu'un changement de palier coute.
        let mut neuf = Renderer::new();
        let mut store = une_carte(&neuf, corps);
        match mesurer(&mut neuf, &mut store, echelle) {
            Some(m) => {
                pixels = m.pixels;
                froids.push(m.froid);
                chauds.push(m.chaud);
            }
            None => {
                froids.push(f64::NAN);
                chauds.push(f64::NAN);
            }
        }
    }
    if pixels.0 == 0 {
        println!("  {echelle:<7.1}  (la texture depasse l'ecran : ce n'est plus un composant)");
        return;
    }
    let kpx = f64::from(pixels.0) * f64::from(pixels.1) / 1000.0;
    let tampon = cout_du_tampon(pixels);
    let (cadre, tout) = (chauds[0], chauds[2]);
    let part_du_cadre = if tout > 0.0 {
        100.0 * cadre / tout
    } else {
        0.0
    };
    println!(
        "  {echelle:<7.1} {:>11} {kpx:>7.0}   {:>6.2} {:>6.2} {:>6.2}   {:>6.2} {:>6.2} {:>6.2}   {tampon:>6.2} {:>7.2} {part_du_cadre:>5.0}%",
        format!("{}x{}", pixels.0, pixels.1),
        froids[0],
        froids[1],
        froids[2],
        chauds[0],
        chauds[1],
        chauds[2],
        tout / (kpx / 1000.0),
    );
}

fn main() {
    println!(
        "Ce qu'une texture de composant coute -- ecran {} x {}, mediane sur {REPRISES} reprises\n",
        ECRAN.0, ECRAN.1
    );
    println!(
        "  {:<7} {:>11} {:>7}   {:^20}   {:^20}   {:>6} {:>7} {:>6}",
        "echelle",
        "texture",
        "kpx",
        "premier rendu",
        "rendus suivants",
        "tampon",
        "ms/Mpx",
        "cadre"
    );
    println!(
        "  {:<7} {:>11} {:>7}   {:>6} {:>6} {:>6}   {:>6} {:>6} {:>6}",
        "", "", "", "vide", "court", "long", "vide", "court", "long"
    );
    for echelle in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0] {
        une_echelle(echelle);
    }
    println!(
        "\n  Lecture : `vide` est le tampon et le cadre, `long` est tout -- leur difference est\n  \
         le texte. `cadre` est la part du cadre dans le rendu complet, une fois le cache chaud.\n"
    );
}
