//! **L'histoire d'un document** (`persist::histoire`) : ce qui s'écrit après la base, et ce
//! qu'on en relit.
//!
//! L'épreuve qui fonde tout est la première : un vrai `Store`, piloté par ses propres gestes,
//! écrit après **chacun** ce que sa file de sortie a produit (JRN-5) ; le fichier relu doit
//! redonner **exactement** le document en mémoire. Si une seule opération modifie le document
//! sans passer par le journal, elle tombe.

use glucose_core::hash::sha256;
use glucose_core::persist::{
    self,
    histoire::{self, nature, Chaine, Genre, Geste, Jalon, Vue},
};
use glucose_core::store::{StackMove, Store};
use glucose_core::types::{
    Annotation, AssetStore, BoardImage, BoardZone, CanvasFolder, Domain, Preset,
};

/// Un fichier en mémoire : la base, puis l'histoire écrite au fil des gestes.
struct Disque {
    octets: Vec<u8>,
    chaine: Chaine,
}

impl Disque {
    fn nouveau(store: &Store) -> Self {
        let octets = persist::encode(&store.project, &AssetStore::new(), 0);
        let chaine = Chaine::de_la_base(&octets).expect("une base neuve a sa table");
        Self { octets, chaine }
    }

    /// Écrit ce que le journal a appliqué depuis la dernière fois, puis la vue.
    fn ecrire(&mut self, store: &mut Store) {
        for transaction in store.journal.prendre_les_ecrits() {
            let g = Geste {
                instant: 1_700_000_000_000,
                auteur: 7,
                transaction,
            };
            let e = self
                .chaine
                .encadrer(nature::GESTE, &histoire::contenu_geste(&g));
            self.octets.extend_from_slice(&e);
        }
        let v = histoire::contenu_vue(&Vue::de(&store.project));
        let e = self.chaine.encadrer(nature::VUE, &v);
        self.octets.extend_from_slice(&e);
    }

    fn relire(&self) -> persist::GlucoseFile {
        persist::decode(&self.octets).expect("l'histoire écrite doit se relire")
    }
}

fn image(id: &str, x: f64) -> BoardImage {
    let mut i = BoardImage::new(id, x, 10.0, 200.0, 150.0);
    i.src = Some(format!("objet:{id}"));
    i
}

