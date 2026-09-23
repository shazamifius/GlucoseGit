//! Ce qu'un fichier déposé sur le canevas devient.
//!
//! # Ce que Glucose Tauri fait, et ce qui manquait ici
//!
//! Glucose Tauri **route** ce qu'on lui dépose (`dropHandler.ts`) : un fichier lisible
//! devient une carte de texte, une image se pose, un dossier devient un miroir, et tout le
//! reste devient une tuile qui mène au fichier. Ici, tout ce qui tombait sur la fenêtre
//! était supposé être une image — un `.rs` déposé produisait un message d'erreur de
//! décodeur, et rien d'autre.
//!
//! # Trois issues, et une seule question posée au décodeur
//!
//! 1. **lisible** — le noyau le dit ([`glucose_core::file_kind`]), et le contenu se pose
//!    en Markdown, dans un bloc marqué de son langage s'il y a lieu ;
//! 2. **image** — on ne le devine pas par son extension : on **tente l'en-tête**. Une liste
//!    d'extensions diverge toujours de ce que le décodeur sait lire, et la divergence se
//!    voit à l'écran ;
//! 3. **le reste** — un lanceur : une tuile carrée, teintée par son extension, qui mène au
//!    fichier. Un dossier en fait partie : on ne sait pas le *lire*, on sait y *mener*.
//!
//! # Pourquoi un lanceur **révèle** au lieu d'ouvrir
//!
//! Glucose Tauri ouvre le fichier dans son application. Ici, le double-clic ouvre
//! l'explorateur du système **sur** le fichier, sélectionné mais pas lancé.
//!
//! Ce n'est pas de la prudence de principe. [`super::links`] refuse déjà tout ce qui n'est
//! ni `http://` ni `https://`, et il écrit pourquoi : un `.glucose` se reçoit de quelqu'un,
//! et rien de ce qu'il contient ne doit pouvoir faire exécuter quoi que ce soit. Or le
//! texte d'une tuile et son `source_file` sont deux champs indépendants : une tuile qui
//! affiche « Rapport.pdf » peut pointer sur autre chose. Ouvrir directement rouvrirait le
//! trou que `links` ferme, dans le même document et par un geste plus banal encore.
//!
//! Révéler montre le **vrai** fichier, avec son vrai nom, et laisse la main. C'est un cas
//! où suivre Glucose Tauri à l'identique serait le suivre dans une faute.

use crate::app::GlucoseApp;
use glucose_core::file_kind;
use glucose_core::types::Annotation;
use std::path::{Path, PathBuf};

/// Décalage entre deux nœuds d'un même lot, en unités monde — la cascade de Glucose Tauri.
const CASCADE: f64 = 28.0;

/// La tuile carrée d'un lanceur, en unités monde (Glucose Tauri : 150 × 140).
const LAUNCHER_SIZE: (f64, f64) = (150.0, 140.0);

/// Le corps d'un lanceur : le nom d'un fichier tient sur une tuile, pas sur une carte.
const LAUNCHER_FONT: f64 = 11.0;

/// Largeur d'une carte née d'un fichier, en unités monde (Glucose Tauri : 520).
const READABLE_WIDTH: f64 = 520.0;

/// Corps d'une carte née d'un fichier : plus petit que la normale, car on y verse du code.
const READABLE_FONT: f64 = 12.0;

/// La teinte d'un lanceur, par extension — reprise de Glucose Tauri (`EXT_COLOR`).
///
/// **Triée**, comme la table du noyau, et pour la même raison : un doublon ou une faute de
/// frappe ne peut pas entrer sans qu'un test le dise.
const ACCENT: &[(&str, &str)] = &[
    ("7z", "#a78bfa"),
    ("abc", "#888888"),
    ("aep", "#9999ff"),
    ("ai", "#ff9a00"),
    ("avi", "#f87171"),
    ("blend", "#e87d0d"),
    ("c", "#60a5fa"),
    ("c4d", "#086adb"),
    ("clip", "#333333"),
    ("cpp", "#60a5fa"),
    ("doc", "#2b579a"),
    ("docx", "#2b579a"),
    ("drp", "#ff6a00"),
    ("exr", "#2dd4a8"),
    ("fbx", "#f87171"),
    ("glb", "#f87171"),
    ("gltf", "#f87171"),
    ("gz", "#a78bfa"),
    ("hip", "#ff6600"),
    ("hipnc", "#ff6600"),
    ("indd", "#ff3366"),
    ("js", "#f59e0b"),
    ("json", "#f59e0b"),
    ("kra", "#2d9b27"),
    ("ma", "#00aaff"),
    ("mb", "#00aaff"),
    ("md", "#888888"),
    ("mkv", "#f87171"),
    ("mov", "#f87171"),
    ("mp4", "#f87171"),
    ("nk", "#ffcc00"),
    ("nuke", "#ffcc00"),
    ("obj", "#f87171"),
    ("pdf", "#e53e3e"),
    ("pptx", "#d24726"),
    ("prproj", "#9999ff"),
    ("psd", "#31a8ff"),
    ("py", "#3b82f6"),
    ("rar", "#a78bfa"),
    ("rs", "#f97316"),
    ("tar", "#a78bfa"),
    ("ts", "#3b82f6"),
    ("txt", "#888888"),
    ("usd", "#fbbf24"),
    ("xcf", "#7b3ebb"),
    ("xls", "#217346"),
    ("xlsx", "#217346"),
    ("zip", "#a78bfa"),
];

