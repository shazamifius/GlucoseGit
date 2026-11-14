//! **PUBLIQUE-1, de bout en bout** dans le vrai Glucose sans fenêtre : un faux relais sur cette
//! machine, « Collaborer », la porte ouverte, une IA qui s'inscrit et demande la porte, la
//! question avec son code, la clé, puis un geste signé de son nom — et la porte retirée, fermée.

use crate::agents::json::Json;
use crate::agents::publique::accords::{self, Etat};
use crate::agents::publique::http::{formulaire, pourcent};
use crate::agents::publique::lien::EtatDuLien;
use crate::agents::Boite;
use crate::app::agents::Agents;
use crate::app::GlucoseApp;
use crate::interactions::question::tests::repondre;
use crate::plateforme::websocket::trames::{self, Decodeur, Trame};
use crate::ui::question::{Reponse, Suite};
use glucose_core::encodage::base64_adresse;
use glucose_core::hash::sha256;
use glucose_core::store::journal::Main;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

const RETOUR: &str = "https://claude.ai/api/mcp/auth_callback";
const VERIFICATEUR: &str = "un-verificateur-pkce-de-plus-de-quarante-trois-signes-ici";

/// **Un faux relais** : il prend le WebSocket de Glucose, répond `pong` à `ping`, passe à Glucose
/// les requêtes que l'épreuve lui confie et rend ses réponses — et dit ce qu'il a vu.
struct FauxRelais {
    racine: String,
    requetes: mpsc::Sender<String>,
    reponses: mpsc::Receiver<String>,
    /// Ce que Glucose a présenté à l'ouverture : le chemin, puis le secret.
    ouverture: mpsc::Receiver<(String, String)>,
    /// Glucose a dit au revoir.
    fermeture: mpsc::Receiver<()>,
}

fn faux_relais() -> FauxRelais {
    let ecoute = TcpListener::bind("127.0.0.1:0").expect("un port local");
    let racine = format!("http://{}", ecoute.local_addr().expect("son adresse"));
    let (requetes, a_passer) = mpsc::channel::<String>();
    let (rendre, reponses) = mpsc::channel();
    let (dire_l_ouverture, ouverture) = mpsc::channel();
    let (dire_la_fermeture, fermeture) = mpsc::channel();
    std::thread::spawn(move || {
        let (mut tcp, _) = ecoute.accept().expect("Glucose se relie");
        let mut lus = Vec::new();
        let mut octet = [0u8; 1];
        while !lus.ends_with(b"\r\n\r\n") {
            tcp.read_exact(&mut octet).expect("en-têtes");
            lus.push(octet[0]);
        }
        let demande = String::from_utf8_lossy(&lus).into_owned();
        let entete = |nom: &str| {
            demande
                .lines()
                .find_map(|l| l.strip_prefix(nom))
                .unwrap_or_default()
                .trim()
                .to_string()
        };
        let chemin = demande.split(' ').nth(1).unwrap_or_default().to_string();
        let secret = entete("Authorization: Bearer ");
        let _ = dire_l_ouverture.send((chemin, secret));
        let reponse = format!(
            "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\r\n",
            trames::acceptation(&entete("Sec-WebSocket-Key: "))
        );
        tcp.write_all(reponse.as_bytes()).expect("ouverture");
        tcp.set_read_timeout(Some(Duration::from_millis(10))).ok();
        let mut d = Decodeur::d_un_client();
        let mut tampon = [0u8; 65536];
        loop {
            while let Ok(r) = a_passer.try_recv() {
                tcp.write_all(&trames::ecrire(&Trame::Texte(r), None))
                    .expect("passer");
            }
            match tcp.read(&mut tampon) {
                Ok(0) => return,
                Ok(n) => d.pousser(&tampon[..n]),
                Err(_) => {}
            }
            while let Ok(Some(t)) = d.suivante() {
                match t {
                    Trame::Texte(p) if p == "ping" => {
                        tcp.write_all(&trames::ecrire(&Trame::Texte("pong".into()), None))
                            .ok();
                    }
                    Trame::Texte(r) => {
                        let _ = rendre.send(r);
                    }
                    Trame::Fermeture(_) => {
                        let _ = dire_la_fermeture.send(());
                        return;
                    }
                    _ => {}
                }
            }
        }
    });
    FauxRelais {
        racine,
        requetes,
        reponses,
        ouverture,
        fermeture,
    }
}

