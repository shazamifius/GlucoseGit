use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::header;
use sha2::{Digest, Sha256};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

mod multiplayer;
use multiplayer::{MpState, SharedMpState};

// ════════════════════════════════════════════════════════════════════════════
// Sécurité (CLEANUP.md SEC-01..09)
// ════════════════════════════════════════════════════════════════════════════

/// Extensions autorisées pour la lecture d'images / fichiers de projet.
const ALLOWED_READ_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "avif", "bmp", "svg",
    "mp4", "mov", "webm", "mkv",
    "glucose", "json",
];

/// Extensions refusées au LANCEMENT (`open_in_app`) — exécutables et scripts.
/// C'est une **deny-list** : un gestionnaire de fichiers doit pouvoir ouvrir
/// « bref tout » (.blend/.kra/.nuke/.indd/.docx/...), SAUF ce qui peut exécuter
/// du code arbitraire au double-clic (protection RCE, cf. SEC-01..09).
const FORBIDDEN_OPEN_EXTS: &[&str] = &[
    "exe", "bat", "cmd", "ps1", "psm1", "psd1", "vbs", "vbe", "js", "jse",
    "wsf", "wsh", "scr", "com", "msi", "msp", "lnk", "url", "inf", "reg",
    "jar", "app", "dmg", "deb", "rpm", "appimage", "sh", "bash", "zsh",
    "hta", "cpl", "msc", "pif", "scf", "gadget", "vb", "vbscript",
];

/// Extensions masquées au SCAN (folder mirror) : binaires/exécutables OS qui
/// n'ont aucun intérêt visuel sur le canvas. On masque le bruit machine MAIS
/// on garde visibles les fichiers source (.js/.ps1/...) — ils restent
/// non-lançables via `FORBIDDEN_OPEN_EXTS`, juste affichés.
const SCAN_HIDE_EXTS: &[&str] = &[
    "exe", "dll", "sys", "com", "scr", "msi", "msp", "ocx", "drv",
    "lnk", "pif", "cpl", "tmp", "bin",
];

/// Vérifie qu'un chemin est dans un des dossiers autorisés (App Data, Documents,
/// Downloads, Pictures, Videos, Desktop) ET qu'il n'utilise pas de chemin UNC.
/// Retourne le chemin canonicalisé si OK, ou Err sinon.
fn validate_scope(path: &str, app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    // 1) Refus chemins UNC (\\server\share). ATTENTION : sous Windows, le
    //    drag-drop natif ET `std::fs::canonicalize` produisent des chemins
    //    préfixés `\\?\` (extended-length) — ce N'EST PAS un chemin réseau.
    //    On strip ce préfixe avant le test, et on rejette explicitement le
    //    vrai UNC étendu `\\?\UNC\server\share`.
    if is_forbidden_unc(path) {
        return Err("Chemins réseau (UNC) non autorisés".into());
    }

    // 2) Canonicalize pour résoudre les ".." et liens symboliques
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("Chemin invalide: {e}"))?;

    // 3) Doit être dans un des roots autorisés
    let path_resolver = app_handle.path();
    let allowed_roots: Vec<PathBuf> = [
        path_resolver.app_data_dir().ok(),
        path_resolver.app_config_dir().ok(),
        path_resolver.document_dir().ok(),
        path_resolver.download_dir().ok(),
        path_resolver.picture_dir().ok(),
        path_resolver.video_dir().ok(),
        path_resolver.desktop_dir().ok(),
    ]
    .into_iter()
    .flatten()
    .filter_map(|p| std::fs::canonicalize(p).ok())
    .collect();

    if !allowed_roots.iter().any(|r| canonical.starts_with(r)) {
        return Err(format!(
            "Chemin hors du scope autorisé: {}",
            canonical.display()
        ));
    }

    Ok(canonical)
}

/// True si `path` est un VRAI chemin réseau UNC (à rejeter). Distingue le
/// préfixe extended-length Windows `\\?\` (produit par `canonicalize` et le
/// drag-drop natif — totalement légitime) du vrai UNC `\\?\UNC\…` ou `\\serveur`.
/// C'était LA cause du « rien ne marche » : `\\?\C:\…` était pris pour de l'UNC
/// et tout (scan, lecture, image, lancement) était refusé.
fn is_forbidden_unc(path: &str) -> bool {
    let is_extended_unc = path.starts_with(r"\\?\UNC\") || path.starts_with(r"\\?\unc\");
    let stripped = path.strip_prefix(r"\\?\").unwrap_or(path);
    is_extended_unc || stripped.starts_with("\\\\") || stripped.starts_with("//")
}

/// Retire le préfixe Windows extended-length `\\?\` pour produire des chemins
/// lisibles côté UI ET compatibles ShellExecute (qui n'aime pas `\\?\`).
/// Sans effet sur macOS/Linux.
fn display_path(p: &Path) -> String {
    let s = p.to_string_lossy();
    match s.strip_prefix(r"\\?\") {
        Some(rest) => rest.to_string(),
        None => s.into_owned(),
    }
}

/// Récupère l'extension d'un chemin en lowercase, ou empty string si absente.
fn get_ext(path: &Path) -> String {
    path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase()
}

// ════════════════════════════════════════════════════════════════════════════
// Commandes Tauri
// ════════════════════════════════════════════════════════════════════════════

/// Bloque les IP privées / loopback / link-local (CLEANUP.md SEC-06 anti-SSRF).
fn is_public_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            !(v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_unspecified()
                || v4.octets()[0] == 0
                // CGN 100.64.0.0/10
                || (v4.octets()[0] == 100 && (64..=127).contains(&v4.octets()[1]))
                // métadonnées cloud AWS/GCP/Azure
                || (v4.octets()[0] == 169 && v4.octets()[1] == 254))
        }
        IpAddr::V6(v6) => {
            !(v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                // unique-local fc00::/7
                || (v6.segments()[0] & 0xfe00) == 0xfc00
                // link-local fe80::/10
                || (v6.segments()[0] & 0xffc0) == 0xfe80)
        }
    }
}

