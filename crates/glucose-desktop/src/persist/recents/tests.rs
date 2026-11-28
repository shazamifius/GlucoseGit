//! Ce que garantit la liste des documents (RECENTS-1, fiche 174).

use super::*;
use crate::epreuve::Dossier;
use crate::persist::atomic::renommer_sans_ecraser;

fn poser(chemin: &Path) {
    if let Some(d) = chemin.parent() {
        std::fs::create_dir_all(d).expect("son dossier");
    }
    std::fs::write(chemin, b"glucose").expect("un document");
}

fn ses_trouves(chemins: &[&Path]) -> Vec<Trouve> {
    chemins
        .iter()
        .map(|p| (p.to_path_buf(), crate::plateforme::numero::de(p)))
        .collect()
}

/// **Les récents dans leur ordre, l'introuvable à sa place, les jamais ouverts ensuite** ; un
/// récent déplacé se retrouve par son numéro, et la liste à récrire le dit.
#[test]
fn recents_1_la_liste_retrouve_le_deplace_et_dit_l_introuvable() {
    let d = Dossier::nouveau("recents");
    let (a, b, efface) = (
        d.join("a.glucose"),
        d.join("b.glucose"),
        d.join("efface.glucose"),
    );
    for p in [&a, &b, &efface] {
        poser(p);
    }
    let connus = vec![
        Connu::numerote(&b),
        Connu::numerote(&a),
        Connu::numerote(&efface),
        Connu {
            chemin: d.join("ancien.glucose"),
            numero: None,
        },
    ];
    // a part ailleurs sous un autre nom ; efface disparaît ; un autre, jamais ouvert, paraît.
    let deplace = d.join("rangé/a renommé.glucose");
    std::fs::create_dir_all(d.join("rangé")).expect("le dossier");
    renommer_sans_ecraser(&a, &deplace).expect("déplacé");
    std::fs::remove_file(&efface).expect("effacé");
    let jamais = d.join("jamais.glucose");
    poser(&jamais);
    // Un autre jamais ouvert, plus ancien : il vient après, quel que soit l'ordre de la fouille.
    let vieux = d.join("vieux.glucose");
    poser(&vieux);
    let il_y_a_un_an = SystemTime::now() - std::time::Duration::from_secs(365 * 86_400);
    let ouvert = std::fs::File::options()
        .write(true)
        .open(&vieux)
        .expect("ouvert");
    ouvert.set_modified(il_y_a_un_an).expect("vieilli");
    let trouves = ses_trouves(&[&deplace, &vieux, &jamais, &b, &jamais]);

    let (documents, a_recrire) = composer(connus, &trouves);
    let vus: Vec<(&Path, Sorte)> = documents
        .iter()
        .map(|doc| (doc.chemin.as_path(), doc.sorte))
        .collect();
    assert_eq!(
        vus,
        [
            (b.as_path(), Sorte::Recent),
            (deplace.as_path(), Sorte::Recent),
            (efface.as_path(), Sorte::Introuvable),
            (d.join("ancien.glucose").as_path(), Sorte::Introuvable),
            (jamais.as_path(), Sorte::JamaisOuvert),
            (vieux.as_path(), Sorte::JamaisOuvert),
        ],
        "les récents, le déplacé à sa nouvelle place, les introuvables, puis les jamais ouverts, \
         une fois chacun, le plus récent d'abord"
    );
    let a_recrire = a_recrire.expect("la liste a appris où est a");
    assert_eq!(a_recrire[1].chemin, deplace, "la liste se corrige");
    assert_eq!(
        a_recrire.len(),
        4,
        "l'introuvable y reste : c'est à lui de l'oublier"
    );
}

/// **Une ligne d'avant gagne son numéro** la première fois que la liste se compose ; une liste
/// qui n'a rien appris ne se récrit pas.
#[test]
fn recents_1_une_ligne_d_avant_gagne_son_numero() {
    let d = Dossier::nouveau("recents-numero");
    let a = d.join("a.glucose");
    poser(&a);
    let d_avant = vec![Connu {
        chemin: a.clone(),
        numero: None,
    }];
    let (_, a_recrire) = composer(d_avant, &[]);
    let a_recrire = a_recrire.expect("un numéro appris");
    assert_eq!(a_recrire[0].numero, crate::plateforme::numero::de(&a));
    assert_eq!(
        composer(a_recrire, &[]).1,
        None,
        "rien de neuf, rien à récrire"
    );
}

