// Écrit l'image décodée, réduite d'un facteur f (moyenne), en PNG : pour la regarder.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let f: usize = a[3].parse().unwrap();
    let i =
        heifer::decode_srgb_rgba8(&std::fs::read(&a[1]).unwrap(), &heifer::Options::new()).unwrap();
    let (w, h) = (i.width as usize, i.height as usize);
    let px = &i.data;
    let (rw, rh) = (w / f, h / f);
    let mut out = vec![0u8; rw * rh * 3];
    for y in 0..rh {
        for x in 0..rw {
            for c in 0..3 {
                let mut s = 0u32;
                for dy in 0..f {
                    for dx in 0..f {
                        s += px[((y * f + dy) * w + x * f + dx) * 4 + c] as u32;
                    }
                }
                out[(y * rw + x) * 3 + c] = (s / (f * f) as u32) as u8;
            }
        }
    }
    let mut e = png::Encoder::new(std::fs::File::create(&a[2]).unwrap(), rw as u32, rh as u32);
    e.set_color(png::ColorType::Rgb);
    e.write_header().unwrap().write_image_data(&out).unwrap();
}
