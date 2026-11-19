// **Claude Code dans son Glucose, par sa porte publique** (fiches 116 et 122) : le chemin de
// faux_claude.mjs, sous l'identité vérifiée de Claude Code (son document d'identité sur claude.ai), pour
// qu'une session lise et écrive le document qu'il a ouvert — ses réponses, nos questions (sa parole du
// 19/11 : « crée plutôt sur Glucose »). La clé ne vit jamais dans le dépôt : ~/.local/share/glucose-cles/.
//   node outils/porte/claude_code.mjs accord <adresse MCP> <fichier du code>  → son clic dans Glucose
//   node outils/porte/claude_code.mjs outils                                  → les outils et leurs schémas
//   node outils/porte/claude_code.mjs appeler <outil> '<arguments JSON>'      → un appel, sa réponse
import { randomBytes, createHash } from "node:crypto";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { homedir } from "node:os";

const DOSSIER = `${homedir()}/.local/share/glucose-cles`;
const CLE = `${DOSSIER}/porte-claude-code.json`;
const CLIENT = "https://claude.ai/oauth/claude-code-client-metadata";
const RETOUR = "http://localhost/callback";
const [commande, ...args] = process.argv.slice(2);

async function accord(mcp, fichierDuCode) {
  let r = await fetch(mcp, {
    method: "POST",
    headers: { "content-type": "application/json", accept: "application/json, text/event-stream" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "tools/list" }),
  });
  const metadonnees = /resource_metadata="([^"]+)"/.exec(r.headers.get("www-authenticate") ?? "")?.[1];
  const ressource = await (await fetch(metadonnees)).json();
  const issuer = new URL(ressource.authorization_servers[0]);
  const guichet = await (await fetch(`${issuer.origin}/.well-known/oauth-authorization-server${issuer.pathname}`)).json();
  const verificateur = randomBytes(32).toString("base64url");
  const params = {
    response_type: "code",
    client_id: CLIENT,
    redirect_uri: RETOUR,
    code_challenge: createHash("sha256").update(verificateur).digest("base64url"),
    code_challenge_method: "S256",
    state: randomBytes(8).toString("hex"),
    resource: ressource.resource,
  };
  const page = guichet.authorization_endpoint;
  r = await fetch(`${page}/demande`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(params) });
  const demande = await r.json();
  if (!demande.code) throw new Error(`demande refusée : ${r.status} ${JSON.stringify(demande)}`);
  // Le code que Glucose montre dans sa question : à lui dire, pour qu'il le compare avant de cliquer.
  writeFileSync(fichierDuCode, String(demande.code));
  console.log("code affiché :", demande.code);
  let redirection;
  // Sa demande vit dix minutes dans Glucose (oauth::DUREE) : on attend un peu moins.
  for (let i = 0; i < 1140 && !redirection; i++) {
    await new Promise((ok) => setTimeout(ok, 500));
    const e = await (await fetch(`${page}/attente`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ demande: demande.demande }) })).json();
    if (e.etat !== "attente") {
      redirection = e.redirection;
      console.log("réponse :", e.etat);
    }
  }
  if (!redirection) throw new Error("pas de réponse en 9 min 30");
  const code = new URL(redirection).searchParams.get("code");
  if (!code) throw new Error(`refusé : ${redirection}`);
  r = await fetch(guichet.token_endpoint, {
    method: "POST",
    headers: { "content-type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({ grant_type: "authorization_code", code, redirect_uri: RETOUR, client_id: CLIENT, code_verifier: verificateur, resource: ressource.resource }),
  });
  const jeton = await r.json();
  if (!jeton.access_token) throw new Error(`échange refusé : ${r.status} ${JSON.stringify(jeton)}`);
  mkdirSync(DOSSIER, { recursive: true, mode: 0o700 });
  writeFileSync(CLE, JSON.stringify({ mcp, cle: jeton.access_token }), { mode: 0o600 });
  console.log("clé reçue, gardée dans", CLE);
}

let numero = 1;
async function appeler(method, params = {}) {
  const { mcp, cle } = JSON.parse(readFileSync(CLE, "utf8"));
  const corps = { jsonrpc: "2.0", id: numero++, method, params: { ...params, _meta: { "io.modelcontextprotocol/protocolVersion": "2026-07-28", "io.modelcontextprotocol/clientInfo": { name: "claude-code", version: "1" } } } };
  const entetes = { authorization: `Bearer ${cle}`, "content-type": "application/json", accept: "application/json, text/event-stream", "mcp-protocol-version": "2026-07-28", "mcp-method": method };
  if (params.name) entetes["mcp-name"] = params.name;
  const rep = await fetch(mcp, { method: "POST", headers: entetes, body: JSON.stringify(corps) });
  const texte = await rep.text();
  try {
    return [rep.status, JSON.parse(texte)];
  } catch {
    return [rep.status, { brut: texte }];
  }
}

if (commande === "accord") {
  await accord(args[0], args[1]);
} else if (commande === "outils") {
  const [s, j] = await appeler("tools/list");
  if (!j.result) console.log(s, JSON.stringify(j));
  else for (const t of j.result.tools) console.log(`## ${t.name}\n${t.description}\n${JSON.stringify(t.inputSchema)}\n`);
} else if (commande === "appeler") {
  const [s, j] = await appeler("tools/call", { name: args[0], arguments: JSON.parse(args[1] ?? "{}") });
  if (j.result) {
    for (const c of j.result.content ?? []) console.log(c.type === "text" ? c.text : JSON.stringify(c));
    if (j.result.isError) process.exitCode = 1;
  } else {
    console.log(s, JSON.stringify(j));
    process.exitCode = 1;
  }
} else {
  console.log("node porte.mjs accord <mcp> <fichier> | outils | appeler <outil> '<json>'");
}
