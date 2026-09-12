//! Affiche la mise en page d'une formule, pour la lire a l'oeil.
use glucose_math::{layout, MathItem, Mode};

fn main() {
    for (f, mode) in [
        (r"\frac{a}{b}", Mode::Inline),
        (r"x^2 + y_1", Mode::Inline),
        (r"\sum_{i=1}^{n} i", Mode::Display),
    ] {
        println!("\n=== {f} ({mode:?}) ===");
        match layout(f, mode) {
            Ok(l) => {
                println!("  boite : w={:.3} h={:.3} d={:.3}", l.width, l.height, l.depth);
                for it in &l.items {
                    match it {
                        MathItem::Glyph { text, x, y, size, family, style } => println!(
                            "  GLYPHE {text:?} x={x:+.3} y={y:+.3} taille={size:.3} {}-{}",
                            family.font_name(), style.suffix()
                        ),
                        MathItem::Rule { x, y, width, height } => println!(
                            "  FILET  x={x:+.3} y={y:+.3} l={width:.3} e={height:.3}"
                        ),
                        MathItem::Path { name, x, y, width, height } => println!(
                            "  FORME  {name:?} x={x:+.3} y={y:+.3} l={width:.3} h={height:.3}"
                        ),
                    }
                }
            }
            Err(e) => println!("  ERREUR {e}"),
        }
    }
}
