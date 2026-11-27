//! **L'histoire gardée ouverte** (fiche 183) : la Time Machine et les IA relisent l'histoire à
//! chaque appel ; elle se prolonge des gestes neufs au lieu de se relire entière.

use super::tests::{application, dossier, image_suivante, noter};
use crate::app::GlucoseApp;
use glucose_core::persist::histoire;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

fn relue(chemin: &Path) -> histoire::Ouvert {
    let fichier = std::fs::File::open(chemin).expect("le fichier existe");
    histoire::ouvrir(&mut std::io::BufReader::new(fichier)).expect("le fichier se relit")
}

fn un_document(d: &Path, nom: &str, notes: usize) -> (GlucoseApp, PathBuf) {
    let chemin = d.join(nom);
    let mut app = application(d);
    app.save_to(chemin.clone());
    for i in 0..notes {
        noter(
            &mut app,
            &format!("{nom}-{i}"),
            &format!("la note {i} de {nom}"),
        );
        image_suivante(&mut app);
    }
    (app, chemin)
}

/// Chaque geste neuf paraît, et l'histoire relue est celle du fichier entier.
#[test]
fn test_histoire_ouverte_suit_chaque_geste() {
    let d = dossier("histoire-ouverte");
    let (mut app, chemin) = un_document(&d, "suivie.glucose", 1);
    let mut avant = app
        .lire_l_histoire()
        .expect("l'histoire se relit")
        .gestes
        .len();
    for i in 0..3 {
        noter(&mut app, &format!("neuve-{i}"), "écrite après la lecture");
        let o = app.lire_l_histoire().expect("l'histoire se relit");
        assert_eq!(o.gestes.len(), avant + 1, "le geste neuf paraît");
        assert_eq!(*o, relue(&chemin), "prolongée, elle est le fichier entier");
        avant = o.gestes.len();
    }
}

/// **Ce qui est déjà lu ne se relit pas** : un octet abîmé dans le premier geste, après la
/// lecture, ne la change pas — une relecture entière s'y arrêterait.
#[test]
fn test_histoire_ouverte_ne_relit_pas_ce_qu_elle_a_lu() {
    let d = dossier("histoire-deja-lue");
    let (mut app, chemin) = un_document(&d, "lue.glucose", 2);
    let o = app.lire_l_histoire().expect("l'histoire se relit");
    let n = o.gestes.len();
    let premier = o.gestes[0].tranche;
    drop(o);
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .open(&chemin)
        .expect("le fichier s'ouvre");
    f.seek(SeekFrom::Start(premier.offset + premier.longueur / 2))
        .expect("le geste est là");
    f.write_all(&[0xA5]).expect("l'octet s'écrit");
    drop(f);
    assert!(relue(&chemin).gestes.len() < n, "relu entier, il s'arrête");
    noter(&mut app, "apres", "écrite après l'octet abîmé");
    let o = app.lire_l_histoire().expect("l'histoire se relit");
    assert_eq!(o.gestes.len(), n + 1, "prolongée, rien de relu");
}

/// **Un fichier remplacé se relit entier** : ce qui suit l'histoire déjà lue ne suit plus sa
/// chaîne — c'est un autre fichier.
#[test]
fn test_histoire_ouverte_un_fichier_remplace_se_relit() {
    let d = dossier("histoire-remplacee");
    let (mut app, chemin) = un_document(&d, "a.glucose", 1);
    app.lire_l_histoire().expect("l'histoire se relit");
    let (_, autre) = un_document(&d, "b.glucose", 4);
    std::fs::write(&chemin, std::fs::read(&autre).expect("b s'écrit")).expect("a remplacé");
    let o = app.lire_l_histoire().expect("l'histoire se relit");
    assert_eq!(
        *o,
        relue(&autre),
        "l'histoire est celle du fichier d'aujourd'hui"
    );
}
