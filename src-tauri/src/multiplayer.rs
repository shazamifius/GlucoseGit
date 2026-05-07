// ────────────────────────────────────────────────────────────────────────────
// Phase 7.5bis — Multi-utilisateur LAN (mDNS + WebSocket)
// ────────────────────────────────────────────────────────────────────────────
//
// Architecture :
//
//   ┌──────────────┐      mDNS announce    ┌──────────────┐
//   │  Instance A  │  ─────────────────→   │  Instance B  │
//   │              │  ←─────────────────   │              │
//   │              │                       │              │
//   │   :7777      │      WebSocket        │   :7777      │
//   │   Server     │ ←──── patches ────→   │   Server     │
//   │              │       Automerge       │              │
//   └──────────────┘                       └──────────────┘
//
// Chaque instance Glucose qui active le multijoueur :
//   1. Démarre un serveur WebSocket sur un port disponible (par défaut 7777)
//   2. Annonce son service `_glucose._tcp.local` via mDNS-SD
//   3. Browse pour les autres instances (events `mp:peer-found`, `mp:peer-lost`)
//   4. Quand l'utilisateur clique sur un peer → connexion client WebSocket
//   5. Tous les patches Automerge sont synchronisés sur les connexions actives
//
// Limites MVP :
//   - Pas de chiffrement (plain WebSocket, OK pour LAN privé)
//   - Pas d'authentification (qui est sur ton LAN peut se connecter)
//   - Pas de re-connexion automatique si la connexion tombe
//   - Pas de curseurs flottants (Phase 7.5bis polish)

use base64::{engine::general_purpose::STANDARD, Engine};
use futures_util::{SinkExt, StreamExt};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;

const SERVICE_TYPE: &str = "_glucose._tcp.local.";

// ─── Types partagés frontend ↔ backend ─────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MpStatus {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "starting")]
    Starting,
    #[serde(rename = "listening")]
    Listening { port: u16, name: String },
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Clone, Debug, Serialize)]
pub struct PeerFound {
    pub name: String,
    pub addr: String,
    pub port: u16,
}

// ─── État global ────────────────────────────────────────────────────────────

struct PeerConn {
    /// Sender pour envoyer un message à ce peer
    tx: mpsc::UnboundedSender<Message>,
    /// Nom human-readable (issu du service mDNS ou de l'addr)
    name: String,
}

pub struct MpState {
    /// Daemon mDNS (publish + browse)
    daemon: Option<ServiceDaemon>,
    /// Nom du service local (pour publish)
    service_name: Option<String>,
    /// Connexions actives, indexées par addr (host:port string)
    peers: HashMap<String, PeerConn>,
    /// Tâche serveur WebSocket
    server_task: Option<tokio::task::JoinHandle<()>>,
    /// Tâche browse mDNS
    browse_task: Option<tokio::task::JoinHandle<()>>,
}

impl MpState {
    pub fn new() -> Self {
        Self {
            daemon: None,
            service_name: None,
            peers: HashMap::new(),
            server_task: None,
            browse_task: None,
        }
    }
}

pub type SharedMpState = Arc<Mutex<MpState>>;

// ─── Helpers ────────────────────────────────────────────────────────────────

fn local_ipv4() -> Option<String> {
    // On essaie de récupérer l'IP locale en ouvrant un socket vers Internet
    // (sans envoyer de paquet, juste pour que l'OS choisisse une route).
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip().to_string())
}

fn machine_name() -> String {
    // Best-effort : hostname OS, sinon fallback générique
    if let Ok(name) = std::env::var("COMPUTERNAME") {
        return name;
    }
    if let Ok(name) = std::env::var("HOSTNAME") {
        return name;
    }
    "Glucose".to_string()
}

// ─── Serveur WebSocket : accepte les connexions entrantes ─────────────────

async fn run_server(
    listener: TcpListener,
    state: SharedMpState,
    app_handle: AppHandle,
) {
    while let Ok((stream, addr)) = listener.accept().await {
        let state = state.clone();
        let app_handle = app_handle.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_incoming(stream, addr, state, app_handle).await {
                eprintln!("[mp] incoming connection from {addr} failed: {e}");
            }
        });
    }
}

