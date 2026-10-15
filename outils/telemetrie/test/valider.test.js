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
  '{"type":"poste","instant_ms":817,"geste":"deplacer la vue","nom":"textures","median_us":14200,"p99_us":21000}',
  '{"type":"pave","instant_ms":900,"quoi":"statut_en_cours","valeur":5}',
  '{"type":"panique","instant_ms":901,"fichier":"crates\\\\glucose-desktop\\\\src\\\\app.rs","ligne":12,"principal":true}',
  '{"type":"plantage","instant_ms":0,"gel":false,"module":"nvoglv64.dll","version_du_module":"32.0.16.1074","code":3221225477,"decalage":4660}',
  '{"type":"carte","instant_ms":40,"moteur":"vulkan","genre":"integree","fabricant":5045,"modele":1946288128,"plafond":8192}',
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
    SESSION.replace('"moteur":"vulkan"', '"moteur":"Mali-G52 de marie"'),
    SESSION.replace('"genre":"integree"', '"genre":"le portable de marie"'),
    SESSION.replace('"fabricant":5045', '"fabricant":"ARM"'),
    SESSION.replace('"plafond":8192', '"plafond":8192,"nom":"Mali-G52"'),
    SESSION.replace('"nom":"textures"', '"nom":"mon document secret"'),
    SESSION.replace('"nom":"textures"', '"nom":"Textures"'),
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

test("une session d'Android passe en arriere-plan, et sa fin s'y dit (BOITE-ANDROID)", () => {
  const android = SESSION
    .replace('"fin":"propre"', '"fin":"arriere_plan"')
    .replace(
      '{"type":"fin","instant_ms":1200}',
      [
        '{"type":"arriere_plan","instant_ms":1100}',
        '{"type":"premier_plan","instant_ms":1150}',
        '{"type":"arriere_plan","instant_ms":1190}',
      ].join(String.fromCharCode(10)),
    );
  const v = validerSession(android);
  assert.equal(v.ok, true, v.raison);
  const inconnue = SESSION.replace('"fin":"propre"', '"fin":"en_vacances"');
  assert.equal(validerSession(inconnue).ok, false);
});

test("la memoire entre, record ou alerte, et ce qui ne se lit pas est null (MEMOIRE-1, 2)", () => {
  const memoire = SESSION.replace(
    '{"type":"fin","instant_ms":1200}',
    [
      '{"type":"memoire","instant_ms":1000,"processus_mo":412,"disponible_mo":2210,"images_mo":180,"offerts_mo":96,"decodages_mo":40,"carte_mo":64}',
      '{"type":"alerte_memoire","instant_ms":1100,"arriere_plan":true}',
      '{"type":"memoire","instant_ms":1100,"processus_mo":null,"disponible_mo":null,"images_mo":0,"offerts_mo":0,"decodages_mo":0,"carte_mo":null}',
      '{"type":"fin","instant_ms":1200}',
    ].join(String.fromCharCode(10)),
  );
  const v = validerSession(memoire);
  assert.equal(v.ok, true, v.raison);
  const cas = [
    memoire.replace('"carte_mo":64', '"carte_mo":64,"nom":"Mali"'),
    memoire.replace('"images_mo":180', '"images_mo":-180'),
    memoire.replace('"images_mo":180', '"images_mo":null'),
    memoire.replace('"arriere_plan":true', '"arriere_plan":"oui"'),
    memoire.replace(',"offerts_mo":96', ""),
    memoire.replace(',"decodages_mo":40', ""),
  ];
  for (const c of cas) assert.equal(validerSession(c).ok, false, c.slice(0, 80));
});

test("la mort qu'Android a gardee entre, par sa raison fermee (SORTIE-1)", () => {
  const sortie = SESSION
    .replace('"fin":"propre"', '"fin":"faute_de_memoire"')
    .replace(
      '{"type":"fin","instant_ms":1200}',
      [
        '{"type":"sortie","instant_ms":0,"raison":"memoire_basse","statut":9,"importance":100,"pss_mo":1840,"rss_mo":2210}',
        '{"type":"sortie","instant_ms":0,"raison":"plantage_natif","statut":null,"importance":400,"pss_mo":0,"rss_mo":0}',
        '{"type":"fin","instant_ms":1200}',
      ].join(String.fromCharCode(10)),
    );
  const v = validerSession(sortie);
  assert.equal(v.ok, true, v.raison);
  const cas = [
    sortie.replace('"raison":"memoire_basse"', '"raison":"le pilote de Mali"'),
    sortie.replace('"rss_mo":2210', '"rss_mo":2210,"description":"lmkd"'),
    sortie.replace('"importance":100,', ""),
    sortie.replace('"pss_mo":1840', '"pss_mo":-1'),
    sortie.replace('"fin":"faute_de_memoire"', '"fin":"oubliee"'),
  ];
  for (const c of cas) assert.equal(validerSession(c).ok, false, c.slice(0, 80));
});

test("ce que le lancement a rouvert entre, par un nom ferme (REPRISE-1)", () => {
  const reprise = SESSION.replace(
    '{"type":"fin","instant_ms":1200}',
    [
      '{"type":"reprise","instant_ms":40,"rouvert":"brouillon","brouillons":2,"tenus":1,"brouillons_mo":9120,"images":6000,"mo":912,"libre_mo":310,"total_mo":null}',
      '{"type":"fin","instant_ms":1200}',
    ].join(String.fromCharCode(10)),
  );
  const v = validerSession(reprise);
  assert.equal(v.ok, true, v.raison);
  const cas = [
    reprise.replace('"rouvert":"brouillon"', '"rouvert":"Canevas 3"'),
    reprise.replace('"mo":912', '"mo":912,"chemin":"/data"'),
    reprise.replace(',"images":6000', ""),
    reprise.replace(',"tenus":1', ""),
  ];
  for (const c of cas) assert.equal(validerSession(c).ok, false, c.slice(0, 80));
});

test("une longue session raccourcie le dit, juste apres son debut (POIDS-1)", () => {
  const lignes = SESSION.split(String.fromCharCode(10));
  lignes.splice(1, 0, '{"type":"coupee","instant_ms":640000,"lignes":9150}');
  const coupee = lignes.join(String.fromCharCode(10));
  const v = validerSession(coupee);
  assert.equal(v.ok, true, v.raison);
  assert.equal(validerSession(coupee.replace('"lignes":9150', '"lignes":"beaucoup"')).ok, false);
});
