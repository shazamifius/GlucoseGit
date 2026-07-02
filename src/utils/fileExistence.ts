import { useState, useEffect } from "react";
import { toAbsolute } from "./pathResolver";

// Cache global des statuts d'existence pour éviter de solliciter le disque à chaque render
const existenceCache = new Map<string, boolean>();
const pendingChecks = new Map<string, Promise<boolean>>();

/**
 * Vérifie de façon asynchrone si un fichier existe à l'aide de Tauri plugin-fs.
 */
export async function checkFileExists(path: string): Promise<boolean> {
  const absPath = toAbsolute(path);
  const cleanPath = absPath.startsWith("file://") ? absPath.slice(7) : absPath;

  if (existenceCache.has(absPath)) {
    return existenceCache.get(absPath)!;
  }
  if (pendingChecks.has(absPath)) {
    return pendingChecks.get(absPath)!;
  }

  const promise = (async () => {
    try {
      const { exists } = await import("@tauri-apps/plugin-fs");
      const res = await exists(cleanPath);
      existenceCache.set(absPath, res);
      return res;
    } catch (e) {
      console.warn("[checkFileExists] Échec de la vérification :", cleanPath, e);
      // En cas d'erreur (ex: permission ou disque réseau déconnecté), on considère qu'il n'existe pas pour l'instant
      existenceCache.set(absPath, false);
      return false;
    } finally {
      pendingChecks.delete(absPath);
    }
  })();

  pendingChecks.set(absPath, promise);
  return promise;
}

/**
 * Hook React pour suivre l'existence d'un fichier lié sans bloquer le rendu.
 */
export function useFileExistence(path: string | undefined): "loading" | "exists" | "broken" {
  const [status, setStatus] = useState<"loading" | "exists" | "broken">("loading");

  useEffect(() => {
    if (!path) {
      setStatus("exists");
      return;
    }

    let active = true;
    
    // Si on a déjà le résultat en cache, on l'affiche immédiatement
    const absPath = toAbsolute(path);
    if (existenceCache.has(absPath)) {
      setStatus(existenceCache.get(absPath) ? "exists" : "broken");
      return;
    }

    setStatus("loading");

    checkFileExists(path).then((exists) => {
      if (active) {
        setStatus(exists ? "exists" : "broken");
      }
    });

    return () => {
      active = false;
    };
  }, [path]);

  return status;
}

/**
 * Invalide le cache d'existence d'un chemin (utile après une réassociation).
 */
export function invalidateExistenceCache(path: string): void {
  const absPath = toAbsolute(path);
  existenceCache.delete(absPath);
}
