#!/usr/bin/env python3
"""
Glucose — serveur de télémétrie auto-hébergé (NixOS).

UN seul port (défaut 24000), sans AUCUNE dépendance externe (stdlib pure) :

  • POST /ingest   — reçoit les lots envoyés par l'app. Protégé par une clé
                     ÉCRITURE-SEULE (header X-Glucose-Key). La fuiter ne permet que
                     d'*envoyer* des logs, jamais d'en *lire*.
  • GET  /health   — sonde de disponibilité (renvoie « ok »).
  • GET  /         — tableau de bord (lag, GPU, erreurs, matériel, versions).
                     Protégé par mot de passe (HTTP Basic). Désactivé tant que
                     GLUCOSE_DASH_PASS n'est pas défini.

Un port unique parce que, sur la box, 24001 est déjà pris (web3game) et seul un
port libre + ouvert au firewall est disponible. Ingest et dashboard cohabitent
donc sur le même port, distingués par le chemin + une auth différente.

Stockage : SQLite (fichier unique). L'app n'envoie QUE des métriques — jamais le
contenu d'un document.

Variables d'environnement :
  GLUCOSE_INGEST_KEY   clé attendue des clients (défaut = clé par défaut de l'app)
  GLUCOSE_DB           chemin du fichier SQLite (défaut ./glucose_telemetry.db)
  GLUCOSE_PORT         port d'écoute (défaut 24000)
  GLUCOSE_BIND         interface (défaut 0.0.0.0 ; 127.0.0.1 pour un test local)
  GLUCOSE_DASH_USER    identifiant du dashboard (défaut « glucose »)
  GLUCOSE_DASH_PASS    mot de passe du dashboard (OBLIGATOIRE pour l'activer)
"""

import base64
import html
import json
import os
import sqlite3
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

# ── Configuration ────────────────────────────────────────────────────────────

INGEST_KEY = os.environ.get(
    "GLUCOSE_INGEST_KEY", "8b0fb5242afe0c916aa92ac71330fa58a35e9e9f5c40e87d"
)
DB_PATH = os.environ.get("GLUCOSE_DB", "glucose_telemetry.db")
BIND = os.environ.get("GLUCOSE_BIND", "0.0.0.0")  # 127.0.0.1 pour un test local
PORT = int(os.environ.get("GLUCOSE_PORT", "24100"))  # 24000/24001 réservés à web3
DASH_USER = os.environ.get("GLUCOSE_DASH_USER", "glucose")
DASH_PASS = os.environ.get("GLUCOSE_DASH_PASS", "")  # vide → dashboard désactivé
MAX_BODY = 2 * 1024 * 1024  # 2 Mo par lot

# Rate-limit anti-flood : N requêtes max par IP sur une fenêtre glissante.
RATE_MAX = int(os.environ.get("GLUCOSE_RATE_MAX", "120"))
RATE_WINDOW = 60.0

_db_lock = threading.Lock()
_rate_lock = threading.Lock()
_rate_hits: dict = {}  # ip -> [timestamps]


def rate_ok(ip: str) -> bool:
    """True si l'IP est sous le quota. En mémoire, borné (anti-DoS mémoire)."""
    now = time.time()
    with _rate_lock:
        hits = _rate_hits.get(ip)
        if hits is None:
            hits = []
            _rate_hits[ip] = hits
        cutoff = now - RATE_WINDOW
        drop = 0
        for t in hits:
            if t >= cutoff:
                break
            drop += 1
        if drop:
            del hits[:drop]
        if len(hits) >= RATE_MAX:
            return False
        hits.append(now)
        # Borne le nombre d'IP suivies (purge les entrées vides si trop nombreuses).
        if len(_rate_hits) > 5000:
            for k in [k for k, v in _rate_hits.items() if not v]:
                del _rate_hits[k]
        return True


# ── Base de données ──────────────────────────────────────────────────────────

def db() -> sqlite3.Connection:
    conn = sqlite3.connect(DB_PATH)
    conn.execute("PRAGMA journal_mode=WAL")
    return conn


