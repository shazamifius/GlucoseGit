//! **Parler à Gemini** (fiche 129) : la question `generateContent`, ce qu'elle rend, ce que ses
//! refus disent. Pur — rien ne parle au réseau ici, tout s'éprouve sans clé.
//!
//! # L'histoire se renvoie telle que Gemini l'a donnée
//!
//! Gemini signe sa pensée : un appel d'outil porte une `thoughtSignature`, qu'il faut lui rendre
//! au tour suivant, faute de quoi il refuse (400 — Codex et VS Code l'ont payé en la perdant).
//! Glucose ne la lit pas : il garde chaque `Content` du modèle **intact** et le renvoie tel quel.
//! Rien à comprendre, rien à perdre. Et l'essai du 22/11 l'a montré : un **autre** modèle accepte
//! la même histoire, signatures comprises — la bascule peut se faire au milieu d'un appel d'outil.
//!
//! # Les outils sont ceux de la porte
//!
//! La liste même que `tools/list` rend aux IA du nuage (une seule liste, `agents::outils`), chaque
//! schéma passé tel quel : `parametersJsonSchema` prend le JSON Schema entier (`items: {}`
//! compris, essayé le 22/11), quand `parameters` n'en prend qu'un sous-ensemble.

use crate::agents::json::Json;
use std::time::Duration;

/// L'adresse de l'API de Google. `generateContent` reste « pleinement prise en charge » à côté de
/// l'API « Interactions », venue en juin 2026 et encore dite bêta par une partie de sa documentation.
pub const ADRESSE: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Les outils de Glucose dans la forme de Gemini, depuis la liste de la porte.
pub fn outils(liste: &Json) -> Json {
    let champ = |o: &Json, cle: &str| o.get(cle).cloned().unwrap_or(Json::Nul);
    let declarations = liste
        .get("tools")
        .and_then(Json::as_liste)
        .unwrap_or(&[])
        .iter()
        .map(|o| {
            Json::objet([
                ("name", champ(o, "name")),
                ("description", champ(o, "description")),
                ("parametersJsonSchema", champ(o, "inputSchema")),
            ])
        })
        .collect();
    Json::Liste(vec![Json::objet([(
        "functionDeclarations",
        Json::Liste(declarations),
    )])])
}

/// **Le corps d'une question** : la consigne, l'histoire entière, les outils — s'il y en a.
pub fn corps(consigne: &str, histoire: &[Json], outils: &Json) -> String {
    let mut champs = vec![
        (
            "systemInstruction",
            Json::objet([("parts", Json::Liste(vec![texte(consigne)]))]),
        ),
        ("contents", Json::Liste(histoire.to_vec())),
    ];
    if outils.as_liste().is_some_and(|l| !l.is_empty()) {
        champs.push(("tools", outils.clone()));
    }
    Json::objet(champs).ecrire()
}

/// Une part de texte.
pub fn texte(t: &str) -> Json {
    Json::objet([("text", Json::from(t))])
}

/// Un message de l'utilisateur — ce qu'il écrit, ou ce que les outils ont rendu.
pub fn de_l_utilisateur(parts: Vec<Json>) -> Json {
    Json::objet([("role", Json::from("user")), ("parts", Json::Liste(parts))])
}

/// Un appel d'outil que le modèle demande.
#[derive(Debug, Clone, PartialEq)]
pub struct AppelDuModele {
    pub id: Option<String>,
    pub nom: String,
    pub arguments: Json,
}

/// **Ce qu'un tour rend** : le `Content` du modèle, intact, pour l'histoire ; ce qu'il dit ; ce
/// qu'il demande aux outils.
#[derive(Debug, Clone, PartialEq)]
pub struct Tour {
    pub contenu: Json,
    pub texte: String,
    pub appels: Vec<AppelDuModele>,
}

