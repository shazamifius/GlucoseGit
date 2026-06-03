// ────────────────────────────────────────────────────────────────────────────
// Autosave — sauvegarde disque automatique débouncée
// ────────────────────────────────────────────────────────────────────────────
//
// Filet anti perte de données : à chaque changement du document (`_doc`), on
// réécrit le fichier `.glucose` courant après un court délai d'inactivité. Actif
// en solo ET en collab (en plus de la persistance serveur/IndexedDB).
//
// Ne déclenche PAS de boîte de dialogue : tant que le projet n'a jamais été
// enregistré (aucun chemin connu), l'autosave reste silencieusement en attente —
// le premier Ctrl+S fixe le chemin, puis l'autosave prend le relais.

import { useEffect, useRef } from "react";
import { useGlucoseStore } from "../store";
import { saveProject } from "./project";

export function useAutosave(
  pathRef: React.MutableRefObject<string | null>,
  delayMs = 1500,
): void {
  const timer = useRef<number | null>(null);
  const lastDoc = useRef(useGlucoseStore.getState()._doc);

  useEffect(() => {
    const unsub = useGlucoseStore.subscribe((state) => {
      if (state._doc === lastDoc.current) return; // pas de changement de doc
      lastDoc.current = state._doc;
      if (!pathRef.current) return; // jamais enregistré → pas d'autosave silencieuse
      if (timer.current !== null) clearTimeout(timer.current);
      timer.current = window.setTimeout(() => {
        timer.current = null;
        const path = pathRef.current;
        if (!path) return;
        saveProject(useGlucoseStore.getState()._doc, path).catch((e) =>
          console.warn("[autosave] échec:", e),
        );
      }, delayMs);
    });
    return () => {
      unsub();
      if (timer.current !== null) clearTimeout(timer.current);
    };
  }, [pathRef, delayMs]);
}
