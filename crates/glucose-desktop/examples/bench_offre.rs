//! **Ce que coûte d'offrir une image à Windows, puis de la reprendre** (fiche 32, étape 4).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_offre
//! ```
//!
//! # La question
//!
//! La mémoire vive de Glucose, ce sont ses images décodées : 1 186 Mo pour les 243 photos du
//! document de l'utilisateur. Les garder toutes est juste tant que personne d'autre n'a besoin
//! de la place ; les rendre trop tôt oblige à redécoder. `OfferVirtualMemory` tranche sans
//! rien choisir : les pages offertes restent là, mais le système les reprend **au moment où
//! une autre application en a besoin**, sans les écrire sur le disque. `ReclaimVirtualMemory`
//! dit ensuite si elles sont intactes.
//!
//! Avant de bâtir un étage dessus, il faut savoir ce que coûtent les deux gestes, et le
//! premier accès aux pixels d'une région reprise — les pages reviennent une à une, et une
//! photo de dix mégaoctets en compte deux mille cinq cents.
//!
//! Le banc passe par les types de l'application ([`Tenue`], [`Offerte`]) : ce qu'il mesure est
//! ce qu'elle fait, et il n'a besoin d'aucun `unsafe` pour le faire.

use glucose_desktop::plateforme::offre::{Offerte, Tenue};
use std::time::{Duration, Instant};

/// La tête d'une pyramide d'épingle : l'original et son premier niveau, en RGBA.
const OCTETS: usize = 10 * 1024 * 1024;
const REGIONS: usize = 24;
const ALLERS_RETOURS: usize = 200;

fn remplie(graine: u8) -> Option<Tenue> {
    let mut r = Tenue::nouvelle(OCTETS)?;
    for (i, o) in r.octets_mut().iter_mut().enumerate() {
        *o = (i as u8).wrapping_mul(31).wrapping_add(graine);
    }
    Some(r)
}

fn somme(octets: &[u8]) -> u64 {
    octets.iter().map(|&o| u64::from(o)).sum()
}

fn travail_mo() -> f64 {
    glucose_desktop::plateforme::empreinte::relever()
        .map_or(0.0, |e| e.memoire_octets as f64 / 1_048_576.0)
}

fn par_region(total: Duration) -> String {
    format!(
        "{:>8.1} us par region",
        total.as_secs_f64() * 1e6 / REGIONS as f64
    )
}

fn main() {
    let t = Instant::now();
    let tenues: Vec<Tenue> = (0..REGIONS as u8).filter_map(remplie).collect();
    println!(
        "{} regions de {} Mo ecrites en {:.1} ms -- memoire de travail {:.0} Mo",
        tenues.len(),
        OCTETS / 1_048_576,
        t.elapsed().as_secs_f64() * 1e3,
        travail_mo()
    );
    let attendu: Vec<u64> = tenues.iter().map(|r| somme(r.octets())).collect();
    let t = Instant::now();
    let chaude: u64 =
        attendu.iter().sum::<u64>() + tenues.iter().map(|r| somme(r.octets())).sum::<u64>();
    let lecture_chaude = t.elapsed();

    let t = Instant::now();
    let offertes: Vec<Offerte> = tenues.into_iter().map(Tenue::offrir).collect();
    println!(
        "offrir                    {}   -- memoire de travail {:.0} Mo",
        par_region(t.elapsed()),
        travail_mo()
    );

    let t = Instant::now();
    let reprises: Vec<Option<Tenue>> = offertes.into_iter().map(Offerte::reprendre).collect();
    println!(
        "reprendre                 {}   -- {}/{} intactes",
        par_region(t.elapsed()),
        reprises.iter().filter(|r| r.is_some()).count(),
        reprises.len()
    );
    let mut reprises: Vec<Tenue> = reprises.into_iter().flatten().collect();

    let t = Instant::now();
    let apres: Vec<u64> = reprises.iter().map(|r| somme(r.octets())).collect();
    println!(
        "lire apres reprise        {}   (lecture chaude {})",
        par_region(t.elapsed()),
        par_region(lecture_chaude / 2).trim()
    );
    println!(
        "contenu {} -- memoire de travail {:.0} Mo (controle {chaude})",
        if apres == attendu {
            "identique"
        } else {
            "ALTERE"
        },
        travail_mo()
    );

    // Une photo au bord de l'écran, qui entre et sort à chaque image.
    let Some(mut r) = reprises.pop() else {
        return;
    };
    let t = Instant::now();
    for _ in 0..ALLERS_RETOURS {
        let Some(reprise) = r.offrir().reprendre() else {
            println!("le systeme a jete la region : l'aller-retour s'arrete");
            return;
        };
        r = reprise;
    }
    println!(
        "offrir puis reprendre, sans lire : {:.1} us l'aller-retour",
        t.elapsed().as_secs_f64() * 1e6 / ALLERS_RETOURS as f64
    );
    let t = Instant::now();
    let mut s = 0u64;
    for _ in 0..ALLERS_RETOURS / 10 {
        let Some(reprise) = r.offrir().reprendre() else {
            return;
        };
        r = reprise;
        s = s.wrapping_add(somme(r.octets()));
    }
    println!(
        "offrir, reprendre et relire 10 Mo : {:.2} ms l'aller-retour ({s})",
        t.elapsed().as_secs_f64() * 1e3 / (ALLERS_RETOURS / 10) as f64
    );
}
