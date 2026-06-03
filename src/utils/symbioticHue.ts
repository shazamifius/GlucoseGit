// Symbiose chromatique — fonction PURE extraite de HtmlAnnotationLayer.
//
// La teinte d'une annotation émerge de sa POSITION (bruit organique 2D par zones
// de ~2000px) puis est tirée vers la moyenne vectorielle circulaire de ses
// voisines (rayon 1200px). `ann.color` non-blanc passe outre côté appelant.
//
// Isolée ici (sans dépendance React/markdown) pour être réutilisable par le
// pipeline d'export et testable sans monter tout le canvas. HtmlAnnotationLayer
// la re-exporte pour compat ascendante.
import { Annotation } from "../types";

export function getSymbioticHue(ann: Annotation, allAnnotations: Annotation[]): number {
  const idHash = (id: string) => {
    let h = 5381;
    for (let i = 0; i < id.length; i++) h = ((h << 5) + h) + id.charCodeAt(i);
    return Math.abs(h);
  };

  // 1. TOUTES LES COULEURS DU MONDE (Bruit Organique)
  const getZoneHue = (x: number, y: number) => {
    const scale = 2000; // Les zones de couleur changent tous les ~2000 pixels
    const cx = x / scale;
    const cy = y / scale;

    const random2D = (ix: number, iy: number) => {
      const dot = ix * 12.9898 + iy * 78.233;
      const sin = Math.sin(dot) * 43758.5453;
      return sin - Math.floor(sin);
    };

    const smooth = (t: number) => t * t * (3 - 2 * t);

    const x0 = Math.floor(cx);
    const x1 = x0 + 1;
    const y0 = Math.floor(cy);
    const y1 = y0 + 1;

    const sx = smooth(cx - x0);
    const sy = smooth(cy - y0);

    const nx0 = random2D(x0, y0) * (1 - sx) + random2D(x1, y0) * sx;
    const nx1 = random2D(x0, y1) * (1 - sx) + random2D(x1, y1) * sx;
    const value = nx0 * (1 - sy) + nx1 * sy;

    // ON DÉBLOQUE TOUT : 0 à 360 degrés !
    return value * 360;
  };

  // Identité de base (Biome + variation unique du bloc)
  let myBaseHue = getZoneHue(ann.x, ann.y) + ((idHash(ann.id) % 80) - 40);
  myBaseHue = ((myBaseHue % 360) + 360) % 360;

  // 2. L'ALGORITHME DE DÉGRADÉ POUSSÉ (Moyenne Vectorielle Circulaire)
  const RAYON = 1200;

  // Pour ne pas créer de couleurs moches ou de sauts étranges, on ne fait pas
  // une bête moyenne mathématique. On utilise des vecteurs (Trigonométrie).
  let sumX = 0;
  let sumY = 0;
  let envWeightSum = 0;

  for (const other of allAnnotations) {
    if (other.id === ann.id || other.type !== "text") continue;

    const dx = ann.x - other.x;
    if (Math.abs(dx) > RAYON) continue;
    const dy = ann.y - other.y;
    if (Math.abs(dy) > RAYON) continue;

    const dist = Math.sqrt(dx * dx + dy * dy);
    if (dist < RAYON) {
      const weight = Math.pow(1 - (dist / RAYON), 2);

      const otherHue = getZoneHue(other.x, other.y) + ((idHash(other.id) % 80) - 40);

      // Conversion de la couleur en vecteur (angle -> x, y)
      const rad = otherHue * (Math.PI / 180);
      sumX += Math.cos(rad) * weight;
      sumY += Math.sin(rad) * weight;

      envWeightSum += weight;
    }
  }

  // 3. SYMBIOSE PARFAITE
  if (envWeightSum > 0) {
    // On retrouve l'angle moyen exact de l'environnement (sans aucun saut au passage par 0)
    let envHue = Math.atan2(sumY, sumX) * (180 / Math.PI);
    if (envHue < 0) envHue += 360;

    // Calcul de l'écart via le chemin le plus court sur la roue des couleurs
    let diff = envHue - myBaseHue;
    if (diff > 180) diff -= 360;
    if (diff < -180) diff += 360;

    // L'environnement tire doucement le bloc vers lui (dégradé très harmonieux)
    const influence = 0.5 * (1 - 1 / (1 + envWeightSum));
    myBaseHue += (diff * influence);
  }

  return ((myBaseHue % 360) + 360) % 360;
}