/// **Lit une réponse 200.** Sa pensée en clair (`thought` : Gemma l'écrit) n'est pas sa réponse.
pub fn lire_le_tour(reponse: &Json) -> Result<Tour, String> {
    let candidat = reponse
        .get("candidates")
        .and_then(Json::as_liste)
        .and_then(<[Json]>::first);
    let Some(candidat) = candidat else {
        // Aucun candidat : la question elle-même est refusée.
        let raison = reponse
            .get("promptFeedback")
            .and_then(|p| p.get("blockReason"))
            .and_then(Json::as_str)
            .unwrap_or("aucune réponse");
        return Err(format!("Gemini n'a pas répondu ({raison})"));
    };
    let contenu = candidat.get("content").cloned().unwrap_or(Json::Nul);
    let mut dit = String::new();
    let mut appels = Vec::new();
    for part in contenu.get("parts").and_then(Json::as_liste).unwrap_or(&[]) {
        if part.get("thought").and_then(Json::as_bool) == Some(true) {
            continue;
        }
        if let Some(t) = part.get("text").and_then(Json::as_str) {
            dit.push_str(t);
        }
        if let Some(appel) = part.get("functionCall") {
            let chaine = |cle: &str| appel.get(cle).and_then(Json::as_str).map(str::to_string);
            appels.push(AppelDuModele {
                id: chaine("id"),
                nom: chaine("name").unwrap_or_default(),
                arguments: appel
                    .get("args")
                    .cloned()
                    .unwrap_or(Json::Objet(Vec::new())),
            });
        }
    }
    if dit.trim().is_empty() && appels.is_empty() {
        let fin = candidat
            .get("finishReason")
            .and_then(Json::as_str)
            .unwrap_or("rien");
        return Err(format!("Gemini n'a rien rendu ({fin})"));
    }
    Ok(Tour {
        contenu,
        texte: dit,
        appels,
    })
}

/// **Ce que rend un outil**, dans l'histoire : son résultat, ou son erreur — que le modèle lit et
/// corrige.
pub fn reponse_d_outil(appel: &AppelDuModele, issue: Result<Json, String>) -> Json {
    let reponse = match issue {
        Ok(v) => Json::objet([("result", v)]),
        Err(e) => Json::objet([("error", Json::from(e))]),
    };
    let mut champs = vec![("name", Json::from(appel.nom.as_str()))];
    if let Some(id) = &appel.id {
        champs.push(("id", Json::from(id.as_str())));
    }
    champs.push(("response", reponse));
    Json::objet([("functionResponse", Json::objet(champs))])
}

/// **Ce qu'un refus dit** : son code, sa première phrase, et quand revenir — le `RetryInfo` que
/// Google joint à un 429 (« 53509s » le 22/11 à 18 h : jusqu'à minuit, heure du Pacifique, quand
/// les quotas du jour repartent).
#[derive(Debug, Clone, PartialEq)]
pub struct Refus {
    pub statut: u16,
    pub message: String,
    pub revenir_dans: Option<Duration>,
}

/// Lit un refus, quel que soit son corps.
pub fn lire_le_refus(statut: u16, corps: &str) -> Refus {
    let json = Json::lire(corps).ok();
    let erreur = json.as_ref().and_then(|j| j.get("error"));
    let message = erreur
        .and_then(|e| e.get("message"))
        .and_then(Json::as_str)
        .and_then(|m| m.lines().map(str::trim).find(|l| !l.is_empty()))
        .map_or_else(|| format!("code {statut}"), str::to_string);
    let revenir_dans = erreur
        .and_then(|e| e.get("details"))
        .and_then(Json::as_liste)
        .unwrap_or(&[])
        .iter()
        .filter_map(|d| d.get("retryDelay").and_then(Json::as_str))
        .find_map(duree);
    Refus {
        statut,
        message,
        revenir_dans,
    }
}

/// « 53509s », « 7.25s » : un `google.protobuf.Duration` écrit en JSON.
fn duree(texte: &str) -> Option<Duration> {
    let secondes: f64 = texte.strip_suffix('s')?.parse().ok()?;
    (secondes.is_finite() && secondes >= 0.0).then(|| Duration::from_secs_f64(secondes))
}
