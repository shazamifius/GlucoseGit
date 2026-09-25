//! **Ce qu'une frappe coûte**, sur une carte comme celles qu'on écrit vraiment.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_saisie
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! Sa chronique du 25/09 (`sortie-chronique-2026-09-25-fleches.txt`) : « éditer du texte »,
//! image médiane **19,48 ms**, dont `textures` **11,59 ms**, sur une carte de **798 kpx** —
//! cinquante et une images par seconde pendant qu'on écrit, la moitié du plancher.
//!
//! `bench_composant` ne ressemble pas à ce cas : son texte « long » tient en deux lignes, et
//! ses grandes textures sont de grosses lettres. Une carte qu'on écrit est un texte **dense**,
//! à zoom normal, sur un écran à 150 % : beaucoup de lignes de petites lettres. C'est elle que
//! ce banc mesure — ce qu'une frappe refait, et de quoi c'est fait.

use glucose_core::store::Store;
use glucose_core::types::{Annotation, Viewport};
use glucose_desktop::interactions::tools::text_card;
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::present::banc_gpu;
use glucose_desktop::present::scene_gpu::SceneGpu;
use glucose_desktop::renderer::composants::Composant;
use glucose_desktop::renderer::{Confie, Regard, Renderer, TextEditSession};
use glucose_desktop::ui::UiState;
use std::time::Instant;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (2160, 1350);
/// L'échelle de son écran.
const DENSITE: f32 = 1.5;
const REPRISES: usize = 101;

/// Un paragraphe de notes, comme on en écrit : des mots courts et longs, des accents.
const PARAGRAPHE: &str = "Les membranes possèdent ce qu'on y dépose : la déplacer emporte son \
contenu, la supprimer le libère. Une flèche désigne un passage exact d'une carte, par ses \
octets et non par son mot, et elle suit l'écriture quand on tape avant ou après lui.";

fn centile(v: &[f64], p: f64) -> f64 {
    let mut t = v.to_vec();
    t.sort_by(f64::total_cmp);
    t[((t.len() as f64 * p) as usize).min(t.len() - 1)]
}

/// Le texte d'une carte dense : `n` paragraphes.
fn dense(n: usize) -> String {
    vec![PARAGRAPHE; n].join("\n")
}

/// Une carte comme la plus grosse de son document du 25/09 : un titre, un sous-titre, deux
/// paragraphes — environ cinq cent soixante octets, sur 550 unités de large.
fn notes() -> String {
    format!("# Idée du projet\n### Ce qui a tout lancé\n{PARAGRAPHE}\n{PARAGRAPHE}")
}

/// Un document d'une carte de `largeur` unités, portant `corps` — ou, si `hauteur` est
/// donnée, vide mais de cette hauteur : sa brume seule.
fn une_carte(
    renderer: &Renderer,
    corps: &str,
    (largeur, zoom): (f64, f64),
    hauteur: Option<f64>,
) -> Store {
    let mut store = Store::new("saisie");
    let board = store.project.active_board_id.clone();
    let mut carte = text_card(
        &renderer.typography,
        &renderer.math,
        "c".to_string(),
        0.0,
        0.0,
        corps.to_string(),
    );
    if let Annotation::Text {
        width,
        height,
        text,
        ..
    } = &mut carte
    {
        *width = Some(largeur);
        // La hauteur qu'on a calculee pour la largeur par defaut ne vaut plus : une hauteur
        // nulle laisse la carte prendre celle que son texte demande a cette largeur
        // (TEXT-FIT-1), comme l'application apres une saisie.
        *height = Some(hauteur.unwrap_or(0.0));
        if hauteur.is_some() {
            text.clear();
        }
    }
    store.add_annotation(&board, carte);
    store.clear_selection();
    store.set_viewport(
        &board,
        Viewport {
            scale: zoom,
            x: 10.0,
            y: 60.0,
        },
    );
    store
}

/// La texture de la carte qu'on écrit, telle que l'image la demanderait.
fn composant(renderer: &mut Renderer, store: &Store, corps: &str) -> Option<Composant> {
    couches(renderer, store, corps)?.composants.first().cloned()
}