impl FauxRelais {
    /// **Passe une requête à Glucose**, et fait tourner sa boucle jusqu'à la réponse — un appel
    /// d'outil l'attend.
    fn passer(
        &self,
        app: &mut GlucoseApp,
        (methode, chemin): (&str, &str),
        cle: Option<&str>,
        corps: &str,
    ) -> (u16, Json) {
        let mut entetes = vec![("content-type", Json::from("application/json"))];
        let porteur = cle.map(|c| format!("Bearer {c}"));
        if let Some(p) = &porteur {
            entetes.push(("authorization", Json::from(p.as_str())));
        }
        let requete = Json::objet([
            ("n", Json::Nombre(1.0)),
            ("methode", Json::from(methode)),
            ("chemin", Json::from(chemin)),
            ("entetes", Json::objet(entetes)),
            ("corps", Json::from(corps)),
        ]);
        self.requetes.send(requete.ecrire()).expect("le relais vit");
        let debut = Instant::now();
        loop {
            app.suivre_les_agents();
            app.suivre_la_porte_publique();
            if let Ok(r) = self.reponses.try_recv() {
                let r = Json::lire(&r).expect("une réponse en JSON");
                let statut = r.get("statut").and_then(Json::as_f64).expect("un statut") as u16;
                let corps = r.get("corps").and_then(Json::as_str).unwrap_or_default();
                return (statut, Json::lire(corps).unwrap_or(Json::Nul));
            }
            assert!(
                debut.elapsed() < Duration::from_secs(10),
                "Glucose ne répond pas à {chemin}"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

fn texte_de_la_question(app: &GlucoseApp) -> String {
    app.ui
        .question
        .as_ref()
        .map(|(q, _)| q.texte.clone())
        .unwrap_or_default()
}

fn attendre_le_lien(app: &GlucoseApp) {
    let debut = Instant::now();
    while app.lancement.agents.publique.as_ref().map(|l| l.etat()) != Some(EtatDuLien::Relie) {
        assert!(
            debut.elapsed() < Duration::from_secs(10),
            "le lien ne se relie pas"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn mcp(id: u32, outil: &str, arguments: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"{outil}","arguments":{arguments},"_meta":{{"io.modelcontextprotocol/protocolVersion":"2026-07-28"}}}}}}"#
    )
}

/// Une requête moderne porte ses en-têtes ; le faux relais ne les passe pas : la voie ancienne
/// (sans `_meta`) suffit à l'épreuve.
fn mcp_ancien(id: u32, outil: &str, arguments: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"{outil}","arguments":{arguments}}}}}"#
    )
}

