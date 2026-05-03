// Level of Detail (LOD) — pilote l'adaptation du rendu selon le zoom.
// Trois lois invariantes : densité visuelle constante, connexion latente, domaine coloré.

export type LOD = "macro" | "meso" | "micro";

// Seuils pensés pour qu'à scale = 1.0 (zoom par défaut) on soit DÉJÀ en micro.
// Macro reste rare (très dézoomé) ; meso est un bref intervalle de transition.
export const LOD_THRESHOLDS = {
  macroToMeso: 0.25,
  mesoToMicro: 0.55,
} as const;

// ⚠️ LOD désactivé temporairement — force "micro" à tous les zooms.
// Pour réactiver, restaurer la logique commentée ci-dessous.
export function computeLOD(_scale: number): LOD {
  return "micro";
  // if (scale < LOD_THRESHOLDS.macroToMeso) return "macro";
  // if (scale < LOD_THRESHOLDS.mesoToMicro) return "meso";
  // return "micro";
}

// Règle anti-spaghetti : une flèche n'est rendue que si AU MOINS une condition s'applique.
// Le LOD module les conditions évaluées :
//   - macro : seules les flèches trans-domaines (en pointillés) et épinglées sont peintes
//   - meso  : + flèches dont source ou cible est dans la sélection
//   - micro : + flèches dont source ou cible est sous le curseur
export interface ArrowVisibilityContext {
  lod: LOD;
  selectedNodeIds: Set<string>;     // ids des nœuds (annotations + images) sélectionnés
  hoveredNodeId: string | null;     // id du nœud sous le curseur (null si aucun)
  transDomainVisible: boolean;       // toggle utilisateur (par défaut true)
}

export interface ArrowVisibilityProbe {
  arrowId: string;
  sourceId?: string;
  targetId?: string;
  pinned?: boolean;
  isTransDomain?: boolean;          // calculé à partir des domaines source/cible (Phase 3)
}

export function shouldRenderArrow(arrow: ArrowVisibilityProbe, ctx: ArrowVisibilityContext): boolean {
  // (d) épinglée — toujours visible
  if (arrow.pinned) return true;

  // (c) trans-domaines — visible à tous les LOD si toggle actif
  if (arrow.isTransDomain && ctx.transDomainVisible) return true;

  // En macro, on s'arrête là (seuls c + d).
  if (ctx.lod === "macro") return false;

  // (a) sélection — meso + micro
  if (arrow.sourceId && ctx.selectedNodeIds.has(arrow.sourceId)) return true;
  if (arrow.targetId && ctx.selectedNodeIds.has(arrow.targetId)) return true;
  if (ctx.selectedNodeIds.has(arrow.arrowId)) return true;

  // En meso, on s'arrête là (a + c + d).
  if (ctx.lod === "meso") return false;

  // (b) hover — micro seulement
  if (ctx.hoveredNodeId) {
    if (arrow.sourceId === ctx.hoveredNodeId) return true;
    if (arrow.targetId === ctx.hoveredNodeId) return true;
  }

  return false;
}
