// **Un faux Glucose** contre un relais (fiche 116) : il se relie, répond, et dit ce qui passe.
//   node outils/porte/essai/faux_glucose.mjs [racine du relais]
import { randomBytes, createHash } from "node:crypto";
const R = process.argv[2] ?? "http://127.0.0.1:8787";
const secret = randomBytes(32);
const id = createHash("sha256").update(secret).digest("hex").slice(0, 32);
const ws = new WebSocket(`${R.replace('http','ws')}/g/${id}/glucose`, { headers: { Authorization: `Bearer ${secret.toString("base64url")}` } });
const vus = [];
ws.onmessage = (e) => {
  if (e.data === "pong") { vus.push("pong"); return; }
  const r = JSON.parse(e.data);
  vus.push(`${r.methode} ${r.chemin} auth=${r.entetes.authorization ?? "-"} cookie=${r.entetes.cookie ?? "-"}`);
  const html = r.chemin === "/piege";
  ws.send(JSON.stringify({ n: r.n, statut: 200, entetes: { "content-type": html ? "text/html" : "application/json", "set-cookie": "x=1" }, corps: html ? "<h1>hameçon</h1>" : JSON.stringify({ recu: r.corps }) }));
};
await new Promise((ok, ko) => { ws.onopen = ok; ws.onerror = ko; });
ws.send("ping");
const a = await fetch(`${R}/g/${id}/mcp`, { method: "POST", headers: { authorization: "Bearer abc", cookie: "secret=1", "content-type": "application/json" }, body: '{"jsonrpc":"2.0"}' });
console.log("mcp", a.status, a.headers.get("content-type"), a.headers.get("set-cookie"), await a.text());
const b = await fetch(`${R}/g/${id}/piege`, { method: "POST", body: "x" });
console.log("piège html", b.status, await b.text());
const c = await fetch(`${R}/.well-known/oauth-authorization-server/g/${id}`);
console.log("découverte", c.status, await c.text());
await new Promise((ok) => setTimeout(ok, 300));
console.log("vus par Glucose :", vus);
// Un second Glucose prend la porte : le premier est fermé.
const ws2 = new WebSocket(`${R.replace('http','ws')}/g/${id}/glucose`, { headers: { Authorization: `Bearer ${secret.toString("base64url")}` } });
const ferme = new Promise((ok) => { ws.onclose = (e) => ok(e.code); });
await new Promise((ok) => { ws2.onopen = ok; });
console.log("le premier fermé, code", await ferme);
ws2.close(); 
await new Promise((ok) => setTimeout(ok, 300));
const d = await fetch(`${R}/g/${id}/mcp`, { method: "POST", body: "{}" });
console.log("après départ", d.status, await d.text());
process.exit(0);
