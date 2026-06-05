import { Assets } from "pixi.js";

async function run() {
  await Assets.init();
  const bytes = new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68, 65, 84, 8, 215, 99, 248, 255, 255, 63, 0, 5, 254, 2, 254, 220, 204, 89, 226, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130]);
  const blob = new Blob([bytes], { type: "image/png" });
  const url = URL.createObjectURL(blob);
  
  console.log("Blob URL:", url);
  try {
    const tex = await Assets.load(url);
    console.log("Loaded directly!", tex.width);
  } catch (e) {
    console.error("Direct load failed:", e.message);
    try {
      const tex2 = await Assets.load({ src: url, format: ".png" });
      console.log("Loaded with format=.png!", tex2.width);
    } catch (e2) {
      console.error("Format load failed:", e2.message);
      try {
        const tex3 = await Assets.load({ src: url, ext: ".png" });
        console.log("Loaded with ext=.png!", tex3.width);
      } catch (e3) {
        console.error("Ext load failed:", e3.message);
      }
    }
  }
}
run();
