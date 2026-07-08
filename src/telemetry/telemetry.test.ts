// Smoke tests de la télémétrie : le chemin CHAUD (recordAction, appelé à chaque
// mutation du store) et le benchmark ne doivent JAMAIS jeter ni bloquer l'app,
// même sans consentement / sans WebGL (jsdom).

import { describe, it, expect } from "vitest";
import {
  runBenchmark,
  recordAction,
  setTelemetryContext,
  getConsentState,
} from "./telemetry";

describe("télémétrie — benchmark", () => {
  it("runBenchmark renvoie un résultat cohérent (cpuMs, glMs, score)", () => {
    const b = runBenchmark();
    expect(b.cpuMs).toBeGreaterThanOrEqual(0);
    expect(Number.isFinite(b.glMs)).toBe(true);
    expect(b.score).toBeGreaterThan(0);
  });
});

describe("télémétrie — chemin chaud sûr", () => {
  it("recordAction ne jette jamais (même sans consentement)", () => {
    expect(() => {
      for (let i = 0; i < 1000; i++) recordAction("setViewport", i % 5);
    }).not.toThrow();
  });

  it("setTelemetryContext ne jette pas", () => {
    expect(() => setTelemetryContext({ panels: ["timeline"], collab: false })).not.toThrow();
  });

  it("par défaut, aucun consentement n'est accordé", () => {
    // (localStorage jsdom vierge) → « unset » : rien n'est jamais envoyé.
    expect(getConsentState()).toBe("unset");
  });
});