#[tauri::command]
async fn fetch_image(url: String) -> Result<String, String> {
    // Parse l'URL et résout les hôtes pour bloquer SSRF.
    let parsed = url::Url::parse(&url).map_err(|e| format!("URL invalide: {e}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("Seuls http/https sont autorisés".into());
    }
    if let Some(host) = parsed.host_str() {
        // Si le host est déjà une IP, vérifier qu'elle est publique
        if let Ok(ip) = host.parse::<IpAddr>() {
            if !is_public_ip(&ip) {
                return Err("IP privée non autorisée".into());
            }
        } else {
            // Sinon, résoudre via DNS et vérifier toutes les IPs
            let resolved = tokio::net::lookup_host((host, 0))
                .await
                .map_err(|e| format!("DNS: {e}"))?;
            for addr in resolved {
                if !is_public_ip(&addr.ip()) {
                    return Err("L'hôte résout vers une IP privée".into());
                }
            }
        }
    }

    // User-Agent navigateur réaliste : certains CDN (Pinterest, Twitter,
    // Instagram…) renvoient 403 sur des UA non-browser. On garde "Glucose"
    // en suffixe pour la transparence (un opérateur curieux peut nous identifier).
    let ua = format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
         (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Glucose/{}",
        env!("CARGO_PKG_VERSION")
    );

    let client = reqwest::Client::builder()
        .user_agent(ua)
        .redirect(reqwest::redirect::Policy::limited(3))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    // Referer = origine racine seulement (scheme://host) — beaucoup de CDN
    // exigent un Referer mais on ne veut pas leaker le path/query de la page
    // d'origine. C'est le compromis pragma SEC-06.
    let referer = parsed
        .host_str()
        .map(|h| format!("{}://{}/", parsed.scheme(), h));

    let mut req = client.get(parsed.as_str());
    if let Some(ref r) = referer {
        req = req.header(header::REFERER, r.clone());
    }
    // Accept large : certains CDN renvoient un HTML/JSON si on n'envoie pas
    // un Accept compatible image.
    req = req.header(header::ACCEPT, "image/avif,image/webp,image/apng,image/*,*/*;q=0.8");

    let resp = req.send().await.map_err(|e| e.to_string())?;

    // Refuse les codes d'erreur HTTP — sinon on bufferise une page d'erreur.
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {} pour {}", status.as_u16(), parsed.as_str()));
    }

    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/png")
        .to_string();
    let mime = content_type
        .split(';')
        .next()
        .unwrap_or("image/png")
        .trim()
        .to_string();

    // Refuse les contenus non-image : sans ce check, une URL de PAGE (texte/html)
    // est encodée comme data:text/html;base64,… qui finit dans `assets/` et ne
    // s'affiche pas. Mieux vaut une vraie erreur que d'écrire un faux asset.
    if !mime.starts_with("image/") {
        return Err(format!(
            "Le serveur a renvoyé du « {} » (pas une image). URL probablement non-image."
        , mime));
    }

    // Limite taille à 25 MB
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    if bytes.len() > 25 * 1024 * 1024 {
        return Err("Image trop volumineuse (> 25 MB)".into());
    }

    let b64 = STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

#[tauri::command]
async fn read_image_file(
    path: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let canonical = validate_scope(&path, &app_handle)?;
    let ext = get_ext(&canonical);
    if !ALLOWED_READ_EXTS.contains(&ext.as_str()) {
        return Err(format!("Extension non autorisée: .{ext}"));
    }
    let bytes = tokio::fs::read(&canonical)
        .await
        .map_err(|e| e.to_string())?;
    if bytes.len() > 100 * 1024 * 1024 {
        return Err("Fichier trop volumineux (> 100 MB)".into());
    }
    let mime = mime_guess::from_path(&canonical)
        .first_or_octet_stream()
        .to_string();
    let b64 = STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

#[tauri::command]
async fn open_in_app(
    path: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 1) Scope check (refuse UNC, chemins hors zones autorisées)
    let canonical = validate_scope(&path, &app_handle)?;

    // 2) Deny-list : on ouvre tout SAUF les exécutables / scripts (RCE).
    let ext = get_ext(&canonical);
    if FORBIDDEN_OPEN_EXTS.contains(&ext.as_str()) {
        return Err(format!(
            "Ouverture refusée pour des raisons de sécurité (.{ext})"
        ));
    }

    // 3) Normalize slashes pour Windows ShellExecute. On retire le préfixe
    //    `\\?\` que canonicalize ajoute : ShellExecute le rejette.
    #[cfg(windows)]
    let path_str = display_path(&canonical).replace('/', "\\");
    #[cfg(not(windows))]
    let path_str = display_path(&canonical);

    open::that(&path_str).map_err(|e| format!("Impossible d'ouvrir «{}»: {}", path_str, e))
}

#[tauri::command]
async fn write_project_file(
    path: String,
    contents: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // On peut écrire dans un nouveau fichier qui n'existe pas encore : on valide
    // le PARENT (qui doit exister) plutôt que le chemin lui-même.
    let path_buf = PathBuf::from(&path);
    let parent = path_buf
        .parent()
        .ok_or_else(|| "Chemin sans parent valide".to_string())?;
    validate_scope(&parent.to_string_lossy(), &app_handle)?;

    // Whitelist d'extensions pour les écritures
    let ext = get_ext(&path_buf);
    if !["glucose", "json"].contains(&ext.as_str()) {
        return Err(format!("Extension non autorisée pour écriture: .{ext}"));
    }
    if contents.len() > 200 * 1024 * 1024 {
        return Err("Contenu trop volumineux (> 200 MB)".into());
    }

    tokio::fs::write(&path, contents)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_project_file(
    path: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let canonical = validate_scope(&path, &app_handle)?;
    let ext = get_ext(&canonical);
    if !["glucose", "json"].contains(&ext.as_str()) {
        return Err(format!("Extension non autorisée: .{ext}"));
    }
    let content = tokio::fs::read_to_string(&canonical)
        .await
        .map_err(|e| e.to_string())?;
    if content.len() > 200 * 1024 * 1024 {
        return Err("Fichier trop volumineux".into());
    }
    Ok(content)
}

// ════════════════════════════════════════════════════════════════════════════
// Phase 7.2 — Format binaire `.glucose` v2 (Automerge)
// ════════════════════════════════════════════════════════════════════════════
//
// Le contenu Automerge n'est PAS du UTF-8 valide → on ne peut pas réutiliser
// `read_project_file` (qui fait `read_to_string`). On expose donc deux
// commandes dédiées qui transportent les bytes en base64 entre Rust et JS.

#[tauri::command]
async fn read_glucose_binary(
    path: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let canonical = validate_scope(&path, &app_handle)?;
    let ext = get_ext(&canonical);
    if !["glucose", "atelier"].contains(&ext.as_str()) {
        return Err(format!("Extension non autorisée pour lecture binaire: .{ext}"));
    }
    let bytes = tokio::fs::read(&canonical)
        .await
        .map_err(|e| e.to_string())?;
    if bytes.len() > 500 * 1024 * 1024 {
        return Err("Fichier trop volumineux (> 500 MB)".into());
    }
    Ok(STANDARD.encode(&bytes))
}

#[tauri::command]
async fn write_glucose_binary(
    path: String,
    base64_data: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    let parent = path_buf
        .parent()
        .ok_or_else(|| "Chemin sans parent valide".to_string())?;
    validate_scope(&parent.to_string_lossy(), &app_handle)?;

    let ext = get_ext(&path_buf);
    if !["glucose", "atelier"].contains(&ext.as_str()) {
        return Err(format!("Extension non autorisée pour écriture binaire: .{ext}"));
    }

    let bytes = STANDARD.decode(base64_data).map_err(|e| e.to_string())?;
    if bytes.len() > 500 * 1024 * 1024 {
        return Err("Contenu trop volumineux".into());
    }
    tokio::fs::write(&path, bytes)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn write_binary_file(
    path: String,
    base64_data: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    let parent = path_buf
        .parent()
        .ok_or_else(|| "Chemin sans parent valide".to_string())?;
    validate_scope(&parent.to_string_lossy(), &app_handle)?;

    // Whitelist d'extensions binaires (export PNG essentiellement)
    let ext = get_ext(&path_buf);
    if !["png", "jpg", "jpeg", "webp", "pdf"].contains(&ext.as_str()) {
        return Err(format!("Extension non autorisée pour export binaire: .{ext}"));
    }

    let bytes = STANDARD.decode(base64_data).map_err(|e| e.to_string())?;
    if bytes.len() > 200 * 1024 * 1024 {
        return Err("Fichier trop volumineux".into());
    }
    tokio::fs::write(&path, bytes)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_video(
    url: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // Vérif basique d'URL (pas de SSRF — on ne télécharge que via yt-dlp qui
    // gère lui-même son réseau, mais on refuse au moins les chemins locaux).
    let parsed = url::Url::parse(&url).map_err(|e| format!("URL invalide: {e}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("Seuls http/https sont autorisés".into());
    }

    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let videos_dir = data_dir.join("videos");
    tokio::fs::create_dir_all(&videos_dir)
        .await
        .map_err(|e| e.to_string())?;

    let yt_dlp = ensure_yt_dlp(&data_dir).await?;
    let out_str = videos_dir
        .join("%(id)s.%(ext)s")
        .to_string_lossy()
        .into_owned();

    // Timeout 5 minutes (CLEANUP.md R-06)
    let cmd_future = tokio::process::Command::new(&yt_dlp)
        .args([
            url.as_str(),
            "--no-playlist",
            "--format",
            "bestvideo[ext=mp4][height<=720]+bestaudio[ext=m4a]/best[ext=mp4][height<=720]/best[height<=720]/best",
            "-o",
            &out_str,
            "--print",
            "after_move:filepath",
            "--no-warnings",
        ])
        .output();

    let output = tokio::time::timeout(std::time::Duration::from_secs(300), cmd_future)
        .await
        .map_err(|_| "Téléchargement vidéo : timeout (5 min)".to_string())?
        .map_err(|e| format!("Impossible de lancer yt-dlp: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let path = stdout.trim().to_string();
    if path.is_empty() {
        return Err("yt-dlp n'a pas retourné de chemin de fichier".into());
    }
    Ok(path)
}

// ════════════════════════════════════════════════════════════════════════════
// yt-dlp pinning + checksum (CLEANUP.md SEC-05)
// ════════════════════════════════════════════════════════════════════════════
//
// Plutôt que de télécharger "latest" sans vérification, on pinne une version
// connue ET on valide le SHA256 contre une const hardcoded. Si l'upstream
// est compromis (cf. xz-utils 2024) ou si l'utilisateur est sur un réseau
// MITM, le hash ne correspondra pas et le binaire sera rejeté.
//
// Pour mettre à jour : changer YT_DLP_VERSION + récupérer les nouveaux SHA256
// sur https://github.com/yt-dlp/yt-dlp/releases/download/<version>/SHA2-256SUMS

const YT_DLP_VERSION: &str = "2024.11.18";

#[cfg(windows)]
const YT_DLP_URL: &str =
    "https://github.com/yt-dlp/yt-dlp/releases/download/2024.11.18/yt-dlp.exe";
#[cfg(target_os = "macos")]
const YT_DLP_URL: &str =
    "https://github.com/yt-dlp/yt-dlp/releases/download/2024.11.18/yt-dlp_macos";
#[cfg(all(not(windows), not(target_os = "macos")))]
const YT_DLP_URL: &str =
    "https://github.com/yt-dlp/yt-dlp/releases/download/2024.11.18/yt-dlp";

// SHA256 officiels de la release 2024.11.18 (source : SHA2-256SUMS upstream).
#[cfg(windows)]
const YT_DLP_SHA256: &str =
    "4d88a8ce1bff829c7167dd21e8b4a8eeb0db1441bc27340f0896bbe781c9c3c0";
#[cfg(target_os = "macos")]
const YT_DLP_SHA256: &str =
    "3ea2a0c1768b630dfad127239882af4ea60ba6116bca0a6ce64974759eba0005";
#[cfg(all(not(windows), not(target_os = "macos")))]
const YT_DLP_SHA256: &str =
    "78b4454c83d0f7efe9b26163e82bede0febf0039ae6bacf2963abcae941ac11a";

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

async fn ensure_yt_dlp(data_dir: &std::path::Path) -> Result<std::path::PathBuf, String> {
    // 1) À côté de l'exécutable (bundle production)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            #[cfg(windows)]
            let c = dir.join("yt-dlp.exe");
            #[cfg(not(windows))]
            let c = dir.join("yt-dlp");
            if c.exists() {
                return Ok(c);
            }
        }
    }

    // 2) Cache app data
    #[cfg(windows)]
    let cached = data_dir.join("yt-dlp.exe");
    #[cfg(not(windows))]
    let cached = data_dir.join("yt-dlp");
    if cached.exists() {
        return Ok(cached);
    }

    // 3) Système (PATH)
    let probe = tokio::process::Command::new(if cfg!(windows) { "yt-dlp.exe" } else { "yt-dlp" })
        .arg("--version")
        .output()
        .await;
    if let Ok(out) = probe {
        if out.status.success() {
            return Ok(std::path::PathBuf::from(if cfg!(windows) {
                "yt-dlp.exe"
            } else {
                "yt-dlp"
            }));
        }
    }

    // 4) Téléchargement de la version PINNED
    tokio::fs::create_dir_all(data_dir)
        .await
        .map_err(|e| e.to_string())?;

    let client = reqwest::Client::builder()
        .user_agent(format!("Glucose/{}", env!("CARGO_PKG_VERSION")))
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(YT_DLP_URL)
        .send()
        .await
        .map_err(|e| format!("Téléchargement yt-dlp impossible: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Téléchargement yt-dlp ({YT_DLP_VERSION}): HTTP {}",
            resp.status()
        ));
    }

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;

    // CLEANUP SEC-05 : vérification SHA256 — refus si l'upstream est compromis
    // ou si on est victime d'un MITM. Aucun fichier n'est écrit en cas d'échec.
    let actual = sha256_hex(&bytes);
    if !actual.eq_ignore_ascii_case(YT_DLP_SHA256) {
        return Err(format!(
            "yt-dlp {} : checksum SHA256 invalide. Attendu {}, reçu {}. \
             Téléchargement annulé pour des raisons de sécurité.",
            YT_DLP_VERSION, YT_DLP_SHA256, actual
        ));
    }
    eprintln!(
        "[ensure_yt_dlp] Verified yt-dlp {} ({} bytes, sha256 OK)",
        YT_DLP_VERSION,
        bytes.len()
    );

    tokio::fs::write(&cached, &bytes)
        .await
        .map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&cached)
            .await
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&cached, perms)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(cached)
}

// ════════════════════════════════════════════════════════════════════════════
// Phase 7.0 — Assets externalisés (CRDT-friendly)
// ════════════════════════════════════════════════════════════════════════════
//
// Plutôt que de stocker les images en base64 inline dans `.glucose` (bloat
// catastrophique pour Automerge), on les sauvegarde dans
// `app_data_dir/assets/<sha256>.<ext>` et on stocke seulement l'identifiant
// logique `asset:<sha256>.<ext>` côté projet.
// Le hash SHA-256 sert de nom : déduplication automatique (deux fois la
// même image = un seul fichier sur disque).

fn assets_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("assets");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

/// Renvoie le chemin absolu du dossier assets (utilisable depuis le frontend
/// via `convertFileSrc`).
#[tauri::command]
async fn get_assets_dir(app_handle: tauri::AppHandle) -> Result<String, String> {
    let dir = assets_dir(&app_handle)?;
    Ok(dir.to_string_lossy().into_owned())
}

/// Sauvegarde un asset (image / petit binaire) dans le dossier assets et
/// renvoie son nom de fichier (`<hash>.<ext>`). La déduplication par hash est
/// automatique : deux appels avec le même contenu écrivent le fichier une
/// seule fois.
///
/// `base64_data` peut être :
///   - une data URL complète (`data:image/png;base64,...`)
///   - le base64 brut (sans préfixe)
/// `ext_hint` : extension souhaitée si pas déductible du data URL (`png`, `jpg`…).
#[tauri::command]
async fn save_asset(
    base64_data: String,
    ext_hint: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // 1) Extraire la partie base64 et l'extension probable
    let (b64, ext) = if let Some(stripped) = base64_data.strip_prefix("data:") {
        // Format : data:<mime>;base64,<payload>
        let mut parts = stripped.splitn(2, ',');
        let header_part = parts.next().unwrap_or("");
        let payload = parts.next().ok_or("data URL malformée".to_string())?;
        // Devine l'extension depuis le mime
        let mime = header_part.split(';').next().unwrap_or("");
        let inferred = match mime {
            "image/png"  => "png",
            "image/jpeg" => "jpg",
            "image/jpg"  => "jpg",
            "image/webp" => "webp",
            "image/gif"  => "gif",
            "image/avif" => "avif",
            "image/bmp"  => "bmp",
            "image/svg+xml" => "svg",
            _ => "",
        };
        let ext = if !inferred.is_empty() { inferred.to_string() } else { sanitize_ext(&ext_hint)? };
        (payload.to_string(), ext)
    } else {
        (base64_data, sanitize_ext(&ext_hint)?)
    };

    // 2) Décoder
    let bytes = STANDARD.decode(&b64).map_err(|e| format!("base64 invalide: {e}"))?;
    if bytes.len() > 100 * 1024 * 1024 {
        return Err("Asset trop volumineux (> 100 MB)".into());
    }

    // 3) Hash SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hex::encode(hasher.finalize());
    // 16 chars d'hex = 64 bits, suffisant pour la dédup et garde des noms courts
    let short = &hash[..16];
    let filename = format!("{short}.{ext}");

    // 4) Écrire si pas déjà là
    let dir = assets_dir(&app_handle)?;
    let path = dir.join(&filename);
    if !path.exists() {
        tokio::fs::write(&path, &bytes).await.map_err(|e| e.to_string())?;
    }

    Ok(filename)
}

/// Lit un asset par son nom de fichier (`<hash>.<ext>`) et renvoie une data URL.
/// Utilisé pour l'export PNG ou les exports textuels qui ont besoin d'inliner.
#[tauri::command]
async fn load_asset(
    filename: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // Sécurité : refuser tout `..` ou séparateur de chemin — on ne lit QUE
    // dans le dossier assets.
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err("Nom de fichier invalide".into());
    }
    let dir = assets_dir(&app_handle)?;
    let path = dir.join(&filename);
    if !path.exists() {
        return Err(format!("Asset introuvable : {filename}"));
    }
    let bytes = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    let mime = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .to_string();
    let b64 = STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}

/// R-FIL-02 (Sprint 2) — entrée d'un scan de répertoire.
#[derive(serde::Serialize)]
struct DirEntryDto {
    /// Chemin absolu canonicalisé.
    path: String,
    /// Nom du fichier ou dossier (sans le chemin).
    name: String,
    /// Taille en octets (0 pour les dossiers).
    size: u64,
    /// True si c'est un sous-dossier.
    is_dir: bool,
    /// Extension en minuscules sans le point (vide si aucune).
    ext: String,
}

/// R-FIL-02 — scanne un dossier (non-récursif) et renvoie ses entrées.
///
/// Garde-fous sécurité :
/// - `path` est validé via `validate_scope` (refus UNC + scope strict).
/// - Les extensions explicitement dangereuses (`FORBIDDEN_OPEN_EXTS`) sont
///   filtrées du résultat — on ne les expose même pas au front pour ne pas
///   tenter le user à les déposer.
/// - `max_files` plafond strict (limite mémoire / payload Tauri).
#[tauri::command]
async fn scan_directory(
    path: String,
    max_files: u32,
    app_handle: tauri::AppHandle,
) -> Result<Vec<DirEntryDto>, String> {
    let canonical = validate_scope(&path, &app_handle)?;

    if !canonical.is_dir() {
        return Err(format!(
            "{} n'est pas un dossier",
            canonical.display()
        ));
    }

    let mut entries = tokio::fs::read_dir(&canonical)
        .await
        .map_err(|e| format!("Lecture du dossier impossible : {e}"))?;

    let limit = max_files.min(20_000) as usize;
    let mut out: Vec<DirEntryDto> = Vec::with_capacity(64);

    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        if out.len() >= limit {
            break;
        }

        let entry_path = entry.path();
        let name = entry
            .file_name()
            .to_string_lossy()
            .into_owned();

        // Skip les fichiers cachés Unix-style et les .DS_Store / Thumbs.db.
        // L'utilisateur préfère ne pas les voir polluer la grille.
        if name.starts_with('.') && name != ".env" && name != ".gitignore" && name != ".gitattributes" {
            continue;
        }
        if name == "Thumbs.db" || name == "desktop.ini" {
            continue;
        }

        let meta = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue, // entrée illisible — on l'ignore silencieusement
        };

        let is_dir = meta.is_dir();
        let size = if is_dir { 0 } else { meta.len() };
        let ext = get_ext(&entry_path);

        // Masque le bruit binaire OS (exe/dll/...) — les sources restent visibles.
        if !is_dir && SCAN_HIDE_EXTS.contains(&ext.as_str()) {
            continue;
        }

        out.push(DirEntryDto {
            path: display_path(&entry_path),
            name,
            size,
            is_dir,
            ext,
        });
    }

    // Tri stable : dossiers d'abord, puis par nom (case-insensitive).
    out.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(out)
}

