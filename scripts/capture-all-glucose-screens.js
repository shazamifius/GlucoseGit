import { chromium } from "playwright";
import fs from "fs";
import path from "path";

async function main() {
  const screensDir = path.resolve("docs/architecture/screens");
  if (!fs.existsSync(screensDir)) {
    fs.mkdirSync(screensDir, { recursive: true });
  }

  const browser = await chromium.launch({
    executablePath: "C:\\Users\\Administrator\\AppData\\Local\\ms-playwright\\chromium-1234\\chrome-win64\\chrome.exe",
    headless: true,
  });

  const context = await browser.newContext({
    viewport: { width: 1920, height: 1080 },
    deviceScaleFactor: 1,
  });

  const page = await context.newPage();
  console.log("1. Navigation vers Glucose (http://localhost:4173/)...");
  await page.goto("http://localhost:4173/", { waitUntil: "networkidle" });
  await page.waitForTimeout(1000);

  // Fermer le dialogue télémétrie si présent
  const refuseTelemetry = page.locator("button:has-text('Non merci')");
  if (await refuseTelemetry.count() > 0 && await refuseTelemetry.isVisible()) {
    console.log("Fermeture dialogue télémétrie...");
    await refuseTelemetry.click();
    await page.waitForTimeout(300);
  }

  // Fermer le bandeau WebModeBanner si présent
  const closeWebBanner = page.locator("button[title='Masquer']");
  if (await closeWebBanner.count() > 0 && await closeWebBanner.isVisible()) {
    console.log("Fermeture bandeau web...");
    await closeWebBanner.click();
    await page.waitForTimeout(300);
  }

  const toolbarBtn = (name) => page.locator(`.glc-toolbar-scroll button:has-text("${name}")`).first();

  console.log("2. Capture 01: Canvas de base complet propre...");
  await page.screenshot({ path: path.join(screensDir, "01_canvas_default.png") });

  console.log("3. Capture 02: Gros plan Toolbar & BoardTabs...");
  await page.screenshot({
    path: path.join(screensDir, "02_toolbar_tabs_crop.png"),
    clip: { x: 0, y: 0, width: 1920, height: 82 },
  });

  // 3. Ouvrir Ordonner (OrganizePanel)
  console.log("4. Capture 03: Panneau Ordonner...");
  const btnOrdonner = toolbarBtn("Ordonner");
  await btnOrdonner.click();
  await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(screensDir, "03_panel_organize_full.png") });

  // 4. Ouvrir Timer (Pomodoro) dans le dock à côté d'Ordonner
  console.log("5. Capture 04: Panneau Pomodoro Timer...");
  const btnTimer = toolbarBtn("Timer");
  await btnTimer.click();
  await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(screensDir, "04_dock_organize_pomodoro.png") });

  // 5. Ouvrir Storyboard
  console.log("6. Capture 05: Panneau Storyboard...");
  const btnStoryboard = toolbarBtn("Storyboard");
  await btnStoryboard.click();
  await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(screensDir, "05_dock_three_panels.png") });

  // Fermer les 3 panneaux du bas
  await btnOrdonner.click();
  await page.waitForTimeout(200);
  await btnTimer.click();
  await page.waitForTimeout(200);
  await btnStoryboard.click();
  await page.waitForTimeout(400);

  // 6. Ouvrir Domaines (Tiroir haut-gauche)
  console.log("7. Capture 06: Panneau Domaines...");
  const btnDomaines = toolbarBtn("Domaines");
  await btnDomaines.click();
  await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(screensDir, "06_drawer_domains.png") });
  await btnDomaines.click();
  await page.waitForTimeout(300);

  // 7. Ouvrir Presets (Tiroir haut-gauche)
  console.log("8. Capture 07: Panneau Presets...");
  const btnPreset = toolbarBtn("Preset");
  await btnPreset.click();
  await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(screensDir, "07_drawer_presets.png") });
  await btnPreset.click();
  await page.waitForTimeout(300);

  // 8. Ouvrir Plugins (Tiroir haut-gauche)
  console.log("9. Capture 08: Panneau Plugins...");
  const btnPlugins = toolbarBtn("Plugins");
  await btnPlugins.click();
  await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(screensDir, "08_drawer_plugins.png") });
  await btnPlugins.click();
  await page.waitForTimeout(300);

  // 9. Ouvrir Collaboration (Multijoueur)
  console.log("10. Capture 09: Panneau Collaborer...");
  const btnCollab = toolbarBtn("Collaborer");
  await btnCollab.click();
  await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(screensDir, "09_panel_multiplayer.png") });
  await btnCollab.click();
  await page.waitForTimeout(300);

  // 10. Recherche (Ctrl+F)
  console.log("11. Capture 10: Recherche Modale...");
  await page.keyboard.press("Control+f");
  await page.waitForTimeout(400);
  await page.keyboard.type("Concept");
  await page.waitForTimeout(300);
  await page.screenshot({ path: path.join(screensDir, "10_search_modal.png") });
  await page.keyboard.press("Escape");
  await page.waitForTimeout(300);

  // 11. Time Machine (Ctrl+H)
  console.log("12. Capture 11: Time Machine Drawer...");
  await page.keyboard.press("Control+h");
  await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(screensDir, "11_timemachine_drawer.png") });
  await page.keyboard.press("Escape");
  await page.waitForTimeout(300);

  // 12. Réglette Temporelle (Shift+R)
  console.log("13. Capture 12: Réglette Temporelle...");
  await page.keyboard.press("Shift+R");
  await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(screensDir, "12_temporal_ruler.png") });
  await page.keyboard.press("Shift+R");
  await page.waitForTimeout(300);

  // 13. Diagnostics HUD (Ctrl+Shift+D)
  console.log("14. Capture 13: Diagnostics HUD...");
  await page.keyboard.press("Control+Shift+D");
  await page.waitForTimeout(500);
  await page.screenshot({ path: path.join(screensDir, "13_diagnostics_hud.png") });
  await page.keyboard.press("Control+Shift+D");
  await page.waitForTimeout(300);

  // 14. Création d'éléments réels sur le canvas
  console.log("15. Capture 14: Création d'un sticky, opérateur ET, et texte sur le canvas...");
  // Sticky 1: outil 'N'
  await page.keyboard.press("n");
  await page.waitForTimeout(150);
  await page.mouse.click(500, 380);
  await page.waitForTimeout(300);

  // Transformer ce sticky en opérateur "ET" (AND) via Alt+1
  await page.keyboard.press("Alt+1");
  await page.waitForTimeout(300);

  // Sticky 2 standard: outil 'N'
  await page.keyboard.press("n");
  await page.waitForTimeout(150);
  await page.mouse.click(750, 360);
  await page.waitForTimeout(300);

  // Texte: outil 'T'
  await page.keyboard.press("t");
  await page.waitForTimeout(150);
  await page.mouse.click(1050, 360);
  await page.waitForTimeout(300);

  // Revenir en sélection: 'V'
  await page.keyboard.press("v");
  await page.waitForTimeout(200);

  // Cliquer sur le sticky 2 pour afficher le cadre de sélection et poignées
  await page.mouse.click(760, 380);
  await page.waitForTimeout(300);

  await page.screenshot({ path: path.join(screensDir, "14_canvas_with_live_elements.png") });

  console.log("=== TOUTES LES CAPTURES RÉELLES ONT ÉTÉ GÉNÉRÉES AVEC SUCCÈS DANS docs/architecture/screens/ ===");
  await browser.close();
}

main().catch(console.error);
