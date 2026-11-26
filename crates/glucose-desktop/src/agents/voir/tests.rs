//! MCP-VOIR — une IA voit une photo du tableau (fiche 179).

use super::*;
use crate::agents::protocole::Appel;
use crate::agents::{Boite, Demande};
use crate::app::agents::Agents;
use crate::app::GlucoseApp;
use crate::interactions::question::tests::repondre;
use crate::persist::objets::Source;
use crate::ui::question::Reponse;
use std::sync::mpsc::channel;
use std::time::Duration;

/// Ce qu'une image de `(l, h)` coûte à un modèle de Claude.
fn jetons((l, h): (u32, u32)) -> u32 {
    l.div_ceil(PATCH) * h.div_ceil(PATCH)
}

/// Une photo unie de `(l, h)` pixels en PNG, d'opacité `alpha`.
fn png((l, h): (u32, u32), alpha: u8) -> Vec<u8> {
    let mut p = Pixmap::new(l, h).expect("une photo");
    p.fill(tiny_skia::Color::from_rgba8(200, 40, 40, alpha));
    p.encode_png().expect("encodée")
}

/// Une photo opaque, rouge à gauche, bleue à droite.
fn deux_moities((l, h): (u32, u32)) -> Vec<u8> {
    let mut p = Pixmap::new(l, h).expect("une photo");
    p.fill(tiny_skia::Color::from_rgba8(200, 40, 40, 255));
    let droite = tiny_skia::Rect::from_xywh((l / 2) as f32, 0.0, (l / 2) as f32, h as f32);
    let mut bleu = tiny_skia::Paint::default();
    bleu.set_color_rgba8(40, 40, 200, 255);
    p.fill_rect(
        droite.expect("un rectangle"),
        &bleu,
        Transform::identity(),
        None,
    );
    p.encode_png().expect("encodée")
}

fn objets_avec(cle: &str, octets: Vec<u8>) -> Arc<Objets> {
    let objets = Objets::nouveau();
    objets.poser(cle, Source::Memoire(Arc::new(octets)));
    objets
}

/// **La taille que Claude lit sans la réduire** : l'exemple de sa documentation (1920×1080 devient
/// 1456×819), et pour toute forme, au plus 1 568 px et 1 568 jetons, un pixel de plus les dépassant.
#[test]
fn mcp_voir_la_taille_que_claude_lit_sans_la_reduire() {
    assert_eq!(taille_pour((1920, 1080), None), (1456, 819));
    for native in [
        (4000, 3000),
        (3000, 4000),
        (1080, 2400),
        (5000, 500),
        (1568, 1568),
    ] {
        let (l, h) = taille_pour(native, None);
        assert!(
            l.max(h) <= COTE && jetons((l, h)) <= JETONS,
            "{native:?} → {l}×{h}"
        );
        let un_de_plus = taille_pour(native, Some(l.max(h) + 1));
        let k = |(a, b): (u32, u32)| f64::from(a) / f64::from(b);
        assert!(
            (k((l, h)) - k(native)).abs() < 0.02,
            "{native:?} : ses proportions"
        );
        assert!(
            l.max(h) == COTE || jetons(un_de_plus) > JETONS,
            "{native:?} → {l}×{h} : elle pouvait être plus grande"
        );
    }
}

/// **Jamais agrandie, et « cote » obéi** : une petite photo reste la sienne ; un côté demandé se
/// tient, sans dépasser la photo ni ce que lit le modèle le plus fin.
#[test]
fn mcp_voir_jamais_agrandie_et_la_cote_obeie() {
    assert_eq!(taille_pour((300, 200), None), (300, 200));
    assert_eq!(taille_pour((1920, 1080), Some(480)), (480, 270));
    assert_eq!(taille_pour((300, 200), Some(900)), (300, 200));
    assert_eq!(
        taille_pour((8000, 4000), Some(9000)),
        (COTE_FIN, COTE_FIN / 2)
    );
    assert_eq!(taille_pour((0, 0), None), (1, 1));
}