/// **JRN-5** — après chaque geste, le fichier relu redonne le document en mémoire.
#[test]
fn test_l_histoire_relue_redonne_le_document_apres_chaque_geste() {
    let mut store = Store::new("essai");
    let b = store.project.active_board_id.clone();
    let mut disque = Disque::nouveau(&store);
    let mut verifier = |store: &mut Store, quoi: &str| {
        disque.ecrire(store);
        let relu = disque.relire();
        assert!(
            relu.project == store.project,
            "après « {quoi} », le fichier relu diffère du document en mémoire"
        );
    };

    store.add_image(&b, image("a", 0.0));
    verifier(&mut store, "ajouter une image");
    store.add_image(&b, image("b", 300.0));
    store.add_image(&b, image("c", 600.0));
    verifier(&mut store, "ajouter deux images");
    store.select_image("a".into(), false);
    store.select_image("c".into(), true);
    store.move_selected(&b, 12.5, -3.0);
    verifier(&mut store, "déplacer une sélection");
    store.move_selection_in_stack(&b, StackMove::Front);
    verifier(&mut store, "passer au premier plan");
    store.duplicate_selected(&b);
    verifier(&mut store, "dupliquer");
    store.toggle_lock_selection(&b);
    verifier(&mut store, "verrouiller");
    store.add_annotation(&b, Annotation::text("t1", 5.0, 5.0, "# une idée"));
    store.add_annotation(&b, Annotation::sticky("s1", 50.0, 5.0, "ET"));
    verifier(&mut store, "ajouter des annotations");
    assert!(store.undo(), "il y a un geste à défaire");
    verifier(&mut store, "défaire");
    assert!(store.undo());
    verifier(&mut store, "défaire encore");
    assert!(store.redo(), "il y a un geste à refaire");
    verifier(&mut store, "refaire");
    store.clear_selection();
    store.select_image("b".into(), false);
    store.delete_selected(&b);
    verifier(&mut store, "supprimer");
    store.rename_board(&b, "Principal renommé");
    let b2 = store.add_board("Annexe");
    verifier(&mut store, "renommer et ajouter un tableau");
    store.add_image(&b2, image("z", 0.0));
    store.set_board_zones(&b, vec![BoardZone::new("slot", 0.0, 0.0, 10.0, 10.0)]);
    store.set_project_name("renommé");
    verifier(&mut store, "zones, nom, image sur l'annexe");
    store
        .try_add_domain(Domain {
            id: "d1".into(),
            name: "Art".into(),
            color: "#ff0000".into(),
            icon: "A".into(),
            created_at: 0,
        })
        .expect("domaine neuf");
    store
        .try_assign_domain_to_node(&b, "a", "d1", 0.5)
        .expect("assignation");
    verifier(&mut store, "domaine et assignation");
    store.create_folder(&b, CanvasFolder::new("", "Dossier", ""));
    verifier(&mut store, "créer un dossier");
    store.add_preset(Preset {
        id: "p".into(),
        name: "Preset".into(),
        description: String::new(),
        slots: Vec::new(),
        is_builtin: false,
        created_at: 0,
    });
    verifier(&mut store, "ajouter un preset");
    // Une mise en page qui change le NOMBRE d'éléments : elle vidait le journal, et le disque
    // n'en aurait jamais rien su.
    store.mutate_board_layout(&b, |board| {
        board.images.pop();
    });
    verifier(&mut store, "une mise en page qui retire une image");
    // MEMB-1 : naître dans une membrane, l'emporter, en sortir au dépôt, la supprimer.
    store.add_annotation(&b, Annotation::membrane("m", -500.0, -500.0, 400.0, 800.0));
    store.add_image(&b, image("dans-m", -450.0));
    let membre = |store: &Store| {
        store
            .active_board()
            .and_then(|board| board.images.iter().find(|i| i.id == "dans-m"))
            .and_then(|i| i.membrane_id.clone())
    };
    assert_eq!(membre(&store).as_deref(), Some("m"), "née dans la membrane");
    verifier(&mut store, "naître dans une membrane");
    store.clear_selection();
    store.select_annotation("m".into(), false);
    store.move_selected(&b, 7.0, 3.0);
    verifier(&mut store, "une membrane emporte son contenu");
    store.clear_selection();
    store.select_image("dans-m".into(), false);
    store.begin_live_edit();
    store.move_selected(&b, 0.0, 900.0);
    store.rattacher_la_selection(&b);
    store.end_live_edit();
    assert_eq!(membre(&store), None, "lâchée dehors");
    verifier(&mut store, "lâcher hors de la membrane");
    store.move_selected(&b, 0.0, -900.0);
    store.rattacher_la_selection(&b);
    store.remove_annotations(&b, &["m"]);
    verifier(&mut store, "supprimer la membrane libère son contenu");
    store.try_remove_board(&b2).expect("l'annexe se supprime");
    verifier(&mut store, "supprimer un tableau");
    // La navigation n'est pas un geste, mais la vue l'emporte.
    store.set_viewport(
        &b,
        glucose_core::types::Viewport {
            x: 40.0,
            y: -8.0,
            scale: 2.5,
        },
    );
    verifier(&mut store, "se déplacer (la vue)");
    while store.undo() {}
    verifier(&mut store, "tout défaire");
}

/// Une fin déchirée — l'écriture interrompue d'une entrée — est ignorée, et dite : tout ce
/// qui la précède se relit.
#[test]
fn test_une_fin_dechiree_est_ignoree_et_ce_qui_precede_se_relit() {
    let mut store = Store::new("x");
    let b = store.project.active_board_id.clone();
    let mut disque = Disque::nouveau(&store);
    store.add_image(&b, image("gardee", 0.0));
    disque.ecrire(&mut store);
    let intact = disque.octets.len();
    let attendu = store.project.clone();
    store.add_image(&b, image("perdue", 1.0));
    disque.ecrire(&mut store);
    // La fin du geste « perdue » : chaque coupe tombe dans son en-tête ou dans son contenu.
    let complet = histoire::ouvrir(&mut std::io::Cursor::new(&disque.octets)).unwrap();
    let dernier = complet.gestes.last().expect("deux gestes écrits").tranche;
    let fin_du_geste = (dernier.offset + dernier.longueur) as usize;
    for coupe in [intact + 3, intact + 17, fin_du_geste - 1] {
        let tronque = &disque.octets[..coupe];
        let o = histoire::ouvrir(&mut std::io::Cursor::new(tronque)).expect("le début se relit");
        assert_eq!(o.projet, attendu, "coupé à {coupe}");
        assert_eq!(
            o.fin, intact as u64,
            "la reprise se fait après la dernière entrée saine"
        );
        assert_eq!(o.fin_ignoree, (coupe - intact) as u64);
    }
}

/// Un octet altéré au milieu de l'histoire arrête la lecture **là** : la chaîne ne suit plus,
/// et rien d'après n'est cru.
#[test]
fn test_un_octet_altere_arrete_la_chaine_a_son_entree() {
    let mut store = Store::new("x");
    let b = store.project.active_board_id.clone();
    let mut disque = Disque::nouveau(&store);
    store.add_image(&b, image("un", 0.0));
    disque.ecrire(&mut store);
    let apres_un = (disque.octets.len(), store.project.clone());
    store.add_image(&b, image("deux", 1.0));
    disque.ecrire(&mut store);
    store.add_image(&b, image("trois", 2.0));
    disque.ecrire(&mut store);
    let mut abime = disque.octets.clone();
    abime[apres_un.0 + 20] ^= 0x40;
    let o = histoire::ouvrir(&mut std::io::Cursor::new(&abime)).unwrap();
    assert_eq!(o.projet, apres_un.1);
    assert_eq!(o.fin, apres_un.0 as u64);
}

