//! **Ce que coûte d'offrir une image à Windows, puis de la réclamer** (fiche 32, étape 4).
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
//! premier accès aux pixels d'une région réclamée — les pages reviennent une à une, et une
//! photo de dix mégaoctets en compte deux mille cinq cents.

#[cfg(windows)]
fn main() {
    mesure::main();
}

#[cfg(not(windows))]
fn main() {
    println!("OfferVirtualMemory n'existe que sous Windows");
}

#[cfg(windows)]
mod mesure {
    use std::time::{Duration, Instant};
    use windows::Win32::System::Memory::{
        OfferVirtualMemory, ReclaimVirtualMemory, VirtualAlloc, VirtualFree, MEM_COMMIT,
        MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VmOfferPriorityNormal,
    };

    /// La tête d'une pyramide d'épingle : l'original et son premier niveau, en RGBA.
    const OCTETS: usize = 10 * 1024 * 1024;
    const REGIONS: usize = 24;
    const ALLERS_RETOURS: usize = 200;

    struct Region {
        debut: *mut u8,
    }

    impl Region {
        fn nouvelle(graine: u8) -> Option<Self> {
            // Sûr : une allocation neuve, rendue par `VirtualFree` dans `drop`.
            let p = unsafe { VirtualAlloc(None, OCTETS, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE) };
            if p.is_null() {
                return None;
            }
            let mut r = Self { debut: p.cast() };
            for (i, o) in r.octets_mut().iter_mut().enumerate() {
                *o = (i as u8).wrapping_mul(31).wrapping_add(graine);
            }
            Some(r)
        }

        fn octets(&self) -> &[u8] {
            // Sûr : la région vit tant que `self` vit.
            unsafe { std::slice::from_raw_parts(self.debut, OCTETS) }
        }

        fn octets_mut(&mut self) -> &mut [u8] {
            // Sûr : comme `octets`, et l'emprunt exclusif interdit toute autre vue.
            unsafe { std::slice::from_raw_parts_mut(self.debut, OCTETS) }
        }

        fn somme(&self) -> u64 {
            self.octets().iter().map(|&o| u64::from(o)).sum()
        }
    }

    impl Drop for Region {
        fn drop(&mut self) {
            // Sûr : l'adresse vient de `VirtualAlloc`, et `MEM_RELEASE` exige une taille nulle.
            let _ = unsafe { VirtualFree(self.debut.cast(), 0, MEM_RELEASE) };
        }
    }

    fn travail_mo() -> f64 {
        glucose_desktop::plateforme::empreinte::relever()
            .map_or(0.0, |e| e.memoire_octets as f64 / 1_048_576.0)
    }

    fn par_region(total: Duration) -> String {
        format!("{:>8.1} us par region", total.as_secs_f64() * 1e6 / REGIONS as f64)
    }

    pub fn main() {
        let t = Instant::now();
        let mut regions: Vec<Region> = (0..REGIONS as u8).filter_map(Region::nouvelle).collect();
        println!(
            "{} regions de {} Mo ecrites en {:.1} ms -- memoire de travail {:.0} Mo",
            regions.len(),
            OCTETS / 1_048_576,
            t.elapsed().as_secs_f64() * 1e3,
            travail_mo()
        );
        let attendu: Vec<u64> = regions.iter().map(Region::somme).collect();

        let t = Instant::now();
        let chaude: u64 = regions.iter().map(Region::somme).sum();
        let lecture_chaude = t.elapsed();

        let t = Instant::now();
        for r in &mut regions {
            let _ = unsafe { OfferVirtualMemory(r.octets_mut(), VmOfferPriorityNormal) };
        }
        println!(
            "offrir                    {}   -- memoire de travail {:.0} Mo",
            par_region(t.elapsed()),
            travail_mo()
        );

        let t = Instant::now();
        let intactes = regions
            .iter()
            .filter(|r| unsafe { ReclaimVirtualMemory(r.octets()) } == 0)
            .count();
        println!(
            "reclamer                  {}   -- {intactes}/{} intactes",
            par_region(t.elapsed()),
            regions.len()
        );

        let t = Instant::now();
        let apres: Vec<u64> = regions.iter().map(Region::somme).collect();
        let lecture_apres = t.elapsed();
        println!(
            "lire apres reclamation    {}   (lecture chaude {})",
            par_region(lecture_apres),
            par_region(lecture_chaude).trim()
        );
        println!(
            "contenu {} -- memoire de travail {:.0} Mo (controle {chaude})",
            if apres == attendu { "identique" } else { "ALTERE" },
            travail_mo()
        );

        // Une photo au bord de l'écran, qui entre et sort à chaque image.
        let r = &mut regions[0];
        let t = Instant::now();
        for _ in 0..ALLERS_RETOURS {
            let _ = unsafe { OfferVirtualMemory(r.octets_mut(), VmOfferPriorityNormal) };
            let _ = unsafe { ReclaimVirtualMemory(r.octets()) };
        }
        println!(
            "offrir puis reclamer, sans lire : {:.1} us l'aller-retour",
            t.elapsed().as_secs_f64() * 1e6 / ALLERS_RETOURS as f64
        );
        let t = Instant::now();
        let mut s = 0u64;
        for _ in 0..ALLERS_RETOURS / 10 {
            let _ = unsafe { OfferVirtualMemory(r.octets_mut(), VmOfferPriorityNormal) };
            let _ = unsafe { ReclaimVirtualMemory(r.octets()) };
            s = s.wrapping_add(r.somme());
        }
        println!(
            "offrir, reclamer et relire 10 Mo : {:.2} ms l'aller-retour ({s})",
            t.elapsed().as_secs_f64() * 1e3 / (ALLERS_RETOURS / 10) as f64
        );
    }
}
