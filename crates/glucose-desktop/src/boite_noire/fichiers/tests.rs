//! Les fichiers du journal technique, lisibles (fiche 58) — dans des dossiers d'épreuve, jamais
//! celui de l'utilisateur.

use super::*;
use std::collections::BTreeSet;

/// Un dossier d'épreuve, effacé à la fin.
struct Dossier(PathBuf);

impl Dossier {
    fn nouveau(nom: &str) -> Self {
        let p = std::env::temp_dir().join(format!("glucose-journal-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).expect("dossier d'épreuve");
        Self(p)
    }
}

impl Drop for Dossier {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// **Un nom se lit** : une date, une heure, le processus, et `.txt` — et il se relit.
#[test]
fn test_un_nom_de_session_se_lit_et_se_relit() {
    let nom = nom_de_session(1_791_477_194_257, 16_656);
    assert!(nom.ends_with(" (16656).txt"), "{nom}");
    assert!(nom.starts_with("2026-10-0"), "{nom}");
    assert!(nom.contains("m14"), "les secondes : {nom}");
    assert_eq!(processus_de(&nom), Some(16_656));
    for autre in [
        LISEZ_MOI,
        "carte.txt",
        "session-00000001791477194257-16656.jsonl",
        "notes (3).txt",
    ] {
        assert!(!est_une_session(autre), "{autre}");
    }
}

/// **La date universelle est exacte**, bissextiles comprises — confrontée à celle de Python.
#[test]
fn test_la_date_universelle_est_exacte() {
    let jour = |ms: u64| date_universelle(ms / 86_400_000);
    assert_eq!(jour(0), (1970, 1, 1));
    assert_eq!(jour(1_791_477_194_257), (2026, 10, 8));
    assert_eq!(jour(951_782_400_000), (2000, 2, 29));
    assert_eq!(jour(4_102_444_800_000), (2100, 1, 1));
}

/// **L'ancien dossier se range dans le nouveau, sans rien perdre** : son nom devient lisible,
/// ses octets ne changent pas, et sa clé — d'où le serveur tire l'identifiant — reste l'ancien
/// nom : une session déjà partie ne repart jamais. Le lisez-moi est posé.
#[test]
fn test_l_ancien_dossier_se_range_sans_rien_perdre() {
    let d = Dossier::nouveau("migrer");
    let ancien = d.0.join("boite-noire");
    std::fs::create_dir_all(&ancien).unwrap();
    let ancien_nom = "session-00000001791477194257-16656.jsonl";
    let texte =
        "{\"type\":\"debut\",\"epoque_ms\":1791477194257}\n{\"type\":\"fin\",\"instant_ms\":9}\n";
    std::fs::write(ancien.join(ancien_nom), texte).unwrap();
    std::fs::write(ancien.join("autre.txt"), "à garder").unwrap();

    let journal = preparer(&d.0).expect("préparé");
    assert_eq!(journal, d.0.join(DOSSIER));
    assert!(!ancien.exists(), "l'ancien dossier est parti");
    let lisible = journal.join(nom_de_session(1_791_477_194_257, 16_656));
    assert_eq!(
        std::fs::read_to_string(&lisible).unwrap(),
        texte,
        "les octets"
    );
    assert_eq!(
        std::fs::read_to_string(journal.join("autre.txt")).unwrap(),
        "à garder"
    );
    let nom = lisible.file_name().unwrap().to_string_lossy().into_owned();
    let debut = debut_du_fichier(&lisible).expect("son début");
    assert_eq!(cle(debut, processus_de(&nom).unwrap()), ancien_nom);
    assert_eq!(
        std::fs::read_to_string(journal.join(LISEZ_MOI)).unwrap(),
        TEXTE_DU_LISEZ_MOI
    );
}

/// **Quand les deux dossiers existent**, l'ancien se vide dans le nouveau, et rien ne s'écrase :
/// un nom déjà pris laisse l'ancien fichier où il est, et l'ancien dossier avec.
#[test]
fn test_les_deux_dossiers_se_rejoignent_sans_rien_ecraser() {
    let d = Dossier::nouveau("rejoindre");
    let (ancien, journal) = (d.0.join("boite-noire"), d.0.join(DOSSIER));
    std::fs::create_dir_all(&ancien).unwrap();
    std::fs::create_dir_all(&journal).unwrap();
    std::fs::write(ancien.join("a.txt"), "ancien a").unwrap();
    std::fs::write(ancien.join("b.txt"), "ancien b").unwrap();
    std::fs::write(journal.join("b.txt"), "nouveau b").unwrap();
    preparer(&d.0).expect("préparé");
    assert_eq!(
        std::fs::read_to_string(journal.join("a.txt")).unwrap(),
        "ancien a"
    );
    assert_eq!(
        std::fs::read_to_string(journal.join("b.txt")).unwrap(),
        "nouveau b"
    );
    assert_eq!(
        std::fs::read_to_string(ancien.join("b.txt")).unwrap(),
        "ancien b"
    );
}

/// Une ligne de chaque sorte que le journal sait écrire.
fn une_ligne_de_chaque_sorte() -> Vec<String> {
    use crate::boite_noire::enregistrement::{ligne_de_panique, ligne_de_plantage, Enregistrement};
    let tous = [
        Enregistrement::Debut {
            epoque_ms: 1,
            version: "2.0.2",
            systeme: "windows",
            architecture: "x86_64",
            demarrage_appareil_ms: Some(0),
        },
        Enregistrement::Precedente {
            instant_ms: 0,
            fin: "propre",
            appareil_redemarre: false,
            duree_ms: 1,
            batterie_pct: Some(1),
            en_charge: Some(true),
        },
        Enregistrement::Episode {
            instant_ms: 0,
            duree_ms: 1,
            geste: "zoomer",
            images: 1,
            median_us: 1,
            p99_us: 1,
            pire_us: 1,
        },
        Enregistrement::Machine {
            instant_ms: 0,
            batterie_pct: Some(1),
            en_charge: Some(false),
        },
        Enregistrement::Pire {
            instant_ms: 0,
            duree_us: 1,
            geste: "zoomer",
        },
        Enregistrement::Pave {
            instant_ms: 0,
            quoi: "contact",
            valeur: 0,
        },
        Enregistrement::Fin { instant_ms: 0 },
    ];
    let plantage = crate::boite_noire::plantage::Plantage {
        gel: false,
        module: Some("m.dll".into()),
        version_du_module: Some("1.0".into()),
        code: Some(1),
        decalage: Some(1),
    };
    let mut lignes: Vec<String> = tous.iter().map(Enregistrement::ligne).collect();
    lignes.push(ligne_de_panique(0, "src/ui.rs", 1, true));
    lignes.push(ligne_de_plantage(0, &plantage));
    lignes
}

/// **Le lisez-moi n'oublie rien** : chaque sorte de ligne a sa section, et chaque champ de
/// chacune sa rangée — un champ ajouté demain sans le dire ferait tomber cette épreuve.
#[test]
fn test_le_lisez_moi_dit_chaque_champ_de_chaque_ligne() {
    use glucose_core::persist::tauri::json;
    let sections: Vec<&str> = TEXTE_DU_LISEZ_MOI.split("\ntype « ").skip(1).collect();
    let mut sortes = BTreeSet::new();
    for ligne in une_ligne_de_chaque_sorte() {
        let v = json::lire(ligne.trim()).expect("du JSON");
        let sorte = v.texte("type").expect("un type").to_string();
        let section = sections
            .iter()
            .find(|s| s.starts_with(&format!("{sorte} »")))
            .unwrap_or_else(|| panic!("le lisez-moi n'a pas de section « {sorte} »"));
        let glucose_core::persist::tauri::Valeur::Carte(champs) = &v else {
            panic!("une ligne est un objet");
        };
        for champ in champs.keys().filter(|c| *c != "type") {
            assert!(
                section
                    .lines()
                    .any(|l| l.starts_with(&format!("  {champ} "))),
                "« {sorte} » : le champ {champ} n'est pas dit"
            );
        }
        sortes.insert(sorte);
    }
    assert_eq!(sortes.len(), 9, "les neuf sortes de lignes : {sortes:?}");
    assert_eq!(sections.len(), 9, "et pas une section de plus");
}

/// **Les répétitions de la bascule plantent leur témoin dans le dossier de Glucose** : le
/// renommage l'avait laissé dans l'ancien, que la migration rangeait — la CI l'a vu tomber.
#[test]
fn test_les_repetitions_plantent_dans_le_dossier_du_journal() {
    let windows = include_str!("../../../../../outils/installeur/commun.ps1");
    let linux = include_str!("../../../../../outils/paquets/une_forme.sh");
    for (nom, script) in [("commun.ps1", windows), ("une_forme.sh", linux)] {
        assert!(
            script.contains(DOSSIER),
            "{nom} ne plante rien dans {DOSSIER}"
        );
        assert!(
            !script.contains(ANCIEN_DOSSIER),
            "{nom} plante encore dans l'ancien dossier"
        );
    }
}