/// Une histoire écrite derrière une autre base ne se relit pas derrière celle-ci : la graine
/// de la chaîne est la table de la base.
#[test]
fn test_une_histoire_ne_se_relit_que_derriere_sa_propre_base() {
    let mut store = Store::new("x");
    let b = store.project.active_board_id.clone();
    let mut disque = Disque::nouveau(&store);
    store.add_image(&b, image("un", 0.0));
    disque.ecrire(&mut store);
    let queue = disque.octets
        [persist::encode(&Store::new("x").project, &AssetStore::new(), 0).len()..]
        .to_vec();
    let mut autre = persist::encode(&Store::new("autre").project, &AssetStore::new(), 0);
    let base_seule = autre.len() as u64;
    autre.extend_from_slice(&queue);
    let o = histoire::ouvrir(&mut std::io::Cursor::new(&autre)).unwrap();
    assert!(
        o.projet.boards[0].images.is_empty(),
        "la queue d'une autre base n'est pas crue"
    );
    assert_eq!(o.fin, base_seule);
}

/// Un instantané évite de tout rejouer, et les gestes qui le suivent s'appliquent dessus.
#[test]
fn test_un_instantane_puis_des_gestes_se_relisent() {
    let mut store = Store::new("x");
    let b = store.project.active_board_id.clone();
    let mut disque = Disque::nouveau(&store);
    for i in 0..5 {
        store.add_image(&b, image(&format!("i{i}"), i as f64));
    }
    disque.ecrire(&mut store);
    let e = disque.chaine.encadrer(
        nature::INSTANTANE,
        &histoire::contenu_instantane(&store.project),
    );
    disque.octets.extend_from_slice(&e);
    store.select_image("i2".into(), false);
    store.delete_selected(&b);
    disque.ecrire(&mut store);
    let o = histoire::ouvrir(&mut std::io::Cursor::new(&disque.octets)).unwrap();
    assert_eq!(o.projet, store.project);
    assert_eq!(o.instantanes.len(), 1);
    assert_eq!(
        o.instantanes[0].apres, 5,
        "l'instantané suit les cinq ajouts"
    );
    assert_eq!(o.gestes.len(), 6);
}

/// Les octets d'une image s'écrivent une fois, se retrouvent par leur empreinte, et une clé
/// s'y lie.
#[test]
fn test_un_objet_et_son_lien_se_retrouvent() {
    let store = Store::new("x");
    let mut disque = Disque::nouveau(&store);
    let pixels = b"\x89PNG des pixels".to_vec();
    let empreinte = sha256(&pixels);
    let e = disque
        .chaine
        .entete_d_objet(&empreinte, pixels.len() as u64)
        .unwrap();
    disque.octets.extend_from_slice(&e);
    disque.octets.extend_from_slice(&pixels);
    let e = disque.chaine.encadrer(
        nature::LIEN,
        &histoire::contenu_lien("objet:photo", &empreinte),
    );
    disque.octets.extend_from_slice(&e);
    let e = disque.chaine.encadrer(
        nature::JALON,
        &histoire::contenu_jalon(&Jalon {
            instant: 3,
            genre: Genre::Enregistrement,
            libelle: "avant".into(),
        }),
    );
    disque.octets.extend_from_slice(&e);
    let relu = disque.relire();
    assert_eq!(relu.assets.get("objet:photo"), Some(&pixels[..]));
    let o = histoire::ouvrir(&mut std::io::Cursor::new(&disque.octets)).unwrap();
    let t = o.objets[&empreinte];
    assert_eq!(
        &disque.octets[t.offset as usize..][..t.longueur as usize],
        &pixels[..]
    );
    assert_eq!(o.jalons.len(), 1);
    assert_eq!(o.jalons[0].1.libelle, "avant");
}

/// Une entrée d'une nature inconnue, qui suit pourtant la chaîne, vient d'une version plus
/// récente : le document est refusé et dit pourquoi, au lieu d'être ouvert à moitié.
#[test]
fn test_une_nature_inconnue_demande_une_version_plus_recente() {
    let store = Store::new("x");
    let mut disque = Disque::nouveau(&store);
    let e = disque.chaine.encadrer(99, b"du futur");
    disque.octets.extend_from_slice(&e);
    let err = histoire::ouvrir(&mut std::io::Cursor::new(&disque.octets))
        .expect_err("refusé")
        .to_string();
    assert!(err.contains("plus récente"), "{err}");
}

