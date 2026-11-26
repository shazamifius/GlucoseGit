//! **Les vidéos du Mac, la file principale servie** (fiche 175 § 9). AVPlayer ne devient « prêt »
//! qu'en passant par la file principale : à la CI de `1b4ac90`, chaque vidéo restait « pas prête
//! après 10 s », statut inconnu, sans erreur. Glucose la sert par la boucle de sa fenêtre ; le
//! harnais de `cargo test` jamais — son fil principal attend ses épreuves. Cette suite, sans
//! harnais, la sert comme Glucose (`CFRunLoopRun` sur le fil principal) et passe sur un autre fil
//! les épreuves vidéo partagées (`renderer/video/epreuves.rs`). Ailleurs, ces épreuves tournent
//! sous le harnais : elle n'a rien à faire.

#[cfg(target_os = "macos")]
fn main() {
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRunLoopRun();
    }
    std::thread::spawn(|| {
        let toutes = glucose_desktop::epreuves_video::TOUTES;
        let mut tombees = Vec::new();
        for (nom, epreuve) in toutes {
            let tenue = std::panic::catch_unwind(epreuve).is_ok();
            println!("test {nom} ... {}", if tenue { "ok" } else { "FAILED" });
            if !tenue {
                tombees.push(nom);
            }
        }
        let tenues = toutes.len() - tombees.len();
        println!("\ntest result: {tenues} passed; {} failed", tombees.len());
        std::process::exit(i32::from(!tombees.is_empty()));
    });
    loop {
        // SAFETY : la boucle du fil principal, qui sert sa file ; elle ne rend la main que vide.
        unsafe { CFRunLoopRun() };
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {}
