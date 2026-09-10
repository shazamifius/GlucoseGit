//! Rasterizer 2D logiciel natif en pur Rust std (0 dépendance).
//!
//! Gère un framebuffer `Vec<u32>` au format `0x00RRGGBB` attendu par softbuffer.
//! Primitives graphiques intégrées :
//! - Remplissage de fond
//! - Grille de points infinie
//! - Rectangles pleins avec alpha blending
//! - Rectangles de contour et pointillés
//! - Tracé de segments et flèches vectorielles orientées
//! - Redimensionnement et blit d'images
//! - Texte bitmap 8x8 intégré

// Table de glyphes bitmap 8x8
include!("font_data.rs");

pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let w = width.max(1);
        let h = height.max(1);
        Self {
            width: w,
            height: h,
            pixels: vec![0; w * h],
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        let w = width.max(1);
        let h = height.max(1);
        if self.width != w || self.height != h {
            self.width = w;
            self.height = h;
            self.pixels.resize(w * h, 0);
        }
    }

    #[inline(always)]
    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    #[inline(always)]
    pub fn set_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height {
            let idx = (y as usize) * self.width + (x as usize);
            self.pixels[idx] = color;
        }
    }

    #[inline(always)]
    pub fn blend_pixel(&mut self, x: i32, y: i32, color: u32, alpha: u8) {
        if alpha == 0 {
            return;
        }
        if alpha == 255 {
            self.set_pixel(x, y, color);
            return;
        }
        if x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height {
            let idx = (y as usize) * self.width + (x as usize);
            let dst = self.pixels[idx];

            let sr = ((color >> 16) & 0xFF) as u32;
            let sg = ((color >> 8) & 0xFF) as u32;
            let sb = (color & 0xFF) as u32;

            let dr = ((dst >> 16) & 0xFF) as u32;
            let dg = ((dst >> 8) & 0xFF) as u32;
            let db = (dst & 0xFF) as u32;

            let a = alpha as u32;
            let inv_a = 255 - a;

            let r = (sr * a + dr * inv_a) / 255;
            let g = (sg * a + dg * inv_a) / 255;
            let b = (sb * a + db * inv_a) / 255;

            self.pixels[idx] = (r << 16) | (g << 8) | b;
        }
    }

    /// Remplissage d'un rectangle plein avec opacité alpha
    pub fn fill_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32, alpha: u8) {
        let left = x0.min(x1).clamp(0, self.width as i32);
        let right = x0.max(x1).clamp(0, self.width as i32);
        let top = y0.min(y1).clamp(0, self.height as i32);
        let bottom = y0.max(y1).clamp(0, self.height as i32);

        if left >= right || top >= bottom {
            return;
        }

        if alpha == 255 {
            for y in top..bottom {
                let row_start = (y as usize) * self.width + (left as usize);
                let row_end = (y as usize) * self.width + (right as usize);
                self.pixels[row_start..row_end].fill(color);
            }
        } else {
            for y in top..bottom {
                for x in left..right {
                    self.blend_pixel(x, y, color, alpha);
                }
            }
        }
    }

    /// Tracé du contour d'un rectangle
    pub fn stroke_rect(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: u32,
        stroke_width: i32,
        alpha: u8,
    ) {
        let left = x0.min(x1);
        let right = x0.max(x1);
        let top = y0.min(y1);
        let bottom = y0.max(y1);
        let sw = stroke_width.max(1);

        // Haut
        self.fill_rect(left, top, right, top + sw, color, alpha);
        // Bas
        self.fill_rect(left, bottom - sw, right, bottom, color, alpha);
        // Gauche
        self.fill_rect(left, top, left + sw, bottom, color, alpha);
        // Droite
        self.fill_rect(right - sw, top, right, bottom, color, alpha);
    }

    /// Tracé d'un contour en pointillés
    pub fn stroke_dashed_rect(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: u32,
        dash_len: i32,
        alpha: u8,
    ) {
        let left = x0.min(x1);
        let right = x0.max(x1);
        let top = y0.min(y1);
        let bottom = y0.max(y1);
        let dash = dash_len.max(2);

        // Ligne horizontale haute
        let mut draw = true;
        let mut x = left;
        while x < right {
            let next_x = (x + dash).min(right);
            if draw {
                self.fill_rect(x, top, next_x, top + 1, color, alpha);
            }
            draw = !draw;
            x = next_x;
        }

        // Ligne horizontale basse
        draw = true;
        x = left;
        while x < right {
            let next_x = (x + dash).min(right);
            if draw {
                self.fill_rect(x, bottom - 1, next_x, bottom, color, alpha);
            }
            draw = !draw;
            x = next_x;
        }

        // Ligne verticale gauche
        draw = true;
        let mut y = top;
        while y < bottom {
            let next_y = (y + dash).min(bottom);
            if draw {
                self.fill_rect(left, y, left + 1, next_y, color, alpha);
            }
            draw = !draw;
            y = next_y;
        }

        // Ligne verticale droite
        draw = true;
        y = top;
        while y < bottom {
            let next_y = (y + dash).min(bottom);
            if draw {
                self.fill_rect(right - 1, y, right, next_y, color, alpha);
            }
            draw = !draw;
            y = next_y;
        }
    }

    /// Tracé de segment de droite (Bresenham avec épaisseur)
    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: u32, width: i32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let half_w = (width - 1) / 2;

        loop {
            if width <= 1 {
                self.set_pixel(x0, y0, color);
            } else {
                self.fill_rect(
                    x0 - half_w,
                    y0 - half_w,
                    x0 + half_w + 1,
                    y0 + half_w + 1,
                    color,
                    255,
                );
            }

            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Tracé d'une flèche avec tête triangulaire
    pub fn draw_arrow(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: u32,
        width: i32,
        head_size: f32,
    ) {
        self.draw_line(x0, y0, x1, y1, color, width);

        // Tête de flèche
        let angle = ((y1 - y0) as f32).atan2((x1 - x0) as f32);
        let arrow_angle = 0.45; // ~26 degrés
        let len = head_size.max(8.0);

        let left_x = (x1 as f32 - len * (angle - arrow_angle).cos()).round() as i32;
        let left_y = (y1 as f32 - len * (angle - arrow_angle).sin()).round() as i32;
        let right_x = (x1 as f32 - len * (angle + arrow_angle).cos()).round() as i32;
        let right_y = (y1 as f32 - len * (angle + arrow_angle).sin()).round() as i32;

        self.draw_line(x1, y1, left_x, left_y, color, width);
        self.draw_line(x1, y1, right_x, right_y, color, width);
        self.draw_line(left_x, left_y, right_x, right_y, color, width);
    }

    /// Tracé de point pour la grille infinie
    pub fn draw_dot(&mut self, cx: i32, cy: i32, radius: i32, color: u32) {
        let r = radius.max(1);
        self.fill_rect(cx - r, cy - r, cx + r + 1, cy + r + 1, color, 255);
    }

    /// Blit d'une image matricielle RGBA avec mise à l'échelle rapide (Nearest Neighbor)
    pub fn blit_image(
        &mut self,
        src_pixels: &[u32],
        src_w: usize,
        src_h: usize,
        dst_x: i32,
        dst_y: i32,
        dst_w: i32,
        dst_h: i32,
    ) {
        if src_w == 0 || src_h == 0 || dst_w <= 0 || dst_h <= 0 {
            return;
        }

        let min_x = dst_x.max(0);
        let max_x = (dst_x + dst_w).min(self.width as i32);
        let min_y = dst_y.max(0);
        let max_y = (dst_y + dst_h).min(self.height as i32);

        if min_x >= max_x || min_y >= max_y {
            return;
        }

        let scale_x = (src_w as f32) / (dst_w as f32);
        let scale_y = (src_h as f32) / (dst_h as f32);

        for y in min_y..max_y {
            let sy = (((y - dst_y) as f32 * scale_y) as usize).min(src_h - 1);
            let row_offset = (y as usize) * self.width;
            let src_row = sy * src_w;

            for x in min_x..max_x {
                let sx = (((x - dst_x) as f32 * scale_x) as usize).min(src_w - 1);
                let pixel = src_pixels[src_row + sx];
                let a = ((pixel >> 24) & 0xFF) as u8;

                if a == 255 {
                    self.pixels[row_offset + (x as usize)] = pixel & 0x00FFFFFF;
                } else if a > 0 {
                    self.blend_pixel(x, y, pixel & 0x00FFFFFF, a);
                }
            }
        }
    }

    /// Rendu d'une ligne de texte bitmap 8x8
    pub fn draw_text(&mut self, text: &str, start_x: i32, start_y: i32, scale: i32, color: u32) {
        let s = scale.max(1);
        let mut cur_x = start_x;

        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            let glyph_idx = if ch.is_ascii() && (ch as usize) >= 32 && (ch as usize) <= 126 {
                (ch as usize) - 32
            } else {
                0
            };

            let glyph = FONT_8X8[glyph_idx];
            for row in 0..8 {
                let row_byte = glyph[row];
                for col in 0..8 {
                    if (row_byte & (1 << (7 - col))) != 0 {
                        self.fill_rect(
                            cur_x + col * s,
                            start_y + (row as i32) * s,
                            cur_x + (col + 1) * s,
                            start_y + (row as i32 + 1) * s,
                            color,
                            255,
                        );
                    }
                }
            }
            cur_x += 8 * s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framebuffer_resize_and_clear() {
        let mut fb = FrameBuffer::new(100, 100);
        fb.clear(0x00FF0000);
        assert_eq!(fb.pixels[0], 0x00FF0000);
        assert_eq!(fb.pixels[99 * 100 + 99], 0x00FF0000);

        fb.resize(50, 50);
        assert_eq!(fb.pixels.len(), 2500);
    }

    #[test]
    fn test_fill_rect_solid_and_alpha() {
        let mut fb = FrameBuffer::new(20, 20);
        fb.clear(0x00000000);

        // Solid fill
        fb.fill_rect(2, 2, 8, 8, 0x0000FF00, 255);
        assert_eq!(fb.pixels[2 * 20 + 2], 0x0000FF00);
        assert_eq!(fb.pixels[0], 0x00000000);

        // Alpha blend 50%
        let mut fb2 = FrameBuffer::new(10, 10);
        fb2.clear(0x00000000);
        fb2.blend_pixel(0, 0, 0x00FFFFFF, 128);
        let p = fb2.pixels[0];
        let r = (p >> 16) & 0xFF;
        assert!(r > 120 && r < 135);
    }

    #[test]
    fn test_stroke_rect_and_dashed() {
        let mut fb = FrameBuffer::new(30, 30);
        fb.clear(0);
        fb.stroke_rect(5, 5, 25, 25, 0x00FFFFFF, 1, 255);
        assert_eq!(fb.pixels[5 * 30 + 5], 0x00FFFFFF);
        assert_eq!(fb.pixels[15 * 30 + 15], 0);

        fb.stroke_dashed_rect(2, 2, 28, 28, 0x0000FF00, 4, 255);
        assert_eq!(fb.pixels[2 * 30 + 2], 0x0000FF00);
    }

    #[test]
    fn test_draw_line_and_arrow() {
        let mut fb = FrameBuffer::new(50, 50);
        fb.clear(0);
        fb.draw_line(0, 0, 49, 49, 0x00FFFFFF, 1);
        assert_eq!(fb.pixels[0], 0x00FFFFFF);
        assert_eq!(fb.pixels[25 * 50 + 25], 0x00FFFFFF);
        assert_eq!(fb.pixels[49 * 50 + 49], 0x00FFFFFF);

        fb.draw_arrow(10, 10, 40, 40, 0x00FF0000, 2, 10.0);
        assert_eq!(fb.pixels[40 * 50 + 40], 0x00FF0000);
    }

    #[test]
    fn test_blit_image() {
        let mut fb = FrameBuffer::new(20, 20);
        fb.clear(0);
        let src = vec![0xFF00FF00; 4]; // 2x2 green image with alpha 255
        fb.blit_image(&src, 2, 2, 5, 5, 4, 4);

        assert_eq!(fb.pixels[5 * 20 + 5], 0x0000FF00);
        assert_eq!(fb.pixels[8 * 20 + 8], 0x0000FF00);
        assert_eq!(fb.pixels[0], 0);
    }

    #[test]
    fn test_draw_text_and_dot() {
        let mut fb = FrameBuffer::new(40, 40);
        fb.clear(0);
        fb.draw_text("A", 10, 10, 1, 0x00FFFFFF);
        fb.draw_dot(20, 20, 2, 0x0038BDF8);

        assert_eq!(fb.pixels[20 * 40 + 20], 0x0038BDF8);
    }
}