/// **Le retour dans le temps** (HISTOIRE-3) : pour chaque point du passé, l'état relu est
/// celui qu'avait le document à ce geste ; restaurer y ramène le document par **un** geste
/// annulable ; et défaire ce geste rend le présent.
#[test]
fn test_chaque_point_du_passe_se_relit_se_restaure_et_se_defait() {
    let mut store = Store::new("temps");
    let b = store.project.active_board_id.clone();
    let mut disque = Disque::nouveau(&store);
    let mut etats = vec![store.project.clone()];
    let geste = |store: &mut Store, disque: &mut Disque, etats: &mut Vec<_>| {
        for t in store.journal.prendre_les_ecrits() {
            let g = Geste {
                instant: etats.len() as i64,
                auteur: 1,
                transaction: t,
            };
            let e = disque
                .chaine
                .encadrer(nature::GESTE, &histoire::contenu_geste(&g));
            disque.octets.extend_from_slice(&e);
            etats.push(store.project.clone());
        }
    };
    store.add_image(&b, image("a", 0.0));
    geste(&mut store, &mut disque, &mut etats);
    store.add_image(&b, image("b", 1.0));
    geste(&mut store, &mut disque, &mut etats);
    // Un instantané au milieu : les points d'après partent de lui, ceux d'avant de la base.
    let e = disque.chaine.encadrer(
        nature::INSTANTANE,
        &histoire::contenu_instantane(&store.project),
    );
    disque.octets.extend_from_slice(&e);
    store.select_image("a".into(), false);
    store.move_selected(&b, 40.0, 2.0);
    geste(&mut store, &mut disque, &mut etats);
    store.add_annotation(&b, Annotation::text("t", 1.0, 1.0, "trois"));
    geste(&mut store, &mut disque, &mut etats);
    store.clear_selection();
    store.select_image("b".into(), false);
    store.delete_selected(&b);
    geste(&mut store, &mut disque, &mut etats);
    assert!(store.undo());
    geste(&mut store, &mut disque, &mut etats);

    let lire = || histoire::ouvrir(&mut std::io::Cursor::new(&disque.octets)).unwrap();
    let o = lire();
    assert_eq!(o.gestes.len(), etats.len() - 1);
    for (k, attendu) in etats.iter().enumerate() {
        let passe = histoire::etat_au_geste(&mut std::io::Cursor::new(&disque.octets), &o, k)
            .expect("chaque point se relit");
        assert_eq!(&passe, attendu, "au geste {k}");
    }
    for k in [0, 2, 4] {
        let mut essai = store.clone();
        let retour = histoire::retour_au_geste(&mut std::io::Cursor::new(&disque.octets), &o, k)
            .expect("le retour se construit");
        assert!(
            essai.appliquer_comme_un_geste(retour),
            "restaurer au geste {k}"
        );
        assert_eq!(essai.project, etats[k], "restauré au geste {k}");
        assert!(essai.undo(), "restaurer est un geste annulable");
        assert_eq!(
            essai.project, store.project,
            "défaire la restauration rend le présent"
        );
    }
}

/// **Le point d'accroche d'un texte en cours de frappe** : la chaîne juste après le dernier
/// geste. Une vue, un jalon ne le déplacent pas — ils ne changent pas ce qu'on tapait ; un
/// geste, si — c'est que la saisie a été validée.
#[test]
fn test_le_dernier_geste_ne_bouge_qu_avec_un_geste() {
    let mut store = Store::new("x");
    let b = store.project.active_board_id.clone();
    let mut disque = Disque::nouveau(&store);
    let graine = disque.chaine;
    let ouvrir = |d: &Disque| histoire::ouvrir(&mut std::io::Cursor::new(&d.octets)).unwrap();
    assert_eq!(
        ouvrir(&disque).dernier_geste,
        graine,
        "sans geste : la graine"
    );

    store.add_image(&b, image("a", 0.0));
    disque.ecrire(&mut store);
    let o = ouvrir(&disque);
    assert_ne!(o.dernier_geste, graine);
    assert_ne!(
        o.dernier_geste, o.chaine,
        "la vue écrite après ne le déplace pas"
    );
    let apres = o.dernier_geste;

    let e = disque.chaine.encadrer(
        nature::JALON,
        &histoire::contenu_jalon(&Jalon {
            instant: 3,
            genre: Genre::Nomme,
            libelle: "ici".into(),
        }),
    );
    disque.octets.extend_from_slice(&e);
    assert_eq!(ouvrir(&disque).dernier_geste, apres, "un jalon non plus");

    store.add_image(&b, image("b", 300.0));
    disque.ecrire(&mut store);
    assert_ne!(ouvrir(&disque).dernier_geste, apres, "un geste, si");
}
