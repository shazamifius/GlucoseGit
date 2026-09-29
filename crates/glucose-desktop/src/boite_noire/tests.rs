//! La boîte noire, éprouvée dans un dossier à elle — jamais dans celui de l'utilisateur.

use super::bilan::{self, Fin};
use super::enregistrement::{ligne_de_panique, Enregistrement};
use super::{sondes, BoiteNoire, DOSSIER};
use crate::chronique::Geste;
use glucose_core::persist::tauri::{json, Valeur};
use std::path::{Path, PathBuf};

/// Un dossier d'épreuve, effacé à la fin.
struct Dossier(PathBuf);

impl Dossier {
    fn nouveau(nom: &str) -> Self {
        let p =
            std::env::temp_dir().join(format!("glucose-boite-noire-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).expect("dossier d'épreuve");
        Self(p)
    }

    fn boite(&self) -> PathBuf {
        self.0.join(DOSSIER)
    }
}

impl Drop for Dossier {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn lire(p: &Path) -> String {
    std::fs::read_to_string(p).expect("session lisible")
}

fn lignes_du_type(texte: &str, nom: &str) -> Vec<Valeur> {
    texte
        .lines()
        .filter_map(|l| json::lire(l).ok())
        .filter(|v| v.texte("type") == Some(nom))
        .collect()
}

/// **Une session fermée se relit propre**, au lancement suivant, avec ses épisodes.
#[test]
fn test_une_session_fermee_se_relit_propre() {
    let d = Dossier::nouveau("propre");
    let (mut b, precedente) = BoiteNoire::ouvrir(&d.0).expect("ouverte");
    assert_eq!(precedente, None, "rien avant la première");
    for us in [1000, 3000, 2000] {
        b.image(Geste::Zoomer, us);
    }
    b.image(Geste::DeplacerLaVue, 500);
    let premiere = b.chemin().to_path_buf();
    b.clore();

    let texte = lire(&premiere);
    let episodes = lignes_du_type(&texte, "episode");
    assert_eq!(episodes.len(), 2, "le zoom, puis le déplacement");
    assert_eq!(episodes[0].texte("geste"), Some("zoomer"));
    assert_eq!(episodes[0].entier("images"), Some(3));
    assert_eq!(episodes[0].entier("pire_us"), Some(3000));
    assert_eq!(lignes_du_type(&texte, "fin").len(), 1);

    let (seconde, precedente) = BoiteNoire::ouvrir(&d.0).expect("rouverte");
    let p = precedente.expect("la session d'avant");
    assert_eq!(p.fin, Fin::Propre);
    assert!(!p.appareil_redemarre);
    let texte = {
        seconde.synchroniser();
        lire(seconde.chemin())
    };
    let dite = lignes_du_type(&texte, "precedente");
    assert_eq!(dite.len(), 1, "le bilan voyage avec la session suivante");
    assert_eq!(dite[0].texte("fin"), Some("propre"));
}

/// **Ce qui est confié est sur le disque avant la fin** : une session copiée en plein vol — ce
/// qu'un plantage laisserait — se relit interrompue, avec ses épisodes clos.
#[test]
fn test_une_session_coupee_se_relit_interrompue() {
    let d = Dossier::nouveau("coupee");
    let (mut b, _) = BoiteNoire::ouvrir(&d.0).expect("ouverte");
    b.image(Geste::GlisserUnNoeud, 4000);
    b.image(Geste::Repos, 100);
    b.synchroniser();
    let en_vol = lire(b.chemin());
    assert_eq!(
        lignes_du_type(&en_vol, "episode").len(),
        1,
        "le glisser, clos"
    );
    let bilan = bilan::bilan(&en_vol, None).expect("une session");
    assert_eq!(bilan.fin, Fin::Interrompue);
    assert!(
        !bilan.appareil_redemarre,
        "sans démarrage connu, on ne dit rien"
    );
}

/// **L'appareil qui a redémarré se dit — et seulement lui** : la dernière trace avant le
/// démarrage présent, oui ; après, non ; une session propre, jamais.
#[test]
fn test_l_appareil_redemarre_se_dit_d_apres_la_derniere_trace() {
    let texte = concat!(
        "{\"type\":\"debut\",\"epoque_ms\":1000000,\"version\":\"x\",\"systeme\":\"x\",",
        "\"architecture\":\"x\",\"demarrage_appareil_ms\":900000}\n",
        "{\"type\":\"machine\",\"instant_ms\":5000,\"batterie_pct\":3,\"en_charge\":false}\n",
        "{\"type\":\"episode\",\"instant_ms\":6000,\"duree_ms\":4000,\"geste\":\"zoomer\",",
        "\"images\":9,\"median_us\":1,\"p99_us\":1,\"pire_us\":1}\n",
    );
    // La dernière trace : 1 000 000 + 6 000.
    let apres = bilan::bilan(texte, Some(1_006_001)).expect("session");
    assert!(apres.appareil_redemarre);
    assert_eq!(apres.machine.batterie_pct, Some(3));
    assert_eq!(apres.machine.en_charge, Some(false));
    assert!(
        apres.dire().contains("3 %, sur batterie"),
        "{}",
        apres.dire()
    );
    let avant = bilan::bilan(texte, Some(1_005_999)).expect("session");
    assert!(!avant.appareil_redemarre);
    // Fermée proprement, puis l'appareil éteint : ce n'est pas la session qui s'est arrêtée.
    let propre = format!("{texte}{{\"type\":\"fin\",\"instant_ms\":7000}}\n");
    let fermee = bilan::bilan(&propre, Some(1_007_001)).expect("session");
    assert_eq!(fermee.fin, Fin::Propre);
    assert!(!fermee.appareil_redemarre);
}

/// **Une ligne coupée par la fin brutale ne cache pas ce qui la précède**, et un fichier sans
/// début n'est pas une session.
#[test]
fn test_une_ligne_coupee_ne_cache_pas_la_fin() {
    let texte = "{\"type\":\"debut\",\"epoque_ms\":10}\n{\"type\":\"machine\",\"instant_ms\":7,\
                 \"batterie_pct\":40,\"en_charge\":true}\n{\"type\":\"episode\",\"insta";
    let b = bilan::bilan(texte, None).expect("session");
    assert_eq!(b.fin, Fin::Interrompue);
    assert_eq!(b.duree_ms, 7);
    assert_eq!(b.machine.batterie_pct, Some(40));
    assert_eq!(bilan::bilan("{\"type\":\"episode\"}\n", None), None);
}

/// **Seule une panique du fil principal finit la session** : un autre fil peut tomber, et
/// Glucose continuer jusqu'à sa fermeture.
#[test]
fn test_une_panique_d_un_autre_fil_ne_finit_pas_la_session() {
    let debut = "{\"type\":\"debut\",\"epoque_ms\":10}\n";
    let ailleurs = ligne_de_panique(5, "src/atelier.rs", 12, false);
    let fin = "{\"type\":\"fin\",\"instant_ms\":9}\n";
    let survecue = bilan::bilan(&format!("{debut}{ailleurs}{fin}"), None).expect("session");
    assert_eq!(survecue.fin, Fin::Propre);
    let muette = bilan::bilan(&format!("{debut}{ailleurs}"), None).expect("session");
    assert_eq!(
        muette.fin,
        Fin::Interrompue,
        "elle s'est arrêtée ailleurs que dans cette panique"
    );
    let principale = ligne_de_panique(5, "src/app.rs", 40, true);
    let tombee = bilan::bilan(&format!("{debut}{principale}"), None).expect("session");
    assert_eq!(tombee.fin, Fin::Panique);
}

/// **Le rangement garde la session d'avant et ce qui a mal fini**, et efface le reste : une
/// ancienne session propre, un fichier qui n'est pas une session.
#[test]
fn test_ranger_garde_la_precedente_et_ce_qui_a_mal_fini() {
    let d = Dossier::nouveau("ranger");
    std::fs::create_dir_all(d.boite()).expect("boîte");
    let ecrire = |nom: &str, texte: &str| {
        let p = d.boite().join(nom);
        std::fs::write(&p, texte).expect("écrite");
        p
    };
    let debut = "{\"type\":\"debut\",\"epoque_ms\":10}\n";
    let fin = "{\"type\":\"fin\",\"instant_ms\":9}\n";
    let ancienne_propre = ecrire(
        "session-00000000000000000001-1.jsonl",
        &format!("{debut}{fin}"),
    );
    let interrompue = ecrire("session-00000000000000000002-1.jsonl", debut);
    let muette = ecrire("session-00000000000000000003-1.jsonl", "rien\n");
    let derniere = ecrire(
        "session-00000000000000000004-1.jsonl",
        &format!("{debut}{fin}"),
    );
    let autre = ecrire("autre-chose.txt", "à ne pas toucher");

    let precedente = bilan::ranger(&d.boite(), None).expect("la dernière");
    assert_eq!(precedente.fin, Fin::Propre);
    assert!(
        !ancienne_propre.exists(),
        "racontée au lancement qui la suivait"
    );
    assert!(interrompue.exists(), "elle a quelque chose à apprendre");
    assert!(!muette.exists(), "elle ne dit rien");
    assert!(derniere.exists(), "c'est elle qu'on raconte");
    assert!(
        autre.exists(),
        "ce qui n'est pas une session ne se touche pas"
    );
}

/// **Chaque ligne est du JSON**, dont le `type` est le nom de l'enregistrement — la panique
/// comprise, même quand son chemin porte ce que JSON doit échapper.
#[test]
fn test_chaque_ligne_est_du_json() {
    let tous = [
        Enregistrement::Debut {
            epoque_ms: 1,
            version: "2.0.1",
            systeme: "windows",
            architecture: "x86_64",
            demarrage_appareil_ms: None,
        },
        Enregistrement::Precedente {
            instant_ms: 0,
            fin: "interrompue",
            appareil_redemarre: true,
            duree_ms: 3,
            batterie_pct: Some(3),
            en_charge: Some(false),
        },
        Enregistrement::Episode {
            instant_ms: 1,
            duree_ms: 2,
            geste: "glisser un noeud",
            images: 3,
            median_us: 4,
            p99_us: 5,
            pire_us: 6,
        },
        Enregistrement::Machine {
            instant_ms: 1,
            batterie_pct: None,
            en_charge: Some(true),
        },
        Enregistrement::Pire {
            instant_ms: 1,
            duree_us: 2,
            geste: "zoomer",
        },
        Enregistrement::Fin { instant_ms: 9 },
    ];
    for e in tous {
        let ligne = e.ligne();
        assert!(ligne.ends_with('\n') && ligne.matches('\n').count() == 1);
        let v = json::lire(ligne.trim_end()).expect("du JSON");
        assert_eq!(v.texte("type"), Some(e.nom()), "{ligne}");
    }
    let chemin = "crates\\glucose \"desktop\"\\src.rs";
    let panique = ligne_de_panique(1, chemin, 7, true);
    let v = json::lire(panique.trim_end()).expect("du JSON");
    assert_eq!(v.texte("fichier"), Some(chemin));
    assert_eq!(v.entier("ligne"), Some(7));
}

/// **Un record s'écrit quand l'image bat le précédent**, et seulement alors.
#[test]
fn test_seules_les_images_record_s_ecrivent_une_a_une() {
    let d = Dossier::nouveau("pires");
    let (mut b, _) = BoiteNoire::ouvrir(&d.0).expect("ouverte");
    for us in [100, 50, 300, 300, 200, 900] {
        b.image(Geste::Zoomer, us);
    }
    b.synchroniser();
    let pires: Vec<i64> = lignes_du_type(&lire(b.chemin()), "pire")
        .iter()
        .filter_map(|v| v.entier("duree_us"))
        .collect();
    assert_eq!(pires, vec![100, 300, 900]);
}

/// **L'état de la machine ne s'écrit que s'il a changé** : jamais deux fois le même de suite.
#[test]
fn test_la_machine_ne_s_ecrit_que_si_elle_change() {
    let d = Dossier::nouveau("machine");
    let (mut b, _) = BoiteNoire::ouvrir(&d.0).expect("ouverte");
    for geste in [Geste::Zoomer, Geste::Repos, Geste::Zoomer, Geste::Repos] {
        b.image(geste, 10);
        b.synchroniser();
    }
    let etats: Vec<(Option<i64>, Option<bool>)> = lignes_du_type(&lire(b.chemin()), "machine")
        .iter()
        .map(|v| (v.entier("batterie_pct"), v.booleen("en_charge")))
        .collect();
    assert!(!etats.is_empty(), "le premier réveil écrit l'état");
    assert!(
        etats.windows(2).all(|w| w[0] != w[1]),
        "jamais deux fois le même de suite : {etats:?}"
    );
}

/// **Les sondes répondent** : l'appareil a démarré avant maintenant, et une batterie dit un
/// pourcentage.
#[test]
fn test_les_sondes_repondent() {
    let etat = sondes::etat_de_la_machine();
    assert!(etat.batterie_pct.is_none_or(|p| p <= 100));
    let demarrage = sondes::demarrage_de_l_appareil_ms();
    #[cfg(any(windows, target_os = "linux"))]
    assert!(demarrage.is_some(), "Windows et Linux le disent");
    if let Some(d) = demarrage {
        assert!(d < sondes::maintenant_ms());
    }
}

/// **Le témoin note une vraie panique** : où, et sur quel fil — jamais son message.
#[test]
fn test_le_temoin_note_une_vraie_panique() {
    let d = Dossier::nouveau("temoin");
    let (b, _) = BoiteNoire::ouvrir(&d.0).expect("ouverte");
    b.temoigner_des_paniques();
    let _ = std::thread::spawn(|| panic!("secret de l'utilisateur")).join();
    b.synchroniser();
    let texte = lire(b.chemin());
    assert!(!texte.contains("secret"), "le message ne s'écrit jamais");
    let paniques = lignes_du_type(&texte, "panique");
    assert!(
        paniques.iter().any(
            |v| v.texte("fichier").is_some_and(|f| f.ends_with("tests.rs"))
                && v.booleen("principal") == Some(false)
        ),
        "{texte}"
    );
}