def init_db() -> None:
    with db() as conn:
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS events (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                received_at INTEGER,   -- epoch ms côté serveur
                cid         TEXT,       -- identifiant client anonyme
                session     TEXT,
                os          TEXT,
                arch        TEXT,
                app_version TEXT,
                kind        TEXT,       -- session_start | perf | error | event
                ts          INTEGER,    -- epoch ms côté client
                data        TEXT        -- événement complet (JSON)
            )
            """
        )
        conn.execute("CREATE INDEX IF NOT EXISTS idx_kind ON events(kind)")
        conn.execute("CREATE INDEX IF NOT EXISTS idx_cid ON events(cid)")
        conn.execute("CREATE INDEX IF NOT EXISTS idx_recv ON events(received_at)")


def store_batch(envelope: dict) -> int:
    """Aplati l'enveloppe {os,arch,app_version,payload:{cid,session,events[]}}
    en une ligne par événement. Renvoie le nombre d'événements stockés."""
    os_ = str(envelope.get("os", "?"))[:32]
    arch = str(envelope.get("arch", "?"))[:32]
    ver = str(envelope.get("app_version", "?"))[:32]
    payload = envelope.get("payload") or {}
    cid = str(payload.get("cid", "?"))[:64]
    session = str(payload.get("session", "?"))[:64]
    events = payload.get("events") or []
    if not isinstance(events, list):
        return 0
    now = int(time.time() * 1000)
    rows = []
    for ev in events[:500]:  # plafond dur par lot
        if not isinstance(ev, dict):
            continue
        kind = str(ev.get("kind", "?"))[:32]
        ts = int(ev.get("ts", now)) if str(ev.get("ts", "")).isdigit() else now
        rows.append((now, cid, session, os_, arch, ver, kind, ts, json.dumps(ev)[:16000]))
    if not rows:
        return 0
    with _db_lock, db() as conn:
        conn.executemany(
            "INSERT INTO events (received_at,cid,session,os,arch,app_version,kind,ts,data)"
            " VALUES (?,?,?,?,?,?,?,?,?)",
            rows,
        )
    return len(rows)


# ── Dashboard (rendu HTML) ───────────────────────────────────────────────────

def _q(conn, sql, params=()):
    return conn.execute(sql, params).fetchall()


def bar(value: float, maximum: float, width: int = 180) -> str:
    w = 0 if maximum <= 0 else int(round(width * value / maximum))
    return (
        f'<span style="display:inline-block;height:10px;width:{w}px;'
        f'background:#4ade80;border-radius:2px;vertical-align:middle"></span>'
    )


def esc(v) -> str:
    return html.escape(str(v))


