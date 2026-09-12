//! Écrit la capture de la scène témoin, pour être **regardée**.
//!
//! ```text
//! cargo run -p glucose-desktop --example capture_temoin
//! cargo run -p glucose-desktop --example capture_temoin -- mon/dossier
//! ```
//!
//! Le test `test_l_empreinte_de_la_scene_temoin_n_a_pas_change` garde l'empreinte de cette
//! image. Quand il échoue, c'est que le rendu a changé — volontairement ou non. Le geste est
//! alors toujours le même : lancer cet exemple, **ouvrir le PNG et le regarder**, puis mettre
//! l'empreinte à jour si le changement est celui qu'on voulait.
//!
//! Garder une empreinte plutôt que l'image elle-même est délibéré : une image de référence
//! versionnée grossirait le dépôt à chaque évolution du rendu, et il y en aura beaucoup.

use glucose_core::hash::{hex_of, sha256};
use glucose_core::synth;
use glucose_desktop::bench;

fn main() {
    let dossier = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let temoin = synth::witness();
    let (w0, h0) = synth::WITNESS_SIZE;

    // La seconde capture est la même scène **vue de plus loin** : c'est elle qui montre ce que
    // le texte devient quand le zoom descend, là où la fiche 03 signalait onze bornes qui
    // figent la police pendant que la boîte, elle, se met à l'échelle.
    let mut dezoome = temoin.clone();
    bench::frame_document(&mut dezoome, 0.45, w0, h0);

    for (nom, store, w, h) in
        [("temoin", &temoin, w0, h0), ("temoin-dezoome", &dezoome, w0, h0)]
    {
        let png = bench::capture(store, w, h);
        let chemin = format!("{dossier}/{nom}.png");
        match std::fs::write(&chemin, &png) {
            Ok(()) => println!(
                "{chemin} — {w}x{h}, {} octets, empreinte {}",
                png.len(),
                &hex_of(&sha256(&png))[..16]
            ),
            Err(e) => eprintln!("{chemin} : {e}"),
        }
    }
}
