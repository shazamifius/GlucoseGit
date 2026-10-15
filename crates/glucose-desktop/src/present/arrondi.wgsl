// La loi des rectangles arrondis de `glucose_core::membrane_forme`, recopiee fonction pour
// fonction, en f32 et dans le meme ordre : la membrane et la brume d'une carte la lisent toutes
// deux (fiche 73). Un seul texte, pose devant chaque nuanceur qui l'emploie.

// `Arrondi::distance` : depuis les BORDS, pour rester juste quand le centre est tres loin.
fn distance(b: vec4<f32>, r: f32, p: vec2<f32>) -> f32 {
    let qx = max(b.x - p.x, p.x - b.z) + r;
    let qy = max(b.y - p.y, p.y - b.w) + r;
    return length(max(vec2<f32>(qx, qy), vec2<f32>(0.0))) + min(max(qx, qy), 0.0) - r;
}

fn couverture_d_un_plein(d: f32) -> f32 {
    return clamp(0.5 - d, 0.0, 1.0);
}

fn couverture_d_un_trait(d: f32, h: f32) -> f32 {
    return max(min(d + 0.5, h) - max(d - 0.5, -h), 0.0);
}