/// La teinte d'un lanceur pour cette extension, ou la teinte neutre de Glucose Tauri.
fn accent_of(ext: &str) -> &'static str {
    ACCENT
        .binary_search_by_key(&ext, |(key, _)| key)
        .map_or("#1a1a2e", |index| ACCENT[index].1)
}

/// Ce qu'un lot déposé a donné, en une phrase.
///
/// Pure, donc testable sans fenêtre : la formulation d'un message est exactement le genre
/// de chose qui se casse sans qu'aucun test ne le voie.
fn compte_rendu(poses: usize, rates: usize) -> String {
    let pluriel = if poses > 1 { "s" } else { "" };
    match (poses, rates) {
        (0, 0) => String::new(),
        (0, n) => format!("Aucun des {n} fichiers n'a pu être posé"),
        (p, 0) => format!("{p} élément{pluriel} posé{pluriel}"),
        (p, n) => format!(
            "{p} élément{pluriel} posé{pluriel}, {n} illisible{}",
            if n > 1 { "s" } else { "" }
        ),
    }
}

/// Le nom d'un fichier, sans son chemin.
fn file_name_of(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .map_or_else(|| path.to_string_lossy().into_owned(), str::to_string)
}

impl GlucoseApp {
    /// Le point du monde où un lot déposé se pose : le centre de ce qu'on regarde.
    ///
    /// **Là où le curseur a lâché quand on le sait**, et le centre de ce qu'on regarde sinon.
    ///
    /// winit **reçoit** la position du curseur — `IDropTarget::Drop` la prend en paramètre —
    /// et la jette : son paramètre s'appelle `_pt`. Et pendant un glisser, Windows ne remonte
    /// aucun mouvement de souris à la fenêtre, donc la dernière position connue date d'avant
    /// le geste, parfois de plusieurs minutes. Poser au centre était le seul choix
    /// **prévisible** qui restât.
    ///
    /// Depuis DEPOT-WEB-1, la cible de dépôt du projet la transmet, et ce module cesse d'être
    /// aveugle — mais seulement là où un pont existe. Le centre reste donc la réponse quand
    /// personne ne l'a dite, et c'est ce que `client` vaut alors.
    pub(crate) fn drop_origin(&self, client: Option<(f64, f64)>) -> (f64, f64) {
        let vp = self.store.viewport();
        if let Some((cx, cy)) = client {
            return crate::canvas::screen_to_world(cx, cy, &vp);
        }
        let (w, h) = self.window.as_ref().map_or((1440.0, 900.0), |window| {
            let size = window.inner_size();
            (f64::from(size.width), f64::from(size.height))
        });
        let header = f64::from(self.ui.header_height());
        crate::canvas::screen_to_world(w / 2.0, (h + header) / 2.0, &vp)
    }

    /// Un lot de fichiers déposés sur le canevas.
    ///
    /// Le lot entier tient dans **une** entrée d'annulation : déposer huit fichiers puis se
    /// raviser est un seul geste, et le défaire demande un seul `Ctrl+Z`.
    pub fn drop_files(&mut self, paths: &[PathBuf]) {
        let origine = self.drop_origin(None);
        self.deposer(paths, &[], origine);
    }

