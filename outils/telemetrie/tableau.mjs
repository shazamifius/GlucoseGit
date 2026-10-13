// **Le tableau de bord de la boîte noire** (fiche 65) : la base lue depuis son compte, comme
// toujours (`wrangler d1 execute` — le Worker n'a aucune porte pour lire), résumée en trois
// fichiers JSON que le tableau de bord cite.
//
//   node outils/telemetrie/tableau.mjs <dossier de sortie>
//
// * `installations.json` : une rangée par installation — son système, sa carte graphique par
//   son nom, ses sessions, comment elles ont fini, sa pire image ;
// * `jours.json` : les sessions reçues, par jour et par système ;
// * `fins.json` : chaque session qui a mal fini, si l'appareil a redémarré, et combien de temps
//   après sa dernière trace — quelques dizaines de secondes disent un redémarrage **pendant**
//   Glucose, des heures un redémarrage sans lien.
//
// **Comment une session a fini** ne se lit pas dans ses propres lignes : c'est la session
// **suivante** de la même installation qui le dit (sa ligne `precedente`). La dernière session
// reçue d'une installation n'a donc pas encore de fin connue.
//
// Les cartes graphiques n'arrivent que par leurs numéros (CARTE-1) ; leurs noms viennent de la
// base publique `pci.ids`, gardée dans le dossier temporaire.

import { execSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ICI = dirname(fileURLToPath(import.meta.url));
const SORTIE = process.argv[2] ?? ".";

/** Les fabricants que `pci.ids` ne nomme pas comme on les connaît (identifiants Khronos). */
const FABRICANTS = {
  0x1002: "AMD",
  0x10de: "NVIDIA",
  0x8086: "Intel",
  0x1010: "Imagination (PowerVR)",
  0x13b5: "Arm (Mali)",
  0x5143: "Qualcomm (Adreno)",
  0x106b: "Apple",
  0x10005: "Mesa",
  0x144d: "Samsung",
};

function lireLaBase() {
  const sql =
    "SELECT installation, session, recue, version, systeme, architecture, lignes FROM sessions";
  // Une seule chaîne, la requête entre guillemets : `npx` est un script de commande sous
  // Windows, qui ne se lance que par un shell, et un shell recolle les arguments.
  const commande = `npx wrangler d1 execute glucose-boite-noire --remote --json --command "${sql}"`;
  const sortie = execSync(commande, {
    cwd: ICI,
    encoding: "utf-8",
    maxBuffer: 64 * 1024 * 1024,
  });
  return JSON.parse(sortie)[0].results;
}

async function pciIds() {
  const chemin = join(tmpdir(), "glucose-pci.ids");
  if (!existsSync(chemin)) {
    const reponse = await fetch("https://pci-ids.ucw.cz/v2.2/pci.ids");
    writeFileSync(chemin, await reponse.text());
  }
  const vendeurs = new Map();
  let courant = null;
  for (const ligne of readFileSync(chemin, "utf-8").split("\n")) {
    if (ligne.startsWith("#") || ligne.trim() === "") continue;
    if (/^[0-9a-f]{4}\s/.test(ligne)) {
      courant = { nom: ligne.slice(4).trim(), modeles: new Map() };
      vendeurs.set(parseInt(ligne.slice(0, 4), 16), courant);
    } else if (/^\t[0-9a-f]{4}\s/.test(ligne) && courant) {
      courant.modeles.set(parseInt(ligne.slice(1, 5), 16), ligne.slice(5).trim());
    } else if (/^C\s/.test(ligne)) {
      break; // les classes de périphériques : plus de fabricants après
    }
  }
  return vendeurs;
}

/** Le nom d'une carte : le nom commercial entre crochets quand `pci.ids` en donne un. */
function nomDeLaCarte(pci, fabricant, modele) {
  const vendeur = pci.get(fabricant);
  const marque = FABRICANTS[fabricant] ?? vendeur?.nom ?? `fabricant 0x${fabricant.toString(16)}`;
  const brut = vendeur?.modeles.get(modele);
  if (!brut) return `${marque}, modèle 0x${modele.toString(16)}`;
  const crochets = brut.match(/\[([^\]]+)\]/);
  return `${marque} ${crochets ? crochets[1] : brut}`;
}

function lignesDe(session) {
  return session.lignes
    .split("\n")
    .filter((l) => l.trim() !== "")
    .map((l) => JSON.parse(l));
}

/** La médiane des médianes des épisodes de ce geste, pondérée par leurs images — une
 * approximation de l'image typique, dite comme telle (on n'additionne jamais des centiles). */