/// **La fouille trouve les documents rangés à la main**, à toute profondeur, et passe les
/// dossiers cachés et ceux qui se déclarent cache.
#[test]
fn recents_1_la_fouille_passe_les_caches() {
    let d = Dossier::nouveau("fouille");
    let range = d.join("Projets/un/deux/trois/range.glucose");
    for p in [
        &range,
        &d.join("haut.glucose"),
        &d.join(".cache/glucose/cache.glucose"),
        &d.join("Projets/target/debug/construit.glucose"),
    ] {
        poser(p);
    }
    std::fs::write(
        d.join("Projets/target/CACHEDIR.TAG"),
        b"Signature: 8a477f597d28d172789f06886806bc55",
    )
    .expect("la marque");
    std::fs::write(d.join("Projets/pas-un-document.txt"), b"").expect("un autre fichier");
    let mut trouves = Vec::new();
    fouille::parcourir(&d, &mut |p| trouves.push(p.to_path_buf()));
    trouves.sort();
    assert_eq!(
        trouves,
        [
            d.join("Projets/un/deux/trois/range.glucose"),
            d.join("haut.glucose")
        ]
    );
}

/// **Ce qu'une rangée dit à droite** : le dossier sous `~`, depuis quand, ou ce qui lui arrive.
#[test]
fn recents_1_la_precision_dit_le_dossier_et_le_temps() {
    let maintenant = std::time::UNIX_EPOCH + std::time::Duration::from_secs(10 * 86_400);
    let document = |chemin: &str, sorte, il_y_a: u64| Document {
        chemin: PathBuf::from(chemin),
        sorte,
        touche: Some(maintenant - std::time::Duration::from_secs(il_y_a)),
    };
    let moi = Some(Path::new("/home/lui"));
    let dire = |doc: &Document| precision(doc, Some(dossier_court(&doc.chemin, moi)), maintenant);
    assert_eq!(
        dire(&document(
            "/home/lui/Images/refs/a.glucose",
            Sorte::Recent,
            3 * 86_400
        )),
        "~/Images/refs · il y a 3 j"
    );
    assert_eq!(
        dire(&document("/home/lui/a.glucose", Sorte::Recent, 120)),
        "~ · il y a 2 min"
    );
    assert_eq!(
        dire(&document("/media/cle/a.glucose", Sorte::Introuvable, 0)),
        "introuvable · /media/cle"
    );
    assert_eq!(
        dire(&document("/home/lui/x/a.glucose", Sorte::JamaisOuvert, 0)),
        "jamais ouvert · ~/x"
    );
    // Au téléphone, aucun dossier : le dossier privé de Glucose ne se lit pas (RECENTS-2).
    let au_telephone = document("/data/user/0/glucose/a.glucose", Sorte::Recent, 120);
    assert_eq!(precision(&au_telephone, None, maintenant), "il y a 2 min");
}

/// **La fouille, de bout en bout** : branchée sur un dossier, elle trouve, réveille la boucle à
/// chaque trouvaille, et le dit encore quand elle a fini — la palette cesse alors d'annoncer
/// qu'elle cherche. Sans racine, elle ne cherche nulle part.
#[test]
fn recents_1_la_fouille_trouve_et_dit_qu_elle_a_fini() {
    let d = Dossier::nouveau("fouille-fil");
    poser(&d.join("un/a.glucose"));
    poser(&d.join("b.glucose"));
    let muette = fouille::Fouille::default();
    muette.lancer();
    assert!(
        !muette.en_cours() && muette.trouves().is_empty(),
        "sans racine, rien"
    );

    let reveils = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let compte = std::sync::Arc::clone(&reveils);
    let mut f = fouille::Fouille::default();
    f.brancher(
        Some(d.to_path_buf()),
        std::sync::Arc::new(move || {
            compte.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }),
    );
    f.lancer();
    let debut = std::time::Instant::now();
    // Le dernier réveil suit la fin (« en_cours » tombe d'abord, pour que la boucle réveillée la
    // voie) : on attend donc la fin ET les réveils — le 28/11, lu entre les deux, il en manquait un.
    let reveille = || reveils.load(std::sync::atomic::Ordering::Acquire);
    while (f.en_cours() || reveille() < 3) && debut.elapsed() < std::time::Duration::from_secs(10) {
        std::thread::yield_now();
    }
    let reveils = reveille();
    assert_eq!(f.trouves().len(), 2);
    assert_eq!(f.version(), 3, "deux trouvailles, puis la fin");
    assert_eq!(reveils, 3, "la boucle réveillée à chaque fois");
}
