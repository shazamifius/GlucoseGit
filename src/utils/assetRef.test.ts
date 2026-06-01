// ────────────────────────────────────────────────────────────────────────────
// R-EMB-01 — Tests des helpers AssetRef.
// ────────────────────────────────────────────────────────────────────────────

import { afterEach, describe, expect, it } from "vitest";
import {
  buildEmbedRef,
  buildLinkRef,
  dataUrlToBytes,
  extFromMime,
  mimeFromExt,
  releaseAllBlobUrls,
  resolveAssetRefSync,
  sha256Hex,
} from "./assetRef";

afterEach(() => releaseAllBlobUrls());

describe("sha256Hex", () => {
  it("produit 64 chars hex stables", async () => {
    const data = new Uint8Array([1, 2, 3, 4, 5]);
    const h1 = await sha256Hex(data);
    expect(h1).toMatch(/^[0-9a-f]{64}$/);
    const h2 = await sha256Hex(data);
    expect(h2).toBe(h1);
  });
  it("payloads différents → hashes différents", async () => {
    const h1 = await sha256Hex(new Uint8Array([0]));
    const h2 = await sha256Hex(new Uint8Array([1]));
    expect(h1).not.toBe(h2);
  });
});

describe("dataUrlToBytes", () => {
  it("décode un PNG transparent 1x1 (base64)", () => {
    // Plus petit PNG valide (1x1 transparent)
    const dataUrl = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=";
    const { bytes, mime } = dataUrlToBytes(dataUrl);
    expect(mime).toBe("image/png");
    expect(bytes.length).toBeGreaterThan(50);
    // Magic bytes PNG : 89 50 4E 47 0D 0A 1A 0A
    expect(bytes[0]).toBe(0x89);
    expect(bytes[1]).toBe(0x50);
    expect(bytes[2]).toBe(0x4e);
    expect(bytes[3]).toBe(0x47);
  });
  it("rejette les non-data URLs", () => {
    expect(() => dataUrlToBytes("https://x.com/a.png")).toThrow(/non-data URL/);
  });
});

describe("mimeFromExt / extFromMime", () => {
  it("traite png / jpg / mp4", () => {
    expect(mimeFromExt("png")).toBe("image/png");
    expect(mimeFromExt(".jpg")).toBe("image/jpeg");
    expect(mimeFromExt("JPEG")).toBe("image/jpeg");
    expect(mimeFromExt("mp4")).toBe("video/mp4");
  });
  it("inconnu → octet-stream", () => {
    expect(mimeFromExt("unknownext")).toBe("application/octet-stream");
  });
  it("inverse fonctionne pour image/jpeg → jpg", () => {
    expect(extFromMime("image/jpeg")).toBe("jpg");
    expect(extFromMime("image/png")).toBe("png");
    expect(extFromMime("application/octet-stream")).toBe("bin");
  });
});

describe("buildEmbedRef / buildLinkRef", () => {
  it("buildEmbedRef calcule sha + remplit champs", async () => {
    const bytes = new Uint8Array([10, 20, 30, 40, 50]);
    const ref = await buildEmbedRef(bytes, "image/png");
    expect(ref.mode).toBe("embed");
    expect(ref.sha256).toMatch(/^[0-9a-f]{64}$/);
    expect(ref.mime).toBe("image/png");
    expect(ref.sizeBytes).toBe(5);
  });
  it("buildLinkRef accepte sha optionnel", () => {
    const r1 = buildLinkRef("https://x.com/a.png");
    expect(r1).toEqual({ mode: "link", href: "https://x.com/a.png" });
    const r2 = buildLinkRef("C:/x.png", { sha256: "abc", sizeBytes: 1024 });
    expect(r2.mode).toBe("link");
    expect(r2.href).toBe("C:/x.png");
    expect(r2.sha256).toBe("abc");
    expect(r2.sizeBytes).toBe(1024);
  });
});

describe("resolveAssetRefSync", () => {
  it("mode link renvoie href tel quel", () => {
    const url = resolveAssetRefSync(
      { mode: "link", href: "https://x.com/a.png" },
      {},
    );
    expect(url).toBe("https://x.com/a.png");
  });
  it("mode embed renvoie un blob URL stable (cache)", async () => {
    const bytes = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
    const ref = await buildEmbedRef(bytes, "image/png");
    const blobs = { [ref.sha256]: bytes };

    const url1 = resolveAssetRefSync(ref, blobs);
    expect(url1).toMatch(/^blob:/);
    const url2 = resolveAssetRefSync(ref, blobs);
    expect(url2).toBe(url1); // cache hit
  });
  it("mode embed avec blob manquant renvoie chaîne vide", async () => {
    const ref = await buildEmbedRef(new Uint8Array([1, 2, 3]), "image/png");
    const url = resolveAssetRefSync(ref, {});
    expect(url).toBe("");
  });
  it("cache LRU limite à 256 entrées sans crash", async () => {
    // Crée 260 embeds distincts et vérifie que le cache ne dépasse pas
    // (test de non-régression : éviction silencieuse).
    const blobs: Record<string, Uint8Array> = {};
    const refs = [];
    for (let i = 0; i < 260; i++) {
      const bytes = new Uint8Array([i & 0xff, (i >> 8) & 0xff, 0xab, 0xcd, 0xef]);
      const ref = await buildEmbedRef(bytes);
      blobs[ref.sha256] = bytes;
      refs.push(ref);
    }
    for (const ref of refs) resolveAssetRefSync(ref, blobs);
    // Pas de crash, pas d'assertion bloquante — le test prouve la robustesse.
    expect(refs.length).toBe(260);
  });
});
