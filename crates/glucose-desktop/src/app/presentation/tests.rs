//! Ce que la présentation note de chaque image, par le vrai chemin de l'application.

use super::*;

/// **Une image en retard sur le tempo est comptée comme ratée par la chronique** (TEMPO-2) —
/// par le vrai chemin : le tempo juge, la présentation le note, le relevé le recopie.
///
/// Les épreuves du tempo et de la chronique ne le prouvent pas à elles deux : un relevé qui
/// recopierait zéro les passait toutes, et le rapport aurait dit « aucune image n'a raté
/// son balayage » sur une session à trente images par seconde. C'est le sabotage qui l'a
/// montré.
#[test]
fn test_une_image_en_retard_est_comptee_ratee_par_la_chronique() {
    let mut app = GlucoseApp::new();
    let periode = std::time::Duration::from_millis(1);
    app.tempo.accorder(periode);
    // Le tempo ne règle que ce qui bouge : un élan en cours.
    app.elan
        .pousser_pan(10_000.0, 0.0, std::time::Instant::now());
    // La première image pose la grille.
    crate::perf::frame_begin();
    app.attendre_l_heure_de_soumettre();
    app.enregistrer_l_image(1_000, (800, 600));
    assert!(
        app.chronique.ratees().is_none(),
        "la premiere image n'a rien rate"
    );
    // La seconde arrive dix périodes plus tard, bien après sa cible.
    std::thread::sleep(periode * 10);
    crate::perf::frame_begin();
    app.attendre_l_heure_de_soumettre();
    app.enregistrer_l_image(1_000, (800, 600));
    let (ratees, _) = app.chronique.ratees().expect("une image ratee");
    assert_eq!(ratees.compte(), 1);
}

/// Une présentation qui rend l'issue qu'on lui dit.
struct Rend(crate::present::Issue);

impl crate::present::Presenter for Rend {
    fn resize(
        &mut self,
        _: std::num::NonZeroU32,
        _: std::num::NonZeroU32,
    ) -> crate::error::DesktopResult<()> {
        Ok(())
    }
    fn present(
        &mut self,
        _: &tiny_skia::Pixmap,
    ) -> crate::error::DesktopResult<crate::present::Issue> {
        Ok(self.0)
    }
    fn nom(&self) -> &'static str {
        "rend"
    }
    fn rythme(&self) -> &'static str {
        "aucun"
    }
}

/// Une sonde qui date ses envois et en rend les verdicts quand on les lui demande.
#[derive(Default, Clone)]
struct Compte(std::sync::Arc<std::sync::Mutex<(Vec<u64>, Vec<crate::chronique::ecran::Retour>)>>);

impl crate::chronique::ecran::Sonde for Compte {
    fn envoyer(&self) -> u64 {
        let mut c = self.0.lock().unwrap();
        let us = 1_000_000 + 16_667 * c.0.len() as u64;
        c.0.push(us);
        us
    }
    fn relever(&self) -> Vec<crate::chronique::ecran::Retour> {
        std::mem::take(&mut self.0.lock().unwrap().1)
    }
}

/// **PARU-1, par le vrai chemin de la présentation** : chaque image confiée est demandée au
/// compositeur avant de partir, et une image que la surface refuse ne compte pas — sa demande
/// vaudra pour la suivante, qui ne retrouvera pas son envoi.
#[test]
fn test_paru_1_chaque_image_confiee_est_demandee_au_compositeur() {
    use crate::chronique::ecran::{Retour, Vue};
    use crate::present::Issue;
    let sonde = Compte::default();
    let mut app = GlucoseApp::new();
    app.pixmap = tiny_skia::Pixmap::new(8, 8);
    app.chronique.ecran.brancher(Some(Box::new(sonde.clone())));
    for issue in [
        Issue::Presentee,
        Issue::Presentee,
        Issue::Cachee,
        Issue::Perdue,
    ] {
        app.presenter = Some(Box::new(Rend(issue)));
        app.presenter_et_noter_le_rythme(std::time::Instant::now(), None);
    }
    let envois = sonde.0.lock().unwrap().0.clone();
    assert_eq!(envois.len(), 4, "quatre images confiées, quatre demandes");
    // Le compositeur répond pour toutes : les refusées ne comptent pas.
    for (i, envoi_us) in envois.into_iter().enumerate() {
        sonde.0.lock().unwrap().1.push(Retour {
            envoi_us,
            vue: Some(Vue {
                us: 2_000_000 + 16_667 * i as u64,
                battement: 100 + i as u64,
                periode_ns: 16_666_667,
            }),
        });
    }
    app.presenter = Some(Box::new(Rend(Issue::Presentee)));
    app.presenter_et_noter_le_rythme(std::time::Instant::now(), None);
    // Et le rapport de la chronique le dit.
    let t = app.chronique.rapport();
    assert!(t.contains("2 image(s) confiee(s) : 2 montree(s)"), "{t}");
}
