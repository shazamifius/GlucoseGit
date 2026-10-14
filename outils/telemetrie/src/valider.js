// La loi de ce qui peut entrer : exactement les lignes que la boîte noire de Glucose écrit
// (crates/glucose-desktop/src/boite_noire/enregistrement.rs), et rien d'autre.
//
// Une ligne de plus, un champ de plus, un texte qui ne ressemble pas à un nom : la session
// entière est refusée. C'est la promesse de la fiche 49 § 2 tenue aussi de ce côté-ci — même un
// programme qui se ferait passer pour Glucose ne peut rien déposer qui porte un mot de quelqu'un.

/** Chaque type de ligne, et la sorte de chacun de ses champs. */
export const TYPES = {
  debut: {
    epoque_ms: "entier",
    version: "version",
    systeme: "systeme",
    architecture: "architecture",
    demarrage_appareil_ms: "entier?",
  },
  precedente: {
    instant_ms: "entier",
    fin: "fin",
    appareil_redemarre: "booleen",
    duree_ms: "entier",
    batterie_pct: "entier?",
    en_charge: "booleen?",
  },
  episode: {
    instant_ms: "entier",
    duree_ms: "entier",
    geste: "geste",
    images: "entier",
    median_us: "entier",
    p99_us: "entier",
    pire_us: "entier",
  },
  // **Où part le temps d'un épisode trop lent** (POSTES-1, fiche 63) : un poste de rendu, par
  // son nom fixé à la compilation, et ce qu'il a coûté pendant l'épisode.
  poste: {
    instant_ms: "entier",
    geste: "geste",
    nom: "poste",
    median_us: "entier",
    p99_us: "entier",
  },
  machine: { instant_ms: "entier", batterie_pct: "entier?", en_charge: "booleen?" },
  pire: { instant_ms: "entier", duree_us: "entier", geste: "geste" },
  pave: { instant_ms: "entier", quoi: "quoi", valeur: "entier" },
  fin: { instant_ms: "entier" },
  // **Glucose passe en arrière-plan, ou en revient** (BOITE-ANDROID, fiche 65) : sous Android,
  // le système le tue sans prévenir — une session qui finit derrière a fini ordinairement.
  arriere_plan: { instant_ms: "entier" },
  premier_plan: { instant_ms: "entier" },
  // **Ce que Glucose occupe, et ce que la machine laisse** (MEMOIRE-1, fiche 66), en
  // mébioctets : un record de la session, ou forcé après une alerte du système. Ce qu'une
  // plateforme ne sait pas lire est `null`.
  memoire: {
    instant_ms: "entier",
    processus_mo: "entier?",
    disponible_mo: "entier?",
    images_mo: "entier",
    offerts_mo: "entier",
    decodages_mo: "entier",
    carte_mo: "entier?",
  },
  // **Le système demande de rendre de la mémoire** (MEMOIRE-2) : sous Android, onTrimMemory.
  alerte_memoire: { instant_ms: "entier", arriere_plan: "booleen" },
  // La carte graphique, par ses numéros et des noms fermés — jamais le nom que donne son pilote
  // (fiche 60) : le fabricant et le modèle sont les identifiants PCI ou Khronos.
  carte: {
    instant_ms: "entier",
    moteur: "moteur",
    genre: "genre",
    fabricant: "entier",
    modele: "entier",
    plafond: "entier",
  },
  panique: { instant_ms: "entier", fichier: "source", ligne: "entier", principal: "booleen" },
  plantage: {
    instant_ms: "entier",
    gel: "booleen",
    module: "module?",
    version_du_module: "version_du_module?",
    code: "entier?",
    decalage: "entier?",
  },
};

/** Le poids le plus lourd d'une session : les siennes pèsent de 20 à 90 Ko. */
export const POIDS_MAX = 512 * 1024;

/**
 * **Chaque nom, pris dans la liste exacte de ce que Glucose écrit.** Un motif comme « des
 * minuscules » laisserait passer « mon document secret » ; une liste fermée, non. Une épreuve de
 * Glucose (`boite_noire::tests`) vérifie que chaque nom qu'il sait écrire est ici.
 */
