// **Un faux claude.ai** contre un vrai Glucose, par le relais (fiche 116) : le chemin entier d'un
// connecteur. node outils/porte/essai/faux_claude.mjs <adresse MCP> <fichier où écrire le code affiché>
import { randomBytes, createHash } from "node:crypto";
import { writeFileSync } from "node:fs";

const [mcp, fichierDuCode] = process.argv.slice(2);
const CLIENT = "https://claude.ai/oauth/mcp-oauth-client-metadata";
const RETOUR = "https://claude.ai/api/mcp/auth_callback";
const dire = (...a) => console.log(new Date().toISOString().slice(11, 19), ...a);

// 1. Sans clé : 401, et où lire la découverte.
let r = await fetch(mcp, {
  method: "POST",
  headers: { "content-type": "application/json", accept: "application/json, text/event-stream" },
  body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "tools/list" }),
});
const defi401 = r.headers.get("www-authenticate");
dire("1. sans clé", r.status, defi401);
const metadonnees = /resource_metadata="([^"]+)"/.exec(defi401 ?? "")?.[1];
const ressource = await (await fetch(metadonnees)).json();
dire("2. ressource", JSON.stringify(ressource));
const issuer = new URL(ressource.authorization_servers[0]);
const guichet = await (await fetch(`${issuer.origin}/.well-known/oauth-authorization-server${issuer.pathname}`)).json();
dire("3. guichet", guichet.client_id_metadata_document_supported, guichet.token_endpoint_auth_methods_supported);

// 2. L'autorisation, comme la page la fait.
const verificateur = randomBytes(32).toString("base64url");
const defi = createHash("sha256").update(verificateur).digest("base64url");
const page = new URL(guichet.authorization_endpoint);
const params = {
  response_type: "code",
  client_id: CLIENT,
  redirect_uri: RETOUR,
  code_challenge: defi,
  code_challenge_method: "S256",
  state: "etat-essai",
  resource: ressource.resource,
};
const html = await fetch(`${page}?${new URLSearchParams(params)}`);
dire("4. la page", html.status, html.headers.get("content-type"));
r = await fetch(`${page}/demande`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(params) });
const demande = await r.json();
dire("5. demande", r.status, JSON.stringify(demande));
writeFileSync(fichierDuCode, demande.code ?? "aucun");
let redirection;
for (let i = 0; i < 240 && !redirection; i++) {
  await new Promise((ok) => setTimeout(ok, 500));
  const e = await (await fetch(`${page}/attente`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ demande: demande.demande }) })).json();
  if (e.etat !== "attente") {
    redirection = e.redirection;
    dire("6. réponse", e.etat, redirection);
  }
}
const retour = new URL(redirection);
const code = retour.searchParams.get("code");
r = await fetch(guichet.token_endpoint, {
  method: "POST",
  headers: { "content-type": "application/x-www-form-urlencoded" },
  body: new URLSearchParams({ grant_type: "authorization_code", code, redirect_uri: RETOUR, client_id: CLIENT, code_verifier: verificateur, resource: ressource.resource }),
});
const jeton = await r.json();
dire("7. clé", r.status, jeton.token_type, jeton.access_token ? "reçue" : JSON.stringify(jeton));

// 3. MCP, comme un client moderne : chaque requête dit ses en-têtes.
async function appeler(id, method, params = {}) {
  const corps = { jsonrpc: "2.0", id, method, params: { ...params, _meta: { "io.modelcontextprotocol/protocolVersion": "2026-07-28", "io.modelcontextprotocol/clientInfo": { name: "faux-claude", version: "1" } } } };
  const entetes = { authorization: `Bearer ${jeton.access_token}`, "content-type": "application/json", accept: "application/json, text/event-stream", "mcp-protocol-version": "2026-07-28", "mcp-method": method };
  if (params.name) entetes["mcp-name"] = params.name;
  const rep = await fetch(mcp, { method: "POST", headers: entetes, body: JSON.stringify(corps) });
  return [rep.status, await rep.json().catch(() => null)];
}
let [s, j] = await appeler(10, "tools/list");
dire("8. outils", s, j?.result?.tools?.map((t) => t.name).join(", "));
[s, j] = await appeler(11, "tools/call", { name: "glucose_poser_texte", arguments: { id: "bonjour-claude", texte: "# Bonjour\n\nJe suis Claude, passé par la porte publique de ce Glucose." } });
dire("9. poser un texte", s, JSON.stringify(j?.result ?? j?.error).slice(0, 200));
[s, j] = await appeler(12, "tools/call", { name: "glucose_poser_page", arguments: { adresse: "https://exemple.org", capture: "/etc/hostname" } });
dire("10. lire un fichier de la machine", s, JSON.stringify(j?.result ?? j?.error).slice(0, 200));
[s, j] = await appeler(13, "tools/call", { name: "glucose_domaines", arguments: {} });
dire("11. les autres fichiers", s, JSON.stringify(j?.error ?? j?.result).slice(0, 200));
[s, j] = await appeler(14, "tools/call", { name: "glucose_lire", arguments: {} });
dire("12. lire", s, JSON.stringify(j?.result).slice(0, 300));
[s, j] = await appeler(15, "tools/call", { name: "glucose_histoire", arguments: { nombre: 3 } });
dire("13. l'histoire", s, JSON.stringify(j?.result).slice(0, 400));