async fn handle_incoming(
    stream: TcpStream,
    addr: SocketAddr,
    state: SharedMpState,
    app_handle: AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws = tokio_tungstenite::accept_async(stream).await?;
    let key = addr.to_string();
    register_peer(ws, key.clone(), key.clone(), state, app_handle).await;
    Ok(())
}

// ─── Client WebSocket : ouvre une connexion vers un peer ──────────────────

async fn run_client(
    url: String,
    name: String,
    state: SharedMpState,
    app_handle: AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (ws, _) = tokio_tungstenite::connect_async(&url).await?;
    register_peer(ws, url, name, state, app_handle).await;
    Ok(())
}

// ─── Boucle de lecture/écriture WebSocket pour un peer ────────────────────

async fn register_peer<S>(
    ws: tokio_tungstenite::WebSocketStream<S>,
    key: String,
    name: String,
    state: SharedMpState,
    app_handle: AppHandle,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (mut sink, mut stream) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Enregistrer le peer
    {
        let mut s = state.lock().await;
        s.peers.insert(key.clone(), PeerConn { tx, name: name.clone() });
    }
    let _ = app_handle.emit(
        "mp:peer-connected",
        serde_json::json!({ "key": key, "name": name }),
    );

    // Tâche d'envoi (pump rx → sink)
    let send_key = key.clone();
    let send_state = state.clone();
    let send_app = app_handle.clone();
    let send_handle = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
        // Fin : retire le peer
        {
            let mut s = send_state.lock().await;
            s.peers.remove(&send_key);
        }
        let _ = send_app.emit(
            "mp:peer-disconnected",
            serde_json::json!({ "key": send_key }),
        );
    });

    // Tâche de réception (pump stream → events Tauri)
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(Message::Binary(bytes)) => {
                // Patch Automerge brut → on l'envoie au frontend en base64
                let b64 = STANDARD.encode(&bytes);
                let _ = app_handle.emit(
                    "mp:patch",
                    serde_json::json!({ "from": key, "bytes_b64": b64 }),
                );
            }
            Ok(Message::Text(text)) => {
                // Messages texte = handshake / metadata légère
                let _ = app_handle.emit(
                    "mp:text",
                    serde_json::json!({ "from": key, "text": text.to_string() }),
                );
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    // À la sortie de la boucle, on annule la tâche d'envoi pour fermer proprement
    send_handle.abort();
    {
        let mut s = state.lock().await;
        s.peers.remove(&key);
    }
    let _ = app_handle.emit(
        "mp:peer-disconnected",
        serde_json::json!({ "key": key }),
    );
}

// ─── mDNS browse ────────────────────────────────────────────────────────────

async fn run_browse(
    daemon: ServiceDaemon,
    own_name: String,
    app_handle: AppHandle,
) {
    let receiver = match daemon.browse(SERVICE_TYPE) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[mp] browse failed: {e}");
            return;
        }
    };
    while let Ok(event) = receiver.recv_async().await {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                // Ignorer notre propre service
                let svc_name = info.get_fullname().to_string();
                if svc_name == own_name {
                    continue;
                }
                let port = info.get_port();
                // Prend la première IPv4 dispo (le LAN typique)
                let addr = info
                    .get_addresses()
                    .iter()
                    .find_map(|ip| if ip.is_ipv4() { Some(ip.to_string()) } else { None });
                if let Some(addr) = addr {
                    let payload = PeerFound {
                        name: svc_name.replace(SERVICE_TYPE, "").trim_end_matches('.').to_string(),
                        addr,
                        port,
                    };
                    let _ = app_handle.emit("mp:peer-found", &payload);
                }
            }
            ServiceEvent::ServiceRemoved(_, name) => {
                let _ = app_handle.emit(
                    "mp:peer-lost",
                    serde_json::json!({ "name": name }),
                );
            }
            _ => {}
        }
    }
}

