//! **Les images d'un document, et où vivent leurs octets** — lu sans jamais y écrire.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example verifier_images -- chemin\du\document.glucose
//! ```
//!
//! # Pourquoi cet outil existe
//!
//! Le 29/09, une image collée ne se montrait plus : « Image [img-paste-1670] ». Une image vit
//! dans le document quand ses octets y sont **scellés** — un objet, et le lien de sa clé vers son
//! empreinte ; sinon, elle ne dépend que d'un fichier extérieur, qui peut disparaître. L'outil dit,
//! pour chaque image, lequel des deux, et si le fichier existe encore. Il ouvre le fichier en
//! lecture : aucun verrou, et il peut lire un document que Glucose tient ouvert.

use glucose_core::hash::sha256;
use glucose_core::persist::histoire;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

fn main() {
    let Some(chemin) = std::env::args().nth(1) else {
        eprintln!("usage : verifier_images <document.glucose>");
        return;
    };
    let f = match std::fs::File::open(&chemin) {
        Ok(f) => f,
        Err(e) => return eprintln!("{chemin} : {e}"),
    };
    let mut lecteur = std::io::BufReader::new(&f);
    let ouvert = match histoire::ouvrir(&mut lecteur) {
        Ok(o) => o,
        Err(e) => return eprintln!("{chemin} ne se lit pas : {e}"),
    };
    let (mut scellees, mut dehors, mut perdues, mut sans_source) = (0, Vec::new(), Vec::new(), 0);
    for tableau in &ouvert.projet.boards {
        for img in &tableau.images {
            let Some(src) = img.src.as_deref() else {
                sans_source += 1;
                continue;
            };
            let scellee = ouvert
                .liens
                .get(src)
                .is_some_and(|e| ouvert.objets.contains_key(e));
            if scellee {
                scellees += 1;
                let empreinte = ouvert.liens[src];
                let t = ouvert.objets[&empreinte];
                let mut octets = vec![0u8; t.longueur as usize];
                let lu = lecteur
                    .seek(SeekFrom::Start(t.offset))
                    .and_then(|_| lecteur.read_exact(&mut octets));
                let decode = match (lu, image::load_from_memory(&octets)) {
                    (Err(e), _) => format!("ILLISIBLE ({e})"),
                    (Ok(()), Ok(i)) => format!("se décode, {} x {}", i.width(), i.height()),
                    (Ok(()), Err(e)) => format!("NE SE DÉCODE PAS ({e})"),
                };
                let fichier = match std::fs::read(src) {
                    Ok(o) if sha256(&o) == empreinte => "fichier identique".to_string(),
                    Ok(o) => format!(
                        "fichier DIFFÉRENT ({} octets contre {} scellés)",
                        o.len(),
                        octets.len()
                    ),
                    Err(_) => "fichier absent".to_string(),
                };
                println!(
                    "{}  {} octets scellés, {decode} ; {fichier}
    {src}",
                    img.id,
                    octets.len()
                );
            } else if Path::new(src).is_file() {
                dehors.push((tableau.name.clone(), img.id.clone(), src.to_string()));
            } else {
                perdues.push((tableau.name.clone(), img.id.clone(), src.to_string()));
            }
        }
    }
    println!(
        "{} tableau(x) ; images scellées dans le document : {scellees} ; hors du document, fichier \
         présent : {} ; hors du document, fichier ABSENT : {} ; sans source : {sans_source}",
        ouvert.projet.boards.len(),
        dehors.len(),
        perdues.len()
    );
    println!(
        "{} objet(s) d'image dans le fichier, {} lien(s)\n",
        ouvert.objets.len(),
        ouvert.liens.len()
    );
    for (titre, liste) in [
        ("HORS DU DOCUMENT, FICHIER PRÉSENT (à sceller)", &dehors),
        ("HORS DU DOCUMENT, FICHIER ABSENT (perdues)", &perdues),
    ] {
        if !liste.is_empty() {
            println!("{titre} :");
            for (tableau, id, src) in liste {
                println!("  {id}  [{tableau}]  {src}");
            }
            println!();
        }
    }
    fichiers_designes(&ouvert.projet);
}

/// **Les nœuds qui désignent un fichier** — une tuile qui y mène, une carte née de son texte —
/// et ce qu'il en reste sur le disque. Une tuile ne porte que le chemin : si le fichier
/// disparaît, elle ne mène plus à rien. Ceux du dossier temporaire du système sont nommés, car
/// Windows peut le vider.
fn fichiers_designes(projet: &glucose_core::types::Project) {
    use glucose_core::types::Annotation;
    let temporaire = std::env::temp_dir();
    let mut lignes = Vec::new();
    for tableau in &projet.boards {
        for a in &tableau.annotations {
            let (sorte, chemin) = match a {
                Annotation::Sticky {
                    source_file: Some(c),
                    ..
                } => ("tuile", c),
                Annotation::Text {
                    source_file: Some(c),
                    ..
                } => ("carte", c),
                _ => continue,
            };
            let etat = match (
                Path::new(chemin).is_file(),
                Path::new(chemin).starts_with(&temporaire),
            ) {
                (false, _) => "fichier ABSENT",
                (true, true) => "fichier présent, dans le DOSSIER TEMPORAIRE",
                (true, false) => "fichier présent",
            };
            lignes.push(format!(
                "  {sorte} {}  [{}]  {etat}\n    {chemin}",
                a.id(),
                tableau.name
            ));
        }
    }
    println!("{} nœud(s) désignent un fichier", lignes.len());
    for l in lignes {
        println!("{l}");
    }
}