function typique(episodes) {
  const tries = episodes.filter((e) => e.images > 0).sort((a, b) => a.median_us - b.median_us);
  const total = tries.reduce((s, e) => s + e.images, 0);
  let cumul = 0;
  for (const e of tries) {
    cumul += e.images;
    if (cumul * 2 >= total) return e.median_us;
  }
  return null;
}

const ms = (us) => (us == null ? null : Math.round(us / 100) / 10);

async function main() {
  const pci = await pciIds();
  const sessions = lireLaBase().map((s) => ({ ...s, l: lignesDe(s) }));
  for (const s of sessions) {
    const debut = s.l.find((x) => x.type === "debut");
    s.debut = debut?.epoque_ms ?? 0;
    s.demarrage = debut?.demarrage_appareil_ms ?? null;
    s.derniereTrace = s.debut + Math.max(0, ...s.l.map((x) => x.instant_ms ?? 0));
    s.precedente = s.l.find((x) => x.type === "precedente");
    s.carte = s.l.find((x) => x.type === "carte");
  }
  const parInstallation = new Map();
  for (const s of sessions) {
    if (!parInstallation.has(s.installation)) parInstallation.set(s.installation, []);
    parInstallation.get(s.installation).push(s);
  }
  const installations = [];
  const fins = [];
  for (const [id, liste] of parInstallation) {
    liste.sort((a, b) => a.debut - b.debut);
    // Comment chaque session a fini : la suivante le dit.
    liste.forEach((s, i) => {
      const suivante = liste[i + 1];
      const p = suivante?.precedente;
      s.fin = p?.fin ?? null;
      s.redemarre = p?.appareil_redemarre ?? null;
      // Le démarrage de l'appareil, noté par la suivante, moins la dernière trace de celle-ci.
      const apres =
        s.redemarre && suivante.demarrage != null
          ? Math.round((suivante.demarrage - s.derniereTrace) / 60000)
          : null;
      if (s.fin && s.fin !== "propre" && s.fin !== "arriere_plan") {
        fins.push({
          installation: id.slice(0, 8),
          jour: s.recue,
          version: s.version,
          systeme: s.systeme,
          fin: s.fin,
          appareil_redemarre: s.redemarre ? "oui" : "non",
          redemarre_apres_min: apres,
        });
      }
    });
    const derniere = liste[liste.length - 1];
    const carte = [...liste].reverse().find((s) => s.carte)?.carte;
    const episodes = liste.flatMap((s) => s.l.filter((x) => x.type === "episode"));
    const deplacer = episodes.filter((e) => e.geste === "deplacer la vue");
    const pire = Math.max(0, ...episodes.map((e) => e.pire_us));
    installations.push({
      installation: id.slice(0, 8),
      systeme: derniere.systeme,
      architecture: derniere.architecture,
      carte: carte ? nomDeLaCarte(pci, carte.fabricant, carte.modele) : "inconnue",
      moteur: carte?.moteur ?? "",
      sessions: liste.length,
      premiere: liste.map((s) => s.recue).sort()[0],
      derniere: liste.map((s) => s.recue).sort().at(-1),
      version: derniere.version,
      fins_propres: liste.filter((s) => s.fin === "propre" || s.fin === "arriere_plan").length,
      fins_anormales: liste.filter(
        (s) => s.fin && s.fin !== "propre" && s.fin !== "arriere_plan",
      ).length,
      redemarrages: liste.filter((s) => s.redemarre).length,
      image_typique_ms: ms(typique(deplacer)),
      pire_image_ms: ms(pire),
    });
  }
  installations.sort((a, b) => b.sessions - a.sessions);
  fins.sort((a, b) => a.jour.localeCompare(b.jour));
  fins.forEach((f, i) => {
    f.n = i + 1;
  });
  const jours = new Map();
  for (const s of sessions) {
    const cle = `${s.recue}|${s.systeme}`;
    jours.set(cle, (jours.get(cle) ?? 0) + 1);
  }
  const parJour = [...jours]
    .map(([cle, n]) => ({ jour: cle.split("|")[0], systeme: cle.split("|")[1], sessions: n }))
    .sort((a, b) => a.jour.localeCompare(b.jour) || a.systeme.localeCompare(b.systeme));
  mkdirSync(SORTIE, { recursive: true });
  writeFileSync(join(SORTIE, "installations.json"), JSON.stringify(installations, null, 1));
  writeFileSync(join(SORTIE, "jours.json"), JSON.stringify(parJour, null, 1));
  writeFileSync(join(SORTIE, "fins.json"), JSON.stringify(fins, null, 1));
  console.log(`${sessions.length} sessions, ${installations.length} installations, ${fins.length} fins anormales`);
  for (const i of installations) console.log(JSON.stringify(i));
}

main();
