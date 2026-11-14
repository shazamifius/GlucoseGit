// Les épreuves de la salle, sans Cloudflare : `node --test outils/porte/test/salle.test.js`.
import { test } from "node:test";
import assert from "node:assert/strict";
import { randomBytes, createHash } from "node:crypto";
import { lireUnMessage, GESTE_MAX } from "../src/messages.js";
import { aiguiller, empreinteDuSecret } from "../src/aiguillage.js";

const ID = "0123456789abcdef0123456789abcdef";

test("une salle s'adresse par son identifiant, et rien d'autre", () => {
  assert.deepEqual(aiguiller(`/s/${ID}/salle`), { sorte: "salle", id: ID });
  for (const non of [`/s/${ID}`, `/s/${ID}/salle/x`, `/s/court/salle`, `/s/${ID}/autre`]) {
    assert.equal(aiguiller(non), null, non);
  }
});

test("l'empreinte d'un secret d'écriture est son SHA-256, et un secret trop court n'en a pas", async () => {
  const secret = randomBytes(16);
  assert.equal(await empreinteDuSecret(secret.toString("base64url")), createHash("sha256").update(secret).digest("hex"));
  assert.equal(await empreinteDuSecret(randomBytes(8).toString("base64url")), null);
  assert.equal(await empreinteDuSecret(""), null);
});

test("la salle lit les trois messages, et refuse le reste", () => {
  assert.deepEqual(lireUnMessage('{"t":"depuis","n":3}'), { t: "depuis", n: 3 });
  assert.deepEqual(lireUnMessage('{"t":"geste","l":"a1","o":"QUJD"}'), { t: "geste", l: "a1", o: "QUJD" });
  assert.deepEqual(lireUnMessage('{"t":"curseur","o":"QUJD"}'), { t: "curseur", o: "QUJD" });
  for (const non of [
    "pas du json",
    '{"t":"depuis","n":-1}',
    '{"t":"geste","o":"QUJD"}',
    '{"t":"geste","l":"a b","o":"QUJD"}',
    '{"t":"geste","l":"a","o":"<script>"}',
    `{"t":"geste","l":"a","o":"${"A".repeat(GESTE_MAX + 4)}"}`,
    `{"t":"curseur","o":"${"A".repeat(5000)}"}`,
    '{"t":"effacer"}',
  ]) {
    assert.ok(lireUnMessage(non).refus, non.slice(0, 40));
  }
});
