//! La [`Valeur`] d'un document Tauri traduite en [`Project`] de Glucose Rust.
//!
//! Les deux modèles portent les mêmes champs, aux noms près (`camelCase` d'un côté,
//! `snake_case` de l'autre) : c'est voulu, la cible visuelle et fonctionnelle de Glucose Rust
//! est exactement Glucose Tauri. La traduction est donc un relevé, champ par champ, et elle
//! a deux règles :
//!
//! * **un champ absent prend la valeur neutre du modèle Rust** — celle qu'un nœud neuf aurait —,
//!   jamais une valeur inventée ;
//! * **ce que le modèle Rust ne sait pas encore porter n'est pas perdu en silence** : il est
//!   compté dans [`Rapport::omis`], et l'utilisateur le lit (règle R1 de la fiche 05).
//!
//! Les images ne portent pas leurs octets ici : chacune reçoit une [`Provenance`], que
//! le bureau résout sur le disque (`glucose_desktop::tauri::resoudre`).

mod annotations;

use crate::persist::tauri::Valeur;
use crate::types::{
    AssetRef, Board, BoardImage, BoardZone, CanvasFolder, Domain, DomainAssignment,
    FolderMirrorSource, FolderSortMode, Preset, PresetSlot, Project, StoryboardPanel,
    TemporalAnchor, Viewport,
};
use std::collections::{BTreeMap, HashMap};

/// D'où viendront les octets d'une image.
#[derive(Debug, Clone, PartialEq)]
pub enum Provenance {
    /// Le magasin de Tauri (`asset:<nom>`) : `%APPDATA%\com.glucose.app\assets\<nom>`, ou le
    /// dossier `objects/` d'un document portable.
    Magasin(String),
    /// Octets portés par le document lui-même (`data:` en base64, ou blob embarqué).
    Octets(Vec<u8>),
    /// Un chemin de fichier, absolu ou relatif au document (le miroir d'un dossier).
    Chemin(String),
    /// Une adresse web : Tauri la chargeait à l'affichage, rien n'est à rapatrier ici.
    Web(String),
    /// Rien ne dit où sont les octets.
    Inconnue,
}

/// Ce que la traduction a vu, et ce qu'elle n'a pas pu garder.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rapport {
    /// Pour chaque image, par son identifiant : d'où viennent ses octets.
    pub provenances: Vec<(String, Provenance)>,
    /// Ce que le modèle Rust ne porte pas encore, et combien de fois : « réglages de
    /// storyboard », « image d'un panneau »… Vide pour un document qui passe en entier.
    pub omis: BTreeMap<&'static str, usize>,
}

impl Rapport {
    fn omettre(&mut self, quoi: &'static str) {
        *self.omis.entry(quoi).or_insert(0) += 1;
    }
}

/// Traduit le projet entier.
pub fn traduire(v: &Valeur) -> (Project, Rapport) {
    let mut r = Rapport::default();
    let blobs = blobs(v);
    let boards: Vec<Board> = v
        .liste("boards")
        .iter()
        .filter(|b| b.est_carte())
        .map(|b| tableau(b, &blobs, &mut r))
        .collect();
    let mut projet = Project::new(v.texte("name").unwrap_or("Projet Glucose"));
    projet.version = v.texte("version").unwrap_or("1.0.0").to_string();
    if !boards.is_empty() {
        projet.boards = boards;
    }
    // Un tableau actif qui n'existe pas se rabat sur le premier : c'est ce que Tauri affichait.
    projet.active_board_id = v
        .texte("activeBoardId")
        .filter(|id| projet.boards.iter().any(|b| b.id == *id))
        .map(str::to_string)
        .unwrap_or_else(|| projet.boards[0].id.clone());
    projet.presets = v.liste("presets").iter().map(preset).collect();
    projet.domains = v.liste("domains").iter().map(domaine).collect();
    projet.collab_url = v.texte("collabUrl").map(str::to_string);
    projet.asset_channel_url = v.texte("assetChannelUrl").map(str::to_string);
    projet.created_at = v.entier("createdAt").unwrap_or(0);
    projet.updated_at = v.entier("updatedAt").unwrap_or(0);
    (projet, r)
}

/// Les blobs embarqués (`project.blobs`, sha256 → octets), s'il y en a.
fn blobs(v: &Valeur) -> HashMap<String, Vec<u8>> {
    match v.champ("blobs") {
        Some(Valeur::Carte(m)) => m
            .iter()
            .filter_map(|(k, v)| match v {
                Valeur::Octets(o) => Some((k.clone(), o.clone())),
                _ => None,
            })
            .collect(),
        _ => HashMap::new(),
    }
}

