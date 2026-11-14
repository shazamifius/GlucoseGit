// **Le relais de la porte publique** (fiche 116) : un Worker et un Durable Object par Glucose,
// sur le compte Cloudflare de l'auteur, gratuits.
//
// Une IA publique (Claude.ai, ChatGPT) ne parle qu'à une adresse `https://` ; un ordinateur
// derrière sa box n'en a pas. Glucose se relie donc **lui-même** au relais, par un WebSocket
// sortant, et le relais lui passe chaque demande qui arrive à son adresse. **Le relais ne lit
// rien, ne garde rien** : ni les clés d'accès (Glucose les donne et les vérifie), ni le contenu.
// Une connexion qui attend ne coûte rien (la mise en sommeil des WebSockets) ; le ping de Glucose
// reçoit son pong sans réveiller l'objet.
//
// Déployé depuis ce dossier : `npx wrangler deploy` (la télémétrie vit à côté, même compte).
// Les épreuves : `node --test outils/porte/test`.

import { DurableObject } from "cloudflare:workers";
import {
  aiguiller,
  empreinteDuSecret,
  entetesPourGlucose,
  erreur,
  identifiantDuSecret,
  reponseDeGlucose,
} from "./aiguillage.js";
export { Salle } from "./salle.js";
import { PAGE, SCRIPT } from "./page.js";

/** Le plus gros corps qu'une demande porte jusqu'à Glucose. */
const CORPS_MAX = 8 << 20;
/** Le plus long qu'une demande attend Glucose — sous la minute de patience des clients. */
const ATTENTE_MS = 55_000;

const SECURITE_DE_LA_PAGE = {
  "content-type": "text/html; charset=utf-8",
  "content-security-policy":
    "default-src 'none'; script-src 'self'; style-src 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'",
  "x-content-type-options": "nosniff",
  "referrer-policy": "no-referrer",
  "cache-control": "no-store",
};

export default {
  async fetch(requete, env) {
    const url = new URL(requete.url);
    const ou = aiguiller(url.pathname);
    if (!ou) {
      return url.pathname === "/"
        ? new Response("Le relais de Glucose.\n", { headers: { "content-type": "text/plain; charset=utf-8" } })
        : erreur(404, "introuvable", "Rien à cette adresse.");
    }
    if (ou.sorte === "script") {
      return new Response(SCRIPT, {
        headers: { "content-type": "text/javascript; charset=utf-8", "x-content-type-options": "nosniff" },
      });
    }
    if (ou.sorte === "salle") return entrerDansLaSalle(requete, env, ou.id);
    if (ou.sorte === "page") {
      if (requete.method !== "GET") return erreur(405, "methode", "La page d'accord se lit, elle ne reçoit rien.");
      return new Response(PAGE, { headers: SECURITE_DE_LA_PAGE });
    }
    if (ou.sorte === "glucose") {
      if (requete.headers.get("upgrade")?.toLowerCase() !== "websocket") {
        return erreur(426, "websocket", "Cette adresse est celle d'un Glucose, par WebSocket.");
      }
      const secret = (requete.headers.get("authorization") ?? "").replace(/^Bearer\s+/i, "");
      if ((await identifiantDuSecret(secret)) !== ou.id) {
        return erreur(403, "secret", "Ce secret n'ouvre pas cette porte.");
      }
    }
    const porte = env.PORTES.get(env.PORTES.idFromName(ou.id));
    if (ou.sorte === "glucose") return porte.fetch(requete);
    const longueur = Number(requete.headers.get("content-length") ?? 0);
    if (longueur > CORPS_MAX) return erreur(413, "trop_lourd", "Cette demande est trop lourde pour le relais.");
    const corps = ["GET", "HEAD"].includes(requete.method) ? "" : await requete.text();
    if (corps.length > CORPS_MAX) return erreur(413, "trop_lourd", "Cette demande est trop lourde pour le relais.");
    return porte.passer({
      methode: requete.method,
      chemin: ou.chemin,
      requete: url.search,
      entetes: entetesPourGlucose(requete.headers),
      corps,
    });
  },
};

/**
 * **Entrer dans une salle** (fiche 116) : par WebSocket. Le secret d'écriture, s'il est présenté,
 * devient son empreinte — calculée ici, jamais crue du client.
 */
async function entrerDansLaSalle(requete, env, id) {
  if (requete.headers.get("upgrade")?.toLowerCase() !== "websocket") {
    return erreur(426, "websocket", "Une salle s'ouvre par WebSocket.");
  }
  const entetes = new Headers(requete.headers);
  entetes.delete("x-glucose-ecrire");
  const secret = (requete.headers.get("authorization") ?? "").replace(/^Bearer\s+/i, "");
  if (secret) {
    const empreinte = await empreinteDuSecret(secret);
    if (!empreinte) return erreur(403, "secret", "Ce secret n'est pas un secret d'écriture.");
    entetes.set("x-glucose-ecrire", empreinte);
  }
  const salle = env.SALLES.get(env.SALLES.idFromName(id));
  return salle.fetch(new Request(requete, { headers: entetes }));
}

/** **La porte d'un Glucose** : sa connexion, et les demandes qui l'attendent. */
export class Porte extends DurableObject {
  constructor(ctx, env) {
    super(ctx, env);
    this.attente = new Map();
    this.suivant = 0;
    // Le ping de Glucose reçoit son pong sans réveiller l'objet : une porte ouverte toute la
    // journée ne coûte rien.
    ctx.setWebSocketAutoResponse(new WebSocketRequestResponsePair("ping", "pong"));
  }

  /** La connexion de Glucose — une seule : la plus récente prend la porte. */
  async fetch(_requete) {
    for (const ancienne of this.ctx.getWebSockets()) {
      ancienne.close(4000, "un autre Glucose a pris cette porte");
    }
    const [client, serveur] = Object.values(new WebSocketPair());
    this.ctx.acceptWebSocket(serveur);
    return new Response(null, { status: 101, webSocket: client });
  }

  /** **Passe une demande à Glucose**, et rend sa réponse — ou ce qui l'en empêche. */
  async passer(demande) {
    const glucose = this.ctx.getWebSockets()[0];
    if (!glucose) {
      return erreur(503, "glucose_ferme", "Le Glucose de cette adresse n'est pas ouvert, ou sa porte est fermée.");
    }
    const n = ++this.suivant;
    const reponse = new Promise((resoudre) => {
      this.attente.set(n, resoudre);
      setTimeout(() => resoudre(null), ATTENTE_MS);
    });
    try {
      glucose.send(JSON.stringify({ n, ...demande }));
    } catch {
      this.attente.delete(n);
      return erreur(503, "glucose_ferme", "Le Glucose de cette adresse vient de se fermer.");
    }
    const r = await reponse;
    this.attente.delete(n);
    if (r?.ferme) return erreur(503, "glucose_ferme", "Le Glucose de cette adresse vient de se fermer.");
    return r ? reponseDeGlucose(r) : erreur(504, "glucose_muet", "Ce Glucose n'a pas répondu à temps.");
  }

  webSocketMessage(_ws, message) {
    let r;
    try {
      r = JSON.parse(typeof message === "string" ? message : new TextDecoder().decode(message));
    } catch {
      return;
    }
    this.attente.get(r?.n)?.(r);
  }

  webSocketClose(ws, code) {
    // Les demandes en route ne recevront plus rien : elles le disent tout de suite.
    for (const [, resoudre] of this.attente) resoudre({ ferme: true });
    this.attente.clear();
    try {
      ws.close(code === 1005 ? 1000 : code, "au revoir");
    } catch {
      // Déjà fermée.
    }
  }
}
