// **L'essai d'une page exportée, dans un vrai navigateur** (EXPORT-HTML-1, fiche 115) : par le
// protocole de débogage de Chromium, sans aucune bibliothèque — ouvre la page, capture,
// touche la flèche nommée, capture, pince à deux doigts, capture ; dit ce qui s'est passé.
//
//   node piloter.mjs <port> <page.html> <sortie> [flèche]   (voir essai_page.sh)
import { writeFileSync } from "node:fs";
const [port, page, sortie, fleche = "s-borne"] = process.argv.slice(2);
const cibles = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
const ws = new WebSocket(cibles.find((c) => c.type === "page").webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
let n = 0;
const attente = new Map();
ws.onmessage = (m) => {
  const d = JSON.parse(m.data);
  if (d.id && attente.has(d.id)) { attente.get(d.id)(d); attente.delete(d.id); }
};
const cdp = (method, params = {}) =>
  new Promise((r) => { const id = ++n; attente.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const capture = async (nom) => {
  const r = await cdp("Page.captureScreenshot", { format: "png" });
  writeFileSync(`${sortie}/${nom}.png`, Buffer.from(r.result.data, "base64"));
};
const evaluer = async (expr) => (await cdp("Runtime.evaluate", { expression: expr, returnByValue: true })).result.result.value;

await cdp("Emulation.setDeviceMetricsOverride", { width: 1180, height: 820, deviceScaleFactor: 1, mobile: false });
await cdp("Page.enable");
await cdp("Page.navigate", { url: `file://${page}` });
await pause(1500);
await capture("1-ouverte");
const avant = await evaluer("document.getElementById('scene').style.transform");
// Le milieu du tracé de la flèche, à l'écran.
const p = await evaluer(`(function(){var g=document.querySelector('.fleche[data-id="${fleche}"] path');
  var b=g.getBoundingClientRect(); return {x:b.left+b.width/2, y:b.top+b.height/2, l:b.width, h:b.height};})()`);
console.log("flèche", JSON.stringify(p));
// Le toucher du protocole : l'élément sous le point est-il bien la flèche ?
console.log("sous le point", await evaluer(`(function(){var e=document.elementFromPoint(${p.x},${p.y});
  return e && e.closest('.fleche') ? e.closest('.fleche').getAttribute('data-id') : String(e && e.tagName);})()`));
for (const type of ["mousePressed", "mouseReleased"]) {
  await cdp("Input.dispatchMouseEvent", { type, x: p.x, y: p.y, button: "left", clickCount: 1 });
}
await pause(400);
console.log("panneau", await evaluer("document.getElementById('panneau').style.display"),
  JSON.stringify(await evaluer("document.getElementById('titre').textContent + ' | ' + document.getElementById('corps').textContent")));
await capture("2-fleche");
// Deux doigts qui s'écartent : un pincement.
const c = { x: 590, y: 420 };
const doigts = (e) => [{ x: c.x - e, y: c.y, id: 1 }, { x: c.x + e, y: c.y, id: 2 }];
await cdp("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: doigts(60) });
for (let e = 70; e <= 180; e += 10) {
  await cdp("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: doigts(e) });
  await pause(16);
}
await cdp("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
await pause(400);
const apres = await evaluer("document.getElementById('scene').style.transform");
console.log("avant", avant, "apres", apres, "panneau", await evaluer("document.getElementById('panneau').style.display"));
await capture("3-pince");
ws.close();