fn texte(v: &Valeur, cle: &str) -> String {
    v.texte(cle).unwrap_or_default().to_string()
}

fn option(v: &Valeur, cle: &str) -> Option<String> {
    v.texte(cle).map(str::to_string)
}

fn tableau(b: &Valeur, blobs: &HashMap<String, Vec<u8>>, r: &mut Rapport) -> Board {
    let mut board = Board::new(texte(b, "id"), texte(b, "name"));
    board.images = b
        .liste("images")
        .iter()
        .filter_map(|i| image(i, blobs, r))
        .collect();
    board.annotations = b
        .liste("annotations")
        .iter()
        .filter_map(|a| annotations::annotation(a, r))
        .collect();
    board.folders = b.liste("folders").iter().map(dossier).collect();
    board.panels = b.liste("panels").iter().map(|p| panneau(p, r)).collect();
    board.zones = b.liste("zones").iter().map(zone).collect();
    if let Some(vp) = b.champ("viewport") {
        board.viewport = vue(vp);
    }
    if let Some(Valeur::Carte(signets)) = b.champ("bookmarks") {
        board.bookmarks = signets
            .iter()
            .filter(|(_, v)| v.est_carte())
            .map(|(k, v)| (k.clone(), vue(v)))
            .collect();
    }
    if b.champ("storyboard").is_some() {
        r.omettre("réglages de storyboard d'un tableau");
    }
    if b.champ("presetId").is_some() {
        r.omettre("preset appliqué à un tableau");
    }
    board.created_at = b.entier("createdAt").unwrap_or(0);
    board.updated_at = b.entier("updatedAt").unwrap_or(0);
    board
}

fn vue(v: &Valeur) -> Viewport {
    Viewport {
        x: v.nombre("x").unwrap_or(0.0),
        y: v.nombre("y").unwrap_or(0.0),
        scale: v.nombre("scale").unwrap_or(1.0),
    }
    .normalized()
}

fn image(i: &Valeur, blobs: &HashMap<String, Vec<u8>>, r: &mut Rapport) -> Option<BoardImage> {
    let Some(id) = i.texte("id") else {
        r.omettre("image sans identifiant");
        return None;
    };
    let largeur = i.nombre("width").unwrap_or(0.0);
    let hauteur = i.nombre("height").unwrap_or(0.0);
    let mut img = BoardImage::new(
        id,
        i.nombre("x").unwrap_or(0.0),
        i.nombre("y").unwrap_or(0.0),
        largeur,
        hauteur,
    );
    img.membrane_id = option(i, "membraneId");
    img.asset = i.champ("asset").and_then(reference);
    img.rotation = i.nombre("rotation").unwrap_or(0.0);
    img.locked = i.booleen("locked").unwrap_or(false);
    img.tags = i
        .liste("tags")
        .iter()
        .filter_map(|t| match t {
            Valeur::Texte(s) => Some(s.clone()),
            _ => None,
        })
        .collect();
    img.slot_id = option(i, "slotId");
    img.source_url = option(i, "sourceUrl");
    img.original_width = i.nombre("originalWidth").unwrap_or(largeur);
    img.original_height = i.nombre("originalHeight").unwrap_or(hauteur);
    img.is_video = i.booleen("isVideo").unwrap_or(false);
    img.fit = option(i, "fit");
    img.domains = assignations(i);
    img.mirror_of = option(i, "mirrorOf");
    img.temporal_anchor = i.champ("temporalAnchor").map(ancre_temporelle);
    let provenance = provenance(img.asset.as_ref(), i.texte("src"), blobs);
    r.provenances.push((img.id.clone(), provenance));
    Some(img)
}

fn reference(a: &Valeur) -> Option<AssetRef> {
    let taille = a.entier("sizeBytes").and_then(|n| u64::try_from(n).ok());
    match a.texte("mode")? {
        "embed" => Some(AssetRef::Embed {
            sha256: a.texte("sha256")?.to_string(),
            mime: texte(a, "mime"),
            size_bytes: taille,
        }),
        "link" => Some(AssetRef::Link {
            href: a.texte("href")?.to_string(),
            sha256: option(a, "sha256"),
            size_bytes: taille,
        }),
        _ => None,
    }
}