/// R-FIL-01/02 — extensions textuelles lisibles inline (doit rester en phase
/// avec TEXT_FILE_EXTS + CODE_FILE_EXTS côté front, cf. dropHandler.ts).
const TEXT_READ_EXTS: &[&str] = &[
    // texte / data
    "txt", "md", "markdown", "json", "jsonl", "csv", "tsv", "log",
    "yaml", "yml", "toml", "ini", "env", "xml", "html", "htm",
    "conf", "cfg", "gitignore", "gitattributes",
    // code
    "ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "rs", "go",
    "c", "cc", "cpp", "h", "hpp", "java", "rb", "php", "swift", "kt",
    "cs", "scala", "sh", "bash", "zsh", "fish", "sql", "lua",
    "r", "jl", "hs", "erl", "ex", "exs", "clj", "fs", "dart", "nim",
    "zig", "asm", "s", "vim", "tex", "bib",
];

/// Plafond de lecture inline d'un fichier texte (100 KB — au-delà on tronque).
const TEXT_INLINE_MAX_BYTES: usize = 100_000;

/// Résultat de lecture d'un fichier texte (contenu + flag de troncature).
#[derive(serde::Serialize)]
struct TextFileDto {
    content: String,
    truncated: bool,
}

/// R-FIL-01/02 — lit un fichier texte/code par chemin absolu (le drag-drop
/// natif nous donne des chemins, pas des objets `File`). Tronque à
/// `TEXT_INLINE_MAX_BYTES` et signale la troncature.
#[tauri::command]
async fn read_text_file_inline(
    path: String,
    app_handle: tauri::AppHandle,
) -> Result<TextFileDto, String> {
    let canonical = validate_scope(&path, &app_handle)?;
    let ext = get_ext(&canonical);
    // Les fichiers spéciaux sans extension (.gitignore, .env) tombent ici via
    // leur nom : on accepte aussi par nom de fichier complet.
    let name = canonical
        .file_name()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let name_ok = matches!(name.as_str(), ".env" | ".gitignore" | ".gitattributes");
    if !TEXT_READ_EXTS.contains(&ext.as_str()) && !name_ok {
        return Err(format!("Extension non lisible en texte: .{ext}"));
    }
    let bytes = tokio::fs::read(&canonical)
        .await
        .map_err(|e| e.to_string())?;
    let truncated = bytes.len() > TEXT_INLINE_MAX_BYTES;
    let slice = if truncated { &bytes[..TEXT_INLINE_MAX_BYTES] } else { &bytes[..] };
    // from_utf8_lossy : un fichier "texte" mal encodé (latin-1, binaire déguisé)
    // ne doit pas faire planter — on remplace les octets invalides.
    let content = String::from_utf8_lossy(slice).into_owned();
    Ok(TextFileDto { content, truncated })
}

