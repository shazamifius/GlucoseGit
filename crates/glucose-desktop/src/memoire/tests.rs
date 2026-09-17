//! Ce que le module de mémoire promet, sur la machine qui fait tourner les tests.

use super::*;

/// Ce test ne vit que là où le module sait lire. Ailleurs, exiger une réponse serait exiger
/// qu'on devine — et c'est précisément ce que le module refuse de faire.
#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios"
))]
#[test]
fn test_la_machine_dit_sa_memoire() {
    let m = Memoire::du_systeme().expect("cette plateforme sait lire sa mémoire");

    // Un gigaoctet : aucune machine capable de compiler ce projet n'en a moins, et une valeur
    // au-dessous signalerait une unité confondue (kibioctets lus comme octets, par exemple).
    assert!(
        m.totale >= 1 << 30,
        "mémoire totale invraisemblable : {} octets",
        m.totale
    );
    assert!(
        m.disponible <= m.totale,
        "le disponible ({}) dépasse la totale ({})",
        m.disponible,
        m.totale
    );
    println!(
        "[memoire] totale {:.1} Go, disponible {:.1} Go, part d'un cache {:.1} Go",
        m.totale as f64 / 1e9,
        m.disponible as f64 / 1e9,
        m.part_pour_un_cache() as f64 / 1e9
    );
}

/// Ce que coûte de relire la mémoire — parce que le magasin le fait à **chaque image**, et
/// qu'une telle décision se prend sur une mesure.
#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios"
))]
#[test]
fn test_relire_la_memoire_est_assez_bon_marche_pour_chaque_image() {
    // Une lecture de chauffe : la première touche le fichier virtuel ou charge le symbole.
    let _ = Memoire::du_systeme();

    let tours = 1000;
    let depart = std::time::Instant::now();
    for _ in 0..tours {
        std::hint::black_box(Memoire::du_systeme());
    }
    let par_lecture = depart.elapsed() / tours;

    println!("[memoire] une lecture coûte {par_lecture:?}");
    // Le budget d'une image à 400 fps est de 2,5 ms. Une lecture qui prend plus d'un centième
    // de ce budget ne pourrait plus se faire à chaque image, et la borne devrait alors être
    // rafraîchie autrement — ce test est là pour qu'on l'apprenne par une mesure et non par
    // une lenteur inexpliquée.
    assert!(
        par_lecture < std::time::Duration::from_micros(25),
        "relire la mémoire coûte {par_lecture:?} : trop cher pour chaque image"
    );
}

#[test]
fn test_un_cache_ne_prend_jamais_plus_que_ce_qu_il_laisse() {
    // C'est l'énoncé de la règle, et il se vérifie sur des valeurs choisies plutôt que sur la
    // machine : le test dirait la même chose sur un téléphone et sur un serveur.
    for disponible in [0u64, 1, 2 << 30, 32 << 30, u64::MAX] {
        let m = Memoire {
            totale: disponible.max(1),
            disponible,
        };
        let part = m.part_pour_un_cache();
        assert!(
            part <= disponible - part,
            "sur {disponible} octets disponibles, le cache prend {part} et n'en laisse que {}",
            disponible - part
        );
    }
}

#[test]
fn test_la_borne_suit_la_machine_au_lieu_d_etre_choisie() {
    // Le défaut que ce test ferme : une borne écrite en dur, ridicule sur une grande machine
    // et mortelle sur une petite. Ici, un rapport de seize entre deux machines donne un
    // rapport de seize entre leurs bornes — la borne n'a aucune existence propre.
    let petite = Memoire {
        totale: 2 << 30,
        disponible: 1 << 30,
    };
    let grande = Memoire {
        totale: 32 << 30,
        disponible: 16 << 30,
    };
    assert_eq!(
        grande.part_pour_un_cache() / petite.part_pour_un_cache(),
        16,
        "la borne doit être proportionnelle à ce qui est disponible"
    );
}

#[test]
fn test_la_borne_se_contracte_quand_la_machine_se_tend() {
    // L'adaptation en temps réel, sur l'axe mémoire : une autre application réclame, le
    // disponible baisse, et la borne baisse avec — sans qu'aucun code de Glucose n'ait été
    // prévenu de quoi que ce soit.
    let avant = Memoire {
        totale: 16 << 30,
        disponible: 12 << 30,
    };
    let pendant = Memoire {
        totale: 16 << 30,
        disponible: 2 << 30,
    };
    assert!(
        pendant.part_pour_un_cache() < avant.part_pour_un_cache(),
        "la borne doit suivre le disponible à la baisse"
    );
}
