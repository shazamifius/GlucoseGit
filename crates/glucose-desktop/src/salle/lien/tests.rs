//! Ce qui part par le fil, tour après tour (B1) : les lettres devant ce qui passe.

use super::*;

/// Un canal qui note ce qui part, et ne reçoit rien.
#[derive(Default)]
struct Note(Vec<String>);

impl Canal for Note {
    fn envoyer(&mut self, texte: &str) -> Result<(), String> {
        self.0.push(texte.to_string());
        Ok(())
    }

    fn recevoir(&mut self, _: Duration) -> Result<Option<String>, String> {
        Ok(None)
    }
}

impl Note {
    /// La sorte de chaque message parti, dans l'ordre.
    fn sortes(&self) -> Vec<String> {
        self.0
            .iter()
            .filter_map(|m| Json::lire(m).ok()?.get("t")?.as_str().map(String::from))
            .collect()
    }
}

fn un_fil() -> (Lien, Fil) {
    let boite = Boite::nouvelle(Arc::new(|| {}));
    relier("http://127.0.0.1:9", Code::neuf([3; 32]), 0, boite)
}

fn passe(id: &str, photo: bool) -> Passe {
    Passe {
        id: id.into(),
        clair: vec![1, 2, 3],
        photo,
    }
}

/// **B1 — un geste ne fait jamais la queue derrière une photo** : dans un même tour, la lettre
/// part d'abord ; puis une seule chose qui passe par tour, et sa sorte se dit en clair.
#[test]
fn b1_les_lettres_passent_devant_les_photos() {
    let (lien, mut fil) = un_fil();
    let expediteur = lien.expediteur();
    expediteur.send(passe("p1", true)).expect("une place");
    lien.envoyer(Ordre::Lettre {
        id: "g1".into(),
        clair: vec![9],
    });
    let mut note = Note::default();
    fil.un_tour(&mut note, true).expect("un tour");
    assert_eq!(note.sortes(), ["geste", "photo"]);
    expediteur.send(passe("p2", false)).expect("une place");
    fil.un_tour(&mut note, true).expect("un tour");
    assert_eq!(note.sortes(), ["geste", "photo", "demande"]);
}

/// **Ce qui passe avant l'« à jour » se perd**, sans bloquer celui qui l'envoie : le bonjour
/// suivant le redemandera.
#[test]
fn b1_ce_qui_passe_avant_l_a_jour_se_perd() {
    let (lien, mut fil) = un_fil();
    lien.expediteur()
        .send(passe("p1", true))
        .expect("une place");
    let mut note = Note::default();
    fil.un_tour(&mut note, false).expect("un tour");
    fil.un_tour(&mut note, true).expect("un tour");
    assert!(note.0.is_empty(), "rien n'est parti : {:?}", note.sortes());
    assert!(
        lien.expediteur().try_send(passe("p2", true)).is_ok(),
        "la place s'est libérée"
    );
}

/// LIEN-JOURNAL (fiche 157 § 5) : après une connexion perdue, la raison du premier essai manqué se
/// dit — avant, seule celle du démarrage se disait, et une salle qui ne revenait plus se taisait.
#[test]
fn la_sortie_dit_le_premier_echec_de_chaque_serie_puis_le_retour() {
    let mut coupure = Coupure::default();
    assert_eq!(
        coupure.echec("nom du serveur").as_deref(),
        Some("pas relié (nom du serveur) ; il réessaie")
    );
    assert_eq!(coupure.echec("nom du serveur"), None, "la série se tait");
    assert_eq!(
        coupure.relie().as_deref(),
        Some("relié de nouveau, après 0 s")
    );
    assert_eq!(coupure.relie(), None, "relié sans coupure : rien à dire");
    coupure.perdue();
    assert_eq!(
        coupure
            .echec("le relais refuse l'ouverture : 503")
            .as_deref(),
        Some("pas relié (le relais refuse l'ouverture : 503) ; il réessaie")
    );
    assert!(
        coupure.relie().is_some(),
        "le retour après une connexion perdue"
    );
    coupure.perdue();
    assert!(
        coupure.relie().is_some(),
        "perdue puis reliée du premier essai : le retour se dit"
    );
}

/// Un canal qui dit tout de suite ce qui part, et ne reçoit rien.
struct Espion(mpsc::Sender<String>);

impl Canal for Espion {
    fn envoyer(&mut self, texte: &str) -> Result<(), String> {
        self.0.send(texte.to_string()).map_err(|e| e.to_string())
    }

    fn recevoir(&mut self, attente: Duration) -> Result<Option<String>, String> {
        std::thread::sleep(attente);
        Ok(None)
    }
}

/// **SYNCHRO-2** : au retour du sommeil, une connexion morte sans le dire se révèle tout de
/// suite — un ping part au tour suivant, au lieu d'attendre ses 20 s.
#[test]
fn synchro_2_eprouve_un_ping_part_tout_de_suite() {
    let (lien, mut fil) = un_fil();
    let (envoi, parti) = mpsc::channel();
    let arret = Arc::clone(&lien.arret);
    let tourne = std::thread::spawn(move || fil.servir(Box::new(Espion(envoi)), &arret));
    lien.eprouver();
    let debut = Instant::now();
    let mut ping = false;
    while !ping && debut.elapsed() < Duration::from_secs(5) {
        ping = parti
            .recv_timeout(Duration::from_millis(50))
            .is_ok_and(|m| m == "ping");
    }
    drop(lien);
    let _ = tourne.join();
    assert!(ping, "aucun ping avant 5 s : le lien attendait ses 20 s");
}

/// **SYNCHRO-2** : coupé, le lien pressé n'attend pas sa pause (jusqu'à 60 s) — une fois.
#[test]
fn synchro_2_eprouve_coupe_la_pause_une_fois() {
    let (lien, fil) = un_fil();
    lien.eprouver();
    let debut = Instant::now();
    assert!(fil.patienter(Duration::from_secs(60), &AtomicBool::new(false)));
    assert!(
        debut.elapsed() < Duration::from_secs(1),
        "{:?}",
        debut.elapsed()
    );
    assert!(!fil.patienter(Duration::from_millis(150), &AtomicBool::new(false)));
}
