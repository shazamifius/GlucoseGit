//! **Ce qu'un document a vécu**, lu dans son fichier — sans jamais y écrire.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example lire_histoire -- chemin\du\document.glucose [combien]
//! ```
//!
//! # Pourquoi cet outil existe
//!
//! Après son vrai plantage du 25/09, l'utilisateur a dit : le texte est revenu, la caméra
//! aussi, mais pas le mode d'édition. Aucune sortie ne gardait la trace de cette relance. Son
//! document, si : chaque geste y est écrit avec **son instant et l'identifiant du lancement**
//! qui l'a fait. On y a lu que le texte tapé avant l'arrêt avait été validé par le lancement
//! **suivant**, onze secondes après la relance, juste avant un jalon — donc que la saisie était
//! bien revenue en édition, et qu'un `Ctrl+S` l'avait fermée (fiche 38 § 2).
//!
//! Il liste les derniers gestes — l'heure (UTC), le lancement, ce qu'ils changent — puis les
//! jalons. Le fichier s'ouvre en lecture : l'outil ne pose aucun verrou d'écriture, et peut
//! donc lire un document que Glucose tient ouvert.

use glucose_core::persist::histoire;
use glucose_core::store::journal::Edit;
use glucose_core::types::Annotation;
use std::io::{Read, Seek, SeekFrom};

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(chemin) = args.next() else {
        eprintln!("usage : lire_histoire <document.glucose> [combien de gestes]");
        return;
    };
    let combien: usize = args.next().and_then(|n| n.parse().ok()).unwrap_or(30);
    let mut f = match std::fs::File::open(&chemin) {
        Ok(f) => f,
        Err(e) => return eprintln!("{chemin} : {e}"),
    };
    let ouvert = match histoire::ouvrir(&mut std::io::BufReader::new(&f)) {
        Ok(o) => o,
        Err(e) => return eprintln!("{chemin} ne se lit pas : {e}"),
    };
    println!(
        "{} geste(s), {} jalon(s), {} instantané(s), {} octet(s) de fin ignorés",
        ouvert.gestes.len(),
        ouvert.jalons.len(),
        ouvert.instantanes.len(),
        ouvert.fin_ignoree
    );
    // MAIN-1 et NOTE-1 (fiche 108) : qui est déclaré, ce qui est écrit, ce que les agents tiennent.
    for a in ouvert.auteurs.values() {
        let gestes = ouvert.gestes.iter().filter(|r| r.auteur == a.id).count();
        let genre = if a.agent { "agent" } else { "humain" };
        println!(
            "auteur {:016x} : {genre} « {} », {gestes} geste(s)",
            a.id, a.nom
        );
    }
    println!(
        "{} écrit(s) ; {} nœud(s) posés par un agent et que l'humain n'a pas touchés\n",
        ouvert.notes.len(),
        ouvert.des_agents.len()
    );
    let debut = ouvert.gestes.len().saturating_sub(combien);
    for (k, repere) in ouvert.gestes.iter().enumerate().skip(debut) {
        let mut octets = vec![0u8; repere.tranche.longueur as usize];
        let lu = f
            .seek(SeekFrom::Start(repere.tranche.offset))
            .and_then(|_| f.read_exact(&mut octets));
        match lu.ok().and_then(|()| histoire::lire_geste(&octets).ok()) {
            Some(g) => println!(
                "#{k:<4} {}  {}\n       {}",
                heure(g.instant),
                qui(&ouvert.auteurs, g.auteur),
                resume(&g.transaction.edits)
            ),
            None => println!("#{k:<4} illisible"),
        }
        for note in ouvert.notes.iter().filter(|n| n.geste == k as u64) {
            let auteur = qui(&ouvert.auteurs, note.auteur);
            let sujets = note.sujets.join(", ");
            println!("       écrit de {auteur} [{sujets}] : {}", note.texte);
        }
    }
    println!();
    for (apres, jalon) in &ouvert.jalons {
        println!("jalon après {apres} geste(s) : « {} »", jalon.libelle);
    }
}

/// Qui est un auteur : l'IA de Glucose, l'agent sous son nom, ou le lancement de Glucose par
/// l'humain (un identifiant que rien ne déclare, MAIN-1).
fn qui(auteurs: &std::collections::HashMap<u64, histoire::Auteur>, id: u64) -> String {
    let auteur = auteurs.get(&id).and_then(histoire::Auteur::en_mots);
    auteur.unwrap_or_else(|| format!("humain, lancement {id:016x}"))
}

/// L'heure d'un instant Unix en millisecondes, en temps universel.
fn heure(instant: i64) -> String {
    let s = (instant / 1000).rem_euclid(86_400);
    format!("{:02}:{:02}:{:02} UTC", s / 3600, (s / 60) % 60, s % 60)
}

/// Ce qu'un geste change, une ligne par modification.
fn resume(edits: &[Edit]) -> String {
    edits
        .iter()
        .map(|e| match e {
            Edit::Annotation { slot, .. } => format!(
                "annotation {} : {} → {}",
                slot.after
                    .as_deref()
                    .or(slot.before.as_deref())
                    .map_or("?", Annotation::id),
                texte(slot.before.as_deref()),
                texte(slot.after.as_deref())
            ),
            autre => {
                let tout = format!("{autre:?}");
                tout.chars().take(90).collect()
            }
        })
        .collect::<Vec<_>>()
        .join("\n       ")
}

/// Le texte d'une annotation, abrégé — ou un tiret quand elle n'existe pas de ce côté.
fn texte(a: Option<&Annotation>) -> String {
    a.and_then(Annotation::own_text).map_or("—".into(), |t| {
        let court: String = t.chars().take(120).collect();
        format!("{court:?}")
    })
}