def render_dashboard() -> str:
    with db() as conn:
        total = _q(conn, "SELECT COUNT(*) FROM events")[0][0]
        clients = _q(conn, "SELECT COUNT(DISTINCT cid) FROM events")[0][0]
        sessions = _q(conn, "SELECT COUNT(DISTINCT session) FROM events")[0][0]
        day_ago = int(time.time() * 1000) - 24 * 3600 * 1000
        active_24h = _q(
            conn, "SELECT COUNT(DISTINCT cid) FROM events WHERE received_at > ?", (day_ago,)
        )[0][0]

        os_rows = _q(
            conn,
            "SELECT os, COUNT(DISTINCT cid) c FROM events GROUP BY os ORDER BY c DESC",
        )
        ver_rows = _q(
            conn,
            "SELECT app_version, COUNT(DISTINCT cid) c FROM events"
            " GROUP BY app_version ORDER BY c DESC",
        )

        # GPU + rendu logiciel : on lit les session_start.
        starts = _q(
            conn,
            "SELECT os, data FROM events WHERE kind='session_start' ORDER BY received_at DESC",
        )
        gpu_count: dict = {}
        software_machines = 0
        wayland_linux = 0
        x11_linux = 0
        for os_, data in starts:
            try:
                d = json.loads(data)
            except Exception:
                continue
            gpu = d.get("gpu") or {}
            r = str(gpu.get("renderer", "inconnu"))
            gpu_count[r] = gpu_count.get(r, 0) + 1
            if gpu.get("software"):
                software_machines += 1
            if os_ == "linux":
                if d.get("wayland") is True:
                    wayland_linux += 1
                elif d.get("wayland") is False:
                    x11_linux += 1
        gpu_sorted = sorted(gpu_count.items(), key=lambda kv: -kv[1])

        # Perf agrégée par OS (à partir des events perf).
        perf = _q(
            conn,
            "SELECT os, data FROM events WHERE kind='perf' ORDER BY received_at DESC LIMIT 5000",
        )
        perf_by_os: dict = {}
        for os_, data in perf:
            try:
                d = json.loads(data)
            except Exception:
                continue
            acc = perf_by_os.setdefault(os_, {"n": 0, "fps": 0.0, "worst": 0, "jank": 0, "stalls": 0})
            acc["n"] += 1
            acc["fps"] += float(d.get("fps", 0) or 0)
            acc["worst"] = max(acc["worst"], int(d.get("worstMs", 0) or 0))
            acc["jank"] += int(d.get("jankFrames", 0) or 0)
            acc["stalls"] += int(d.get("stalls", 0) or 0)

        # Dernières erreurs.
        errors = _q(
            conn,
            "SELECT received_at, os, app_version, data FROM events"
            " WHERE kind='error' ORDER BY received_at DESC LIMIT 60",
        )

    # ── HTML ──
    max_os = max((c for _, c in os_rows), default=1)
    out = []
    out.append(
        "<!doctype html><meta charset=utf-8><title>Glucose · Télémétrie</title>"
        "<meta http-equiv=refresh content=30>"
        "<style>"
        "body{background:#0d0d0d;color:#d4d4dd;font:13px/1.6 system-ui,sans-serif;margin:0;padding:24px 28px}"
        "h1{font-size:18px;margin:0 0 2px}h2{font-size:14px;color:#9a9aa0;margin:26px 0 8px;"
        "border-bottom:1px solid #26262e;padding-bottom:5px;text-transform:uppercase;letter-spacing:.06em}"
        ".cards{display:flex;gap:14px;flex-wrap:wrap;margin-top:14px}"
        ".card{background:#16161a;border:1px solid #26262e;border-radius:8px;padding:12px 16px;min-width:120px}"
        ".card .n{font-size:26px;color:#fff;font-weight:600}.card .l{color:#7d7d8c;font-size:11px}"
        "table{border-collapse:collapse;width:100%;max-width:960px}"
        "td,th{text-align:left;padding:4px 10px 4px 0;border-bottom:1px solid #1c1c22;vertical-align:top}"
        "th{color:#7d7d8c;font-weight:500;font-size:11px;text-transform:uppercase;letter-spacing:.05em}"
        ".bad{color:#f87171}.ok{color:#4ade80}.mono{font-family:ui-monospace,monospace;font-size:11.5px}"
        "</style>"
    )
    out.append(f"<h1>Glucose · Télémétrie</h1><div class=l style='color:#7d7d8c'>Actualisé {esc(time.strftime('%Y-%m-%d %H:%M:%S'))} · auto-refresh 30 s</div>")

    out.append("<div class=cards>")
    for label, val in [
        ("clients uniques", clients),
        ("actifs 24 h", active_24h),
        ("sessions", sessions),
        ("événements", total),
        ("rendu LOGICIEL", software_machines),
    ]:
        cls = " bad" if label == "rendu LOGICIEL" and val else ""
        out.append(f"<div class=card><div class='n{cls}'>{esc(val)}</div><div class=l>{esc(label)}</div></div>")
    out.append("</div>")

    # OS
    out.append("<h2>Systèmes</h2><table>")
    for os_, c in os_rows:
        out.append(f"<tr><td style='width:120px'>{esc(os_)}</td><td>{bar(c, max_os)} {esc(c)}</td></tr>")
    out.append("</table>")
    if wayland_linux or x11_linux:
        out.append(
            f"<div class=l style='margin-top:6px;color:#7d7d8c'>Linux : "
            f"{esc(wayland_linux)} Wayland · {esc(x11_linux)} X11</div>"
        )

    # Versions
    out.append("<h2>Versions</h2><table>")
    for v, c in ver_rows:
        out.append(f"<tr><td style='width:160px' class=mono>{esc(v)}</td><td>{esc(c)} client(s)</td></tr>")
    out.append("</table>")

    # GPU
    out.append("<h2>GPU / Renderer</h2><table><tr><th>Renderer</th><th>Machines</th></tr>")
    for r, c in gpu_sorted:
        soft = any(t in r.lower() for t in ("llvmpipe", "swrast", "software", "softpipe"))
        cls = " class=bad" if soft else ""
        tag = " ⚠ logiciel" if soft else ""
        out.append(f"<tr><td class=mono{cls}>{esc(r)}{tag}</td><td>{esc(c)}</td></tr>")
    out.append("</table>")

    # Perf
    out.append("<h2>Performance (moyenne)</h2><table>")
    out.append("<tr><th>OS</th><th>FPS moy.</th><th>Pire frame</th><th>Frames lentes</th><th>Gels</th><th>éch.</th></tr>")
    for os_, a in sorted(perf_by_os.items()):
        n = a["n"] or 1
        fps = a["fps"] / n
        fps_cls = " class=bad" if fps < 30 else (" class=ok" if fps >= 55 else "")
        worst_cls = " class=bad" if a["worst"] > 100 else ""
        out.append(
            f"<tr><td>{esc(os_)}</td><td{fps_cls}>{fps:.0f}</td>"
            f"<td{worst_cls}>{esc(a['worst'])} ms</td><td>{esc(a['jank'])}</td>"
            f"<td>{esc(a['stalls'])}</td><td>{esc(a['n'])}</td></tr>"
        )
    out.append("</table>")

    # Erreurs
    out.append("<h2>Dernières erreurs</h2><table>")
    out.append("<tr><th>Quand</th><th>OS</th><th>Version</th><th>Message</th></tr>")
    for recv, os_, ver, data in errors:
        try:
            d = json.loads(data)
        except Exception:
            d = {}
        when = time.strftime("%m-%d %H:%M", time.localtime(recv / 1000))
        msg = d.get("message", "")
        out.append(
            f"<tr><td class=mono>{esc(when)}</td><td>{esc(os_)}</td>"
            f"<td class=mono>{esc(ver)}</td><td class='mono bad'>{esc(msg)}</td></tr>"
        )
    out.append("</table>")
    if not errors:
        out.append("<div class=l style='color:#4ade80'>Aucune erreur remontée 🎉</div>")

    return "".join(out)


