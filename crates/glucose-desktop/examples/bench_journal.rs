//! Ce que la **sauvegarde en cours de route** de la chronique coûte, et combien de fois elle a lieu.
//!
//! # La question, et pourquoi elle se pose maintenant
//!
//! Trois sessions de terrain portent le même poste inexpliqué : **763 à 13 099 ms « à ne pas
//! dessiner »**, c'est-à-dire entre la fin d'une image et le début de la suivante, hors de
//! tout code de rendu. La fiche 24 § 12 le nomme « jamais instrumenté » ; la fiche 25 § 9.4
//! ajoute qu'on sait désormais que ce n'est **pas** un refus de surface.
//!
//! Or `about_to_wait` — le seul endroit de la boucle qui s'exécute là — appelle
//! `sauver_la_chronique_si_besoin`, qui **écrit le rapport entier sur disque** dès qu'une
//! image entre dans les trente-deux plus lentes de la session.
//!
//! Ce banc ne suppose rien de la réponse. Il chiffre deux choses que personne n'a mesurées :
//!
//! 1. **combien de fois** la sauvegarde se déclenche sur une session, ce qui ne dépend que de
//!    la mécanique du top-32 et de l'ordre des images ;
//! 2. **ce qu'un déclenchement coûte** : composer le rapport, puis l'écrire là où
//!    l'application l'écrit vraiment — `%TEMP%\glucose-chronique\`, antivirus compris.
//!
//! Les deux séparément, parce qu'ils appellent des réponses opposées : composer trop souvent
//! se corrige en espaçant, composer trop cher se corrige en composant moins.
//!
//! `cargo run --release -p glucose-desktop --example bench_journal`

use glucose_desktop::chronique::{Chronique, Geste, Instantane};
use std::time::{Duration, Instant};

/// La session réelle du 22/09 au soir, telle que sa chronique la décrit.
///
/// Les durées viennent des centiles publiés — médiane 5,79 ms, p90 11,59, p99 16,38, pire
/// 409,73 — et leurs effectifs reconstituent la forme de la distribution sur 731 images.
/// Ce n'est pas une loi choisie : c'est le terrain, rejoué.
const TERRAIN: [(u32, usize); 6] = [
    (4_870, 300),
    (5_790, 250),
    (8_190, 100),
    (11_590, 55),
    (16_380, 19),
    (409_730, 7),
];

/// Un générateur déterministe, pour que deux exécutions du banc donnent le même nombre.
///
/// L'ordre décide du nombre de déclenchements : des durées croissantes en donneraient une par
/// image, des durées décroissantes trente-deux. C'est l'ordre **quelconque** qui répond, et il
/// doit être reproductible pour que la mesure se relise.
struct Melange(u64);

impl Melange {
    fn suivant(&mut self) -> u64 {
        // Le générateur congruentiel de Numerical Recipes : deux constantes, aucune liberté.
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.0 >> 33
    }
}

/// Les durées du terrain, dans un ordre quelconque et reproductible.
fn session() -> Vec<u32> {
    let mut durees: Vec<u32> = TERRAIN
        .iter()
        .flat_map(|(us, combien)| std::iter::repeat_n(*us, *combien))
        .collect();
    let mut melange = Melange(0x5eed);
    for i in (1..durees.len()).rev() {
        let j = (melange.suivant() as usize) % (i + 1);
        durees.swap(i, j);
    }
    durees
}

