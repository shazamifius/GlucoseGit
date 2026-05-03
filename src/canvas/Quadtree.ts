// Spatial hash for viewport culling — O(cells_in_viewport) query, O(n) build
export class SpatialHash {
  private grid = new Map<string, string[]>();
  private readonly cellSize: number;
  // CLEANUP P-07 — Set réutilisable pour queryIds() afin d'éviter
  // d'allouer un nouveau Set à chaque frame (60+/s en pan/zoom).
  private readonly _queryResult = new Set<string>();

  constructor(cellSize = 2000) {
    this.cellSize = cellSize;
  }

  build(images: readonly { id: string; x: number; y: number; width: number; height: number }[]) {
    this.grid.clear();
    for (const img of images) {
      // Sprites use anchor(0.5) so x/y is center
      const minCi = Math.floor((img.x - img.width  / 2) / this.cellSize);
      const maxCi = Math.floor((img.x + img.width  / 2) / this.cellSize);
      const minCj = Math.floor((img.y - img.height / 2) / this.cellSize);
      const maxCj = Math.floor((img.y + img.height / 2) / this.cellSize);
      for (let ci = minCi; ci <= maxCi; ci++) {
        for (let cj = minCj; cj <= maxCj; cj++) {
          const key = `${ci},${cj}`;
          let cell = this.grid.get(key);
          if (!cell) { cell = []; this.grid.set(key, cell); }
          cell.push(img.id);
        }
      }
    }
  }

  /**
   * Returns IDs whose cell overlaps the rect [x, y, x+w, y+h] + margin.
   * ATTENTION : retourne le **même Set** d'un appel à l'autre (réutilisation
   * mémoire pour éviter la pression GC). Si l'appelant doit conserver le
   * résultat, il doit en faire une copie.
   */
  queryIds(x: number, y: number, w: number, h: number, margin = 0): Set<string> {
    this._queryResult.clear();
    const minCi = Math.floor((x - margin) / this.cellSize);
    const maxCi = Math.floor((x + w + margin) / this.cellSize);
    const minCj = Math.floor((y - margin) / this.cellSize);
    const maxCj = Math.floor((y + h + margin) / this.cellSize);
    for (let ci = minCi; ci <= maxCi; ci++) {
      for (let cj = minCj; cj <= maxCj; cj++) {
        const ids = this.grid.get(`${ci},${cj}`);
        if (ids) for (const id of ids) this._queryResult.add(id);
      }
    }
    return this._queryResult;
  }

  clear() { this.grid.clear(); }
}
