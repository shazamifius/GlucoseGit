//! Ce que le magasin promet : ne jamais bloquer, ne jamais redemander en vain, et rendre à la
//! machine ce qu'elle réclame sans jamais lâcher ce qui est à l'écran.

use super::*;
use tiny_skia::Pixmap;

fn photo_temoin(nom: &str) -> String {
    let dossier = std::env::temp_dir().join("glucose-magasin-tests");
    std::fs::create_dir_all(&dossier).expect("dossier de test");
    let chemin = dossier.join(nom);
    if !chemin.exists() {
        let mut brute = image::RgbaImage::new(48, 32);
        for (x, _, px) in brute.enumerate_pixels_mut() {
            *px = image::Rgba([(x * 5 % 256) as u8, 90, 160, 255]);
        }
        brute.save(&chemin).expect("écriture de la photo témoin");
    }
    chemin.to_string_lossy().to_string()
}

/// Fait tourner le magasin comme le rendu le fait — récolter, puis réclamer — jusqu'à ce que
/// l'image soit là, sans jamais boucler indéfiniment.
fn attendre(magasin: &mut Magasin, src: &str) -> bool {
    let depart = std::time::Instant::now();
    while depart.elapsed() < std::time::Duration::from_secs(30) {
        magasin.recolter();
        if magasin.pyramide(src).is_some() {
            return true;
        }
        std::thread::yield_now();
    }
    false
}

/// Pose une entrée directement dans le cache, sans passer par le disque.
///
/// Les tests d'éviction portent sur la **règle**, pas sur le décodage : la fabriquer ici les
/// rend instantanés et indépendants de ce qu'une machine met à lire un fichier.
fn poser_entree(magasin: &mut Magasin, nom: &str, cote: u32, cout_ms: u64, vue: u64) {
    let pixmap = Pixmap::new(cote, cote).expect("une image");
    magasin.cache.insert(
        nom.to_string(),
        Entree {
            pyramide: Pyramide::nouvelle(pixmap),
            cout: Duration::from_millis(cout_ms),
            vue,
        },
    );
}

#[test]
fn test_la_premiere_demande_ne_bloque_pas_et_rend_rien() {
    let src = photo_temoin("magasin-premiere.png");
    let mut magasin = Magasin::nouveau();

    // C'est tout l'invariant DECODE-1 : la première demande revient immédiatement, les mains
    // vides. Une mesure de durée serait fragile ; ce qui se vérifie, c'est qu'aucune image
    // n'est rendue alors qu'aucun décodage n'a encore pu aboutir.
    assert!(
        magasin.pyramide(&src).is_none(),
        "le rendu n'attend pas le décodage"
    );
    assert_eq!(magasin.en_travail(), 1, "mais la demande est bien partie");
}

#[test]
fn test_l_image_finit_par_arriver_et_reste() {
    let src = photo_temoin("magasin-arrivee.png");
    let mut magasin = Magasin::nouveau();

    assert!(attendre(&mut magasin, &src), "l'image doit finir par arriver");
    assert_eq!(magasin.en_travail(), 0, "le chantier est vide");

    let pyramide = magasin.pyramide(&src).expect("elle est dans le cache");
    assert_eq!(
        (pyramide.native().width(), pyramide.native().height()),
        (48, 32)
    );
    assert!(magasin.octets() > 0, "elle occupe de la place, et on le sait");
}

#[test]
fn test_un_fichier_illisible_ne_se_redemande_jamais() {
    let absent = std::env::temp_dir()
        .join("glucose-magasin-tests")
        .join("rien-ici.png")
        .to_string_lossy()
        .to_string();
    let mut magasin = Magasin::nouveau();

    assert!(magasin.pyramide(&absent).is_none());
    magasin.attendre_le_chantier();

    assert!(magasin.echecs.contains(&absent), "l'échec est retenu");
    // R-29 : sans le cache négatif, chaque image redemanderait ce fichier, et l'atelier
    // passerait sa vie à rouvrir un fichier qui n'existe pas.
    assert!(magasin.pyramide(&absent).is_none());
    assert_eq!(magasin.en_travail(), 0, "aucune nouvelle demande n'est partie");
}

