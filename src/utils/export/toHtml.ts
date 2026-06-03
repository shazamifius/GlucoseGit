// Export HTML autonome — un SEUL fichier .html que n'importe qui ouvre dans son
// navigateur (zéro installation) : pan/zoom à la souris, texte sélectionnable,
// clic sur une flèche → sa description. Images intégrées en dataURL (via le SVG).
//
// Visuellement identique aux exports SVG/PNG : on réutilise sceneToSvg comme
// rendu, on l'enrobe d'un viewport interactif + un panneau de description.
import { ExportScene } from "./scene";
import { sceneToSvg } from "./toSvg";

function escAttr(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

export function sceneToHtml(scene: ExportScene): string {
  const svg = sceneToSvg(scene, { transparent: false });

  // Métadonnées des flèches pour le panneau de description (clic).
  const arrowMeta = scene.arrows
    .filter((a) => a.label || a.longText || a.predicate)
    .map((a) => ({ id: a.id, label: a.label ?? "", longText: a.longText ?? "", predicate: a.predicate ?? "" }));
  const metaJson = JSON.stringify(arrowMeta).replace(/</g, "\\u003c");

  const title = `${scene.projectName} — ${scene.boardName}`;

  return `<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>${escAttr(title)}</title>
<style>
  * { box-sizing: border-box; }
  html, body { margin: 0; height: 100%; background: ${scene.background}; color: #e8e8f0;
    font-family: system-ui, -apple-system, "Segoe UI", sans-serif; overflow: hidden; }
  #bar { position: fixed; top: 0; left: 0; right: 0; height: 44px; z-index: 10;
    display: flex; align-items: center; gap: 14px; padding: 0 16px;
    background: rgba(15,15,20,0.82); backdrop-filter: blur(8px);
    border-bottom: 1px solid rgba(255,255,255,0.08); }
  #bar b { font-size: 14px; font-weight: 650; }
  #bar .hint { font-size: 12px; color: #8a8a99; margin-left: auto; }
  #bar button { background: #23232b; color: #cfcfda; border: 1px solid #3a3a46;
    border-radius: 7px; padding: 5px 10px; font-size: 12px; cursor: pointer; }
  #bar button:hover { background: #2d2d38; }
  #viewport { position: fixed; inset: 44px 0 0 0; overflow: hidden; cursor: grab; }
  #viewport.grabbing { cursor: grabbing; }
  #stage { position: absolute; top: 0; left: 0; transform-origin: 0 0; will-change: transform; }
  #stage svg { display: block; }
  .arrow { cursor: pointer; }
  #panel { position: fixed; right: 16px; bottom: 16px; width: min(380px, 90vw);
    max-height: 50vh; overflow: auto; z-index: 11; display: none;
    background: rgba(20,20,28,0.96); border: 1px solid #34344a; border-radius: 12px;
    padding: 14px 16px; box-shadow: 0 12px 40px rgba(0,0,0,0.55); }
  #panel h4 { margin: 0 0 8px; font-size: 13px; color: #9aa0c0; letter-spacing: .4px;
    text-transform: uppercase; }
  #panel .body { font-size: 14px; line-height: 1.55; color: #dfe2f0; }
  #panel .body h1, #panel .body h2, #panel .body h3 { font-size: 15px; margin: 10px 0 4px; }
  #panel .body strong { color: #fff; }
  #panel .close { position: absolute; top: 8px; right: 10px; cursor: pointer;
    color: #777; font-size: 18px; line-height: 1; }
</style>
</head>
<body>
  <div id="bar">
    <b>${escAttr(title)}</b>
    <button id="reset">Recadrer</button>
    <span class="hint">Molette = zoom · glisser = déplacer · clic sur une flèche = description</span>
  </div>
  <div id="viewport"><div id="stage">${svg}</div></div>
  <div id="panel"><span class="close" id="panelClose">×</span><h4 id="panelTitle"></h4><div class="body" id="panelBody"></div></div>

<script>
(function () {
  var ARROWS = ${metaJson};
  var meta = {}; ARROWS.forEach(function (a) { meta[a.id] = a; });
  var PREDS = { est_precurseur: "est précurseur de", contredit: "contredit", herite_de: "hérite de", inspire: "inspire", depend_de: "dépend de", illustre: "illustre" };

  var vp = document.getElementById("viewport");
  var stage = document.getElementById("stage");
  var svg = stage.querySelector("svg");
  var W = parseFloat(svg.getAttribute("width")) || 1000;
  var H = parseFloat(svg.getAttribute("height")) || 800;

  var state = { x: 0, y: 0, s: 1 };
  function apply() { stage.style.transform = "translate(" + state.x + "px," + state.y + "px) scale(" + state.s + ")"; }

  function fit() {
    var rect = vp.getBoundingClientRect();
    var s = Math.min(rect.width / W, rect.height / H) * 0.94;
    state.s = s > 0 ? s : 1;
    state.x = (rect.width - W * state.s) / 2;
    state.y = (rect.height - H * state.s) / 2;
    apply();
  }

  // Zoom molette centré sur le curseur
  vp.addEventListener("wheel", function (e) {
    e.preventDefault();
    var rect = vp.getBoundingClientRect();
    var mx = e.clientX - rect.left, my = e.clientY - rect.top;
    var factor = Math.exp(-e.deltaY * 0.0015);
    var ns = Math.max(0.02, Math.min(40, state.s * factor));
    var k = ns / state.s;
    state.x = mx - (mx - state.x) * k;
    state.y = my - (my - state.y) * k;
    state.s = ns;
    apply();
  }, { passive: false });

  // Pan par glisser
  var drag = null;
  vp.addEventListener("pointerdown", function (e) {
    drag = { px: e.clientX, py: e.clientY, ox: state.x, oy: state.y, moved: false };
    vp.classList.add("grabbing");
    vp.setPointerCapture(e.pointerId);
  });
  vp.addEventListener("pointermove", function (e) {
    if (!drag) return;
    var dx = e.clientX - drag.px, dy = e.clientY - drag.py;
    if (Math.abs(dx) + Math.abs(dy) > 3) drag.moved = true;
    state.x = drag.ox + dx; state.y = drag.oy + dy; apply();
  });
  vp.addEventListener("pointerup", function (e) {
    var wasDrag = drag && drag.moved;
    drag = null; vp.classList.remove("grabbing");
    if (wasDrag) return;
    // clic simple : test flèche
    var g = e.target.closest ? e.target.closest(".arrow") : null;
    if (g) showPanel(g.getAttribute("data-id"));
    else hidePanel();
  });

  // Mini-formateur Markdown (gras/italique/titres/puces) pour les descriptions
  function mdToHtml(src) {
    var escd = src.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    var lines = escd.split(/\\n/);
    var out = [], inList = false;
    for (var i = 0; i < lines.length; i++) {
      var l = lines[i].trim();
      if (!l) { if (inList) { out.push("</ul>"); inList = false; } continue; }
      var h = /^(#{1,3})\\s+(.*)$/.exec(l);
      if (h) { if (inList) { out.push("</ul>"); inList = false; } out.push("<h3>" + inline(h[2]) + "</h3>"); continue; }
      if (/^[-*]\\s+/.test(l)) { if (!inList) { out.push("<ul>"); inList = true; } out.push("<li>" + inline(l.replace(/^[-*]\\s+/, "")) + "</li>"); continue; }
      if (inList) { out.push("</ul>"); inList = false; }
      out.push("<p>" + inline(l) + "</p>");
    }
    if (inList) out.push("</ul>");
    return out.join("");
  }
  function inline(s) {
    return s.replace(/\\*\\*([^*]+)\\*\\*/g, "<strong>$1</strong>")
            .replace(/\\*([^*]+)\\*/g, "<em>$1</em>")
            .replace(/\`([^\`]+)\`/g, "<code>$1</code>");
  }

  var panel = document.getElementById("panel");
  function showPanel(id) {
    var m = meta[id];
    if (!m) { hidePanel(); return; }
    var t = m.predicate ? (PREDS[m.predicate] || m.predicate) : (m.label || "Lien");
    document.getElementById("panelTitle").textContent = t;
    var body = m.label && m.predicate ? ("**" + m.label + "**\\n\\n" + m.longText) : m.longText;
    document.getElementById("panelBody").innerHTML = body ? mdToHtml(body) : "<p style='color:#888'>(pas de description)</p>";
    panel.style.display = "block";
  }
  function hidePanel() { panel.style.display = "none"; }
  document.getElementById("panelClose").addEventListener("click", hidePanel);
  document.getElementById("reset").addEventListener("click", fit);
  window.addEventListener("resize", function () { /* garde l'état, juste re-clamp possible */ });

  fit();
})();
</script>
</body>
</html>`;
}
