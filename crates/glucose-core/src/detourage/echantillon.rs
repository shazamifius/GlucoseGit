//! **Changer la taille d'un plan de valeurs** (DETOURER-1) : une seule loi pour réduire l'image vers
//! l'entrée du modèle et remettre le masque à la taille de l'image.
//!
//! Le filtre triangle (bilinéaire), **élargi du rapport d'échelle quand on réduit** : chaque pixel de
//! sortie est alors la moyenne pondérée de tous les pixels d'entrée qu'il couvre, et non de ses deux
//! voisins — une image de 4 000 pixels réduite à 320 sans crénelage. C'est le « bilinéaire » de
//! Pillow ; quand on agrandit, il redevient l'interpolation entre deux voisins.

/// Les poids d'un axe : pour chaque sortie, la première entrée et les poids des suivantes.
fn poids(n: usize, m: usize) -> Vec<(usize, Vec<f32>)> {
    let echelle = n as f64 / m as f64;
    let largeur = echelle.max(1.0);
    (0..m)
        .map(|i| {
            let centre = (i as f64 + 0.5) * echelle;
            let debut = (centre - largeur).floor().max(0.0) as usize;
            let fin = ((centre + largeur).ceil() as usize).min(n);
            let mut w: Vec<f32> = (debut..fin)
                .map(|j| (1.0 - ((j as f64 + 0.5 - centre) / largeur).abs()).max(0.0) as f32)
                .collect();
            let somme: f32 = w.iter().sum();
            if somme > 0.0 {
                w.iter_mut().for_each(|x| *x /= somme);
            }
            (debut, w)
        })
        .collect()
}

/// `plan` (`l × h`, ligne après ligne) à la taille `l2 × h2`.
pub fn reechantillonner(
    plan: &[f32],
    (l, h): (usize, usize),
    (l2, h2): (usize, usize),
) -> Vec<f32> {
    let (px, py) = (poids(l, l2), poids(h, h2));
    let mut large = Vec::with_capacity(l2 * h);
    for ligne in plan.chunks_exact(l) {
        for (debut, w) in &px {
            large.push(
                w.iter()
                    .zip(&ligne[*debut..])
                    .map(|(a, b)| a * b)
                    .sum::<f32>(),
            );
        }
    }
    let mut sortie = vec![0.0; l2 * h2];
    for (y, (debut, w)) in py.iter().enumerate() {
        let dest = &mut sortie[y * l2..(y + 1) * l2];
        for (k, &wk) in w.iter().enumerate() {
            let source = &large[(debut + k) * l2..(debut + k + 1) * l2];
            dest.iter_mut().zip(source).for_each(|(d, s)| *d += wk * s);
        }
    }
    sortie
}
