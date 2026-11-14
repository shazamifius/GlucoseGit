// **L'aiguillage du relais** (fiche 116) — pur : une adresse entre, une décision sort.
//
// Chaque Glucose a son adresse, `/g/<identifiant>`, l'identifiant étant les 32 premiers chiffres
// hexadécimaux du SHA-256 de son secret. Le relais ne connaît que trois choses : la connexion de
// Glucose (`/g/<id>/glucose`), la page qui dit « va dans Glucose » (`/g/<id>/authorize`), et la
// porte de chaque Glucose, où tout le reste passe sans être lu.
//
// Les documents de découverte d'OAuth vivent à la racine du domaine, le chemin de leur sujet à la
// suite (RFC 8414 § 3.1, RFC 9728 § 3.1) : ils vont, eux aussi, au Glucose qu'ils nomment.

const IDENTIFIANT = /^[0-9a-f]{32}$/;

/** Le décodage base64, des deux alphabets, sans `=`. `null` si le texte n'en est pas. */
function octetsDuBase64(texte) {
  if (!/^[A-Za-z0-9_\-+/]+=*$/.test(texte)) return null;
  const normal = texte.replace(/-/g, "+").replace(/_/g, "/").replace(/=+$/, "");
  try {
    return Uint8Array.from(atob(normal + "=".repeat((4 - (normal.length % 4)) % 4)), (c) =>
      c.charCodeAt(0),
    );
  } catch {
    return null;
  }
}

/** **L'empreinte d'un secret d'écriture** (une salle, fiche 116) : SHA-256 en hexadécimal, ou `null`. */
export async function empreinteDuSecret(secret) {
  const octets = octetsDuBase64(secret ?? "");
  if (!octets || octets.length < 16 || octets.length > 64) return null;
  const e = new Uint8Array(await crypto.subtle.digest("SHA-256", octets));
  return Array.from(e, (o) => o.toString(16).padStart(2, "0")).join("");
}

/** **L'identifiant d'un secret** : ce que Glucose prouve en le présentant. */
export async function identifiantDuSecret(secret) {
  const octets = octetsDuBase64(secret ?? "");
  if (!octets || octets.length !== 32) return null;
  const empreinte = new Uint8Array(await crypto.subtle.digest("SHA-256", octets));
  return Array.from(empreinte.slice(0, 16), (o) => o.toString(16).padStart(2, "0")).join("");
}

/**
 * **Où va cette adresse** :
 * - `{ sorte: "glucose", id }` — la connexion d'un Glucose ;
 * - `{ sorte: "page", id }` — la page d'accord ;
 * - `{ sorte: "porte", id, chemin }` — ce qui passe à Glucose, `chemin` relatif à son adresse ;
 * - `{ sorte: "script" }` — le script de la page ;
 * - `{ sorte: "salle", id }` — la salle d'un canevas partagé ;
 * - `null` — rien ici.
 */
export function aiguiller(pathname) {
  const decoupe = pathname.split("/").filter((m) => m !== "");
  if (decoupe.length === 1 && decoupe[0] === "porte.js") return { sorte: "script" };
  const [tete, ...reste] = decoupe;
  if (tete === ".well-known") {
    const [document, g, id] = reste;
    const sujets = {
      "oauth-protected-resource": "/.well-known/oauth-protected-resource",
      "oauth-authorization-server": "/.well-known/oauth-authorization-server",
      "openid-configuration": "/.well-known/oauth-authorization-server",
    };
    if (g !== "g" || !IDENTIFIANT.test(id ?? "") || !sujets[document]) return null;
    return { sorte: "porte", id, chemin: sujets[document] };
  }
  // Une salle de canevas partagé (fiche 116) : son identifiant vient du code d'invitation.
  if (tete === "s" && IDENTIFIANT.test(reste[0] ?? "") && reste.length === 2 && reste[1] === "salle") {
    return { sorte: "salle", id: reste[0] };
  }
  if (tete !== "g" || !IDENTIFIANT.test(reste[0] ?? "")) return null;
  const [id, ...suite] = reste;
  const chemin = "/" + suite.join("/");
  if (chemin === "/glucose") return { sorte: "glucose", id };
  if (chemin === "/authorize") return { sorte: "page", id };
  return { sorte: "porte", id, chemin };
}

/** Les en-têtes d'une demande qui comptent pour Glucose ; aucun autre ne passe. */
const ENTETES_DEMANDE = [
  "authorization",
  "content-type",
  "accept",
  "origin",
  "mcp-protocol-version",
  "mcp-method",
  "mcp-name",
  "mcp-session-id",
  "last-event-id",
];

export function entetesPourGlucose(headers) {
  const choisis = {};
  for (const [nom, valeur] of headers) {
    const n = nom.toLowerCase();
    if (ENTETES_DEMANDE.includes(n) || n.startsWith("mcp-param-")) choisis[n] = valeur;
  }
  return choisis;
}

/** Les sortes de corps que Glucose peut rendre. Jamais de HTML : rien ne s'affiche en son nom. */
const TYPES_PERMIS = /^(application\/json|text\/event-stream)\s*(;|$)/i;
const ENTETES_REPONSE = ["content-type", "www-authenticate", "allow", "cache-control"];

/**
 * **La réponse de Glucose, telle que le relais la rend** : son statut, les seuls en-têtes
 * permis, un corps JSON ou rien. Une réponse qui sortirait de ce cadre devient un `502` : le
 * domaine du relais ne sert jamais une page que Glucose aurait écrite.
 */
export function reponseDeGlucose(r) {
  const statut = Number.isInteger(r?.statut) && r.statut >= 200 && r.statut <= 599 ? r.statut : 0;
  const type = r?.entetes?.["content-type"];
  const corps = typeof r?.corps === "string" && r.corps !== "" ? r.corps : null;
  if (!statut || (corps !== null && !TYPES_PERMIS.test(type ?? ""))) {
    return erreur(502, "glucose_illisible", "Ce Glucose a répondu quelque chose que le relais ne transmet pas.");
  }
  const headers = new Headers({ "cache-control": "no-store", "x-content-type-options": "nosniff" });
  for (const nom of ENTETES_REPONSE) {
    const v = r.entetes?.[nom];
    if (typeof v === "string") headers.set(nom, v);
  }
  return new Response(statut === 204 || statut === 202 ? null : corps, { status: statut, headers });
}

/** Une erreur du relais lui-même, à la forme d'OAuth (RFC 6749 § 5.2). */
export function erreur(statut, code, description) {
  return new Response(JSON.stringify({ error: code, error_description: description }), {
    status: statut,
    headers: { "content-type": "application/json", "cache-control": "no-store" },
  });
}
