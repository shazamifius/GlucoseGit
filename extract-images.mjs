// Extraction one-shot des images embarquées d'un .glucose (doc Automerge v2).
// Lit project.blobs (sha256 -> bytes) + croise avec les AssetRef des images
// pour le mime, sniffe les magic bytes pour l'extension, écrit chaque image.
import { next as Automerge } from "@automerge/automerge";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const SRC = process.argv[2];
const OUT = process.argv[3];
if (!SRC || !OUT) {
  console.error("usage: node extract-images.mjs <src.glucose> <outDir>");
  process.exit(1);
}

mkdirSync(OUT, { recursive: true });

console.log("Lecture du fichier…");
const bytes = new Uint8Array(readFileSync(SRC));
console.log(`  ${(bytes.length / 1024 / 1024).toFixed(1)} Mo`);

console.log("Chargement du document Automerge…");
const doc = Automerge.load(bytes);

// Normalise un éventuel proxy Automerge en vrai Uint8Array.
function toU8(raw) {
  if (raw instanceof Uint8Array) return raw;
  if (ArrayBuffer.isView(raw)) return new Uint8Array(raw.buffer, raw.byteOffset, raw.byteLength);
  const len = raw?.length ?? Object.keys(raw || {}).length;
  const u8 = new Uint8Array(len);
  for (let i = 0; i < len; i++) u8[i] = raw[i] ?? 0;
  return u8;
}

// Sniff l'extension depuis les magic bytes.
function sniffExt(b) {
  if (b.length >= 8 && b[0] === 0x89 && b[1] === 0x50 && b[2] === 0x4e && b[3] === 0x47) return "png";
  if (b.length >= 3 && b[0] === 0xff && b[1] === 0xd8 && b[2] === 0xff) return "jpg";
  if (b.length >= 12 && b[0] === 0x52 && b[1] === 0x49 && b[2] === 0x46 && b[3] === 0x46
      && b[8] === 0x57 && b[9] === 0x45 && b[10] === 0x42 && b[11] === 0x50) return "webp";
  if (b.length >= 6 && b[0] === 0x47 && b[1] === 0x49 && b[2] === 0x46) return "gif";
  if (b.length >= 12 && b[4] === 0x66 && b[5] === 0x74 && b[6] === 0x79 && b[7] === 0x70) return "mp4"; // ftyp
  if (b.length >= 5 && b[0] === 0x3c && (b[1] === 0x73 || b[1] === 0x3f)) return "svg"; // <s / <?
  return "bin";
}

// 1) Récupère la table sha -> mime/nom depuis les images des boards.
const metaBySha = new Map();
let imgCount = 0;
const boards = doc.boards || [];
for (const b of boards) {
  for (const img of (b.images || [])) {
    imgCount++;
    const a = img.asset;
    const sha = a?.sha256 || (typeof img.src === "string" && img.src.startsWith("asset:") ? img.src.slice(6).split(".")[0] : null);
    if (sha) {
      if (!metaBySha.has(sha)) metaBySha.set(sha, { mime: a?.mime, name: img.name || img.label || img.title });
    }
  }
}
console.log(`  ${boards.length} board(s), ${imgCount} référence(s) d'image`);

// 2) Dump tous les blobs embarqués.
const blobs = doc.blobs || {};
const shas = Object.keys(blobs);
console.log(`  ${shas.length} blob(s) embarqué(s) à extraire`);

const manifest = [];
let i = 0;
for (const sha of shas) {
  i++;
  const u8 = toU8(blobs[sha]);
  const ext = sniffExt(u8);
  const idx = String(i).padStart(3, "0");
  const fname = `img_${idx}_${sha.slice(0, 12)}.${ext}`;
  writeFileSync(join(OUT, fname), u8);
  manifest.push({ index: i, sha, file: fname, bytes: u8.length, ext, mime: metaBySha.get(sha)?.mime, name: metaBySha.get(sha)?.name });
  if (i % 20 === 0) console.log(`  …${i}/${shas.length}`);
}

writeFileSync(join(OUT, "_manifest.json"), JSON.stringify(manifest, null, 2));
const totalMb = (manifest.reduce((s, m) => s + m.bytes, 0) / 1024 / 1024).toFixed(1);
console.log(`\n✅ ${manifest.length} image(s) extraite(s) (${totalMb} Mo) vers:\n   ${OUT}`);