export const NOMS = {
  geste: [
    "repos", "deplacer la vue", "zoomer", "glisser un noeud", "redimensionner", "dessiner",
    "selectionner", "editer du texte", "decoder des images", "animer la camera",
  ],
  fin: ["propre", "panique", "interrompue", "plantee", "gelee", "arriere_plan"],
  quoi: [
    "contact", "cadre_refuse", "interaction_debut", "interaction_fin",
    "statut_en_construction", "statut_actif", "statut_desactive", "statut_en_cours",
    "statut_inertie", "statut_pret", "statut_suspendu", "statut_inconnu",
  ],
  // Les postes de rendu, tels que `perf::stage` les marque dans le code de Glucose (POSTES-1) :
  // une épreuve de Glucose (`cliquets_suite`) vérifie que chaque marque est ici.
  poste: [
    "acquerir", "agrandir", "annotations", "ariane", "atelier", "bande", "blit", "clear",
    "composants", "cull", "docks", "effacer", "encoder", "etages", "fleches", "folders", "grid",
    "grille", "halos", "hues", "images", "index", "lueurs", "membranes", "minimap",
    "occlusion", "ornements", "poses", "present", "reclamer", "recolte", "region", "releve",
    "relever", "reperes", "report", "soumettre", "teintes", "tempo", "textures", "tuile", "ui",
    "vignettes", "zones",
  ],
  // Le moteur et le genre d'une carte, tels que `wgpu` les distingue.
  moteur: ["vulkan", "gl", "dx12", "metal", "webgpu", "aucun"],
  genre: ["integree", "dediee", "virtuelle", "processeur", "autre"],
  // `std::env::consts::OS` et `ARCH`, tels que Rust les nomme.
  systeme: ["windows", "linux", "macos", "android", "ios", "freebsd", "netbsd", "openbsd"],
  architecture: ["x86_64", "x86", "aarch64", "arm", "riscv64", "wasm32", "powerpc64", "loongarch64"],
};

// Une version : `2.0.2-dev`, `2.0.2-beta.1`. Une version d'un module de Windows : `32.0.16.1074`.
const VERSION = /^\d{1,5}\.\d{1,5}\.\d{1,5}(-(dev|alpha|beta|rc)(\.\d{1,5})?)?$/;
const VERSION_DU_MODULE = /^\d{1,5}(\.\d{1,5}){0,3}$/;
// Le chemin d'un fichier source de Glucose ou d'une caisse, tel que le compilateur l'écrit.
const SOURCE = /^[A-Za-z0-9_.:\\/ -]{1,200}$/;
// Un nom de module de Windows : un nom de fichier, jamais un chemin.
const MODULE = /^[A-Za-z0-9_.-]{1,64}$/;

function conforme(sorte, v) {
  const facultatif = sorte.endsWith("?");
  if (v === null) return facultatif;
  switch (sorte.replace("?", "")) {
    case "entier":
      return Number.isSafeInteger(v) && v >= 0;
    case "booleen":
      return typeof v === "boolean";
    case "geste":
    case "poste":
    case "fin":
    case "quoi":
    case "moteur":
    case "genre":
    case "systeme":
    case "architecture":
      return NOMS[sorte.replace("?", "")].includes(v);
    case "version":
      return typeof v === "string" && VERSION.test(v);
    case "version_du_module":
      return typeof v === "string" && VERSION_DU_MODULE.test(v);
    case "source":
      return typeof v === "string" && SOURCE.test(v);
    case "module":
      return typeof v === "string" && MODULE.test(v);
    default:
      return false;
  }
}

/**
 * Valide une session : du texte, une ligne JSON par enregistrement. Rend `{ ok: true, debut }`
 * — la ligne de début, qui dit la version et le système — ou `{ ok: false, raison }`.
 */
export function validerSession(texte) {
  if (typeof texte !== "string" || texte.length === 0) return { ok: false, raison: "vide" };
  if (new TextEncoder().encode(texte).length > POIDS_MAX) return { ok: false, raison: "trop lourde" };
  let debut = null;
  const lignes = texte.split("\n").filter((l) => l.length > 0);
  for (const [i, l] of lignes.entries()) {
    let v;
    try {
      v = JSON.parse(l);
    } catch {
      return { ok: false, raison: `ligne ${i + 1} : pas du JSON` };
    }
    if (v === null || typeof v !== "object" || Array.isArray(v)) {
      return { ok: false, raison: `ligne ${i + 1} : pas un objet` };
    }
    const loi = TYPES[v.type];
    if (!loi) return { ok: false, raison: `ligne ${i + 1} : type inconnu` };
    const cles = Object.keys(v).filter((k) => k !== "type");
    for (const k of cles) {
      if (!(k in loi)) return { ok: false, raison: `ligne ${i + 1} : champ inconnu ${k}` };
      if (!conforme(loi[k], v[k])) return { ok: false, raison: `ligne ${i + 1} : champ ${k}` };
    }
    for (const k of Object.keys(loi)) {
      if (!(k in v) && !loi[k].endsWith("?")) {
        return { ok: false, raison: `ligne ${i + 1} : champ manquant ${k}` };
      }
    }
    if (v.type === "debut") {
      if (i !== 0) return { ok: false, raison: "le debut n'est pas en tete" };
      debut = v;
    }
  }
  if (!debut) return { ok: false, raison: "aucun debut" };
  return { ok: true, debut };
}

/** Un identifiant d'installation ou de session : tiré au hasard par Glucose, 32 chiffres hex. */
export function identifiant(v) {
  return typeof v === "string" && /^[0-9a-f]{32}$/.test(v);
}
