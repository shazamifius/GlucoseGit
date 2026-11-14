// **La page d'accord** (fiche 116) : celle qu'une IA publique ouvre dans le navigateur quand on
// ajoute l'adresse d'un Glucose. Elle ne demande rien — ni mot de passe, ni compte : elle montre
// un code, et dit d'aller dans Glucose, où la même question attend avec le même code. Le clic
// dans Glucose est la preuve : seule la personne devant ce Glucose peut le donner.
//
// Elle est **fixe**, servie par le relais lui-même : rien de ce qu'un Glucose répond ne s'y écrit
// autrement que par `textContent`, et elle ne quitte la page que vers l'adresse de retour que
// l'IA a donnée en l'ouvrant. Le langage de `style.md` : une feuille noire, du texte, un filet.

export const PAGE = `<!doctype html>
<html lang="fr">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Glucose — ta porte</title>
<style>
  :root { color-scheme: dark; }
  body { margin: 0; min-height: 100vh; display: grid; place-items: center; background: #0d0d0d;
         color: #e6e6e6; font: 16px/1.5 system-ui, sans-serif; }
  main { width: min(420px, calc(100vw - 32px)); border: 1px solid #2a2a2a; background: #111;
         padding: 24px; box-sizing: border-box; }
  h1 { margin: 0 0 16px; font: 600 11px/1 ui-monospace, monospace; letter-spacing: 1.2px;
       text-transform: uppercase; color: #7c7c7c; }
  #code { font: 600 44px/1.2 ui-monospace, monospace; letter-spacing: 12px; margin: 20px 0;
          text-align: center; }
  #qui { margin: 0; }
  #dire { margin: 0; color: #7c7c7c; font-size: 14px; }
  .fini #code { color: #7c7c7c; }
</style>
<main>
  <h1>Glucose · la porte</h1>
  <p id="qui">Une IA demande l'accès à ton Glucose…</p>
  <p id="code" aria-live="polite">· · ·</p>
  <p id="dire">Va dans ton Glucose : la même question t'y attend, avec ce code.</p>
</main>
<script src="/porte.js"></script>
</html>
`;

export const SCRIPT = `"use strict";
(async () => {
  const base = location.pathname.replace(/\\/authorize$/, "");
  const params = Object.fromEntries(new URLSearchParams(location.search));
  const qui = document.getElementById("qui");
  const code = document.getElementById("code");
  const dire = document.getElementById("dire");
  const fin = (texte) => { document.body.classList.add("fini"); dire.textContent = texte; };
  const appeler = async (chemin, corps) => {
    const r = await fetch(base + chemin, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(corps),
    });
    return [r.status, await r.json().catch(() => ({}))];
  };
  // On ne quitte la page que vers l'adresse de retour que l'IA a donnée.
  const partir = (adresse) => {
    if (typeof adresse === "string" && params.redirect_uri && adresse.startsWith(params.redirect_uri)) {
      location.replace(adresse);
      return true;
    }
    return false;
  };
  let demande;
  try {
    const [statut, d] = await appeler("/authorize/demande", params);
    if (statut !== 200) {
      code.textContent = "—";
      fin(d.error_description || "Ton Glucose n'a pas pu recevoir cette demande.");
      return;
    }
    demande = d;
  } catch {
    fin("Le relais ne répond pas.");
    return;
  }
  qui.textContent = "« " + demande.client.nom + " » (" + demande.client.hote +
    ") demande l'accès au document ouvert dans ton Glucose.";
  code.textContent = demande.code.split("").join(" ");
  for (;;) {
    await new Promise((r) => setTimeout(r, 1000));
    let e;
    try {
      const [, reponse] = await appeler("/authorize/attente", { demande: demande.demande });
      e = reponse;
    } catch {
      continue;
    }
    if (e.etat === "attente") continue;
    if (e.etat === "accorde" && partir(e.redirection)) { fin("Accordé. Tu peux revenir à ton IA."); return; }
    if (e.etat === "refuse") { partir(e.redirection); fin("Refusé dans Glucose."); return; }
    fin(e.error_description || "Cette demande n'existe plus : recommence depuis ton IA.");
    return;
  }
})();
`;
