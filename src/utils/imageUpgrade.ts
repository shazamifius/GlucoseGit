/**
 * CDN URL upgrade rules — no API key required.
 * Given an image URL, returns a list of higher-resolution candidate URLs
 * to try in order (best quality first). The caller fetches them and takes
 * the first successful response.
 */

const CDN_RULES: Array<{
  name: string;
  test:       (url: string) => boolean;
  candidates: (url: string) => string[];
}> = [
  // ── Pinterest ─────────────────────────────────────────────────────────────
  // https://i.pinimg.com/236x/ab/cd/ef/hash.jpg  →  originals/ → 1200x/ → 736x/
  {
    name: "Pinterest",
    test: (u) => /i\.pinimg\.com\//.test(u),
    candidates: (u) => {
      const m = u.match(/i\.pinimg\.com\/[^/]+\/(.+)/);
      if (!m) return [];
      const path = m[1].split("?")[0];
      return [
        `https://i.pinimg.com/originals/${path}`,
        `https://i.pinimg.com/1200x/${path}`,
        `https://i.pinimg.com/736x/${path}`,
      ];
    },
  },

  // ── Twitter / X ───────────────────────────────────────────────────────────
  // ?format=jpg&name=medium  →  name=orig
  // :small / :large suffix   →  :orig
  {
    name: "Twitter",
    test: (u) => /pbs\.twimg\.com\/media\//.test(u),
    candidates: (u) => {
      const q = u.match(/(https:\/\/pbs\.twimg\.com\/media\/[^?]+)\?format=(\w+)&name=\w+/);
      if (q) return [`${q[1]}?format=${q[2]}&name=orig`, `${q[1]}?format=${q[2]}&name=large`];
      const s = u.match(/(https:\/\/pbs\.twimg\.com\/media\/.+?)(?::\w+)?$/);
      if (s) return [`${s[1]}:orig`, `${s[1]}:large`];
      return [];
    },
  },

  // ── Reddit ────────────────────────────────────────────────────────────────
  // https://preview.redd.it/HASH.jpg?width=…  →  https://i.redd.it/HASH.jpg
  {
    name: "Reddit",
    test: (u) => /preview\.redd\.it\//.test(u),
    candidates: (u) => {
      const m = u.match(/preview\.redd\.it\/([^?]+)/);
      return m ? [`https://i.redd.it/${m[1]}`] : [];
    },
  },

  // ── Imgur ─────────────────────────────────────────────────────────────────
  // Single-letter suffix: s b t m l h  →  remove suffix = original
  {
    name: "Imgur",
    test: (u) => /i\.imgur\.com\/\w+[sbtlmh]\.\w+/.test(u),
    candidates: (u) => {
      const up = u.replace(/(i\.imgur\.com\/\w+)[sbtlmh](\.\w+)/, "$1$2");
      return up !== u ? [up] : [];
    },
  },

  // ── Tumblr ────────────────────────────────────────────────────────────────
  // _250 / _400 / _500 suffix  →  _raw → _1280
  {
    name: "Tumblr",
    test: (u) => /media\.tumblr\.com\//.test(u),
    candidates: (u) => {
      const ext = (u.match(/\.(jpg|jpeg|png|gif|webp)(\?|$)/i) ?? [, ".jpg"])[1];
      const base = u.replace(/_\d+(\.(?:jpg|jpeg|png|gif|webp))(\?.*)?$/i, "");
      return base !== u
        ? [`${base}_raw${ext}`, `${base}_1280${ext}`]
        : [];
    },
  },

  // ── Wallhaven thumbnails ──────────────────────────────────────────────────
  // https://th.wallhaven.cc/small/ab/abcdef.jpg  →  full size
  {
    name: "Wallhaven",
    test: (u) => /th\.wallhaven\.cc\/small\//.test(u),
    candidates: (u) => {
      const m = u.match(/th\.wallhaven\.cc\/small\/(\w+)\/(\w+)\.(\w+)/);
      if (!m) return [];
      return [
        `https://w.wallhaven.cc/full/${m[1]}/wallhaven-${m[2]}.jpg`,
        `https://w.wallhaven.cc/full/${m[1]}/wallhaven-${m[2]}.png`,
      ];
    },
  },

  // ── DeviantArt (wixmp CDN) ────────────────────────────────────────────────
  // Strip /v1/fill/w_NNN,h_NNN,... path to get original
  {
    name: "DeviantArt",
    test: (u) => /wixmp\.com\/f\//.test(u),
    candidates: (u) => {
      const m = u.match(/(https:\/\/images-wixmp[^/]*\.wixmp\.com\/f\/[^/]+\/[^/]+\.(?:jpg|jpeg|png|gif|webp))/i);
      return m ? [m[1]] : [];
    },
  },

  // ── ArtStation ────────────────────────────────────────────────────────────
  // /small/ or /medium/  →  /large/ → /4k/
  {
    name: "ArtStation",
    test: (u) => /artstation\.com\/p\/assets\//.test(u),
    candidates: (u) => [
      u.replace(/\/(small|medium|large|4k)\//, "/4k/"),
      u.replace(/\/(small|medium|large|4k)\//, "/large/"),
    ].filter((c) => c !== u),
  },
];

/**
 * Returns candidate URLs to try for higher-resolution versions of `url`,
 * ordered best-first. Returns [] if the URL doesn't match any known CDN.
 * The caller should try each in order and use the first successful fetch.
 */
export function getCDNCandidates(url: string): string[] {
  const rule = CDN_RULES.find((r) => r.test(url));
  if (!rule) return [];
  return rule.candidates(url).filter(Boolean);
}
