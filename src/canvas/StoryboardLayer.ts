import { Container, Graphics, Text, Sprite, Texture, Assets } from "pixi.js";
import { StoryboardPanel, StoryboardSettings } from "../types";

export class StoryboardLayer {
  private container: Container;
  private panels = new Map<string, Container>();
  private thumbs = new Map<string, string>(); // panelId -> imageId for loaded textures

  constructor(parent: Container) {
    this.container = new Container();
    parent.addChild(this.container);
  }

  async sync(
    panels: StoryboardPanel[],
    settings: StoryboardSettings | undefined,
    selectedIds: string[],
    getImageSrc: (imageId: string) => string | undefined,
  ) {
    if (!settings) {
      this.clear();
      return;
    }

    const current = new Set(panels.map((p) => p.id));
    this.panels.forEach((obj, id) => {
      if (!current.has(id)) {
        this.container.removeChild(obj);
        obj.destroy({ children: true });
        this.panels.delete(id);
        this.thumbs.delete(id);
      }
    });

    for (const panel of panels) {
      const sel = selectedIds.includes(panel.id);
      const existing = this.panels.get(panel.id);
      if (existing) {
        existing.x = panel.x;
        existing.y = panel.y;
        this.updatePanel(existing, panel, settings, sel, getImageSrc);
      } else {
        const obj = new Container();
        obj.x = panel.x;
        obj.y = panel.y;
        this.buildPanel(obj, panel, settings, sel, getImageSrc);
        this.container.addChild(obj);
        this.panels.set(panel.id, obj);
      }
    }
  }

  private updatePanel(
    c: Container,
    panel: StoryboardPanel,
    settings: StoryboardSettings,
    sel: boolean,
    getImageSrc: (id: string) => string | undefined,
  ) {
    c.removeChildren().forEach((ch) => (ch as any).destroy?.({ children: true }));
    this.buildPanel(c, panel, settings, sel, getImageSrc);
  }

  private buildPanel(
    c: Container,
    panel: StoryboardPanel,
    _settings: StoryboardSettings,
    sel: boolean,
    getImageSrc: (id: string) => string | undefined,
  ) {
    const w = panel.width;
    const h = panel.height;
    const descH = 40;

    // Frame background
    const g = new Graphics();
    g.rect(0, 0, w, h);
    g.fill({ color: 0x1a1a1a, alpha: 1 });
    if (sel) {
      g.rect(-2, -2, w + 4, h + 4);
      g.stroke({ color: 0xffffff, width: 1.5, alpha: 0.8 });
    } else {
      g.rect(0, 0, w, h);
      g.stroke({ color: 0x333333, width: 1, alpha: 0.8 });
    }
    c.addChild(g);

    // Panel number badge
    const numBg = new Graphics();
    numBg.rect(0, 0, 24, 18);
    numBg.fill({ color: 0x222222, alpha: 0.9 });
    numBg.x = 4;
    numBg.y = 4;
    c.addChild(numBg);

    const numTxt = new Text({
      text: String(panel.order + 1),
      style: { fontSize: 10, fill: 0x888888, fontWeight: "bold" },
    });
    numTxt.x = 4 + 12;
    numTxt.y = 4 + 9;
    numTxt.anchor.set(0.5);
    c.addChild(numTxt);

    // Thumbnail image (if assigned)
    if (panel.imageId) {
      const src = getImageSrc(panel.imageId);
      if (src) {
        Assets.load(src).then((tex: Texture) => {
          if (!c.parent) return;
          const thumb = new Sprite(tex);
          thumb.x = 0;
          thumb.y = 0;
          thumb.width = w;
          thumb.height = h;
          thumb.alpha = 0.7;
          // Insert behind number badge (index 1 = after background)
          c.addChildAt(thumb, 1);
        }).catch(() => {});
      }
    }

    // Description area below frame
    const descBg = new Graphics();
    descBg.rect(0, h, w, descH);
    descBg.fill({ color: 0x111111, alpha: 1 });
    c.addChild(descBg);

    const descTxt = new Text({
      text: panel.description || "",
      style: { fontSize: 10, fill: 0x777777, wordWrap: true, wordWrapWidth: w - 8 },
    });
    descTxt.x = 4;
    descTxt.y = h + 4;
    c.addChild(descTxt);
  }

  clear() {
    this.container.removeChildren().forEach((ch) => (ch as any).destroy?.({ children: true }));
    this.panels.clear();
    this.thumbs.clear();
  }

  destroy() {
    this.container.destroy({ children: true });
  }
}
