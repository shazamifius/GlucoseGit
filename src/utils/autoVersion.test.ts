// ────────────────────────────────────────────────────────────────────────────
// Git #1 Phase 3 — Tests du déclencheur de jalons AUTO « à l'ampleur ».
// On mocke ./versions pour ne pas toucher Tauri : on vérifie la LOGIQUE
// (accumulation, seuil, remise à zéro), pas l'I/O disque (couverte par versions).
// ────────────────────────────────────────────────────────────────────────────

import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("./versions", () => ({
  saveVersion: vi.fn(async () => ({ path: "p", file: "f", time: 0, kind: "auto", label: "l" })),
  pruneAutoVersions: vi.fn(async () => 0),
}));

import { LIMITS } from "../constants";
import {
  noteSavedDelta,
  resetAutoVersionAccumulator,
  _peekAutoAccum,
  maybeCreateAutoVersion,
} from "./autoVersion";
import { saveVersion, pruneAutoVersions } from "./versions";

const THRESH = LIMITS.AUTO_VERSION_DELTA_BYTES;
const fakeDoc = {} as never;

beforeEach(() => {
  resetAutoVersionAccumulator();
  vi.clearAllMocks();
});

describe("autoVersion — accumulateur d'ampleur", () => {
  it("accumule et lit le volume", () => {
    noteSavedDelta(100);
    noteSavedDelta(50);
    expect(_peekAutoAccum()).toBe(150);
  });

  it("reset remet à zéro", () => {
    noteSavedDelta(100);
    resetAutoVersionAccumulator();
    expect(_peekAutoAccum()).toBe(0);
  });

  it("ignore les deltas nuls ou négatifs", () => {
    noteSavedDelta(0);
    noteSavedDelta(-5);
    expect(_peekAutoAccum()).toBe(0);
  });
});

describe("autoVersion — déclenchement au seuil", () => {
  it("sous le seuil → aucun jalon auto, compteur inchangé", async () => {
    noteSavedDelta(THRESH - 1);
    await maybeCreateAutoVersion("/x.glucose", fakeDoc);
    expect(saveVersion).not.toHaveBeenCalled();
    expect(pruneAutoVersions).not.toHaveBeenCalled();
    expect(_peekAutoAccum()).toBe(THRESH - 1);
  });

  it("au seuil → jalon auto écrit + élagage + compteur remis à zéro", async () => {
    noteSavedDelta(THRESH);
    await maybeCreateAutoVersion("/x.glucose", fakeDoc);
    expect(saveVersion).toHaveBeenCalledOnce();
    expect(saveVersion).toHaveBeenCalledWith(
      "/x.glucose", fakeDoc, expect.stringContaining("auto"), "auto",
    );
    expect(pruneAutoVersions).toHaveBeenCalledWith("/x.glucose", LIMITS.AUTO_VERSION_KEEP);
    expect(_peekAutoAccum()).toBe(0);
  });
});