# ── Serveur HTTP (port unique) ───────────────────────────────────────────────

class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def _send(self, code: int, body: bytes, ctype: str = "text/plain; charset=utf-8", extra=None):
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        for k, v in (extra or {}).items():
            self.send_header(k, v)
        self.end_headers()
        if body:
            self.wfile.write(body)

    def _text(self, code: int, msg: str):
        self._send(code, msg.encode("utf-8"))

    def _dash_auth_ok(self) -> bool:
        expected = "Basic " + base64.b64encode(f"{DASH_USER}:{DASH_PASS}".encode()).decode()
        return self.headers.get("Authorization", "") == expected

    def _rl(self) -> bool:
        """Applique le rate-limit. Renvoie False (et répond 429) si dépassé."""
        ip = self.client_address[0] if self.client_address else "?"
        if rate_ok(ip):
            return True
        self._text(429, "trop de requêtes")
        return False

    def do_GET(self):  # noqa: N802
        if not self._rl():
            return
        if self.path == "/health":
            self._text(200, "ok")
            return
        if self.path in ("/", "/index.html", "/dashboard"):
            if not DASH_PASS:
                self._text(503, "Dashboard désactivé : définis GLUCOSE_DASH_PASS.")
                return
            if not self._dash_auth_ok():
                self._send(401, b"", extra={"WWW-Authenticate": 'Basic realm="Glucose"'})
                return
            self._send(200, render_dashboard().encode("utf-8"), "text/html; charset=utf-8")
            return
        self._text(404, "")

    def do_POST(self):  # noqa: N802
        if not self._rl():
            return
        if self.path != "/ingest":
            self._text(404, "")
            return
        if self.headers.get("X-Glucose-Key", "") != INGEST_KEY:
            self._text(401, "clé invalide")
            return
        try:
            length = int(self.headers.get("Content-Length", "0"))
        except ValueError:
            length = 0
        if length <= 0 or length > MAX_BODY:
            self._text(413, "corps invalide")
            return
        raw = self.rfile.read(length)
        try:
            envelope = json.loads(raw.decode("utf-8"))
        except Exception:
            self._text(400, "JSON invalide")
            return
        try:
            n = store_batch(envelope)
        except Exception as e:  # noqa: BLE001
            self._text(500, f"erreur stockage: {e}")
            return
        self._text(200, f"ok {n}")

    def log_message(self, *args):  # silence les logs par requête
        pass


# ── Démarrage ────────────────────────────────────────────────────────────────

def main() -> None:
    init_db()
    server = ThreadingHTTPServer((BIND, PORT), Handler)
    print(f"[glucose-telemetry] écoute     → {BIND}:{PORT}")
    print(f"[glucose-telemetry]   ingest   → POST /ingest (clé)")
    print(f"[glucose-telemetry]   dashboard→ GET  /       "
          f"({'protégé' if DASH_PASS else 'DÉSACTIVÉ : pas de mot de passe'})")
    print(f"[glucose-telemetry]   santé    → GET  /health")
    print(f"[glucose-telemetry] base       → {DB_PATH}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[glucose-telemetry] arrêt.")


if __name__ == "__main__":
    main()
