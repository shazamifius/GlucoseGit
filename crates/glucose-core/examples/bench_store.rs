//! Banc de mesure du noyau — 0 dépendance, `std` uniquement.
//!
//! Lancer : `cargo run --release --example bench_store`
//!
//! Il répond à trois questions, avec des chiffres et non des impressions :
//!
//! 1. **Combien coûte une mutation ?** `push_undo` clone le projet entier à chaque appel, et il
//!    est appelé depuis 34 points de mutation. La loi L3 dit « une modification coûte la taille
//!    de la modification, jamais celle du document » : ce banc mesure l'écart.
//! 2. **Combien pèse un nœud, réellement ?** Pas `size_of`, qui ignore le tas : un allocateur
//!    compteur donne les octets effectivement alloués, chaînes et vecteurs compris.
//! 3. **Combien coûte retrouver un nœud ?** Le store cherche par balayage linéaire.

use glucose_core::quadtree::SpatialHash;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, BoardImage};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

// ── Allocateur compteur ──────────────────────────────────────────────────────
// Mesure les octets réellement vivants sur le tas. C'est la seule façon honnête de peser un
// modèle dont les champs sont des `String` et des `Vec` : `size_of` n'en voit que les en-têtes.

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

fn live_bytes() -> usize {
    LIVE.load(Ordering::Relaxed)
}

// ── Génération déterministe ──────────────────────────────────────────────────
// Pas de dépendance à un générateur aléatoire : une suite congruentielle suffit, et elle rend
// le banc strictement reproductible d'une exécution à l'autre.

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    fn coord(&mut self, span: f64) -> f64 {
        (self.next() % 1_000_000) as f64 / 1_000_000.0 * span
    }
}

const BOARD: &str = "main";

