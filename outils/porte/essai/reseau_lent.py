"""**Un réseau lent** entre un Glucose et le relais (fiche 117) : ce qui descend du relais vers
Glucose arrive `delai` secondes plus tard, dans l'ordre ; ce qui monte passe tout de suite.

C'est ainsi qu'on provoque un vrai conflit dans l'essai du canevas partagé : un participant derrière
ce réseau agit sans avoir encore vu le geste d'un autre — ce que vit quelqu'un sur une connexion
lente. Le chemin et l'autorisation passent tels quels.

    nix shell --impure --expr '(import (builtins.getFlake "nixpkgs") {}).python3.withPackages
      (p: [p.websockets])' -c python3 outils/porte/essai/reseau_lent.py ws://127.0.0.1:8787 8788 3
"""

import asyncio
import sys

from websockets.asyncio.client import connect
from websockets.asyncio.server import serve

HAUT = sys.argv[1].rstrip("/")
PORT = int(sys.argv[2])
DELAI = float(sys.argv[3])


async def relayer(bas):
    chemin = bas.request.path
    entetes = [(k, v) for k, v in bas.request.headers.raw_items() if k.lower() == "authorization"]
    async with connect(HAUT + chemin, additional_headers=entetes, max_size=None) as haut:
        boucle = asyncio.get_running_loop()
        file = asyncio.Queue()

        async def monter():
            async for m in bas:
                await haut.send(m)

        async def lire():
            async for m in haut:
                file.put_nowait((boucle.time() + DELAI, m))

        async def descendre():
            while True:
                quand, m = await file.get()
                attente = quand - boucle.time()
                if attente > 0:
                    await asyncio.sleep(attente)
                await bas.send(m)

        taches = [asyncio.create_task(t()) for t in (monter, lire, descendre)]
        await asyncio.wait(taches, return_when=asyncio.FIRST_COMPLETED)
        for t in taches:
            t.cancel()


async def principal():
    async with serve(relayer, "127.0.0.1", PORT, max_size=None):
        print(f"réseau lent : 127.0.0.1:{PORT} → {HAUT}, {DELAI} s vers le bas", flush=True)
        await asyncio.Future()


asyncio.run(principal())
