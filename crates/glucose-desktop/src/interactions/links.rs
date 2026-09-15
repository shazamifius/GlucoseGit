//! Suivre un lien, depuis le canevas.
//!
//! # Pourquoi `Ctrl`, et pas le clic seul
//!
//! Sur un canevas, cliquer une carte la sélectionne — c'est le geste le plus fréquent de
//! tous. Si un clic sur un lien ouvrait un navigateur, on ne pourrait plus attraper une carte
//! qui en contient un sans partir ailleurs. `Ctrl`+clic est la convention des éditeurs de
//! code pour exactement cette raison : le texte reste du texte, et le lien s'ouvre quand on
//! le demande.
//!
//! # Ouvrir sans dépendance
//!
//! Chaque système a sa commande — `start` sur Windows, `open` sur macOS, `xdg-open` ailleurs.
//! Trois lignes, contre une caisse de plus pour le même résultat : c'est le cas que la charte
//! tranche dans l'autre sens, puisqu'on peut très bien faire sans.
//!
//! **Rien n'est lancé sans être vérifié** : seuls `http://` et `https://` partent. Une source
//! `.glucose` est un fichier comme un autre, qu'on peut recevoir de quelqu'un ; un
//! `[innocent](file:///…)` ou un schéma inventé ne doit pas pouvoir faire exécuter quoi que
//! ce soit par un simple `Ctrl`+clic.

use crate::app::GlucoseApp;
use crate::renderer::richtext::TextMode;
use glucose_core::text::{inline_spans, SpanRole};

/// Les seuls schémas qu'un clic peut ouvrir.
const SCHEMES: [&str; 2] = ["http://", "https://"];

impl GlucoseApp {
    /// `Ctrl`+clic sur un lien l'ouvre. Rend `true` si le clic a été pris.
    pub fn click_link_at(&mut self, screen: (f64, f64)) -> bool {
        if !self.modifiers.control_key() {
            return false;
        }
        let Some(url) = self.link_under(screen) else {
            return false;
        };
        // Rien à dire quand ça marche : le navigateur s'ouvre, et cela se voit. Un échec,
        // lui, ne se voit pas du tout — c'est le seul cas où l'utilisateur a besoin qu'on
        // lui parle.
        if !open_url(&url) {
            self.ui.show_toast("Ce lien n'a pas pu être ouvert");
        }
        self.mark_dirty();
        true
    }

    /// L'adresse du lien sous le curseur, s'il y en a un.
    ///
    /// La carte est lue **au repos** : c'est la vue où l'adresse est effacée, donc celle que
    /// l'utilisateur a sous les yeux quand il vise le texte du lien.
    pub(crate) fn link_under(&self, screen: (f64, f64)) -> Option<String> {
        let (wx, wy) = crate::canvas::screen_to_world(
            screen.0,
            screen.1,
            &self.store.active_board()?.viewport,
        );
        let candidate = self.pick_candidate_at(wx, wy)?;
        let texte = self.editable_text_of(&candidate.id)?;
        let offset = self.offset_in_card(&candidate.id, &texte, screen, TextMode::Rendered)?;
        url_at(&texte, offset)
    }
}

/// L'adresse du lien qui couvre l'octet `at`, s'il y en a un.
///
/// Le texte d'un lien porte `LINK` ; son adresse est le signe qui le suit — `](url)`. On
/// cherche donc la tranche visée, puis le premier signe après elle.
pub fn url_at(source: &str, at: usize) -> Option<String> {
    let spans = inline_spans(source);
    let vise = spans.iter().position(|s| {
        at >= s.start && at < s.end && s.role == SpanRole::Text && s.emphasis.link()
    })?;
    let ferme = spans[vise..]
        .iter()
        .find(|s| s.role == SpanRole::Marker && s.slice(source).starts_with(']'))?;
    let brut = ferme.slice(source);
    let dedans = brut.strip_prefix("](")?.strip_suffix(')')?;
    SCHEMES
        .iter()
        .any(|s| dedans.starts_with(s))
        .then(|| dedans.to_string())
}

/// Confie l'adresse au système. Rend `false` si la commande n'a pas pu partir.
/// Ouvre `url` avec le navigateur du système. Rend `false` si rien n'a pu être lancé.
///
/// # Deux fautes que la version précédente faisait
///
/// Elle rendait `spawn().is_ok()`, c'est-à-dire **« cmd a démarré »** — jamais « le lien
/// s'est ouvert ». Un échec de `start` à l'intérieur de `cmd` passait donc pour un succès, et
/// le seul message prévu pour l'utilisateur ne s'affichait pas. Un bouton qui ment, et qui
/// ment même à celui qui l'a écrit.
///
/// Et elle faisait clignoter une console : `cmd` ouvre une fenêtre que personne n'a demandée.
#[cfg(target_os = "windows")]
fn open_url(url: &str) -> bool {
    use std::os::windows::process::CommandExt;
    /// Lancer sans ouvrir de console (`CREATE_NO_WINDOW`).
    const SANS_CONSOLE: u32 = 0x0800_0000;

    // `start` est une commande interne de `cmd`, d'où le détour ; le `""` est le titre de
    // fenêtre que `start` attend en premier et qu'il confondrait sinon avec l'adresse.
    let par_cmd = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .creation_flags(SANS_CONSOLE)
        .status()
        .is_ok_and(|code| code.success());
    if par_cmd {
        return true;
    }
    // Repli sans interprète de commandes : l'adresse n'est plus une ligne de commande, donc
    // plus rien à échapper. C'est la voie qu'emprunte Windows lui-même pour un raccourci.
    std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .creation_flags(SANS_CONSOLE)
        .spawn()
        .is_ok()
}

/// Ouvre `url` avec le navigateur du système. Rend `false` si rien n'a pu être lancé.
#[cfg(not(target_os = "windows"))]
fn open_url(url: &str) -> bool {
    let programme = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(programme)
        .arg(url)
        .spawn()
        .is_ok()
}

#[cfg(test)]
mod tests;