/// Ce que l'image confie à la carte graphique quand on écrit `corps`, curseur au bout.
fn couches(renderer: &mut Renderer, store: &Store, corps: &str) -> Option<Confie> {
    let mut dessous = Pixmap::new(ECRAN.0, ECRAN.1)?;
    let mut dessus = Pixmap::new(ECRAN.0, ECRAN.1)?;
    let mut ui = UiState::new();
    ui.scale_factor = DENSITE;
    let guides = glucose_core::smart_align::SnapGuides::default();
    let saisie = TextEditSession {
        ann_id: "c".into(),
        buffer: corps.to_string(),
        selection: glucose_core::text::Selection::at(corps.len()),
        goal_x: None,
        blink_timer: Instant::now(),
        curseur_visible: true,
    };
    let confie = renderer.rendre_les_couches(
        &mut dessous.as_mut(),
        &mut dessus.as_mut(),
        store,
        (&mut ui, Pointer { x: 0.0, y: 0.0 }),
        SceneOverlay {
            editing: (!corps.is_empty()).then_some(&saisie),
            ..SceneOverlay::sans_rien(&guides)
        },
        Regard {
            degradation_permise: false,
            en_mouvement: false,
        },
    );
    Some(confie)
}

/// Le minimum et la médiane d'un rendu de cette texture, le cache de glyphes chaud.
///
/// **Le minimum dit le coût propre** d'un travail déterministe : tout ce qui s'y ajoute vient
/// d'ailleurs — un autre programme, la fréquence du processeur. Les deux sont rapportés : un
/// écart qui grandit entre eux est un avertissement sur la mesure, pas sur le code.
fn chronometrer(renderer: &Renderer, c: &Composant) -> ((f64, f64), (u32, u32)) {
    let premier = c.rendre(renderer.kit()).expect("une texture");
    let pixels = (premier.width(), premier.height());
    let mut t = Vec::with_capacity(REPRISES);
    for _ in 0..REPRISES {
        let debut = Instant::now();
        let rendu = c.rendre(renderer.kit());
        t.push(debut.elapsed().as_secs_f64() * 1000.0);
        drop(rendu);
    }
    ((centile(&t, 0.0), centile(&t, 0.5)), pixels)
}

fn main() {
    au_processeur();
    a_froid();
    par_la_carte();
}

/// **Un rendu à froid** : les glyphes retracés depuis leurs contours, parce que le cache ne
/// les a pas — ou ne les a plus. Et combien de variantes une image laisse dans le cache,
/// interface comprise : le cache est partagé, et plafonné.
fn a_froid() {
    println!("\n  A froid, et ce que le cache de glyphes garde :\n");
    for (largeur, zoom) in [(550.0, 1.0), (550.0, 2.3)] {
        let corps = notes();
        let mut renderer = Renderer::new();
        let store = une_carte(&renderer, &corps, (largeur, zoom), None);
        let Some(c) = composant(&mut renderer, &store, &corps) else {
            continue;
        };
        let apres_l_interface = renderer.typography.cached_glyph_count();
        let debut = Instant::now();
        let rendu = c.rendre(renderer.kit());
        let froid = debut.elapsed().as_secs_f64() * 1000.0;
        drop(rendu);
        let avec_la_carte = renderer.typography.cached_glyph_count();
        println!(
            "  ses notes {largeur:>7} {zoom:>4}   premier rendu {froid:.2} ms ; cache : {apres_l_interface} variantes apres l'interface, {avec_la_carte} avec la carte"
        );
    }
}