#[test]
fn test_ce_qui_est_a_l_ecran_ne_s_evince_jamais() {
    // La propriété qui empêche le cache de battre : évincer une image qu'on vient de dessiner
    // obligerait à la redemander à l'image suivante, indéfiniment.
    let mut magasin = Magasin::nouveau();
    magasin.ouvrir();
    let a_l_ecran = magasin.image;

    poser_entree(&mut magasin, "visible", 256, 1, a_l_ecran);
    poser_entree(&mut magasin, "oubliee", 256, 1, a_l_ecran - 1);

    // Une borne de zéro : tout ce qui peut partir doit partir. Ne restera que ce qui est
    // protégé — et rien d'autre ne le protège que d'avoir été vu à cette image.
    magasin.ramener_sous(0);

    assert!(
        magasin.cache.contains_key("visible"),
        "une image dessinée à cette passe ne se rend pas"
    );
    assert!(
        !magasin.cache.contains_key("oubliee"),
        "une image qui n'est plus à l'écran se rend"
    );
}

#[test]
fn test_on_rend_d_abord_ce_qui_coute_le_moins_a_refaire() {
    // Pas « le plus ancien » : ce que sa reconstruction coûterait, rapporté à ce qu'il occupe.
    // Les deux termes sont mesurés — l'un par l'atelier, l'autre par la pyramide.
    let mut magasin = Magasin::nouveau();
    magasin.ouvrir();
    let ancienne = magasin.image - 1;

    // Même taille, donc seule l'utilité les distingue.
    poser_entree(&mut magasin, "chere", 256, 300, ancienne);
    poser_entree(&mut magasin, "bon-marche", 256, 1, ancienne);

    let une = Pyramide::nouvelle(Pixmap::new(256, 256).expect("une image")).octets() as u64;
    // Une borne qui ne laisse la place qu'à une seule des deux.
    magasin.ramener_sous(une);

    assert!(
        magasin.cache.contains_key("chere"),
        "on garde ce qui coûterait cher à refaire"
    );
    assert!(
        !magasin.cache.contains_key("bon-marche"),
        "on rend ce qui se refait pour rien"
    );
    assert_eq!(magasin.evincees(), 1);
}

#[test]
fn test_le_cache_ne_rend_rien_tant_qu_il_tient_dans_la_borne() {
    let mut magasin = Magasin::nouveau();
    magasin.ouvrir();
    let ancienne = magasin.image - 1;
    poser_entree(&mut magasin, "a", 64, 5, ancienne);
    poser_entree(&mut magasin, "b", 64, 5, ancienne);

    // Une borne large : rien ne doit bouger. Un cache qui évince alors qu'il a la place
    // paierait des décodages pour rien.
    magasin.ramener_sous(u64::MAX);

    assert_eq!(magasin.cache.len(), 2);
    assert_eq!(magasin.evincees(), 0);
}

#[test]
fn test_l_eviction_s_arrete_des_que_la_borne_est_tenue() {
    // Une éviction qui ne s'arrête pas viderait le cache à la première tension mémoire, et
    // tout serait à redécoder — c'est le battement qu'on veut éviter.
    let mut magasin = Magasin::nouveau();
    magasin.ouvrir();
    let ancienne = magasin.image - 1;
    for i in 0..8 {
        poser_entree(&mut magasin, &format!("img-{i}"), 128, 10, ancienne);
    }
    let une = Pyramide::nouvelle(Pixmap::new(128, 128).expect("une image")).octets() as u64;

    magasin.ramener_sous(une * 5);

    assert!(magasin.octets() as u64 <= une * 5, "la borne est tenue");
    assert!(
        magasin.cache.len() >= 5,
        "mais on n'a pas rendu plus que nécessaire : {} restantes",
        magasin.cache.len()
    );
}
