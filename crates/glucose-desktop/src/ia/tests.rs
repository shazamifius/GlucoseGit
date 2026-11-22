//! Les épreuves des IA gratuites (fiches 128 et 129) : le classement, la loi de Gemini, la bascule,
//! la discussion, et la clé — sans le réseau, sur des réponses réelles de Google recopiées le 22/11.

use super::bascule::{self, Bascule, Issue};
use super::cles::{self, Cles};
use super::discussion;
use super::gemini::{self, Refus};
use super::modeles::{self, Fournisseur, Modele};
use crate::agents::json::Json;
use std::time::Duration;

/// Les noms que la clé de l'utilisateur ouvrait le 22/11 (`supportedGenerationMethods` compris).
const LISTE_DU_22_11: [&str; 30] = [
    "gemini-2.5-flash",
    "gemini-2.5-pro",
    "gemini-2.5-flash-preview-tts",
    "gemma-4-26b-a4b-it",
    "gemma-4-31b-it",
    "gemini-flash-latest",
    "gemini-pro-latest",
    "gemini-2.5-flash-lite",
    "gemini-2.5-flash-image",
    "gemini-3-flash-preview",
    "gemini-3.1-pro-preview",
    "gemini-3.1-pro-preview-customtools",
    "gemini-3.1-flash-lite",
    "gemini-3.1-flash-lite-image",
    "gemini-nano-banana-2.1",
    "gemini-3.5-flash",
    "gemini-3.5-flash-lite",
    "gemini-omni-1.2-flash",
    "gemini-3.5-transcribe",
    "gemini-3.6-flash",
    "gemini-3.7-flash",
    "gemini-3.8-flash",
    "gemini-4-argon",
    "gemini-4-hydrogen",
    "gemini-3.8-flash-tts",
    "gemini-robotics-er-2-preview",
    "gemini-2.5-computer-use-preview-10-2025",
    "antigravity-preview-09-2026",
    "deep-research-preview-04-2026",
    "lyria-3.5",
];

fn liste(noms: &[&str]) -> Json {
    let modeles = noms
        .iter()
        .map(|n| {
            Json::objet([
                ("name", Json::from(format!("models/{n}"))),
                ("displayName", Json::from(format!("Nom de {n}"))),
                (
                    "supportedGenerationMethods",
                    Json::Liste(vec![Json::from("generateContent")]),
                ),
            ])
        })
        .collect();
    Json::objet([("models", Json::Liste(modeles))])
}

/// **Le classement de la fiche 129** : la version qui suit `gemini-` d'abord, la gamme ensuite,
/// Gemma en dernier ; ni les familles à part, ni les spécialités, ni les alias.
#[test]
fn test_les_modeles_de_conversation_du_22_11_dans_l_ordre() {
    let ids: Vec<String> = modeles::gemini(&liste(&LISTE_DU_22_11))
        .into_iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(
        ids,
        [
            "gemini-4-argon",
            "gemini-4-hydrogen",
            "gemini-3.8-flash",
            "gemini-3.7-flash",
            "gemini-3.6-flash",
            "gemini-3.5-flash",
            "gemini-3.5-flash-lite",
            "gemini-3.1-pro-preview",
            "gemini-3.1-flash-lite",
            "gemini-3-flash-preview",
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "gemini-2.5-flash-lite",
            "gemma-4-26b-a4b-it",
            "gemma-4-31b-it",
        ]
    );
}

/// **Preuve à l'envers** : la loi de la fiche 128 lisait les deux premiers nombres du nom — le
/// robot (« 2 ») et l'omni (« 1.2 ») entraient, et passaient devant des modèles de conversation.
#[test]
fn test_une_famille_a_part_ne_prend_pas_son_numero_pour_une_version() {
    let ancienne = |id: &str| {
        let mut n = id
            .trim_start_matches("gemini-")
            .split(['-', '.'])
            .filter_map(|m| m.parse::<u32>().ok());
        (n.next().unwrap_or(0), n.next().unwrap_or(0))
    };
    assert_eq!(ancienne("gemini-robotics-er-2-preview"), (2, 0));
    assert_eq!(ancienne("gemini-omni-1.2-flash"), (1, 2));
    assert_eq!(modeles::rang("gemini-robotics-er-2-preview"), None);
    assert_eq!(modeles::rang("gemini-omni-1.2-flash"), None);
    assert_eq!(modeles::rang("gemini-flash-latest"), None);
    assert!(modeles::rang("gemma-4-31b-it") < modeles::rang("gemini-2.5-flash-lite"));
}

