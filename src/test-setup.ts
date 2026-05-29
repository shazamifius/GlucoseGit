// Setup commun pour tous les tests Vitest.
//
// - matchMedia : jsdom ne l'implémente pas, certains composants Tailwind / PixiJS
//   en ont besoin.
// - ResizeObserver : idem.
// - URL.createObjectURL : utilisé par certains imports d'image.
// - crypto.randomUUID : fallback pour nanoid si nécessaire (déjà importé).
// - console.error : on intercepte pour faire échouer le test en cas d'erreur
//   React (incluant les hooks violations) — c'est PRÉCISÉMENT ce qu'on
//   cherche à détecter avec les smoke tests composants.

import "@testing-library/jest-dom/vitest";

if (typeof window !== "undefined") {
  if (!window.matchMedia) {
    window.matchMedia = (query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
    });
  }
  if (!window.ResizeObserver) {
    // Stub minimal pour jsdom (qui n'implémente pas ResizeObserver).
    class StubResizeObserver {
      observe() {}
      unobserve() {}
      disconnect() {}
    }
    window.ResizeObserver = StubResizeObserver as unknown as typeof ResizeObserver;
  }
  if (!URL.createObjectURL) {
    let counter = 0;
    URL.createObjectURL = () => `blob:test-${counter++}`;
  }
  // jsdom ne fournit pas le contexte canvas 2D. ColorPicker et Minimap en
  // ont besoin. On stubbe le minimum nécessaire pour qu'un mount ne crash pas.
  const proto = HTMLCanvasElement.prototype as unknown as {
    getContext: (kind: string) => unknown;
  };
  const realGetContext = proto.getContext;
  proto.getContext = function (kind: string) {
    if (kind !== "2d") {
      return typeof realGetContext === "function" ? realGetContext.call(this, kind) : null;
    }
    return {
      canvas: this,
      fillStyle: "#000", strokeStyle: "#000", lineWidth: 1,
      globalAlpha: 1, font: "10px sans-serif",
      createImageData: (w: number, h: number) => ({
        data: new Uint8ClampedArray(w * h * 4), width: w, height: h,
        colorSpace: "srgb" as const,
      }),
      putImageData: () => {},
      getImageData: (_x: number, _y: number, w: number, h: number) => ({
        data: new Uint8ClampedArray(w * h * 4), width: w, height: h,
        colorSpace: "srgb" as const,
      }),
      clearRect: () => {}, fillRect: () => {}, strokeRect: () => {},
      beginPath: () => {}, closePath: () => {},
      moveTo: () => {}, lineTo: () => {}, arc: () => {},
      stroke: () => {}, fill: () => {},
      save: () => {}, restore: () => {},
      translate: () => {}, scale: () => {}, rotate: () => {},
      setTransform: () => {}, resetTransform: () => {},
      drawImage: () => {},
      measureText: (text: string) => ({ width: text.length * 6 }),
      fillText: () => {}, strokeText: () => {},
      setLineDash: () => {}, getLineDash: () => [],
    };
  };
}
