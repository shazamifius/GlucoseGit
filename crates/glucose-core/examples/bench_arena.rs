//! Le banc qui décide de l'arène — 0 dépendance, `std` uniquement.
//!
//! Lancer : `cargo run --release --example bench_arena`
//!
//! La fiche 11 § RQ-1 annonce « ~32 octets par nœud au lieu de ~512 » et un modèle qui tient
//! dix millions de nœuds. Ce sont des multiplications faites sur une feuille, pas des mesures :
//! personne n'avait chargé dix millions de nœuds. Ce banc le fait, des deux côtés, sur les
//! mêmes nœuds, et publie l'écart.
//!
//! Quatre questions, une colonne chacune :
//!
//! 1. **Combien pèse le document ?** Allocateur compteur : les octets réellement vivants.
//! 2. **Combien coûte le construire ?** C'est là que se paient les allocations par nœud.
//! 3. **Combien coûte retrouver un nœud ?** Balayage de `String` contre accès tableau.
//! 4. **Combien coûte une requête de viewport ?** C'est le geste dominant, à chaque frame.

use glucose_core::arena::text::TextArena;
use glucose_core::arena::{Arena, Box2, Kind, NodeId};
use glucose_core::fixed::Fx;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, BoardImage};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

// ── Allocateur compteur ──────────────────────────────────────────────────────

static LIVE: AtomicUsize = AtomicUsize::new(0);

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        LIVE.fetch_add(l.size(), Ordering::Relaxed);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size(), Ordering::Relaxed);
        System.dealloc(p, l)
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;

