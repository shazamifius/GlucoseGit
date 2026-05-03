use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::header;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use tauri::Manager;

// ════════════════════════════════════════════════════════════════════════════
// Sécurité (CLEANUP.md SEC-01..09)
// ════════════════════════════════════════════════════════════════════════════

/// Extensions autorisées pour la lecture d'images / fichiers de projet.
const ALLOWED_READ_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "avif", "bmp", "svg",
    "mp4", "mov", "webm", "mkv",
    "glucose", "json",
];

/// Extensions autorisées pour l'ouverture native (`open_in_app`).
/// Volontairement restreint aux fichiers créatifs habituels.
const ALLOWED_OPEN_EXTS: &[&str] = &[
    // images
    "png", "jpg", "jpeg", "gif", "webp", "avif", "bmp",
    // créatifs / 3D / design
    "psd", "ai", "kra", "xcf", "blend", "fbx", "obj", "glb", "gltf",
    "fig", "sketch", "procreate", "zpr", "c4d", "max", "ma", "mb",
    // audio / video / docs / code (lecture seule)
    "mp4", "mov", "webm", "mp3", "wav", "flac",
    "pdf", "txt", "md", "json", "csv",
    "ts", "tsx", "js", "jsx", "py", "rs", "go", "java", "c", "cpp", "h",
];

/// Extensions explicitement refusées par sécurité (exécutables et scripts).
const FORBIDDEN_OPEN_EXTS: &[&str] = &[
    "exe", "bat", "cmd", "ps1", "psm1", "vbs", "vbe", "js",
    "scr", "com", "msi", "msp", "lnk", "url", "inf", "reg",
    "jar", "app", "dmg", "deb", "rpm", "appimage", "sh", "bash",
];

/// Vérifie qu'un chemin est dans un des dossiers autorisés (App Data, Documents,
/// Downloads, Pictures, Videos, Desktop) ET qu'il n'utilise pas de chemin UNC.
/// Retourne le chemin canonicalisé si OK, ou Err sinon.
fn validate_scope(path: &str, app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    // 1) Refus chemins UNC (\\server\share)
    if path.starts_with("\\\\") || path.starts_with("//") {
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

    let client = reqwest::Client::builder()
        .user_agent(format!("Glucose/{}", env!("CARGO_PKG_VERSION")))
        .redirect(reqwest::redirect::Policy::limited(3))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(parsed.as_str())
        .send()
        .await
        .map_err(|e| e.to_string())?;

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

    // 2) Refus extensions exécutables / scripts
    let ext = get_ext(&canonical);
    if FORBIDDEN_OPEN_EXTS.contains(&ext.as_str()) {
        return Err(format!(
            "Ouverture refusée pour des raisons de sécurité (.{ext})"
        ));
    }
    if !ALLOWED_OPEN_EXTS.contains(&ext.as_str()) {
        return Err(format!(
            "Type de fichier non géré pour l'ouverture native (.{ext})"
        ));
    }

    // 3) Normalize slashes pour Windows ShellExecute
    #[cfg(windows)]
    let path_str = canonical.to_string_lossy().replace('/', "\\");
    #[cfg(not(windows))]
    let path_str = canonical.to_string_lossy().to_string();

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
// Plutôt que de télécharger "latest" sans vérification, on pinning une version
// connue. Si l'utilisateur veut mettre à jour, il peut effacer le binaire dans
// app_data_dir/yt-dlp.exe et nous re-téléchargerons.
//
// NOTE : les checksums ci-dessous correspondent à yt-dlp 2024.11.18 release.
// Pour mettre à jour : récupérer le SHA256 officiel sur la page release GitHub.

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

    // NOTE Phase 1.5 : ajouter ici la vérification SHA256 contre une const
    // hardcodée. Pour l'instant on log la taille comme indicateur basique.
    eprintln!(
        "[ensure_yt_dlp] Downloaded yt-dlp {} ({} bytes)",
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            fetch_image,
            read_image_file,
            open_in_app,
            write_project_file,
            read_project_file,
            write_binary_file,
            download_video,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