    /// **Tout ce qu'un dépôt apporte, posé en une fois** — des fichiers, des adresses, ou les
    /// deux (DEPOT-WEB-1).
    ///
    /// `(ox, oy)` est le point du **monde** où le lot se pose : l'appelant le tire du curseur
    /// ([`Self::drop_origin`]), ou de l'annonce d'un téléchargement, qui l'a figé au lâcher.
    ///
    /// **Une seule fonction pour les deux natures**, et c'est ce que le cliquet des toasts a
    /// imposé : un dépôt est un geste, un geste rend **un** compte-rendu. Une seconde fonction
    /// avec son propre message aurait dit deux fois la même chose de deux façons.
    pub(crate) fn deposer(&mut self, paths: &[PathBuf], liens: &[String], (ox, oy): (f64, f64)) {
        // **Un raccourci Internet est une adresse**, d'ou qu'il vienne (DEPOT-WEB-2) : il
        // rejoint les liens, et se pose en carte qu'on peut suivre plutot qu'en carte qui
        // porte son nom de fichier.
        let (fichiers, adresses) = crate::plateforme::moisson::lire_les_raccourcis(paths);
        let liens: Vec<String> = liens.iter().cloned().chain(adresses).collect();
        let paths = fichiers.as_slice();
        let attendus = paths.len() + liens.len();
        if attendus == 0 {
            return;
        }
        let board = self.store.project.active_board_id.clone();

        self.store.begin_live_edit();
        let mut placed = 0usize;
        for path in paths {
            let offset = placed as f64 * CASCADE;
            if self.place_dropped(&board, path, (ox + offset, oy + offset)) {
                placed += 1;
            }
        }
        for lien in &liens {
            let offset = placed as f64 * CASCADE;
            self.place_link(&board, lien, (ox + offset, oy + offset));
            placed += 1;
        }
        self.store.end_live_edit();

        // **Un** compte-rendu pour le lot, échecs compris. Déposer huit fichiers ne doit pas
        // produire huit messages : ce qui s'est passé se dit d'une phrase, et celui qui a
        // raté s'y compte au lieu de s'annoncer tout seul au milieu des autres.
        self.ui.show_toast(compte_rendu(placed, attendus - placed));
        self.mark_dirty();
    }

    /// **Une adresse déposée, posée en carte et cliquable** (DEPOT-WEB-1).
    ///
    /// Elle s'écrit en Markdown `[adresse](adresse)`, et non en texte nu : `links::url_at`
    /// ne suit que ce que la syntaxe désigne comme un lien, et une adresse écrite nue
    /// resterait du texte que `Ctrl`+clic ignore. Poser un lien qui ne s'ouvre pas serait un
    /// bouton qui ment (fiche 05 § 5.4), sous la forme d'une carte.
    fn place_link(&mut self, board: &str, adresse: &str, (x, y): (f64, f64)) {
        let aid = self.store.generate_id("text");
        // La fabrique **mesure** la carte : une adresse longue se dessinerait sur trois lignes
        // et ne se cliquerait que sur une si on posait une hauteur en dur — ce que le collage
        // de texte a déjà payé une fois.
        let ann = super::tools::text_card(
            &self.renderer.typography,
            &self.renderer.math,
            aid,
            x,
            y,
            crate::plateforme::moisson::lien_markdown(adresse),
        );
        self.store.add_annotation(board, ann);
    }

    /// Pose un fichier au point donné. Rend `false` si rien n'a pu l'être.
    fn place_dropped(&mut self, board: &str, path: &Path, (x, y): (f64, f64)) -> bool {
        let name = file_name_of(path);

        // 1. Lisible : le noyau le dit, sans toucher au disque.
        if !path.is_dir() && file_kind::readable(&name).is_some() {
            return self.place_readable(board, path, &name, (x, y));
        }
        // 2. Image : on ne la devine pas par son extension, on tente son en-tête — et c'est
        // le décodeur lui-même qui répond, donc aucune liste ne peut en diverger.
        if !path.is_dir() && self.place_image_file(board, path, (x, y)).is_ok() {
            return true;
        }
        // 3. Le reste — un dossier compris : une tuile qui mène au fichier.
        self.place_launcher(board, path, &name, (x, y));
        true
    }

