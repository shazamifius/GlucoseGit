// Les épreuves du serveur : `node --test outils/telemetrie/test` — sans dépendance.
import { test } from "node:test";
import assert from "node:assert/strict";
import { identifiant, POIDS_MAX, validerSession } from "../src/valider.js";

// Une session telle que la boîte noire l'écrit, ligne pour ligne.
const SESSION = [
  '{"type":"debut","epoque_ms":1791386404045,"version":"2.0.2-dev","systeme":"windows","architecture":"x86_64","demarrage_appareil_ms":1791362086291}',
  '{"type":"precedente","instant_ms":0,"fin":"propre","appareil_redemarre":false,"duree_ms":268681,"batterie_pct":100,"en_charge":true}',
  '{"type":"machine","instant_ms":0,"batterie_pct":null,"en_charge":null}',
  '{"type":"pire","instant_ms":817,"duree_us":27484,"geste":"zoomer"}',
  '{"type":"episode","instant_ms":817,"duree_ms":13,"geste":"deplacer la vue","images":1,"median_us":27484,"p99_us":27484,"pire_us":27484}',
  '{"type":"pave","instant_ms":900,"quoi":"statut_en_cours","valeur":5}',
  '{"type":"panique","instant_ms":901,"fichier":"crates\\\\glucose-desktop\\\\src\\\\app.rs","ligne":12,"principal":true}',
  '{"type":"plantage","instant_ms":0,"gel":false,"module":"nvoglv64.dll","version_du_module":"32.0.16.1074","code":3221225477,"decalage":4660}',
  '{"type":"fin","instant_ms":1200}',
].join("\n") + "\n";

test("une session de la boite noire entre, et dit sa version", () => {
  const v = validerSession(SESSION);
  assert.equal(v.ok, true, v.raison);
  assert.equal(v.debut.version, "2.0.2-dev");
});

test("un mot de l'utilisateur ne passe pas, ou qu'il se cache", () => {
  const cas = [
    SESSION.replace('"geste":"zoomer"', '"geste":"Mon Document Secret.glucose"'),
    SESSION.replace('"geste":"zoomer"', '"geste":"mon document secret"'),
    SESSION.replace('"quoi":"statut_en_cours"', '"quoi":"bonjour marie"'),
    SESSION.replace('"version":"2.0.2-dev"', '"version":"2.0.2-bonjour marie"'),
    SESSION.replace('"version":"2.0.2-dev"', '"version":"2.0.2-marie"'),
    SESSION.replace('"fin":"propre"', '"fin":"propre","note":"bonjour"'),
    SESSION.replace('"module":"nvoglv64.dll"', '"module":"C:\\\\Users\\\\marie\\\\x.dll"'),
    SESSION + '{"type":"texte","contenu":"bonjour"}\n',
    SESSION.replace('"images":1', '"images":"1"'),
    SESSION.replace('"valeur":5', '"valeur":-5'),
  ];
  for (const c of cas) assert.equal(validerSession(c).ok, false, c.slice(0, 80));
});

test("un champ attendu manquant, ou le debut absent, refuse la session", () => {
  assert.equal(validerSession(SESSION.replace(',"duree_us":27484', "")).ok, false);
  assert.equal(validerSession(SESSION.split("\n").slice(1).join("\n")).ok, false);
});

test("une session trop lourde est refusee", () => {
  const remplissage = '{"type":"fin","instant_ms":1}\n'.repeat(POIDS_MAX / 20);
  assert.equal(validerSession(SESSION + remplissage).raison, "trop lourde");
});

test("un identifiant est 32 chiffres hexadecimaux, rien d'autre", () => {
  assert.equal(identifiant("0123456789abcdef0123456789abcdef"), true);
  assert.equal(identifiant("0123456789ABCDEF0123456789abcdef"), false);
  assert.equal(identifiant("x".repeat(32)), false);
  assert.equal(identifiant(undefined), false);
});