/// Crée une annotation texte. Quatorze champs dont douze ne disent rien : il n'existe aucun
/// constructeur dans le noyau, et chaque site de création recopie ce bloc. C'est la
/// « répétition structurelle » de la fiche 05, mesurable ici même.
fn text_at(id: &str, x: f64, y: f64) -> Annotation {
    Annotation::Text {
        id: id.to_string(),
        x,
        y,
        width: Some(320.0),
        height: Some(90.0),
        text: "Un fait, écrit à la main, comme sur le canva.".to_string(),
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

/// Remplit un store de `n` nœuds (moitié images, moitié textes) répartis sur un carré.
///
/// La construction se fait **sous transaction live** : sans elle, chaque ajout clonerait tout
/// le document et le remplissage serait quadratique — impossible à mener jusqu'à 10⁶.
fn build(n: usize, span: f64) -> Store {
    let mut store = Store::new("bench");
    let mut rng = Lcg(0x5eed);
    store.begin_live_edit();
    for i in 0..n {
        let (x, y) = (rng.coord(span), rng.coord(span));
        if i % 2 == 0 {
            let mut img = BoardImage::new(format!("img-{i}"), x, y, 200.0, 150.0);
            img.tags.push("bench".to_string());
            store.add_image(BOARD, img);
        } else {
            store.add_annotation(BOARD, text_at(&format!("txt-{i}"), x, y));
        }
    }
    store.end_live_edit();
    store.clear_selection();
    // Charger un document n'est pas un geste annulable : `load_project` vide la pile, et le
    // banc doit mesurer le modèle, pas l'historique de sa construction.
    store.journal.clear();
    store
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

fn run(n: usize) {
    let span = (n as f64).sqrt() * 400.0;

    let before = live_bytes();
    let t = Instant::now();
    let mut store = build(n, span);
    let build_ms = ms(t);
    let model_bytes = live_bytes().saturating_sub(before);

    // ── 1. Le coût d'UNE mutation ordinaire, hors transaction live ──────────
    // C'est le geste de tous les jours : déplacer une carte, changer un texte. Chaque appel
    // passe par `push_undo`, qui clone le document entier.
    // Un élément du MILIEU de la liste, pas le premier : chercher `img-0` ferait mentir la
    // mesure, le balayage le trouvant à la première comparaison.
    let id = format!("img-{}", (n / 2) & !1);
    let before_undo = live_bytes();
    let t = Instant::now();
    store.update_image(BOARD, &id, |i| i.x += 1.0);
    let mutate_ms = ms(t);
    let undo_bytes = live_bytes().saturating_sub(before_undo);

    // Coût projeté d'une pile pleine : 200 clones (LIMITS.UNDO_DEPTH).
    let journal_kb = store.journal.weight() as f64 / 1024.0;

    // ── 1 bis. Le geste réel : saisir une carte et la déplacer ──────────────
    // Un glisser complet — `begin_live_edit`, trente déplacements, `end_live_edit`. C'est ce
    // que fait la main de l'utilisateur, et c'est la mesure qui compte vraiment.
    store.select_image(id.clone(), false);

    // La SAISIE seule. C'est ici que le filet d'ouverture clonait le document entier.
    // Mesurée à part, elle prouve la loi par invariance : si elle vaut la même chose à 10³
    // et à 10⁶ nœuds, alors ∂T/∂n = 0 — et cette conclusion ne dépend pas de la vitesse de
    // la machine, contrairement à une comparaison avec une exécution antérieure.
    let t = Instant::now();
    store.begin_live_edit();
    let grab_us = ms(t) * 1000.0;

    let t = Instant::now();
    for _ in 0..30 {
        store.move_selected(BOARD, 1.0, 0.0);
    }
    store.end_live_edit();
    let drag_ms = ms(t);
    store.clear_selection();

    // ── 2. Retrouver un nœud par identifiant (balayage linéaire) ────────────
    store.begin_live_edit(); // neutralise le clone, on ne mesure que la recherche
    let mut rng = Lcg(0xbeef);
    let probes = 200.min(n);
    let t = Instant::now();
    for _ in 0..probes {
        let k = (rng.next() as usize % n.max(1) / 2) * 2;
        store.update_image(BOARD, &format!("img-{k}"), |i| i.rotation += 0.0);
    }
    let lookup_us = ms(t) * 1000.0 / probes as f64;
    store.end_live_edit();

    // ── 3. Index spatial et requête de viewport ─────────────────────────────
    let board = store.project.boards.iter().find(|b| b.id == BOARD).unwrap();
    let t = Instant::now();
    let mut hash = SpatialHash::new(512.0);
    hash.index_board(board);
    let index_ms = ms(t);

    let t = Instant::now();
    let hits = hash.query_ids(span / 2.0, span / 2.0, 1920.0, 1080.0, 0.0);
    let query_us = ms(t) * 1000.0;

    println!(
        "{n:>9} | {build_ms:>9.1} | {:>8.1} | {:>7.0} | {mutate_ms:>9.2} | {grab_us:>8.2} | {drag_ms:>9.1} | {:>10.2} | {journal_kb:>10.1} | {lookup_us:>9.1} | {index_ms:>8.1} | {query_us:>9.1} | {:>6}",
        model_bytes as f64 / 1_048_576.0,
        model_bytes as f64 / n as f64,
        undo_bytes as f64 / 1_048_576.0,
        hits.len(),
    );
}

fn main() {
    println!("Banc du noyau Glucose — mesures réelles, allocateur compteur, 0 dépendance\n");
    println!(
        "{:>9} | {:>9} | {:>8} | {:>7} | {:>9} | {:>8} | {:>9} | {:>10} | {:>10} | {:>9} | {:>8} | {:>9} | {:>6}",
        "nœuds",
        "build ms",
        "modèle Mo",
        "o/nœud",
        "1 mutat.ms",
        "saisie µs",
        "30 pas ms",
        "alloué Mo",
        "journal Ko",
        "lookup µs",
        "index ms",
        "query µs",
        "vus",
    );
    println!("{}", "-".repeat(139));

    for n in [1_000usize, 10_000, 100_000, 1_000_000] {
        run(n);
    }

    println!("\nLecture :");
    println!("  • « alloué Mo »  = mémoire allouée par UNE seule mutation ordinaire.");
    println!("  • « saisie µs »  = begin_live_edit seul. Doit être constant : c'est la preuve de ∂T/∂n = 0.
  • « 30 pas ms »  = trente déplacements + end_live_edit, une fois la carte saisie.");
    println!(
        "  • « lookup µs »  = retrouver un nœud par identifiant (balayage linéaire du board)."
    );
    println!("  • Loi L3 : une modification doit coûter la taille de la modification, pas celle du document.");

    println!("\nSauvegarde (fiche 09 § 3–4) — 1 000 nœuds, actifs de 1 Mo :\n");
    println!(
        "{:>9} | {:>9} | {:>10} | {:>10}",
        "actifs Mo", "encode ms", "fichier Mo", "alloué Mo"
    );
    println!("{}", "-".repeat(49));
    for mb in [0usize, 16, 64, 256] {
        run_save(mb);
    }
    println!("\nLecture :");
    println!("  • « encode ms »  = construire les octets du fichier, sans I/O. Une sauvegarde qui ne change");
    println!("    rien au document devrait coûter la modification (zéro), pas le volume des actifs.");
    println!("  • « alloué Mo »  = mémoire vivante au sortir de l'encodage, au-delà du modèle et des actifs :");
    println!("    ce que le fichier en construction a coûté en copies.");
}

/// Une sauvegarde : `persist::encode` sur un projet de 1 000 nœuds portant `assets_mb`
/// mégaoctets d'actifs. Fiche 09 § 4.2 : l'empreinte d'un actif devrait se calculer une fois,
/// à l'import ; elle l'est aujourd'hui à chaque enregistrement, sur chaque octet.
fn run_save(assets_mb: usize) {
    use glucose_core::persist::encode;
    use glucose_core::types::AssetStore;

    let store = build(1_000, 10_000.0);
    // Des actifs de 1 Mo, au contenu distinct. Les bits **hauts** du générateur : ses bits
    // bas ont une période de 512 Ko, et une première version de ce banc produisait 256 actifs
    // identiques — dédupliqués en un seul blob, un fichier de 1 Mo, et une mesure qui mentait.
    let mut assets = AssetStore::new();
    let mut rng = Lcg(7);
    for i in 0..assets_mb {
        let bytes: Vec<u8> = (0..1_048_576).map(|_| (rng.next() >> 40) as u8).collect();
        assets.insert(format!("photo-{i}.jpg"), bytes);
    }

    let before = live_bytes();
    let t = Instant::now();
    let file = encode(&store.project, &assets, 0);
    let encode_ms = ms(t);
    let peak_extra = live_bytes().saturating_sub(before);
    let file_mb = file.len() as f64 / 1_048_576.0;
    drop(file);

    println!(
        "{assets_mb:>9} | {encode_ms:>9.1} | {file_mb:>10.1} | {:>10.1}",
        peak_extra as f64 / 1_048_576.0
    );
}