/// **Ce qu'elle voit est la photo** : une JPEG de la taille dite, ses couleurs gardées ; une photo
/// qui a de la transparence part en PNG ; une photo absente le dit.
#[test]
fn mcp_voir_l_image_envoyee_est_la_photo() {
    let objets = objets_avec("collee:grande", deux_moities((3000, 2000)));
    let (dit, octets, sorte) = image_pour_une_ia("collee:grande", &objets, None).expect("vue");
    assert_eq!(sorte, "image/jpeg");
    let n = |cle| dit.get(cle).and_then(Json::as_f64).expect(cle) as u32;
    let lue = image::load_from_memory(&octets)
        .expect("une JPEG")
        .to_rgb8();
    assert_eq!((lue.width(), lue.height()), (n("largeur"), n("hauteur")));
    assert_eq!(
        (n("largeur"), n("hauteur")),
        taille_pour((3000, 2000), None)
    );
    assert!(dit
        .get("native")
        .is_some_and(|v| v.ecrire() == "[3000,2000]"));
    // Toute la photo, réduite : la frontière de ses deux moitiés reste au milieu.
    let milieu = lue.width() / 2;
    let [r, g, b] = lue.get_pixel(milieu - 30, 10).0;
    assert!(r > 180 && g < 70 && b < 70, "à gauche, rouge : {r} {g} {b}");
    let [r, g, b] = lue.get_pixel(milieu + 30, 10).0;
    assert!(r < 70 && g < 70 && b > 180, "à droite, bleu : {r} {g} {b}");

    let objets = objets_avec("collee:voile", png((40, 30), 128));
    let (_, octets, sorte) = image_pour_une_ia("collee:voile", &objets, None).expect("vue");
    assert_eq!(sorte, "image/png");
    let lue = image::load_from_memory(&octets)
        .expect("une PNG")
        .to_rgba8();
    assert_eq!(lue.get_pixel(5, 5).0[3], 128, "sa transparence gardée");

    assert!(image_pour_une_ia("collee:absente", &objets, None).is_err());
}

/// **Par la porte, comme un agent** : l'appel passe par la boîte et son accord, la photo se lit
/// sur un fil, et la réponse porte ce qu'elle est puis l'image elle-même.
#[test]
fn mcp_voir_un_agent_voit_la_photo_par_la_porte() {
    let mut app = GlucoseApp::new();
    let boite = Boite::nouvelle(Arc::new(|| {}));
    app.lancement.agents = Agents::avec_la_boite(boite.clone());
    let b = app.store.active_board_id().to_string();
    let mut img = glucose_core::types::BoardImage::new("capture", 0.0, 0.0, 400.0, 225.0);
    img.src = Some("collee:capture".into());
    app.store.add_image(&b, img);
    let octets = Arc::new(png((1920, 1080), 255));
    app.disque
        .objets
        .poser("collee:capture", Source::Memoire(octets));

    let demander = |boite: &Boite, arguments: &str| {
        let (envoi, recue) = channel();
        boite.deposer(Demande {
            appel: Appel {
                id: Json::Nombre(1.0),
                nom: "glucose_voir".into(),
                arguments: Json::lire(arguments).expect("JSON"),
                client: "proteo".into(),
                moderne: true,
                portee: Default::default(),
            },
            reponse: envoi,
        });
        recue
    };
    let recue = demander(&boite, r#"{"id":"capture"}"#);
    app.suivre_les_agents();
    repondre(&mut app, Reponse::Oui);
    let ligne = recue
        .recv_timeout(Duration::from_secs(60))
        .expect("le fil répond");
    let r = Json::lire(&ligne).expect("JSON");
    let contenu = (r.get("result").and_then(|r| r.get("content")))
        .and_then(Json::as_liste)
        .expect("son contenu");
    let texte = contenu[0]
        .get("text")
        .and_then(Json::as_str)
        .expect("texte");
    let dit = Json::lire(texte).expect("ce qu'elle est");
    assert_eq!(dit.get("id").and_then(Json::as_str), Some("capture"));
    let image = &contenu[1];
    assert_eq!(image.get("type").and_then(Json::as_str), Some("image"));
    assert_eq!(
        image.get("mimeType").and_then(Json::as_str),
        Some("image/jpeg")
    );
    let data = image
        .get("data")
        .and_then(Json::as_str)
        .expect("ses octets");
    let jpeg = glucose_core::encodage::base64(data).expect("du base64");
    let lue = image::load_from_memory(&jpeg).expect("une JPEG");
    assert_eq!((lue.width(), lue.height()), (1456, 819));

    // Un nœud qui n'est pas une photo : l'agent l'apprend, sans image.
    let recue = demander(&boite, r#"{"id":"rien"}"#);
    app.suivre_les_agents();
    let ligne = recue.recv_timeout(Duration::from_secs(5)).expect("répondu");
    assert!(ligne.contains(r#""isError":true"#) && ligne.contains("pas une photo"));
}
