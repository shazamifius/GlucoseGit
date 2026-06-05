// Mesure le coût de A.save() (= le freeze à chaque navigation/autosave) AVANT/APRÈS
// la sortie des images du doc. Lecture seule sur le fichier source.
import { next as Automerge } from "@automerge/automerge";
import { readFileSync } from "node:fs";

const SRC = process.argv[2];
const bytes = new Uint8Array(readFileSync(SRC));
console.log(`Fichier : ${(bytes.length / 1024 / 1024).toFixed(1)} Mo\n`);

const doc = Automerge.load(bytes);

// --- AVANT : A.save() sur le doc actuel (historique + 117 images embarquées)
let t = performance.now();
const savedNow = Automerge.save(doc);
const tNow = performance.now() - t;
console.log(`AVANT  (doc actuel)  : A.save() = ${tNow.toFixed(0)} ms   → ${(savedNow.length/1024/1024).toFixed(1)} Mo`);

// --- APRÈS : doc neuf SANS blobs, refs embed→link, SANS historique (= ce que fait la migration)
function strip(d) {
  const boards = (d.boards || []).map((b) => ({
    ...b,
    images: (b.images || []).map((img) => {
      if (img.asset?.mode === "embed") {
        const { sha256, sizeBytes } = img.asset;
        return { ...img, asset: { mode: "link", href: `asset:${sha256.slice(0,16)}.png`, sha256, sizeBytes } };
      }
      return img;
    }),
  }));
  const plain = { ...d, boards };
  delete plain.blobs;
  return plain;
}

// Reconstruit un plain JS propre (sans proxy Automerge) via le même marqueur que asPlain
const plainRaw = JSON.parse(JSON.stringify(doc, (_k, v) => (v instanceof Uint8Array ? null : v)));
const stripped = strip(plainRaw);

let fresh = Automerge.init();
fresh = Automerge.change(fresh, "init", (dd) => { Object.assign(dd, stripped); });

t = performance.now();
const savedAfter = Automerge.save(fresh);
const tAfter = performance.now() - t;
console.log(`APRÈS  (doc « du dur ») : A.save() = ${tAfter.toFixed(0)} ms   → ${(savedAfter.length/1024).toFixed(0)} Ko`);

console.log(`\n→ Coût par navigation : ${tNow.toFixed(0)} ms  ⇒  ${tAfter.toFixed(0)} ms  (${(tNow/Math.max(tAfter,1)).toFixed(0)}× plus rapide)`);
console.log(`→ Taille du .glucose  : ${(savedNow.length/1024/1024).toFixed(1)} Mo  ⇒  ${(savedAfter.length/1024).toFixed(0)} Ko`);