#[test]
fn test_le_nom_qu_il_se_donne_est_celui_qu_on_lit() {
    let m = &modeles::gemini(&liste(&["gemini-3.8-flash"]))[0];
    assert_eq!(m.nom, "Nom de gemini-3.8-flash");
    assert_eq!(m.fournisseur, Fournisseur::Google);
    assert!(modeles::gemini(&Json::lire("{}").unwrap()).is_empty());
}

/// La réponse réelle de `gemini-3.5-flash-lite` du 22/11 à un appel d'outil.
const TOUR_DU_22_11: &str = r#"{"candidates":[{"content":{"parts":[{"functionCall":{"name":"glucose_poser_texte","args":{"texte":"bonjour"},"id":"call_17555550451108075839"},"thoughtSignature":"Eo4BCosBAWkUfRNPhhLWvVNwR4BA2dNy"}],"role":"model"},"finishReason":"STOP","index":0}],"modelVersion":"gemini-3.5-flash-lite"}"#;

/// **L'histoire se renvoie telle que Gemini l'a donnée** : le `Content` gardé est celui de la
/// réponse, sa signature de pensée comprise, à l'octet.
#[test]
fn test_un_tour_garde_son_contenu_intact_et_lit_son_appel() {
    let reponse = Json::lire(TOUR_DU_22_11).unwrap();
    let tour = gemini::lire_le_tour(&reponse).unwrap();
    let attendu = reponse.get("candidates").unwrap().as_liste().unwrap()[0]
        .get("content")
        .unwrap();
    assert_eq!(&tour.contenu, attendu);
    assert!(tour.contenu.ecrire().contains("thoughtSignature"));
    assert_eq!(tour.texte, "");
    assert_eq!(tour.appels.len(), 1);
    assert_eq!(tour.appels[0].nom, "glucose_poser_texte");
    assert_eq!(
        tour.appels[0].id.as_deref(),
        Some("call_17555550451108075839")
    );
    assert_eq!(
        tour.appels[0].arguments.get("texte").and_then(Json::as_str),
        Some("bonjour")
    );
}