fn live() -> usize {
    LIVE.load(Ordering::Relaxed)
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

// ── La même scène, des deux côtés ────────────────────────────────────────────
//
// Une grille carrée au maillage de 400 px, un nœud sur deux étant une image. C'est la forme la
// plus favorable à l'ancien modèle : les nœuds sont contigus en mémoire dans l'ordre où ils ont
// été posés, donc son balayage est déjà séquentiel.

const MAILLAGE: i32 = 400;
const BOARD: &str = "main";
/// Le texte que porte chaque nœud textuel, des deux côtés du banc.
const TEXTE: &str = "Un nœud de banc";

fn cote(n: usize) -> i32 {
    (n as f64).sqrt().ceil() as i32
}

fn texte(id: &str, x: f64, y: f64) -> Annotation {
    Annotation::Text {
        id: id.into(),
        x,
        y,
        width: Some(260.0),
        height: Some(80.0),
        text: TEXTE.into(),
        font_size: None,
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

fn build_ancien(n: usize) -> Store {
    let mut s = Store::new("bench");
    let c = cote(n);
    s.begin_live_edit();
    for i in 0..n {
        let (x, y) = (
            (i as i32 % c * MAILLAGE) as f64,
            (i as i32 / c * MAILLAGE) as f64,
        );
        if i % 2 == 0 {
            s.add_image(
                BOARD,
                BoardImage::new(format!("img-{i}"), x, y, 200.0, 150.0),
            );
        } else {
            s.add_annotation(BOARD, texte(&format!("txt-{i}"), x, y));
        }
    }
    s.end_live_edit();
    s.clear_selection();
    s.journal.clear();
    s
}

fn build_arene(n: usize) -> (Arena, TextArena) {
    let mut a = Arena::with_capacity(n);
    // Un nœud sur deux est textuel : l'arène de texte est dimensionnée pour eux.
    let mut t = TextArena::with_capacity(n, n / 2 * TEXTE.len());
    let c = cote(n);
    for i in 0..n {
        let (x, y) = (i as i32 % c * MAILLAGE, i as i32 / c * MAILLAGE);
        let (kind, w, h) = if i % 2 == 0 {
            (Kind::Image, 200, 150)
        } else {
            (Kind::Text, 260, 80)
        };
        let b = Box2::new(
            Fx::from_px(x),
            Fx::from_px(y),
            Fx::from_px(w),
            Fx::from_px(h),
        );
        let id = a.spawn(kind, b, NodeId::NONE);
        if kind == Kind::Text {
            t.set(id, TEXTE);
        }
    }
    (a, t)
}

// ── Les quatre mesures ───────────────────────────────────────────────────────

struct Mesure {
    octets: usize,
    build_ms: f64,
    lookup_us: f64,
    query_us: f64,
    vus: usize,
}

/// La vue : un écran de 1920 × 1080 au centre de la grille, à l'échelle 1.
fn vue(n: usize) -> (f64, f64, f64, f64) {
    let milieu = (cote(n) / 2 * MAILLAGE) as f64;
    (milieu, milieu, 1920.0, 1080.0)
}

fn mesurer_ancien(n: usize) -> Mesure {
    let avant = live();
    let t = Instant::now();
    let store = build_ancien(n);
    let build_ms = ms(t);
    let octets = live().saturating_sub(avant);

    // Retrouver cent nœuds par identifiant, pris au milieu du document.
    let board = store.project.boards.iter().find(|b| b.id == BOARD).unwrap();
    let sondes = 100.min(n);
    let t = Instant::now();
    let mut trouves = 0usize;
    for k in 0..sondes {
        let cible = format!("img-{}", (n / 2 + k * 7) / 2 * 2);
        if board.images.iter().any(|i| i.id == cible) {
            trouves += 1;
        }
    }
    let lookup_us = ms(t) * 1000.0 / sondes as f64;
    assert!(trouves > 0, "les sondes doivent trouver quelque chose");

    // Requête de viewport par balayage du modèle — ce que fait le renderer sans index.
    let (vx, vy, vw, vh) = vue(n);
    let t = Instant::now();
    let mut vus = 0usize;
    for i in &board.images {
        if i.x <= vx + vw && vx <= i.x + i.width && i.y <= vy + vh && vy <= i.y + i.height {
            vus += 1;
        }
    }
    for a in &board.annotations {
        let (w, h) = (260.0, 80.0);
        if a.x() <= vx + vw && vx <= a.x() + w && a.y() <= vy + vh && vy <= a.y() + h {
            vus += 1;
        }
    }
    let query_us = ms(t) * 1000.0;

    Mesure {
        octets,
        build_ms,
        lookup_us,
        query_us,
        vus,
    }
}

fn mesurer_arene(n: usize) -> Mesure {
    let avant = live();
    let t = Instant::now();
    let (arene, textes) = build_arene(n);
    let build_ms = ms(t);
    let octets = live().saturating_sub(avant);
    let mo = |o: usize| o as f64 / 1_048_576.0;
    println!(
        "         |           | dont tronc {:>5.1} Mo, intervalles {:>5.1} Mo, texte {:>5.1} Mo",
        mo(n * Arena::BYTES_PER_NODE),
        mo(textes.slots() * TextArena::BYTES_PER_NODE),
        mo(textes.bytes_used()),
    );

    let sondes = 100.min(n);
    let t = Instant::now();
    let mut trouves = 0usize;
    for k in 0..sondes {
        let id = NodeId::from_index((n / 2 + k * 7) % n);
        if arene.alive(id) {
            trouves += 1;
        }
    }
    let lookup_us = ms(t) * 1000.0 / sondes as f64;
    assert!(trouves > 0);

    let (vx, vy, vw, vh) = vue(n);
    let v = Box2::new(
        Fx::from_f64(vx),
        Fx::from_f64(vy),
        Fx::from_f64(vw),
        Fx::from_f64(vh),
    );
    let mut out = Vec::new();
    let t = Instant::now();
    arene.cull(v, &mut out);
    let query_us = ms(t) * 1000.0;

    Mesure {
        octets,
        build_ms,
        lookup_us,
        query_us,
        vus: out.len(),
    }
}

fn ligne(nom: &str, n: usize, m: &Mesure) {
    println!(
        "{nom:<8} | {n:>9} | {:>9.1} | {:>7.0} | {:>9.0} | {:>10.2} | {:>10.1} | {:>6}",
        m.octets as f64 / 1_048_576.0,
        m.octets as f64 / n as f64,
        m.build_ms,
        m.lookup_us,
        m.query_us,
        m.vus,
    );
}

fn main() {
    // `bench_arena [n] [ancien|arene]` — le second argument ne mesure qu'un modèle. À dix
    // millions de nœuds, l'ancien demande cinq gigaoctets pour le document et autant pour la
    // transaction ouverte qui l'enregistre : pouvoir les peser séparément évite de mesurer la
    // pression mémoire de la machine au lieu du modèle.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tailles: Vec<usize> = match args.first() {
        Some(v) => vec![v.parse().expect("un nombre de nœuds")],
        None => vec![10_000, 100_000, 1_000_000],
    };
    let seul = args.get(1).map(String::as_str);

    println!("Banc de l'arène — ancien modèle contre tableaux de champs, mêmes nœuds\n");
    println!(
        "{:<8} | {:>9} | {:>9} | {:>7} | {:>9} | {:>10} | {:>10} | {:>6}",
        "modèle", "nœuds", "mémoire Mo", "o/nœud", "build ms", "lookup µs", "query µs", "vus"
    );
    println!("{}", "-".repeat(86));

    for n in tailles {
        if seul == Some("arene") {
            ligne("arène", n, &mesurer_arene(n));
            continue;
        }
        let a = mesurer_ancien(n);
        ligne("ancien", n, &a);
        if seul == Some("ancien") {
            continue;
        }
        let b = mesurer_arene(n);
        ligne("arène", n, &b);
        println!(
            "{:<8} | {:>9} | {:>8.1}× | {:>6.1}× | {:>8.1}× | {:>9.1}× | {:>9.1}× | {}",
            "rapport",
            "",
            a.octets as f64 / b.octets.max(1) as f64,
            a.octets as f64 / b.octets.max(1) as f64,
            a.build_ms / b.build_ms.max(1e-9),
            a.lookup_us / b.lookup_us.max(1e-9),
            a.query_us / b.query_us.max(1e-9),
            if a.vus == b.vus {
                "mêmes nœuds vus"
            } else {
                "ÉCART DE RÉSULTAT"
            },
        );
        println!("{}", "-".repeat(86));
    }

    println!("\nLecture :");
    println!(
        "  • « o/nœud »   = le document entier divisé par le nombre de nœuds, texte compris des"
    );
    println!("    deux côtés. L'arène ne range AUCUN identifiant lisible : le NodeId EST l'identité, et les"
    );
    println!(
        "    noms ne survivent que dans le format de fichier. L'ancien modèle porte une String par"
    );
    println!(
        "    nœud : c'est une part de l'écart, et elle est assumée, pas oubliée.");
    println!(
        "  • « lookup µs » = retrouver un nœud par son identifiant. L'ancien balaie et compare"
    );
    println!("    des chaînes ; l'arène indexe un tableau.");
    println!(
        "  • « query µs »  = balayer tout le document pour trouver ce qui croise l'écran, SANS"
    );
    println!("    index spatial des deux côtés. C'est la comparaison honnête de la disposition");
    println!("    mémoire seule — l'index arrive à l'étape B.3 et changera les deux colonnes.");
    println!("  • « vus » doit être identique des deux côtés, sinon la comparaison ne vaut rien.");
}