/// Une chronique remplie d'une session entière, et le nombre de sauvegardes qu'elle a demandées.
fn rejouer(durees: &[u32]) -> (Chronique, usize) {
    let mut chronique = Chronique::nouvelle();
    chronique
        .rythme
        .observer_la_machine(Duration::from_micros(4_166), "fifo (cale sur le balayage)");
    let postes: Vec<usize> = ["tempo", "textures", "present", "docks", "blit", "effacer"]
        .iter()
        .filter_map(|nom| chronique.poste(nom))
        .collect();

    let mut sauvegardes = 0;
    for (i, duree_us) in durees.iter().enumerate() {
        let mut vu = Instantane {
            duree_us: *duree_us,
            geste: Geste::TOUS
                .iter()
                .position(|g| *g == Geste::Zoomer)
                .unwrap_or(0) as u8,
            noeuds: 22,
            ..Instantane::default()
        };
        // Les postes se partagent l'image, comme sur le terrain : le tempo en tient le gros.
        for (rang, poste) in postes.iter().enumerate() {
            vu.postes_us[*poste] = duree_us / (2 + rang as u32);
        }
        vu.intervalle_us = duree_us + 6_890;
        vu.tempo_balayages = 3;
        vu.tempo_attente_us = 6_890;
        vu.reveils = 1 << (i % 8);
        chronique.enregistrer(vu);
        if chronique.du_neuf() {
            sauvegardes += 1;
        }
    }
    (chronique, sauvegardes)
}

/// La médiane et le pire d'une série de durées.
fn resume(mut mesures: Vec<Duration>) -> (Duration, Duration) {
    mesures.sort_unstable();
    let median = mesures[mesures.len() / 2];
    let pire = *mesures.last().expect("au moins une mesure");
    (median, pire)
}

fn main() {
    let durees = session();
    let (chronique, sauvegardes) = rejouer(&durees);
    let rapport = chronique.rapport();

    println!("== Ce que la sauvegarde en cours de route coute ==\n");
    println!("session rejouee : {} images", durees.len());
    println!(
        "sauvegardes demandees : {sauvegardes}  -- soit une image sur {:.1}",
        durees.len() as f64 / sauvegardes.max(1) as f64
    );
    println!("taille du rapport : {} octets\n", rapport.len());

    // **Composer le rapport.** Il parcourt tous les histogrammes, trie les postes de chaque
    // geste et met en forme trente-deux images lentes.
    let mut compositions = Vec::new();
    for _ in 0..40 {
        let depart = Instant::now();
        let texte = chronique.rapport();
        compositions.push(depart.elapsed());
        std::hint::black_box(texte.len());
    }
    let (compo_med, compo_pire) = resume(compositions);

    // **L'ecrire la ou l'application l'ecrit**, pour que l'antivirus et le systeme de
    // fichiers soient dans la mesure. Un banc qui ecrit ailleurs mesure autre chose.
    let chemin = std::env::temp_dir()
        .join("glucose-chronique")
        .join("banc-journal.txt");
    if let Some(dossier) = chemin.parent() {
        std::fs::create_dir_all(dossier).expect("le dossier de la chronique");
    }
    let mut ecritures = Vec::new();
    for _ in 0..40 {
        let depart = Instant::now();
        std::fs::write(&chemin, &rapport).expect("ecriture de la chronique");
        ecritures.push(depart.elapsed());
    }
    let (ecrit_med, ecrit_pire) = resume(ecritures);
    let _ = std::fs::remove_file(&chemin);

    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    println!("             median      pire");
    println!(
        "composer   {:7.2}ms {:7.2}ms",
        ms(compo_med),
        ms(compo_pire)
    );
    println!(
        "ecrire     {:7.2}ms {:7.2}ms",
        ms(ecrit_med),
        ms(ecrit_pire)
    );
    let total_med = ms(compo_med) + ms(ecrit_med);
    let total_pire = ms(compo_pire) + ms(ecrit_pire);
    println!("total      {total_med:7.2}ms {total_pire:7.2}ms\n");

    println!(
        "sur la session : {:.0} ms au median, {:.0} ms si chaque sauvegarde etait la pire",
        total_med * sauvegardes as f64,
        total_pire * sauvegardes as f64
    );
    println!(
        "soit {:.2} ms par image en moyenne, hors de tout rendu",
        total_med * sauvegardes as f64 / durees.len() as f64
    );
}