/// Classification d'un chemin droppé (fichier vs dossier + métadonnées).
#[derive(serde::Serialize)]
struct PathInfoDto {
    path: String,
    name: String,
    is_dir: bool,
    ext: String,
    size: u64,
    /// True si lisible inline (texte/code).
    is_text: bool,
    /// True si image affichable.
    is_image: bool,
}

/// R-FIL — classe une liste de chemins droppés (le natif ne dit pas si c'est un
/// dossier). Permet au front de router : dossier→mirror, texte→inline,
/// image→embed, autre→launcher. Valide chaque chemin (scope) et ignore
/// silencieusement les chemins hors-scope / illisibles.
#[tauri::command]
async fn classify_paths(
    paths: Vec<String>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<PathInfoDto>, String> {
    const IMAGE_EXTS: &[&str] = &[
        "png", "jpg", "jpeg", "gif", "webp", "avif", "svg", "bmp", "tif", "tiff",
    ];
    let mut out = Vec::with_capacity(paths.len());
    for p in paths {
        // Hors-scope → on saute (pas d'erreur fatale : un drop mixte peut
        // contenir un chemin interdit sans invalider les autres).
        let canonical = match validate_scope(&p, &app_handle) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let meta = match std::fs::metadata(&canonical) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let is_dir = meta.is_dir();
        let ext = get_ext(&canonical);
        let name = canonical
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name_lower = name.to_lowercase();
        let is_text = !is_dir
            && (TEXT_READ_EXTS.contains(&ext.as_str())
                || matches!(name_lower.as_str(), ".env" | ".gitignore" | ".gitattributes"));
        let is_image = !is_dir && IMAGE_EXTS.contains(&ext.as_str());
        out.push(PathInfoDto {
            path: display_path(&canonical),
            name,
            is_dir,
            ext,
            size: if is_dir { 0 } else { meta.len() },
            is_text,
            is_image,
        });
    }
    Ok(out)
}

/// R-FIL-02 v2 — nœud d'un scan récursif (arbre de dossiers).
#[derive(serde::Serialize)]
struct DirNodeDto {
    path: String,
    name: String,
    is_dir: bool,
    ext: String,
    size: u64,
    /// Date de dernière modification (epoch secondes) — pour le tri R-FIL-03.
    modified: u64,
    /// R-FIL-02 v2 — contenu texte inline pour les fichiers texte/code sous la
    /// limite (sinon None → launcher). Permet d'afficher un .md/.txt comme
    /// bloc texte directement dans le folder, sans round-trip par fichier.
    text: Option<String>,
    /// Enfants directs (vide pour un fichier ou au-delà de max_depth).
    children: Vec<DirNodeDto>,
}

/// Lit le contenu texte inline d'un fichier si éligible (extension texte/code,
/// taille sous la limite, budget global restant). Décrémente le budget.
fn read_inline_text(
    path: &Path,
    ext: &str,
    name: &str,
    size: u64,
    text_budget: &mut usize,
) -> Option<String> {
    let name_lower = name.to_lowercase();
    let is_text = TEXT_READ_EXTS.contains(&ext)
        || matches!(name_lower.as_str(), ".env" | ".gitignore" | ".gitattributes");
    if !is_text || size as usize > TEXT_INLINE_MAX_BYTES || *text_budget == 0 {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let take = bytes.len().min(TEXT_INLINE_MAX_BYTES).min(*text_budget);
    *text_budget -= take;
    Some(String::from_utf8_lossy(&bytes[..take]).into_owned())
}

/// True si on doit ignorer cette entrée (cachés, Thumbs.db, exécutables).
fn skip_entry(name: &str, ext: &str, is_dir: bool) -> bool {
    if name.starts_with('.')
        && name != ".env"
        && name != ".gitignore"
        && name != ".gitattributes"
    {
        return true;
    }
    if name == "Thumbs.db" || name == "desktop.ini" {
        return true;
    }
    if !is_dir && SCAN_HIDE_EXTS.contains(&ext) {
        return true;
    }
    false
}

fn mtime_secs(meta: &std::fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Récursion synchrone (exécutée dans spawn_blocking). `budget` est un plafond
/// global d'entrées partagé entre tous les niveaux pour éviter d'exploser sur
/// une arborescence énorme. `depth` borne la profondeur.
fn scan_dir_rec(
    dir: &Path,
    depth: u32,
    max_depth: u32,
    budget: &mut usize,
    text_budget: &mut usize,
) -> Vec<DirNodeDto> {
    if depth >= max_depth || *budget == 0 {
        return Vec::new();
    }
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut entries: Vec<std::fs::DirEntry> = rd.filter_map(|e| e.ok()).collect();
    // Tri stable : dossiers d'abord puis nom (le front re-trie selon R-FIL-03).
    entries.sort_by(|a, b| {
        let ad = a.path().is_dir();
        let bd = b.path().is_dir();
        match (ad, bd) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a
                .file_name()
                .to_string_lossy()
                .to_lowercase()
                .cmp(&b.file_name().to_string_lossy().to_lowercase()),
        }
    });

    let mut out = Vec::new();
    for entry in entries {
        if *budget == 0 {
            break;
        }
        let p = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let ext = get_ext(&p);
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let is_dir = meta.is_dir();
        if skip_entry(&name, &ext, is_dir) {
            continue;
        }
        *budget -= 1;
        let size = if is_dir { 0 } else { meta.len() };
        let text = if is_dir {
            None
        } else {
            read_inline_text(&p, &ext, &name, size, text_budget)
        };
        let children = if is_dir {
            scan_dir_rec(&p, depth + 1, max_depth, budget, text_budget)
        } else {
            Vec::new()
        };
        out.push(DirNodeDto {
            path: display_path(&p),
            name,
            is_dir,
            ext,
            size,
            modified: mtime_secs(&meta),
            text,
            children,
        });
    }
    out
}

/// R-FIL-02 v2 — scanne un dossier **récursivement** et renvoie l'arbre complet
/// (borné par `max_entries` et `max_depth`). Sert à construire un folder miroir
/// avec sous-dossiers navigables.
///
/// Sécurité : `validate_scope` sur la racine (refus UNC + scope strict). La
/// récursion reste sous la racine canonicalisée (les liens symboliques vers
/// l'extérieur ne re-passent pas par validate_scope, donc on borne la
/// profondeur pour éviter les cycles).
#[tauri::command]
async fn scan_tree(
    path: String,
    max_entries: u32,
    max_depth: u32,
    app_handle: tauri::AppHandle,
) -> Result<DirNodeDto, String> {
    let canonical = validate_scope(&path, &app_handle)?;
    if !canonical.is_dir() {
        return Err(format!("{} n'est pas un dossier", canonical.display()));
    }
    let depth_cap = max_depth.min(16);
    let entry_cap = max_entries.min(40_000) as usize;

    let root = canonical.clone();
    let node = tokio::task::spawn_blocking(move || {
        let mut budget = entry_cap;
        // Budget global d'octets de texte inline (4 MB) — borne la lecture
        // de contenu pour ne pas exploser sur un dossier plein de gros .md/.csv.
        let mut text_budget: usize = 4 * 1024 * 1024;
        let children = scan_dir_rec(&root, 0, depth_cap, &mut budget, &mut text_budget);
        let name = root
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.to_string_lossy().into_owned());
        let modified = std::fs::metadata(&root)
            .as_ref()
            .map(mtime_secs)
            .unwrap_or(0);
        DirNodeDto {
            path: display_path(&root),
            name,
            is_dir: true,
            ext: String::new(),
            size: 0,
            modified,
            text: None,
            children,
        }
    })
    .await
    .map_err(|e| format!("Scan interrompu: {e}"))?;

    Ok(node)
}

/// Whitelist d'extensions acceptées pour `save_asset`.
fn sanitize_ext(ext: &str) -> Result<String, String> {
    let lower = ext.trim_start_matches('.').to_lowercase();
    const ALLOWED: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "avif", "bmp", "svg"];
    if !ALLOWED.contains(&lower.as_str()) {
        return Err(format!("Extension d'asset non autorisée : .{lower}"));
    }
    Ok(lower)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mp_state: SharedMpState = Arc::new(Mutex::new(MpState::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(mp_state)
        .invoke_handler(tauri::generate_handler![
            fetch_image,
            read_image_file,
            open_in_app,
            write_project_file,
            read_project_file,
            write_binary_file,
            download_video,
            // Phase 7.0 — assets externalisés
            save_asset,
            load_asset,
            get_assets_dir,
            // R-FIL-02 (Sprint 2) — drop d'un dossier OS = folder mirror
            scan_directory,
            scan_tree,
            // R-FIL (Sprint 2) — drag-drop natif : classification + lecture texte
            classify_paths,
            read_text_file_inline,
            // Phase 7.2 — format binaire `.glucose` v2 (Automerge)
            read_glucose_binary,
            write_glucose_binary,
            // Phase 7.5bis — multi-utilisateur LAN
            multiplayer::mp_start,
            multiplayer::mp_stop,
            multiplayer::mp_connect,
            multiplayer::mp_send_patch,
            multiplayer::mp_peers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ════════════════════════════════════════════════════════════════════════════
// Tests unitaires des helpers purs (sécurité chemins + utilitaires)
// ════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::Path;

    #[test]
    fn extended_length_prefix_is_not_unc() {
        // RÉGRESSION : le préfixe \\?\ (canonicalize + drag-drop natif Windows)
        // NE doit PAS être pris pour de l'UNC. C'était LE bug « rien ne marche ».
        assert!(!is_forbidden_unc(r"\\?\C:\Users\me\scene.blend"));
        assert!(!is_forbidden_unc(r"C:\Users\me\scene.blend"));
        assert!(!is_forbidden_unc("/home/me/scene.blend"));
    }

    #[test]
    fn real_unc_is_rejected() {
        assert!(is_forbidden_unc(r"\\server\share\file"));
        assert!(is_forbidden_unc(r"\\?\UNC\server\share"));
        assert!(is_forbidden_unc(r"\\?\unc\server\share"));
        assert!(is_forbidden_unc("//server/share"));
    }

    #[test]
    fn display_path_strips_extended_prefix() {
        // ShellExecute n'aime pas \\?\ → display_path doit le retirer.
        assert_eq!(display_path(Path::new(r"\\?\C:\Users\me")), r"C:\Users\me");
        assert_eq!(display_path(Path::new(r"C:\Users\me")), r"C:\Users\me");
    }

    #[test]
    fn get_ext_is_lowercased() {
        assert_eq!(get_ext(Path::new("Scene.BLEND")), "blend");
        assert_eq!(get_ext(Path::new("archive.tar.GZ")), "gz");
        assert_eq!(get_ext(Path::new("README")), "");
    }

    #[test]
    fn public_ip_blocks_private_and_cloud_metadata() {
        assert!(is_public_ip(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
        assert!(!is_public_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert!(!is_public_ip(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5))));
        assert!(!is_public_ip(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
        // 169.254.169.254 = métadonnées cloud (AWS/GCP/Azure) — anti-SSRF.
        assert!(!is_public_ip(&IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254))));
    }
}
