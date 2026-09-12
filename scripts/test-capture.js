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
  console.log("Navigating to http://localhost:4173/...");
  await page.goto("http://localhost:4173/", { waitUntil: "networkidle" });
  await page.waitForTimeout(1500);

  console.log("Taking initial screenshot...");
  await page.screenshot({ path: path.join(screensDir, "01_canvas_initial.png") });
  console.log("Initial screenshot saved to docs/architecture/screens/01_canvas_initial.png");

  await browser.close();
}

main().catch(console.error);