#[test]
fn publique_1_une_ia_du_nuage_entre_par_le_clic_de_l_utilisateur_signe_puis_sort() {
    let relais = faux_relais();
    let dossier = std::env::temp_dir().join(format!("glucose-publique-1-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dossier);
    std::fs::create_dir_all(&dossier).expect("un dossier");
    let mut app = GlucoseApp::new();
    app.disque.porte_publique = dossier.join(accords::FICHIER);
    app.lancement.agents = Agents::avec_la_boite(Boite::nouvelle(Arc::new(|| {})));
    app.lancement.agents.relais = Some(relais.racine.clone());

    // « Collaborer » : fermée, puis ouverte — l'adresse se montre, le bouton s'allume.
    app.montrer_la_porte_publique();
    assert!(texte_de_la_question(&app).starts_with("Fermée."));
    repondre(&mut app, Reponse::Oui);
    assert!(app.ui.porte_ouverte);
    let barre = crate::ui::layout_topbar(1920.0, &app.ui, &app.renderer.typography, 0);
    let collaborer = barre
        .buttons
        .iter()
        .find(|b| b.action == crate::ui::UiAction::ToggleCollab);
    assert!(collaborer.is_some_and(|b| b.active), "Collaborer s'allume");
    let etat = Etat::lire(&std::fs::read_to_string(&app.disque.porte_publique).expect("écrit"))
        .expect("lisible");
    assert!(etat.ouverte);
    let adresse = format!("{}/g/{}/mcp", relais.racine, etat.identifiant());
    assert!(
        texte_de_la_question(&app).contains(&adresse),
        "{}",
        texte_de_la_question(&app)
    );
    repondre(&mut app, Reponse::Annuler);

    // Glucose se relie lui-même, et prouve son identifiant par son secret.
    let (chemin, secret) = relais
        .ouverture
        .recv_timeout(Duration::from_secs(10))
        .expect("relié");
    assert_eq!(chemin, format!("/g/{}/glucose", etat.identifiant()));
    assert_eq!(secret, etat.secret_texte());
    attendre_le_lien(&app);

    // Une IA s'inscrit, demande la porte : la question montre son nom, d'où elle vient, le code.
    let inscription = format!(r#"{{"client_name":"Claude","redirect_uris":["{RETOUR}"]}}"#);
    let (s, j) = relais.passer(&mut app, ("POST", "/register"), None, &inscription);
    assert_eq!(s, 201);
    let client_id = j
        .get("client_id")
        .and_then(Json::as_str)
        .expect("inscrite")
        .to_string();
    let defi = base64_adresse(&sha256(VERIFICATEUR.as_bytes()));
    let params = Json::objet([
        ("response_type", Json::from("code")),
        ("client_id", Json::from(client_id.as_str())),
        ("redirect_uri", Json::from(RETOUR)),
        ("code_challenge", Json::from(defi)),
        ("code_challenge_method", Json::from("S256")),
        ("state", Json::from("s1")),
    ]);
    let (s, d) = relais.passer(
        &mut app,
        ("POST", "/authorize/demande"),
        None,
        &params.ecrire(),
    );
    assert_eq!(s, 200, "{d:?}");
    let code_affiche = d
        .get("code")
        .and_then(Json::as_str)
        .expect("un code")
        .to_string();
    let demande = d
        .get("demande")
        .and_then(Json::as_str)
        .expect("demande")
        .to_string();
    app.suivre_la_porte_publique();
    let Some((q, Suite::AccordPublic(_))) = &app.ui.question else {
        panic!("la question d'accord");
    };
    assert!(
        q.texte.contains("« Claude », depuis claude.ai"),
        "{}",
        q.texte
    );
    let espace: Vec<String> = code_affiche.chars().map(String::from).collect();
    assert!(
        q.texte.contains(&espace.join(" ")),
        "le code de sa page : {}",
        q.texte
    );
    repondre(&mut app, Reponse::Choix(0));

    // La page reçoit le code d'autorisation ; l'IA l'échange contre sa clé.
    let attente = Json::objet([("demande", Json::from(demande))]).ecrire();
    let (_, e) = relais.passer(&mut app, ("POST", "/authorize/attente"), None, &attente);
    let redirection = e
        .get("redirection")
        .and_then(Json::as_str)
        .expect("accordée")
        .to_string();
    let code = formulaire(redirection.split_once('?').expect("paramètres").1)
        .into_iter()
        .find(|(k, _)| k == "code")
        .expect("un code")
        .1;
    let echange = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&code_verifier={}",
        pourcent(&code),
        pourcent(RETOUR),
        pourcent(&client_id),
        VERIFICATEUR
    );
    let (s, t) = relais.passer(&mut app, ("POST", "/token"), None, &echange);
    assert_eq!(s, 200, "{t:?}");
    let cle = t
        .get("access_token")
        .and_then(Json::as_str)
        .expect("une clé")
        .to_string();

    // Son geste : posé, signé « Claude (claude.ai) », et le témoin le dit.
    let pose = mcp_ancien(
        1,
        "glucose_poser_texte",
        r#"{"id":"du-nuage","texte":"Bonjour depuis le nuage"}"#,
    );
    let (s, r) = relais.passer(&mut app, ("POST", "/mcp"), Some(&cle), &pose);
    assert_eq!(s, 200, "{r:?}");
    let b = app.store.project.active_board_id.clone();
    assert!(app.store.project.annotation(&b, "du-nuage").is_some());
    let mains: Vec<Main> = app
        .store
        .journal
        .prendre_les_ecrits_signes()
        .into_iter()
        .map(|e| e.main)
        .collect();
    assert_eq!(mains, [Main::Agent("Claude (claude.ai)".into())]);
    assert_eq!(
        app.ui.toast_message(),
        Some("Claude (claude.ai) travaille dans ton canevas")
    );

    // Elle ne lit pas les fichiers de cet ordinateur : une vraie image, qu'un agent d'ici poserait.
    let image = dossier.join("capture.png");
    image::RgbaImage::from_pixel(4, 4, image::Rgba([200, 30, 30, 255]))
        .save(&image)
        .expect("une image");
    let arguments = Json::objet([
        ("id", Json::from("page-du-nuage")),
        ("adresse", Json::from("https://exemple.org")),
        ("capture", Json::from(image.display().to_string())),
    ]);
    let capture = mcp_ancien(2, "glucose_poser_page", &arguments.ecrire());
    let photos = |app: &GlucoseApp| app.store.active_board().map_or(0, |t| t.images.len());
    let avant = photos(&app);
    let (_, r) = relais.passer(&mut app, ("POST", "/mcp"), Some(&cle), &capture);
    let resultat = r.get("result").expect("un résultat");
    assert_eq!(resultat.get("isError"), Some(&Json::Bool(true)), "{r:?}");
    assert_eq!(photos(&app), avant, "aucune photo posée");
    // Ni les domaines de tes autres fichiers : celui d'un fichier voisin ne se nomme pas.
    let mut voisin = GlucoseApp::new();
    let pose = r#"{"id":"v","texte":"voisin","domaines":[{"nouveau":"secret-du-voisin"}]}"#;
    voisin
        .executer_pour_un_agent("glucose_poser_texte", &Json::lire(pose).expect("JSON"))
        .expect("posé chez le voisin");
    let recense = glucose_core::recensement::recenser(
        "/home/quelqu-un/voisin.glucose",
        &voisin.store.project,
    );
    app.disque.recenseur.etat.poser(recense);
    let lire = mcp_ancien(5, "glucose_lire", r#"{"domaine":"absent"}"#);
    let (_, r) = relais.passer(&mut app, ("POST", "/mcp"), Some(&cle), &lire);
    let dit = r.ecrire();
    assert!(dit.contains("absent"), "{dit}");
    assert!(
        !dit.contains("secret-du-voisin") && !dit.contains("quelqu-un"),
        "{dit}"
    );
    // Une requête moderne sans ses en-têtes est refusée, sans rien poser.
    let (s, _) = relais.passer(
        &mut app,
        ("POST", "/mcp"),
        Some(&cle),
        &mcp(3, "glucose_lire", "{}"),
    );
    assert_eq!(s, 400);

    // Retirée : sa clé ne vaut plus rien, et le fichier ne la connaît plus.
    app.montrer_la_porte_publique();
    assert!(texte_de_la_question(&app).contains("Claude (claude.ai) — lire et écrire"));
    repondre(&mut app, Reponse::Choix(1));
    repondre(&mut app, Reponse::Choix(0));
    let (s, _) = relais.passer(
        &mut app,
        ("POST", "/mcp"),
        Some(&cle),
        &mcp_ancien(4, "glucose_lire", "{}"),
    );
    assert_eq!(s, 401);
    let relu = Etat::lire(&std::fs::read_to_string(&app.disque.porte_publique).expect("écrit"))
        .expect("lisible");
    assert!(relu.accords.is_empty());

    // Fermée : le lien dit au revoir, le bouton s'éteint, le fichier garde la porte fermée.
    app.montrer_la_porte_publique();
    repondre(&mut app, Reponse::Choix(2));
    assert!(!app.ui.porte_ouverte);
    relais
        .fermeture
        .recv_timeout(Duration::from_secs(10))
        .expect("le lien s'est fermé");
    let relu = Etat::lire(&std::fs::read_to_string(&app.disque.porte_publique).expect("écrit"))
        .expect("lisible");
    assert!(!relu.ouverte);
    assert_eq!(
        relu.secret, etat.secret,
        "le secret, donc l'adresse, ne change pas"
    );
    let _ = std::fs::remove_dir_all(&dossier);
}