/// D'où viennent les octets d'une image : la référence d'abord (le modèle de Tauri depuis
/// septembre), le champ `src` d'avant sinon.
fn provenance(
    asset: Option<&AssetRef>,
    src: Option<&str>,
    blobs: &HashMap<String, Vec<u8>>,
) -> Provenance {
    match asset {
        Some(AssetRef::Embed { sha256, .. }) => match blobs.get(sha256) {
            Some(octets) => Provenance::Octets(octets.clone()),
            None => Provenance::Inconnue,
        },
        Some(AssetRef::Link { href, .. }) => super::provenance::provenance_de(href),
        None => src.map_or(Provenance::Inconnue, super::provenance::provenance_de),
    }
}

fn assignations(v: &Valeur) -> Vec<DomainAssignment> {
    v.liste("domains")
        .iter()
        .filter_map(|d| {
            Some(DomainAssignment {
                domain_id: d.texte("domainId")?.to_string(),
                weight: d.nombre("weight").unwrap_or(1.0),
            })
        })
        .collect()
}

fn ancre_temporelle(v: &Valeur) -> TemporalAnchor {
    let debut = v.entier("start").unwrap_or(0);
    TemporalAnchor {
        start: debut,
        end: v.entier("end").unwrap_or(debut),
        label: option(v, "label"),
    }
}

fn dossier(f: &Valeur) -> CanvasFolder {
    let mut d = CanvasFolder::new(texte(f, "id"), texte(f, "name"), texte(f, "childBoardId"));
    if let Some(c) = f.texte("color") {
        d.color = c.to_string();
    }
    d.x = f.nombre("x").unwrap_or(d.x);
    d.y = f.nombre("y").unwrap_or(d.y);
    d.width = f.nombre("width").unwrap_or(d.width);
    d.height = f.nombre("height").unwrap_or(d.height);
    d.mirror_of = option(f, "mirrorOf");
    d.mirror_source = f.champ("mirrorSource").map(|m| FolderMirrorSource {
        root_path: texte(m, "rootPath"),
        mode: m.texte("mode").unwrap_or("snapshot").to_string(),
        last_scanned_at: m.entier("lastScannedAt").unwrap_or(0),
        pattern: option(m, "pattern"),
        recursive: m.booleen("recursive").unwrap_or(false),
        sort_by: m.texte("sortBy").and_then(tri),
        pending_scan: m.booleen("pendingScan").unwrap_or(false),
    });
    d
}

fn tri(s: &str) -> Option<FolderSortMode> {
    Some(match s {
        "name-asc" => FolderSortMode::NameAsc,
        "name-desc" => FolderSortMode::NameDesc,
        "type" => FolderSortMode::Type,
        "size-desc" => FolderSortMode::SizeDesc,
        "size-asc" => FolderSortMode::SizeAsc,
        "modified-desc" => FolderSortMode::ModifiedDesc,
        "modified-asc" => FolderSortMode::ModifiedAsc,
        _ => return None,
    })
}

fn panneau(p: &Valeur, r: &mut Rapport) -> StoryboardPanel {
    if p.champ("imageId").is_some() {
        r.omettre("image assignée à un panneau de storyboard");
    }
    let mut panel = StoryboardPanel::new(
        texte(p, "id"),
        p.entier("order").unwrap_or(0) as i32,
        p.nombre("x").unwrap_or(0.0),
        p.nombre("y").unwrap_or(0.0),
        p.nombre("width").unwrap_or(0.0),
        p.nombre("height").unwrap_or(0.0),
    );
    panel.description = texte(p, "description");
    panel
}

fn zone(z: &Valeur) -> BoardZone {
    BoardZone::new(
        texte(z, "slotId"),
        z.nombre("x").unwrap_or(0.0),
        z.nombre("y").unwrap_or(0.0),
        z.nombre("width").unwrap_or(0.0),
        z.nombre("height").unwrap_or(0.0),
    )
}

fn preset(p: &Valeur) -> Preset {
    Preset {
        id: texte(p, "id"),
        name: texte(p, "name"),
        description: texte(p, "description"),
        slots: p
            .liste("slots")
            .iter()
            .map(|s| PresetSlot {
                id: texte(s, "id"),
                name: texte(s, "name"),
                color: texte(s, "color"),
                description: texte(s, "description"),
                order: s.entier("order").unwrap_or(0) as i32,
            })
            .collect(),
        is_builtin: p.booleen("isBuiltin").unwrap_or(false),
        created_at: p.entier("createdAt").unwrap_or(0),
    }
}

fn domaine(d: &Valeur) -> Domain {
    Domain {
        id: texte(d, "id"),
        name: texte(d, "name"),
        color: texte(d, "color"),
        icon: texte(d, "icon"),
        created_at: d.entier("createdAt").unwrap_or(0),
    }
}
