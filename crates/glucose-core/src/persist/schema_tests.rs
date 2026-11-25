//! **Le plus petit schéma qui dit tout** (VIDEO-2, fiche 170) : sa beta.6 lit encore ce qui ne
//! parle pas des choix d'une vidéo — le document, sa lettre à la salle, son histoire.

use super::*;
use crate::salle::lettre::{Corps, Lettre};
use crate::store::journal::Main;
use crate::types::{BoardImage, ChoixVideo};

/// Un projet qui porte une vidéo, avec ces choix.
fn avec_une_video(choix: ChoixVideo) -> Project {
    let mut p = Project::new("schéma");
    let mut video = BoardImage::new("v", 0.0, 0.0, 480.0, 270.0);
    video.is_video = true;
    video.video = choix;
    p.boards[0].images.push(video);
    p
}

const AUTO: ChoixVideo = ChoixVideo {
    lecture_auto: true,
    a_cote: false,
};
const A_COTE: ChoixVideo = ChoixVideo {
    lecture_auto: false,
    a_cote: true,
};

#[test]
fn video_2_sans_choix_le_document_s_ecrit_au_schema_6_avec_au_7() {
    for (choix, schema) in [
        (ChoixVideo::default(), MIROIR_DEPUIS),
        (AUTO, VIDEO_DEPUIS),
        (A_COTE, VIDEO_DEPUIS),
    ] {
        let p = avec_une_video(choix);
        let relu = decode(&encode(&p, &AssetStore::new(), 0)).expect("relu");
        assert_eq!(relu.manifest.document_version, schema, "{choix:?}");
        assert_eq!(relu.project, p, "rien de perdu : {choix:?}");
    }
}

/// Une lettre de la salle : ce que le téléphone, resté en beta.6, reçoit de son PC.
#[test]
fn video_2_la_lettre_a_la_salle_s_ecrit_au_plus_petit_schema() {
    for (choix, schema) in [(ChoixVideo::default(), MIROIR_DEPUIS), (AUTO, VIDEO_DEPUIS)] {
        let l = Lettre {
            marque: "a7".into(),
            nom: "Alice".into(),
            couleur: 4,
            main: Main::Humain,
            corps: Corps::Base(Box::new(avec_une_video(choix))),
        };
        let octets = l.ecrire();
        assert_eq!(u16::from_le_bytes([octets[0], octets[1]]), schema);
        assert_eq!(Lettre::lire(&octets).expect("relue"), l);
    }
}

#[test]
fn video_2_l_histoire_s_ecrit_au_plus_petit_schema() {
    for (choix, schema) in [
        (ChoixVideo::default(), MIROIR_DEPUIS),
        (A_COTE, VIDEO_DEPUIS),
    ] {
        let p = avec_une_video(choix);
        let octets = histoire::contenu_instantane(&p);
        assert_eq!(u16::from_le_bytes([octets[0], octets[1]]), schema);
        assert_eq!(histoire::lire_instantane(&octets).expect("relu"), p);
    }
}

#[test]
fn video_2_un_geste_de_l_histoire_s_ecrit_au_plus_petit_schema() {
    use crate::store::journal::{Edit, Slot, Transaction};
    for (choix, schema) in [(ChoixVideo::default(), MIROIR_DEPUIS), (AUTO, VIDEO_DEPUIS)] {
        let video = avec_une_video(choix).boards[0].images[0].clone();
        let g = histoire::geste::Geste {
            instant: 1,
            auteur: 2,
            transaction: Transaction {
                edits: vec![Edit::Image {
                    board: "main".into(),
                    slot: Slot {
                        index: 0,
                        before: None,
                        after: Some(Box::new(video)),
                    },
                }],
            },
        };
        let octets = histoire::contenu_geste(&g);
        assert_eq!(u16::from_le_bytes([octets[0], octets[1]]), schema);
        assert_eq!(histoire::lire_geste(&octets).expect("relu"), g);
    }
}
