// Les épreuves du relais : `node --test outils/porte/test` — sans dépendance.
import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash, randomBytes } from "node:crypto";
import { aiguiller, entetesPourGlucose, identifiantDuSecret, reponseDeGlucose } from "../src/aiguillage.js";

const ID = "0123456789abcdef0123456789abcdef";

test("chaque adresse va où elle doit, et rien d'autre ne passe", () => {
  assert.deepEqual(aiguiller(`/g/${ID}/glucose`), { sorte: "glucose", id: ID });
  assert.deepEqual(aiguiller(`/g/${ID}/authorize`), { sorte: "page", id: ID });
  assert.deepEqual(aiguiller(`/g/${ID}/mcp`), { sorte: "porte", id: ID, chemin: "/mcp" });
  assert.deepEqual(aiguiller(`/g/${ID}/authorize/attente`), {
    sorte: "porte",
    id: ID,
    chemin: "/authorize/attente",
  });
  assert.deepEqual(aiguiller(`/.well-known/oauth-protected-resource/g/${ID}/mcp`), {
    sorte: "porte",
    id: ID,
    chemin: "/.well-known/oauth-protected-resource",
  });
  assert.deepEqual(aiguiller(`/.well-known/oauth-authorization-server/g/${ID}`), {
    sorte: "porte",
    id: ID,
    chemin: "/.well-known/oauth-authorization-server",
  });
  assert.deepEqual(aiguiller(`/.well-known/openid-configuration/g/${ID}`)?.chemin, "/.well-known/oauth-authorization-server");
  assert.deepEqual(aiguiller("/porte.js"), { sorte: "script" });
  for (const refusee of [
    "/",
    "/g/court/mcp",
    `/g/${ID.toUpperCase()}/mcp`,
    `/x/${ID}/mcp`,
    `/.well-known/autre/g/${ID}`,
    `/.well-known/oauth-authorization-server/h/${ID}`,
  ]) {
    assert.equal(aiguiller(refusee), null, refusee);
  }
});

test("l'identifiant d'un secret est le début de son SHA-256, comme Glucose le calcule", async () => {
  const secret = randomBytes(32);
  const attendu = createHash("sha256").update(secret).digest("hex").slice(0, 32);
  assert.equal(await identifiantDuSecret(secret.toString("base64url")), attendu);
  assert.equal(await identifiantDuSecret(secret.toString("base64")), attendu);
  assert.equal(await identifiantDuSecret(randomBytes(16).toString("base64url")), null);
  assert.equal(await identifiantDuSecret("pas du base64 !"), null);
  assert.equal(await identifiantDuSecret(undefined), null);
});

test("seuls les en-têtes utiles passent à Glucose", () => {
  const h = new Headers({
    Authorization: "Bearer x",
    "MCP-Protocol-Version": "2026-07-28",
    "Mcp-Param-Region": "eu",
    Cookie: "secret=1",
    "CF-Connecting-IP": "203.0.113.9",
  });
  assert.deepEqual(entetesPourGlucose(h), {
    authorization: "Bearer x",
    "mcp-protocol-version": "2026-07-28",
    "mcp-param-region": "eu",
  });
});

test("le relais ne sert jamais une page que Glucose aurait écrite", async () => {
  const json = reponseDeGlucose({
    statut: 401,
    entetes: { "content-type": "application/json", "www-authenticate": 'Bearer resource_metadata="x"', "set-cookie": "a=b" },
    corps: "{}",
  });
  assert.equal(json.status, 401);
  assert.equal(json.headers.get("www-authenticate"), 'Bearer resource_metadata="x"');
  assert.equal(json.headers.get("set-cookie"), null);
  assert.equal(await json.text(), "{}");
  const html = reponseDeGlucose({ statut: 200, entetes: { "content-type": "text/html" }, corps: "<script>" });
  assert.equal(html.status, 502);
  const sansType = reponseDeGlucose({ statut: 200, entetes: {}, corps: "<h1>" });
  assert.equal(sansType.status, 502);
  const notification = reponseDeGlucose({ statut: 202, entetes: {}, corps: "" });
  assert.equal(notification.status, 202);
  assert.equal(reponseDeGlucose({ statut: 101 }).status, 502);
  assert.equal(reponseDeGlucose(null).status, 502);
});
