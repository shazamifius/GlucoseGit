// **Ce qu'une salle accepte d'un client** (fiche 116) — pur, éprouvé sans Cloudflare.

/** Le plus gros geste scellé, en signes base64 : sous les 2 Mo d'une valeur de la salle. */
export const GESTE_MAX = 1_500_000;
/** Ce qu'un rattrapage rend d'un coup ; le client redemande depuis le dernier reçu. */
export const RATTRAPAGE_MAX = 2000;
const BASE64 = /^[A-Za-z0-9+/_-]*={0,2}$/;
const LOCAL = /^[A-Za-z0-9_-]{1,64}$/;

/** **Lit un message de client**, ou rend pourquoi il ne passe pas. Pur. */
export function lireUnMessage(texte) {
  let m;
  try {
    m = JSON.parse(texte);
  } catch {
    return { refus: "pas du JSON" };
  }
  if (m?.t === "depuis" && Number.isSafeInteger(m.n) && m.n >= 0) return { t: "depuis", n: m.n };
  const scelle = typeof m?.o === "string" && m.o.length <= GESTE_MAX && BASE64.test(m.o);
  if (m?.t === "geste" && scelle && LOCAL.test(m.l ?? "")) return { t: "geste", l: m.l, o: m.o };
  if (m?.t === "curseur" && scelle && m.o.length <= 4096) return { t: "curseur", o: m.o };
  return { refus: "message inconnu ou trop gros" };
}

