//! **COLLER-3 — une image collée entre dans l'histoire avec ses octets, sans aucun fichier.**
//!
//! Son PNG passait par le dossier temporaire du système, que le scribe relisait : deux fils se
//! passaient un fichier (COLLER-2), et ces fichiers restaient pour toujours là où Windows fait
//! son ménage. Ses octets sont désormais **promis** par l'atelier et attendus par le scribe, à
//! leur place dans sa file.

use super::tests::{application, dossier, image_suivante};
use crate::app::GlucoseApp;
use glucose_core::persist::histoire;

/// Des pixels comme le presse-papiers les donne : huit sur six, chacun différent.
fn pixels() -> crate::interactions::presse_papiers::Pixels {
    let rgba: Vec<u8> = (0..8u32 * 6)
        .flat_map(|i| [(i * 5) as u8, (i * 3) as u8, 200, 255])
        .collect();
    crate::interactions::presse_papiers::Pixels {
        width: 8,
        height: 6,
        bytes: rgba,
    }
}

/// Colle ces pixels au centre, et rend la clé de l'image posée.
fn coller(app: &mut GlucoseApp) -> String {
    let b = app.store.project.active_board_id.clone();
    app.coller_image(&b, &pixels(), (0.0, 0.0));
    app.store.project.boards[0]
        .images
        .last()
        .and_then(|i| i.src.clone())
        .expect("une image collée")
}

/// **Scellée dès l'image suivante, avant le geste qui la pose** — quel que soit le temps que
/// l'atelier met à l'encoder : le scribe l'attend. Dans le fichier, ses octets précèdent le
/// geste ; et sa clé ne désigne aucun fichier.
#[test]
fn test_une_image_collee_se_scelle_avant_le_geste_qui_la_pose() {
    let d = dossier("coller-3-avant");
    let chemin = d.join("colle.glucose");
    let mut app = application(&d);
    app.save_to(chemin.clone());
    let cle = coller(&mut app);
    assert!(cle.starts_with("collee:"), "un nom, pas un chemin : {cle}");
    image_suivante(&mut app);
    assert!(
        app.disque.objets.est_scellee(&cle),
        "le scribe a attendu la promesse"
    );
    let octets = app.disque.objets.lire(&cle).expect("ses octets");
    let relue = image::load_from_memory(&octets).expect("un PNG").to_rgba8();
    assert_eq!(relue.as_raw(), &pixels().bytes, "les pixels collés");
    assert!(app.fermer_le_document());

    let o = histoire::ouvrir(&mut std::fs::File::open(&chemin).unwrap()).unwrap();
    let empreinte = o.liens.get(&cle).expect("son lien");
    let objet = o.objets.get(empreinte).expect("son objet");
    let geste = o.gestes.first().expect("le geste qui la pose");
    assert!(
        objet.offset < geste.tranche.offset,
        "ses octets ({}) précèdent le geste ({}) : un arrêt entre les deux laisse un document \
         sans elle, jamais une image sans octets",
        objet.offset,
        geste.tranche.offset
    );
}

/// **Une promesse abandonnée ne retient pas le scribe** : l'atelier qui disparaît sans l'avoir
/// tenue la rend, le scribe le dit, et la suite s'écrit.
#[test]
fn test_une_promesse_abandonnee_ne_retient_pas_le_scribe() {
    use crate::persist::objets::{Promesse, Source};
    let d = dossier("coller-3-abandon");
    let mut app = application(&d);
    app.save_to(d.join("abandon.glucose"));
    let (promesse, parole) = Promesse::nouvelle();
    app.disque
        .objets
        .poser("collee:perdue", Source::Promise(promesse));
    drop(parole);
    let b = app.store.project.active_board_id.clone();
    let mut img = glucose_core::types::BoardImage::new("i", 0.0, 0.0, 8.0, 6.0);
    img.src = Some("collee:perdue".into());
    app.store.add_image(&b, img);
    image_suivante(&mut app);
    assert!(!app.disque.objets.est_scellee("collee:perdue"));
    // L'échec se dit à l'écran : le toast l'a dit à la fin de cette image si le scribe allait
    // plus vite que le fil qui dessine, ou le dit à la suivante. Lire l'erreur au scribe, comme
    // le faisait cette épreuve, c'était lire ce que l'application avait peut-être déjà pris
    // (tombée 2 fois sur 40 ici, et sur la machine de GitHub).
    app.consigner();
    let dit = app.ui.toast_message();
    assert!(
        dit.is_some_and(|m| m.contains("n'a pas abouti")),
        "l'échec se dit : {dit:?}"
    );
}

/// **Copier une image collée, puis la coller, la duplique** : le lot la porte, et le collage dans
/// le même document lui garde sa clé — il n'existe aucun fichier à relire, et ses octets ne
/// sont pas recopiés (fiche 51 § 2).
#[test]
fn test_copier_coller_une_image_collee_la_duplique() {
    let d = dossier("coller-3-copier");
    let mut app = application(&d);
    app.save_to(d.join("copie.glucose"));
    let cle = coller(&mut app);
    image_suivante(&mut app);
    let id = app.store.project.boards[0].images[0].id.clone();
    let cartes = app.store.project.boards[0].annotations.len();
    app.store.set_selected_image_ids(vec![id]);
    app.copy_selection(false);
    app.suivre_les_echanges(true);
    app.paste_from_clipboard();
    app.suivre_les_echanges(true);
    let images = &app.store.project.boards[0].images;
    assert_eq!(images.len(), 2, "une image de plus");
    assert_eq!(images[1].src.as_deref(), Some(cle.as_str()));
    assert_ne!(images[1].id, images[0].id);
    assert_eq!(
        app.store.project.boards[0].annotations.len(),
        cartes,
        "et pas une carte qui porterait sa clé"
    );
}
