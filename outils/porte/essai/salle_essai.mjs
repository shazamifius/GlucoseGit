// **L'essai d'une salle** (fiche 116) contre un relais : un écrivain, un lecteur, un retardataire,
// un faux écrivain. node outils/porte/essai/salle_essai.mjs [racine du relais]
import { randomBytes, createHash } from "node:crypto";
const R = (process.argv[2] ?? "http://127.0.0.1:8787").replace(/^http/, "ws");
const id = randomBytes(16).toString("hex");
const ecrire = randomBytes(16).toString("base64url");
const adresse = `${R}/s/${id}/salle`;
const ouvrir = (secret) =>
  new Promise((ok, ko) => {
    const ws = new WebSocket(adresse, secret ? { headers: { Authorization: `Bearer ${secret}` } } : undefined);
    ws.recus = [];
    ws.onmessage = (e) => ws.recus.push(JSON.parse(e.data));
    ws.onopen = () => ok(ws);
    ws.onerror = () => ko(new Error("refusé"));
  });
const attendre = (ms) => new Promise((ok) => setTimeout(ok, ms));
const scelle = (texte) => Buffer.from(texte).toString("base64");
const verdicts = [];
const dire = (nom, vrai) => verdicts.push(`${vrai ? "OUI" : "NON"}  ${nom}`);

const alice = await ouvrir(ecrire);
const bob = await ouvrir(); // lecteur
alice.send(JSON.stringify({ t: "geste", l: "a1", o: scelle("la base") }));
alice.send(JSON.stringify({ t: "geste", l: "a2", o: scelle("un texte posé") }));
await attendre(300);
dire("Alice reçoit ses gestes numérotés 1 et 2", JSON.stringify(alice.recus.map((m) => m.n)) === "[1,2]");
dire("Bob, lecteur, les reçoit aussi", JSON.stringify(bob.recus.map((m) => m.n)) === "[1,2]");
bob.send(JSON.stringify({ t: "geste", l: "b1", o: scelle("Bob écrit") }));
await attendre(200);
dire("Bob ne peut pas écrire", bob.recus.at(-1)?.t === "refus");
alice.send(JSON.stringify({ t: "curseur", o: scelle("x=3") }));
await attendre(200);
dire("le curseur d'Alice passe à Bob, pas à elle", bob.recus.at(-1)?.t === "curseur" && alice.recus.at(-1)?.t !== "curseur");
const carol = await ouvrir();
carol.send(JSON.stringify({ t: "depuis", n: 0 }));
await attendre(300);
dire("Carol, arrivée en retard, rattrape 1 et 2 puis « à jour »", JSON.stringify(carol.recus.map((m) => m.n)) === "[1,2,2]" && carol.recus.at(-1).t === "a_jour");
dire("la salle rend les octets tels quels (le client les scelle)", Buffer.from(carol.recus[1].o, "base64").toString() === "un texte posé");
const dave = await ouvrir();
dave.send(JSON.stringify({ t: "depuis", n: 1 }));
await attendre(300);
dire("Dave, qui avait le 1, ne rattrape que le 2", JSON.stringify(dave.recus.map((m) => m.n)) === "[2,2]");
dave.close();
let faux = "accepté";
try {
  await ouvrir(randomBytes(16).toString("base64url"));
} catch {
  faux = "refusé";
}
dire("un faux secret d'écriture est refusé", faux === "refusé");
// Un client qui enverrait lui-même l'empreinte du vrai secret : le relais la recalcule, jamais ne la croit.
const empreinte = createHash("sha256").update(Buffer.from(ecrire, "base64url")).digest("hex");
const malin = await new Promise((ok, ko) => {
  const ws = new WebSocket(adresse, { headers: { "x-glucose-ecrire": empreinte } });
  ws.recus = [];
  ws.onmessage = (e) => ws.recus.push(JSON.parse(e.data));
  ws.onopen = () => ok(ws);
  ws.onerror = () => ko(new Error("refusé"));
});
malin.send(JSON.stringify({ t: "geste", l: "m1", o: scelle("je me fais passer pour Alice") }));
await attendre(300);
dire("une empreinte envoyée par le client ne fait pas un écrivain", malin.recus.at(-1)?.t === "refus");
for (const ws of [alice, bob, carol, malin]) ws.close();
console.log(verdicts.join("\n"));
process.exit(verdicts.every((v) => v.startsWith("OUI")) ? 0 : 1);
