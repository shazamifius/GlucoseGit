//! **Prolonger vaut rouvrir** (fiche 183) : une histoire ouverte, prolongée des entrées écrites
//! depuis, est exactement ce que [`super::ouvrir`] rend du fichier entier — à chaque frontière
//! d'entrée, et d'un seul pas comme de proche en proche.

use super::super::{
    contenu_auteur, contenu_geste, contenu_instantane, contenu_jalon, contenu_note, contenu_vue,
    nature, Auteur, Chaine, Genre, Geste, Jalon, Note, Vue,
};
use super::{ouvrir, prolonger, Ouvert};
use crate::store::Store;
use crate::types::{AssetStore, BoardImage};
use std::io::Cursor;

/// Le fichier en mémoire, et la fin de chacune de ses entrées.
struct Fichier {
    octets: Vec<u8>,
    chaine: Chaine,
    bornes: Vec<usize>,
}

impl Fichier {
    fn ajouter(&mut self, n: u8, contenu: &[u8]) {
        let e = self.chaine.encadrer(n, contenu);
        self.octets.extend_from_slice(&e);
        self.bornes.push(self.octets.len());
    }

    fn gestes(&mut self, store: &mut Store, auteur: u64) {
        for (i, transaction) in store.journal.prendre_les_ecrits().into_iter().enumerate() {
            let g = Geste {
                instant: 1_700_000_000_000 + i as i64,
                auteur,
                transaction,
            };
            self.ajouter(nature::GESTE, &contenu_geste(&g));
        }
        self.ajouter(nature::VUE, &contenu_vue(&Vue::de(&store.project)));
    }
}

fn image(id: &str, x: f64) -> BoardImage {
    let mut i = BoardImage::new(id, x, 10.0, 200.0, 150.0);
    i.src = Some(format!("objet:{id}"));
    i
}

fn lu(octets: &[u8]) -> Ouvert {
    ouvrir(&mut Cursor::new(octets)).expect("le fichier se relit")
}

/// Une histoire où passe chaque nature qui change ce qu'on relit : des gestes de l'humain et
/// d'un agent, sa déclaration, un écrit, un jalon, des vues, un geste qui ne se rejoue pas, puis
/// un instantané qui repart d'un état neuf.
fn une_histoire() -> Fichier {
    let mut store = Store::new("x");
    let b = store.project.active_board_id.clone();
    let octets = crate::persist::encode(&store.project, &AssetStore::new(), 0);
    let chaine = Chaine::de_la_base(&octets).expect("une base neuve a sa table");
    let mut f = Fichier {
        octets,
        chaine,
        bornes: Vec::new(),
    };
    store.add_image(&b, image("a", 0.0));
    f.gestes(&mut store, 7);
    let agent = Auteur::d_un_agent("proteo");
    f.ajouter(nature::AUTEUR, &contenu_auteur(&agent));
    store.add_image(&b, image("b", 300.0));
    f.gestes(&mut store, agent.id);
    let note = Note {
        instant: 1_700_000_000_009,
        auteur: agent.id,
        geste: 1,
        texte: "à côté de la première".into(),
        sujets: vec!["b".into()],
    };
    f.ajouter(nature::NOTE, &contenu_note(&note));
    let jalon = Jalon {
        instant: 3,
        genre: Genre::Enregistrement,
        libelle: "avant".into(),
    };
    f.ajouter(nature::JALON, &contenu_jalon(&jalon));
    // Un geste écrit d'un autre document : rejoué ici par le rang de sa case (JRN-2), il retire
    // une image qui n'est pas la sienne, et le geste suivant, qui ne trouve plus sa case, échoue.
    let mut ailleurs = Store::new("y");
    let tableau = ailleurs.project.active_board_id.clone();
    ailleurs.add_image(&tableau, image("z", 0.0));
    ailleurs.journal.prendre_les_ecrits();
    ailleurs.select_image("z".into(), false);
    ailleurs.delete_selected(&tableau);
    f.gestes(&mut ailleurs, 7);
    store.add_image(&b, image("c", 600.0));
    f.gestes(&mut store, 7);
    // Après l'échec, d'autres gestes : le rejeu ne les applique plus, prolongé ou rouvert.
    store.add_image(&b, image("d", 900.0));
    f.gestes(&mut store, 7);
    f.ajouter(nature::INSTANTANE, &contenu_instantane(&store.project));
    store.select_image("a".into(), false);
    store.delete_selected(&b);
    f.gestes(&mut store, agent.id);
    f
}

#[test]
fn prolonger_d_un_pas_rend_ce_que_rouvrir_rend() {
    let f = une_histoire();
    let entier = lu(&f.octets);
    assert_eq!(entier.gestes.len(), 6, "l'histoire porte ses six gestes");
    assert_eq!(
        entier.geste_en_echec, None,
        "l'instantané a repris le rejeu"
    );
    for &borne in &f.bornes {
        let mut o = lu(&f.octets[..borne]);
        prolonger(&mut Cursor::new(&f.octets), &mut o).expect("le fichier a grandi");
        assert_eq!(o, entier, "prolongée depuis l'octet {borne}");
    }
}

#[test]
fn prolonger_de_proche_en_proche_suit_chaque_entree() {
    let f = une_histoire();
    let mut o = lu(&f.octets[..f.bornes[0]]);
    let mut echecs = 0;
    for &borne in &f.bornes {
        prolonger(&mut Cursor::new(&f.octets[..borne]), &mut o).expect("le fichier a grandi");
        assert_eq!(o, lu(&f.octets[..borne]), "jusqu'à l'octet {borne}");
        echecs += usize::from(o.geste_en_echec.is_some());
    }
    assert!(echecs > 0, "le geste d'ailleurs a échoué en chemin");
}

#[test]
fn un_fichier_raccourci_ne_se_prolonge_pas() {
    let f = une_histoire();
    let mut o = lu(&f.octets);
    let avant = f.bornes[f.bornes.len() - 2];
    assert!(prolonger(&mut Cursor::new(&f.octets[..avant]), &mut o).is_err());
}
