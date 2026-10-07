// **Le serveur de la boîte noire qui voyage** (fiches 49 et 54) : un Worker de Cloudflare et sa
// base D1, dans le compte de l'utilisateur, gratuits.
//
// Deux portes, et aucune pour lire : on lit la base depuis son compte (`wrangler d1 execute`),
// jamais par Internet.
//
//   POST   /v1/sessions                 {installation, session, lignes} -> 201, ou 200 si déjà là
//   DELETE /v1/installations/<id>       tout ce que cette installation a envoyé            -> 200
//
// **Aucune adresse IP n'est gardée** : le Worker ne lit ni `CF-Connecting-IP` ni aucun en-tête
// qui la porte, et la base n'a pas de colonne pour elle. Le jour de réception est gardé, pas
// l'heure.

import { identifiant, validerSession } from "./valider.js";

const ENTETES = { "content-type": "text/plain; charset=utf-8" };

function reponse(statut, texte) {
  return new Response(texte, { status: statut, headers: ENTETES });
}

async function deposer(requete, env) {
  let corps;
  try {
    corps = await requete.json();
  } catch {
    return reponse(400, "pas du JSON");
  }
  const { installation, session, lignes } = corps ?? {};
  if (!identifiant(installation) || !identifiant(session)) {
    return reponse(400, "identifiant");
  }
  const v = validerSession(lignes);
  if (!v.ok) return reponse(422, v.raison);
  const jour = new Date().toISOString().slice(0, 10);
  const r = await env.BASE.prepare(
    "INSERT OR IGNORE INTO sessions (installation, session, recue, version, systeme, architecture, lignes) VALUES (?, ?, ?, ?, ?, ?, ?)",
  )
    .bind(installation, session, jour, v.debut.version, v.debut.systeme, v.debut.architecture, lignes)
    .run();
  return r.meta.changes > 0 ? reponse(201, "recue") : reponse(200, "deja recue");
}

async function effacer(id, env) {
  if (!identifiant(id)) return reponse(400, "identifiant");
  await env.BASE.prepare("DELETE FROM sessions WHERE installation = ?").bind(id).run();
  return reponse(200, "effacee");
}

export default {
  async fetch(requete, env) {
    const url = new URL(requete.url);
    if (requete.method === "POST" && url.pathname === "/v1/sessions") {
      return deposer(requete, env);
    }
    const m = url.pathname.match(/^\/v1\/installations\/([^/]+)$/);
    if (requete.method === "DELETE" && m) {
      return effacer(m[1], env);
    }
    return reponse(404, "rien ici");
  },
};