/// Gemma écrit sa pensée en clair (`thought: true`) : ce n'est pas sa réponse.
#[test]
fn test_la_pensee_en_clair_ne_se_dit_pas() {
    let r = Json::lire(r#"{"candidates":[{"content":{"role":"model","parts":[{"text":"The user wants…","thought":true},{"text":"Voilà."}]}}]}"#).unwrap();
    assert_eq!(gemini::lire_le_tour(&r).unwrap().texte, "Voilà.");
    let vide = Json::lire(
        r#"{"candidates":[{"finishReason":"MALFORMED_FUNCTION_CALL","content":{"parts":[]}}]}"#,
    )
    .unwrap();
    assert!(gemini::lire_le_tour(&vide)
        .unwrap_err()
        .contains("MALFORMED_FUNCTION_CALL"));
    let bloque = Json::lire(r#"{"promptFeedback":{"blockReason":"SAFETY"}}"#).unwrap();
    assert!(gemini::lire_le_tour(&bloque)
        .unwrap_err()
        .contains("SAFETY"));
}

/// Le 429 réel de `gemini-4-argon`, le 22/11 à 18 h 09 (abrégé) : « limit: 0 », et le délai.
const REFUS_DU_22_11: &str = r#"{"error":{"code":429,"message":"You exceeded your current quota, please check your plan and billing details.\n* Quota exceeded for metric: generate_content_free_tier_requests, limit: 0","status":"RESOURCE_EXHAUSTED","details":[{"@type":"type.googleapis.com/google.rpc.Help","links":[]},{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[]},{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"53509s"}]}}"#;

#[test]
fn test_un_refus_dit_quand_revenir() {
    let r = gemini::lire_le_refus(429, REFUS_DU_22_11);
    assert_eq!(r.statut, 429);
    assert_eq!(r.revenir_dans, Some(Duration::from_secs(53509)));
    assert_eq!(
        r.message,
        "You exceeded your current quota, please check your plan and billing details."
    );
    let court = gemini::lire_le_refus(429, r#"{"error":{"details":[{"retryDelay":"7.25s"}]}}"#);
    assert_eq!(court.revenir_dans, Some(Duration::from_millis(7250)));
    let nu = gemini::lire_le_refus(503, "pas du json");
    assert_eq!((nu.revenir_dans, nu.message.as_str()), (None, "code 503"));
}

/// **Les outils sont ceux de la porte** : chaque outil qu'une IA du nuage peut appeler, avec son
/// schéma entier — aucune liste à part, qui divergerait.
#[test]
fn test_les_outils_de_gemini_sont_ceux_de_la_porte() {
    use crate::agents::outils::{self, Portee};
    let porte = outils::liste_pour(Portee::publique(true));
    let attendus = porte.get("tools").unwrap().as_liste().unwrap();
    let gemini = gemini::outils(&porte);
    let declarations = gemini.as_liste().unwrap()[0]
        .get("functionDeclarations")
        .unwrap()
        .as_liste()
        .unwrap();
    assert_eq!(declarations.len(), attendus.len());
    assert!(!declarations.is_empty());
    for (d, o) in declarations.iter().zip(attendus) {
        assert_eq!(d.get("name"), o.get("name"));
        assert_eq!(d.get("parametersJsonSchema"), o.get("inputSchema"));
        assert!(matches!(
            d.get("parametersJsonSchema"),
            Some(Json::Objet(_))
        ));
    }
    // Ce qu'une IA du nuage n'a pas, Gemini ne l'a pas non plus.
    assert!(declarations
        .iter()
        .all(|d| d.get("name").and_then(Json::as_str) != Some("glucose_domaines")));
}

#[test]
fn test_une_question_sans_outils_n_en_annonce_aucun() {
    let h = [gemini::de_l_utilisateur(vec![gemini::texte("bonjour")])];
    assert!(!gemini::corps("c", &h, &Json::Liste(Vec::new())).contains("tools"));
    let avec = gemini::outils(&crate::agents::outils::liste());
    assert!(gemini::corps("c", &h, &avec).contains("functionDeclarations"));
}

#[test]
fn test_la_reponse_d_un_outil_rend_son_identifiant() {
    let appel = gemini::AppelDuModele {
        id: Some("call_1".into()),
        nom: "glucose_lire".into(),
        arguments: Json::Objet(Vec::new()),
    };
    let ok = gemini::reponse_d_outil(&appel, Ok(Json::from("rien"))).ecrire();
    assert_eq!(
        ok,
        r#"{"functionResponse":{"name":"glucose_lire","id":"call_1","response":{"result":"rien"}}}"#
    );
    let ko = gemini::reponse_d_outil(&appel, Err("non".into())).ecrire();
    assert!(ko.contains(r#""response":{"error":"non"}"#));
}

// ── La bascule ──────────────────────────────────────────────────────────────

fn trois() -> Bascule {
    let m = |id: &str, nom: &str| Modele {
        fournisseur: Fournisseur::Google,
        id: id.into(),
        nom: nom.into(),
        rang: 0,
    };
    Bascule::nouvelle(vec![m("a", "Alpha"), m("b", "Bêta"), m("c", "Gamma")])
}

fn refus(statut: u16, revenir_dans: Option<Duration>) -> Issue {
    Issue::Refuse(Refus {
        statut,
        message: format!("refus {statut}"),
        revenir_dans,
    })
}

const MIDI: u64 = 1_800_000_000_000;

/// **BASCULE-1 et 2, sa demande du 21/11** : le premier atteint sa limite, le second prend le
/// relais **et le dit** ; quand le premier se réveille — à l'heure que Google a donnée —, il
/// reprend la main, **et c'est dit aussi**.
#[test]
fn test_bascule_le_relais_puis_le_retour_se_disent() {
    let mut b = trois();
    assert_eq!(b.premier(MIDI), Some(0));
    assert_eq!(b.apprendre(0, &Issue::Repondu, MIDI), Ok(None));
    let heure = Duration::from_secs(3600);
    b.apprendre(0, &refus(429, Some(heure)), MIDI).unwrap();
    assert_eq!(b.premier(MIDI), Some(1));
    let relais = b.apprendre(1, &Issue::Repondu, MIDI).unwrap().unwrap();
    assert_eq!(
        relais,
        "Alpha ne répond plus (sa limite gratuite est atteinte, il revient dans 1 h 00) : Bêta \
         prend le relais."
    );
    // Une seconde avant son heure, il dort encore ; à son heure, il est le premier.
    assert_eq!(b.premier(MIDI + 3_599_000), Some(1));
    assert_eq!(b.premier(MIDI + 3_600_000), Some(0));
    let retour = b.apprendre(0, &Issue::Repondu, MIDI + 3_600_000).unwrap();
    assert_eq!(
        retour.as_deref(),
        Some("Alpha est de retour : il reprend la main.")
    );
    // Le même qui répond encore ne se redit pas.
    assert_eq!(b.apprendre(0, &Issue::Repondu, MIDI + 3_601_000), Ok(None));
}

/// Un 503 cède **cette question** ; à la suivante, il est de nouveau le premier — aucune durée
/// choisie.
#[test]
fn test_bascule_une_surcharge_cede_la_question_seulement() {
    let mut b = trois();
    b.apprendre(0, &refus(503, None), MIDI).unwrap();
    assert_eq!(b.premier(MIDI), Some(1));
    b.nouvelle_question();
    assert_eq!(b.premier(MIDI), Some(0));
    // Un 429 sans délai, un 400 : pareil — un 400 ne vide pas la liste.
    for statut in [429, 400, 500] {
        b.apprendre(0, &refus(statut, None), MIDI).unwrap();
        assert_eq!(b.premier(MIDI), Some(1), "{statut}");
        b.nouvelle_question();
        assert_eq!(b.premier(MIDI), Some(0), "{statut}");
    }
}

/// Un 404 sort pour de bon ; un 403 arrête tout : la clé est refusée.
#[test]
fn test_bascule_un_modele_retire_sort_une_cle_refusee_arrete() {
    let mut b = trois();
    b.apprendre(0, &refus(404, None), MIDI).unwrap();
    b.nouvelle_question();
    assert_eq!(b.premier(MIDI + 86_400_000), Some(1));
    let arret = b.apprendre(1, &refus(403, None), MIDI).unwrap_err();
    assert!(arret.starts_with("Google refuse la clé"));
}

#[test]
fn test_bascule_personne_dit_quand_le_premier_revient() {
    let mut b = trois();
    b.apprendre(0, &refus(429, Some(Duration::from_secs(150))), MIDI)
        .unwrap();
    b.apprendre(1, &refus(429, Some(Duration::from_secs(40))), MIDI)
        .unwrap();
    b.apprendre(2, &refus(404, None), MIDI).unwrap();
    assert_eq!(b.premier(MIDI), None);
    assert_eq!(
        b.personne(MIDI),
        "Aucun modèle gratuit ne répond pour l'instant : le premier revient dans 40 s."
    );
    assert_eq!(bascule::dans(MIDI + 125_000, MIDI), "dans 3 min");
    assert_eq!(bascule::dans(MIDI + 53_509_000, MIDI), "dans 14 h 51");
}

// ── La discussion ───────────────────────────────────────────────────────────

/// **DISCUSSION-2** : après une question interrompue, la suivante rejoint le message de
/// l'utilisateur au lieu d'en ouvrir un second.
#[test]
fn test_une_question_rejoint_le_dernier_message_de_l_utilisateur() {
    let mut h = Vec::new();
    discussion::ajouter_la_question(&mut h, "un");
    discussion::ajouter_la_question(&mut h, "deux");
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].get("parts").unwrap().as_liste().unwrap().len(), 2);
    h.push(Json::lire(r#"{"role":"model","parts":[{"text":"oui"}]}"#).unwrap());
    discussion::ajouter_la_question(&mut h, "trois");
    assert_eq!(h.len(), 3);
}

/// **DISCUSSION-1** : la ligne que la porte écrit à tout agent se relit en ce que l'outil a rendu —
/// par l'écrivain même de la porte.
#[test]
fn test_la_ligne_de_la_porte_se_relit_en_ce_que_l_outil_a_rendu() {
    use crate::agents::outils::Portee;
    use crate::agents::protocole;
    let appel = protocole::Appel {
        id: Json::Nombre(1.0),
        nom: "glucose_lire".into(),
        arguments: Json::Objet(Vec::new()),
        client: "Gemini".into(),
        moderne: true,
        portee: Portee::publique(true),
    };
    let rendu = Json::objet([("noeuds", Json::Liste(vec![Json::from("text-1")]))]);
    let ligne = protocole::reponse_d_outil(&appel, Ok(rendu.clone()));
    assert_eq!(discussion::issue_de_la_ligne(&ligne), Ok(rendu));
    let ligne = protocole::reponse_d_outil(&appel, Err("aucun nœud text-9".into()));
    assert_eq!(
        discussion::issue_de_la_ligne(&ligne),
        Err("aucun nœud text-9".into())
    );
}

/// **Une conversation se décrit sans sa clé** : elle ne paraît dans aucun journal, même d'épreuve.
#[test]
fn test_une_conversation_se_decrit_sans_sa_cle() {
    let temoin = format!(
        "{:?}",
        discussion::Conversation::nouvelle("CLE-SECRETE".into())
    );
    assert!(!temoin.contains("CLE-SECRETE"), "{temoin}");
    assert!(temoin.starts_with("Conversation"));
}

// ── La clé ──────────────────────────────────────────────────────────────────

fn dossier_d_essai(nom: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("glucose-ia-{nom}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn test_la_cle_se_relit_telle_qu_ecrite() {
    let d = dossier_d_essai("relire");
    let chemin = d.join("ia-cles.json");
    let cles = Cles {
        google: Some("AQ.exemple-google".into()),
    };
    cles::ecrire_dans(&chemin, &cles).unwrap();
    assert_eq!(cles::lire_depuis(&chemin), cles);
}

#[test]
fn test_un_fichier_absent_casse_ou_vide_ne_branche_rien() {
    let d = dossier_d_essai("absent");
    assert_eq!(cles::lire_depuis(&d.join("rien.json")), Cles::default());
    let casse = d.join("casse.json");
    std::fs::write(&casse, "{ pas du json").unwrap();
    assert_eq!(cles::lire_depuis(&casse), Cles::default());
    std::fs::write(&casse, r#"{"google":"   "}"#).unwrap();
    assert_eq!(cles::lire_depuis(&casse), Cles::default());
}

#[cfg(unix)]
#[test]
fn test_le_fichier_de_la_cle_est_a_ses_seuls_droits() {
    use std::os::unix::fs::PermissionsExt;
    let d = dossier_d_essai("droits");
    let chemin = d.join("ia-cles.json");
    let cle = Cles {
        google: Some("k".into()),
    };
    cles::ecrire_dans(&chemin, &cle).unwrap();
    let mode = std::fs::metadata(&chemin).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

// ── Le vrai réseau, à la main ───────────────────────────────────────────────

/// Attend la fin d'une question, et rend tout ce qui est arrivé.
fn jusqu_a_la_fin(
    en_route: &discussion::EnRoute,
    boite: Option<&crate::agents::Boite>,
) -> Vec<discussion::Evenement> {
    let mut tout = Vec::new();
    loop {
        // Les appels d'outils reçoivent une réponse d'essai, comme la boucle la donnerait.
        if let Some(b) = boite {
            for d in b.relever() {
                let rendu = Json::objet([("id", Json::from("texte-1"))]);
                let _ = d.reponse.send(crate::agents::protocole::reponse_d_outil(
                    &d.appel,
                    Ok(rendu),
                ));
            }
        }
        match en_route.evenements.recv_timeout(Duration::from_millis(50)) {
            Ok(e @ discussion::Evenement::Fin { .. }) => {
                tout.push(e);
                return tout;
            }
            Ok(e) => tout.push(e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => return tout,
        }
    }
}

/// **Le vrai Google** : `GLUCOSE_CLE_GOOGLE` porte la clé, écrite nulle part. Une question qui
/// demande un geste : l'appel passe par la boîte, la réponse revient, le modèle conclut. Coûte deux
/// ou trois requêtes au premier modèle qui répond — à la main seulement :
/// `cargo test -p glucose-desktop -- --ignored essai_reel`.
#[test]
#[ignore = "touche le réseau et une vraie clé"]
fn essai_reel_une_question_qui_pose_un_texte() {
    // La conversation se décrit sans sa clé : ce journal d'épreuve peut l'imprimer.
    let temoin = format!(
        "{:?}",
        discussion::Conversation::nouvelle("CLE-SECRETE".into())
    );
    assert!(!temoin.contains("CLE-SECRETE"));
    let cle = std::env::var("GLUCOSE_CLE_GOOGLE").expect("GLUCOSE_CLE_GOOGLE");
    let boite = crate::agents::Boite::nouvelle(std::sync::Arc::new(|| {}));
    let c = discussion::Conversation::nouvelle(cle);
    let en_route = discussion::poser(
        c,
        "Pose un texte qui dit « bonjour » sur le tableau.".into(),
        boite.clone(),
    );
    let tout = jusqu_a_la_fin(&en_route, Some(&boite));
    for e in &tout {
        eprintln!("{e:?}");
    }
    assert!(tout
        .iter()
        .any(|e| matches!(e, discussion::Evenement::Geste { echec: None, .. })));
    let Some(discussion::Evenement::Fin {
        erreur,
        conversation,
    }) = tout.last()
    else {
        panic!("pas de fin");
    };
    assert_eq!(erreur, &None);
    assert!(conversation.histoire.len() >= 3);
}
