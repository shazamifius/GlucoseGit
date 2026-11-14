// **La salle d'un canevas partagé** (fiche 116 § 8, le greffier aveugle de la fiche 113) : un
// Durable Object par canevas. Elle **numérote** les gestes, les **garde** et les **passe** à chacun —
// sans jamais pouvoir les lire : chaque geste arrive scellé (ChaCha20-Poly1305) par la clé que porte
// le code d'invitation, et que la salle ne voit pas.
//
// * **Écrire se prouve** : le code « écrire » porte un secret ; la salle n'en garde que l'empreinte
//   (le premier écrivain la fixe, à la création). Le code « lire » n'a que la clé du chiffre.
// * **Un retardataire rattrape** : `depuis n` rend chaque geste gardé après n, dans l'ordre, puis
//   `a_jour`. Le premier geste d'une salle est la base (le document au moment du partage), en
//   morceaux si elle est grosse : une valeur de la salle tient 2 Mo.
// * **Les curseurs passent**, sans être gardés ni numérotés.
//
// Messages (JSON, texte) :
//   → {"t":"depuis","n":k}                 ← {"t":"geste","n":i,"l":…,"o":"<scellé base64>"}…, {"t":"a_jour","n":dernier}
//   → {"t":"geste","l":"<id local>","o":…}  ← à tous : {"t":"geste","n":dernier+1,"l":…,"o":…}
//   → {"t":"curseur","o":…}                 ← aux autres : {"t":"curseur","o":…}
//                                           ← {"t":"refus","pourquoi":…} (lecteur qui écrit, message trop gros)

import { DurableObject } from "cloudflare:workers";
import { lireUnMessage, RATTRAPAGE_MAX } from "./messages.js";

export class Salle extends DurableObject {
  constructor(ctx, env) {
    super(ctx, env);
    this.sql = ctx.storage.sql;
    this.sql.exec("CREATE TABLE IF NOT EXISTS gestes (n INTEGER PRIMARY KEY, l TEXT NOT NULL, o TEXT NOT NULL)");
    this.sql.exec("CREATE TABLE IF NOT EXISTS reglages (cle TEXT PRIMARY KEY, valeur TEXT NOT NULL)");
  }

  dernier() {
    return this.sql.exec("SELECT coalesce(max(n), 0) AS n FROM gestes").one().n;
  }

  /** **Un participant entre** : écrivain s'il prouve le secret d'écriture (`empreinte`), lecteur sinon. */
  async fetch(requete) {
    const empreinte = requete.headers.get("x-glucose-ecrire");
    let ecrivain = false;
    if (empreinte) {
      const connue = this.sql.exec("SELECT valeur FROM reglages WHERE cle = 'ecrire'").toArray()[0]?.valeur;
      if (!connue) {
        this.sql.exec("INSERT INTO reglages (cle, valeur) VALUES ('ecrire', ?)", empreinte);
        ecrivain = true;
      } else if (connue === empreinte) {
        ecrivain = true;
      } else {
        return new Response("ce secret n'écrit pas dans cette salle", { status: 403 });
      }
    }
    const [client, serveur] = Object.values(new WebSocketPair());
    this.ctx.acceptWebSocket(serveur);
    serveur.serializeAttachment({ ecrivain });
    return new Response(null, { status: 101, webSocket: client });
  }

  webSocketMessage(ws, message) {
    const m = lireUnMessage(typeof message === "string" ? message : new TextDecoder().decode(message));
    const dire = (o) => ws.send(JSON.stringify(o));
    if (m.refus) return dire({ t: "refus", pourquoi: m.refus });
    if (m.t === "depuis") {
      const lignes = this.sql
        .exec("SELECT n, l, o FROM gestes WHERE n > ? ORDER BY n LIMIT ?", m.n, RATTRAPAGE_MAX)
        .toArray();
      for (const g of lignes) dire({ t: "geste", n: g.n, l: g.l, o: g.o });
      return dire({ t: "a_jour", n: this.dernier(), complet: lignes.length < RATTRAPAGE_MAX });
    }
    if (m.t === "curseur") {
      const passe = JSON.stringify({ t: "curseur", o: m.o });
      for (const autre of this.ctx.getWebSockets()) if (autre !== ws) autre.send(passe);
      return;
    }
    if (!ws.deserializeAttachment()?.ecrivain) return dire({ t: "refus", pourquoi: "ce code ne fait que lire" });
    const n = this.dernier() + 1;
    this.sql.exec("INSERT INTO gestes (n, l, o) VALUES (?, ?, ?)", n, m.l, m.o);
    const numerote = JSON.stringify({ t: "geste", n, l: m.l, o: m.o });
    for (const chacun of this.ctx.getWebSockets()) chacun.send(numerote);
  }

  webSocketClose(ws, code) {
    try {
      ws.close(code === 1005 ? 1000 : code, "au revoir");
    } catch {
      // Déjà fermée.
    }
  }
}