    /// Le contenu d'un fichier lisible, posé en Markdown sur une carte.
    fn place_readable(&mut self, board: &str, path: &Path, name: &str, (x, y): (f64, f64)) -> bool {
        // Un échec ne parle pas ici : c'est le compte-rendu du lot qui le dira.
        let Ok(raw) = std::fs::read(path) else {
            return false;
        };
        let truncated = raw.len() > file_kind::INLINE_MAX_BYTES;
        // Un fichier « lisible » peut très bien ne pas être de l'UTF-8 valide : un `.txt`
        // hérité d'une autre époque, par exemple. On ne le refuse pas — on remplace ce qui
        // ne se lit pas, ce que `from_utf8_lossy` fait exactement.
        let text = String::from_utf8_lossy(&raw);
        let content = file_kind::truncate_on_char_boundary(&text, file_kind::INLINE_MAX_BYTES);

        let id = self.store.generate_id("txt");
        let markdown = file_kind::as_markdown(name, content, truncated);
        let height = crate::renderer::card::text_card_fit_height(
            &self.renderer.typography,
            &self.renderer.math,
            &markdown,
            READABLE_WIDTH,
        );
        let mut card = Annotation::text(id, x, y, markdown);
        if let Annotation::Text {
            width: w,
            height: h,
            font_size,
            source_file,
            ..
        } = &mut card
        {
            *w = Some(READABLE_WIDTH);
            *h = Some(height);
            *font_size = Some(READABLE_FONT);
            // Le chemin est gardé : la carte sait d'où elle vient, et le double-clic peut
            // y revenir même quand le contenu a été tronqué.
            *source_file = Some(path.to_string_lossy().into_owned());
        }
        self.store.add_annotation(board, card);
        true
    }

    /// Une tuile qui mène au fichier, teintée par son extension.
    fn place_launcher(&mut self, board: &str, path: &Path, name: &str, (x, y): (f64, f64)) {
        let id = self.store.generate_id("src");
        let mut tile = Annotation::sticky(id, x, y, name);
        if let Annotation::Sticky {
            width,
            height,
            font_size,
            color,
            bg_color,
            source_file,
            ..
        } = &mut tile
        {
            *width = Some(LAUNCHER_SIZE.0);
            *height = Some(LAUNCHER_SIZE.1);
            *font_size = Some(LAUNCHER_FONT);
            *color = Some("#cccccc".to_string());
            *bg_color = Some(accent_of(&file_kind::extension(name)).to_string());
            *source_file = Some(path.to_string_lossy().into_owned());
        }
        self.store.add_annotation(board, tile);
    }

    /// Ce qu'un double-clic ouvre sur un **lanceur** : son fichier, révélé dans l'explorateur.
    ///
    /// La règle est ici, dans le geste, et non dans les données : une carte de texte garde
    /// elle aussi le chemin dont elle vient — c'est vrai, et c'est utile — mais elle
    /// **s'édite** au double-clic, comme toute carte de texte. Seule une tuile, qui ne
    /// contient qu'un nom de fichier, a mieux à offrir que d'éditer ce nom.
    ///
    /// Rend `false` si le nœud n'est pas un lanceur. Un lanceur dont l'explorateur refuse
    /// de démarrer rend `true` quand même, et le dit : le geste a bien été pris.
    pub(crate) fn reveal_launcher_target(&mut self, id: &str) -> bool {
        let Some(path) = self.store.launcher_target(id).map(str::to_string) else {
            return false;
        };
        if reveal(&path) {
            return true;
        }
        self.ui
            .show_toast("Impossible d'ouvrir l'explorateur".to_string());
        true
    }
}

/// Montre un fichier dans l'explorateur du système, sélectionné mais **pas lancé**.
///
/// `explorer /select,"…"` sélectionne l'entrée dans son dossier ; la syntaxe sans espace
/// après la virgule est celle qu'attend l'Explorateur, et elle n'est pas négociable. Son
/// code de retour ne dit rien d'utile — il vaut 1 même quand la fenêtre s'ouvre —, donc on
/// ne juge que le démarrage du processus.
#[cfg(target_os = "windows")]
fn reveal(path: &str) -> bool {
    use std::os::windows::process::CommandExt;
    /// `CREATE_NO_WINDOW` : pas de console noire qui clignote derrière la fenêtre.
    const SANS_CONSOLE: u32 = 0x0800_0000;
    std::process::Command::new("explorer")
        .raw_arg(format!("/select,\"{path}\""))
        .creation_flags(SANS_CONSOLE)
        .spawn()
        .is_ok()
}

/// Montre un fichier dans le Finder, sélectionné mais **pas lancé**.
#[cfg(target_os = "macos")]
fn reveal(path: &str) -> bool {
    std::process::Command::new("open")
        .args(["-R", path])
        .spawn()
        .is_ok()
}

/// Ouvre le dossier qui contient le fichier — aucun gestionnaire de fichiers n'est commun à
/// tous les bureaux Linux, mais tous savent ouvrir un dossier.
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn reveal(path: &str) -> bool {
    let parent = Path::new(path)
        .parent()
        .map_or_else(|| path.to_string(), |p| p.to_string_lossy().into_owned());
    std::process::Command::new("xdg-open")
        .arg(parent)
        .spawn()
        .is_ok()
}

#[cfg(test)]
mod tests;