/// **Le rendu seul**, au processeur : ce qu'une frappe refait, et ce que la brume en prend.
fn au_processeur() {
    println!(
        "Ce qu'une frappe refait -- ecran {} x {} a {} %, mediane sur {REPRISES} reprises\n",
        ECRAN.0,
        ECRAN.1,
        DENSITE * 100.0
    );
    println!(
        "  {:>15} {:>7} {:>4} {:>10} {:>5}   {:>15}   {:>15}   {:>6}",
        "carte", "largeur", "zoom", "texture", "kpx", "tout min / med", "brume min / med", "ns/px"
    );
    let cas = [
        ("3 paragraphes", dense(3), 240.0, 1.0),
        ("6 paragraphes", dense(6), 400.0, 1.0),
        ("10 paragraphes", dense(10), 500.0, 1.0),
        ("14 paragraphes", dense(14), 500.0, 1.0),
        ("ses notes", notes(), 550.0, 1.0),
        ("ses notes", notes(), 550.0, 2.3),
    ];
    for (nom, corps, largeur, zoom) in cas {
        let mut renderer = Renderer::new();
        let store = une_carte(&renderer, &corps, (largeur, zoom), None);
        let Some(c) = composant(&mut renderer, &store, &corps) else {
            println!("  {nom:>15} {largeur:>7} {zoom:>4}   (pas un composant)");
            continue;
        };
        let (tout, pixels) = chronometrer(&renderer, &c);
        // La même boîte, vide : ce qui reste est la brume et l'anneau. La texture déborde
        // de la carte d'une marge de chaque côté ; à un pixel près, la brume est la même.
        let hauteur = f64::from(pixels.1 - 8) / zoom;
        let mut neuf = Renderer::new();
        let vide = une_carte(&neuf, "", (largeur, zoom), Some(hauteur));
        let brume = composant(&mut neuf, &vide, "")
            .map(|c| chronometrer(&neuf, &c).0)
            .unwrap_or((f64::NAN, f64::NAN));
        let kpx = f64::from(pixels.0) * f64::from(pixels.1) / 1000.0;
        println!(
            "  {nom:>15} {largeur:>7} {zoom:>4} {:>10} {kpx:>5.0}   {:>6.2} / {:>6.2}   {:>6.2} / {:>6.2}   {:>6.1}",
            format!("{}x{}", pixels.0, pixels.1),
            tout.0,
            tout.1,
            brume.0,
            brume.1,
            tout.0 * 1e6 / (kpx * 1000.0),
        );
    }
    println!(
        "\n  Lecture : `tout` est la texture qu'une frappe refait ; `brume` la meme boite sans \
         texte. En millisecondes ; ns/px sur le minimum."
    );
}

/// **Une frappe entière sur la carte graphique** : le rendu de la texture, sa création et son
/// téléversement — exactement ce que le poste `textures` de la chronique chronomètre
/// (`SceneGpu::assurer`), sur une scène qui dure d'une image à l'autre, comme dans
/// l'application. Une lettre de plus à chaque image.
fn par_la_carte() {
    let Some((peripherique, file)) = banc_gpu::carte() else {
        println!("\n  (pas de carte graphique : la seconde partie ne se mesure pas)");
        return;
    };
    println!("\n  Une frappe par image sur la carte graphique -- le poste `textures` :\n");
    for (nom, largeur, zoom) in [("ses notes", 550.0, 1.0), ("ses notes", 550.0, 2.3)] {
        let texte = notes();
        let mut renderer = Renderer::new();
        let mut scene = SceneGpu::nouvelle(&peripherique, banc_gpu::FORMAT);
        let mut temps = Vec::new();
        // Chaque frappe allonge le texte d'un caractère, depuis la moitié de la carte.
        let debuts: Vec<usize> = texte.char_indices().map(|(i, _)| i).collect();
        for &fin in &debuts[debuts.len() / 2..] {
            let corps = &texte[..fin];
            let store = une_carte(&renderer, corps, (largeur, zoom), None);
            let Some(confie) = couches(&mut renderer, &store, corps) else {
                continue;
            };
            let textures = confie.textures();
            scene.ouvrir();
            let debut = Instant::now();
            scene.assurer(
                &peripherique,
                &file,
                (&textures, std::time::Duration::MAX),
                &|cle| confie.pixels(&renderer, cle),
            );
            temps.push(debut.elapsed().as_secs_f64() * 1000.0);
            scene.preparer(
                &peripherique,
                &file,
                (ECRAN.0 as f32, ECRAN.1 as f32),
                &textures,
            );
            file.submit(None);
            // La carte rattrape entre deux images, comme à l'écran.
            let attente = wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            };
            peripherique.poll(attente).ok();
        }
        println!(
            "  {nom:>15} {largeur:>7} {zoom:>4}   {} frappes : min {:.2} ms, mediane {:.2}, p90 {:.2}, pire {:.2}",
            temps.len(),
            centile(&temps, 0.0),
            centile(&temps, 0.5),
            centile(&temps, 0.9),
            centile(&temps, 1.0),
        );
    }
}
