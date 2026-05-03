// CLEANUP R-01 — Schéma de validation Zod pour les fichiers .glucose
//
// Garantit qu'un projet chargé respecte la structure attendue. Tout champ
// inconnu est ignoré (passthrough) ; tout champ obligatoire manquant ou de
// mauvais type fait échouer la validation et déclenche un fallback dans le
// store (toast + chargement annulé).

import { z } from "zod";

const PointSchema = z.object({ x: z.number().finite(), y: z.number().finite() });

const DomainAssignmentSchema = z.object({
  domainId: z.string(),
  weight: z.number().min(0).max(1),
});

const DomainSchema = z.object({
  id: z.string(),
  name: z.string(),
  color: z.string(),
  icon: z.string(),
  createdAt: z.number(),
});

const BoardImageSchema = z.object({
  id: z.string(),
  src: z.string(),
  x: z.number().finite(),
  y: z.number().finite(),
  width: z.number().finite().positive(),
  height: z.number().finite().positive(),
  rotation: z.number().finite(),
  locked: z.boolean(),
  tags: z.array(z.string()),
  slotId: z.string().optional(),
  sourceUrl: z.string().optional(),
  originalWidth: z.number().finite().positive(),
  originalHeight: z.number().finite().positive(),
  isVideo: z.boolean().optional(),
  domains: z.array(DomainAssignmentSchema).optional(),
  mirrorOf: z.string().optional(),
});

const AnnotationTypeSchema = z.enum(["text", "sticky", "arrow", "membrane"]);

const ArrowPredicateSchema = z.enum([
  "est_precurseur",
  "contredit",
  "herite_de",
  "inspire",
  "depend_de",
  "illustre",
]);

const AnnotationSchema = z.object({
  id: z.string(),
  type: AnnotationTypeSchema,
  x: z.number().finite(),
  y: z.number().finite(),
  text: z.string().optional(),
  fontSize: z.number().optional(),
  color: z.string().optional(),
  bgColor: z.string().optional(),
  width: z.number().finite().optional(),
  height: z.number().finite().optional(),
  x2: z.number().finite().optional(),
  y2: z.number().finite().optional(),
  arrowType: z.enum(["straight", "curved"]).optional(),
  arrowBidirectional: z.boolean().optional(),
  predicate: ArrowPredicateSchema.optional(),
  strokeWidth: z.number().optional(),
  waypoints: z.array(PointSchema).optional(),
  sourceId: z.string().optional(),
  targetId: z.string().optional(),
  sourceBlockId: z.string().optional(),
  targetBlockId: z.string().optional(),
  sourceTextSel: z.string().optional(),
  targetTextSel: z.string().optional(),
  sourceFile: z.string().optional(),
  cursorPos: z.number().optional(),
  pinned: z.boolean().optional(),
  domains: z.array(DomainAssignmentSchema).optional(),
  mirrorOf: z.string().optional(),
  longText: z.string().optional(),
  targetBoardId: z.string().optional(),
  operator: z.enum(["AND", "OR", "BUT", "BECAUSE"]).optional(),
});

const StoryboardPanelSchema = z.object({
  id: z.string(),
  order: z.number(),
  description: z.string(),
  imageId: z.string().optional(),
  x: z.number().finite(),
  y: z.number().finite(),
  width: z.number().finite().positive(),
  height: z.number().finite().positive(),
});

const StoryboardSettingsSchema = z.object({
  aspectRatio: z.enum(["16:9", "4:3", "2.35:1", "1:1", "9:16"]),
  panelWidth: z.number().positive(),
  cols: z.number().int().positive(),
  gap: z.number(),
});

const PresetSlotSchema = z.object({
  id: z.string(),
  name: z.string(),
  color: z.string(),
  description: z.string(),
  order: z.number(),
});

const PresetSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string(),
  slots: z.array(PresetSlotSchema),
  isBuiltin: z.boolean(),
  createdAt: z.number(),
});

const BoardZoneSchema = z.object({
  slotId: z.string(),
  x: z.number().finite(),
  y: z.number().finite(),
  width: z.number().finite().positive(),
  height: z.number().finite().positive(),
});

const CanvasFolderSchema = z.object({
  id: z.string(),
  name: z.string(),
  color: z.string(),
  x: z.number().finite(),
  y: z.number().finite(),
  width: z.number().finite().positive(),
  height: z.number().finite().positive(),
  childBoardId: z.string(),
  mirrorOf: z.string().optional(),
});

const ViewportSchema = z.object({
  x: z.number().finite(),
  y: z.number().finite(),
  scale: z.number().finite().positive(),
});

const BoardSchema = z.object({
  id: z.string(),
  name: z.string(),
  images: z.array(BoardImageSchema),
  annotations: z.array(AnnotationSchema),
  panels: z.array(StoryboardPanelSchema),
  storyboard: StoryboardSettingsSchema.optional(),
  viewport: ViewportSchema,
  presetId: z.string().optional(),
  zones: z.array(BoardZoneSchema),
  folders: z.array(CanvasFolderSchema).default([]),
  bookmarks: z.record(z.string(), ViewportSchema).optional(),
  createdAt: z.number(),
  updatedAt: z.number(),
});

export const ProjectSchema = z.object({
  version: z.string(),
  name: z.string(),
  boards: z.array(BoardSchema).min(1),
  activeBoardId: z.string(),
  presets: z.array(PresetSchema),
  domains: z.array(DomainSchema).optional().default([]),
  createdAt: z.number(),
  updatedAt: z.number(),
});

/**
 * Valide et normalise un objet supposé être un Project.
 * - Si OK : renvoie le projet typé prêt à charger.
 * - Si KO : renvoie une erreur descriptive.
 */
export function parseProjectFile(raw: unknown): { ok: true; project: import("../types").Project } | { ok: false; error: string } {
  const result = ProjectSchema.safeParse(raw);
  if (!result.success) {
    const first = result.error.issues[0];
    const path = first.path.join(".");
    return { ok: false, error: `${path || "root"}: ${first.message}` };
  }
  // Vérification croisée : activeBoardId doit correspondre à un board existant
  if (!result.data.boards.some((b) => b.id === result.data.activeBoardId)) {
    return { ok: false, error: "activeBoardId ne correspond à aucun board" };
  }
  return { ok: true, project: result.data as unknown as import("../types").Project };
}