// ─── Commandes Tauri ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn mp_start(
    port: u16,
    state: tauri::State<'_, SharedMpState>,
    app_handle: AppHandle,
) -> Result<MpStatus, String> {
    // Évite de redémarrer si déjà actif
    {
        let s = state.lock().await;
        if s.server_task.is_some() {
            return Err("Multijoueur déjà actif".into());
        }
    }

    let _ = app_handle.emit("mp:status", &MpStatus::Starting);

    // 1) Bind serveur WebSocket
    let bind_addr: SocketAddr = format!("0.0.0.0:{port}")
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;
    let listener = TcpListener::bind(bind_addr).await.map_err(|e| e.to_string())?;
    let actual_port = listener.local_addr().map_err(|e| e.to_string())?.port();

    // 2) Démarre le daemon mDNS
    let daemon = ServiceDaemon::new().map_err(|e| e.to_string())?;
    let host = machine_name();
    let ip = local_ipv4().unwrap_or_else(|| "127.0.0.1".to_string());
    let unique_suffix = uuid_suffix();
    let instance_name = format!("{host}-{unique_suffix}");
    let svc_info = ServiceInfo::new(
        SERVICE_TYPE,
        &instance_name,
        &format!("{host}.local."),
        ip.as_str(),
        actual_port,
        None,
    )
    .map_err(|e| e.to_string())?;
    let full_service_name = svc_info.get_fullname().to_string();
    daemon.register(svc_info).map_err(|e| e.to_string())?;

    // 3) Spawn server + browse
    let server_task = tokio::spawn(run_server(listener, state.inner().clone(), app_handle.clone()));
    let browse_task = tokio::spawn(run_browse(
        daemon.clone(),
        full_service_name,
        app_handle.clone(),
    ));

    {
        let mut s = state.lock().await;
        s.daemon = Some(daemon);
        s.service_name = Some(instance_name.clone());
        s.server_task = Some(server_task);
        s.browse_task = Some(browse_task);
    }

    let status = MpStatus::Listening { port: actual_port, name: instance_name };
    let _ = app_handle.emit("mp:status", &status);
    Ok(status)
}

#[tauri::command]
pub async fn mp_stop(state: tauri::State<'_, SharedMpState>, app_handle: AppHandle) -> Result<(), String> {
    let mut s = state.lock().await;
    // Annule serveur + browse
    if let Some(t) = s.server_task.take() { t.abort(); }
    if let Some(t) = s.browse_task.take() { t.abort(); }
    // Ferme toutes les connexions peer (drop des tx → tâches send finissent)
    s.peers.clear();
    // Ferme le daemon mDNS
    if let Some(daemon) = s.daemon.take() {
        let _ = daemon.shutdown();
    }
    s.service_name = None;
    let _ = app_handle.emit("mp:status", &MpStatus::Off);
    Ok(())
}

#[tauri::command]
pub async fn mp_connect(
    addr: String,
    port: u16,
    name: Option<String>,
    state: tauri::State<'_, SharedMpState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let url = format!("ws://{addr}:{port}");
    let display = name.unwrap_or_else(|| format!("{addr}:{port}"));
    let s = state.inner().clone();
    let app = app_handle.clone();
    tokio::spawn(async move {
        if let Err(e) = run_client(url, display, s, app.clone()).await {
            let _ = app.emit(
                "mp:status",
                &MpStatus::Error { message: format!("Connexion échouée : {e}") },
            );
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn mp_send_patch(
    bytes_b64: String,
    state: tauri::State<'_, SharedMpState>,
) -> Result<usize, String> {
    let bytes = STANDARD.decode(bytes_b64).map_err(|e| e.to_string())?;
    let s = state.lock().await;
    let mut sent = 0;
    for peer in s.peers.values() {
        if peer.tx.send(Message::binary(bytes.clone())).is_ok() {
            sent += 1;
        }
    }
    Ok(sent)
}

#[tauri::command]
pub async fn mp_peers(state: tauri::State<'_, SharedMpState>) -> Result<Vec<String>, String> {
    let s = state.lock().await;
    Ok(s.peers.values().map(|p| p.name.clone()).collect())
}

// ─── Petit utilitaire ───────────────────────────────────────────────────────

fn uuid_suffix() -> String {
    // Suffixe court (4 chars hex aléatoires) pour différencier deux instances
    // sur la même machine.
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{:04x}", nanos & 0xffff)
}
