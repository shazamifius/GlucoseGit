//! **MCP-VOIR — voir une photo** (fiche 179 ; `relais-rep-mcp`, point 7 : *« voir une photo : une
//! petite image envoyée à l'IA qui la demande »*). Une IA lisait le texte de ses notes, jamais ses
//! captures d'écran : un bogue montré en image lui restait invisible.
//!
//! La photo se lit, se réduit et s'encode **sur un fil à part**, qui répond lui-même à l'agent : la
//! boucle qui dessine n'attend jamais le décodage d'une photo de douze mégapixels. Elle se lit comme
//! le magasin la lit (`atelier::decoder` : tous les formats de Glucose, l'orientation comprise).
//!
//! **Sa taille** est celle qu'un modèle de Claude lit sans la réduire lui-même — au plus 1 568 px de
//! long et 1 568 jetons, une image coûtant ⌈l/28⌉ × ⌈h/28⌉ jetons (la documentation de la vision
//! de Claude, platform.claude.com/docs/en/build-with-claude/vision-coordinates, lue le 26/11 :
//! 1920×1080 y devient 1456×819) : au-delà, le modèle la réduit, et ce qu'on envoie en plus ne sert
//! à rien. Les modèles depuis Opus 4.7 lisent plus fin (2 576 px, 4 784 jetons), trois fois plus
//! cher : les jetons d'abord (sa priorité du 24/11), « cote » en demande une autre. **Sa forme** : JPEG, la plus légère à porter ; PNG si elle a de la
//! transparence, que le JPEG perdrait.

use crate::agents::json::Json;
use crate::agents::protocole::{self, Appel};
use crate::persist::objets::Objets;
use crate::renderer::photo::Pyramide;
use std::sync::{mpsc, Arc};
use tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};

/// Le côté d'un « patch » de la vision de Claude : une image coûte ⌈l/28⌉ × ⌈h/28⌉ jetons.
const PATCH: u32 = 28;
/// Le plus long côté qu'un modèle de Claude lit sans réduire l'image (le palier standard).
const COTE: u32 = 1568;
/// Les jetons qu'il lit sans la réduire.
const JETONS: u32 = 1568;
/// Le plus long côté que lit le modèle le plus fin : « cote » ne demande jamais plus.
const COTE_FIN: u32 = 2576;

/// **La taille à envoyer**, proportions gardées, jamais plus grande que la photo : par défaut la
/// plus grande que Claude lit sans la réduire ; avec `cote`, le plus long côté demandé, jusqu'à
/// [`COTE_FIN`].
pub fn taille_pour((l, h): (u32, u32), cote: Option<u32>) -> (u32, u32) {
    let (l, h) = (l.max(1), h.max(1));
    let grand = l.max(h);
    let mut cote_voulu = cote.unwrap_or(COTE).clamp(1, grand.min(COTE_FIN));
    let taille = |c: u32| {
        let k = f64::from(c) / f64::from(grand);
        let mesure = |x: u32| ((f64::from(x) * k).round() as u32).max(1);
        (mesure(l), mesure(h))
    };
    let jetons = |(a, b): (u32, u32)| a.div_ceil(PATCH) * b.div_ceil(PATCH);
    while cote.is_none() && cote_voulu > 1 && jetons(taille(cote_voulu)) > JETONS {
        cote_voulu -= 1;
    }
    taille(cote_voulu)
}

/// **Ce qu'une IA voit de cette photo** : ce qu'elle est (sa taille envoyée, sa taille vraie), ses
/// octets et leur type.
pub fn image_pour_une_ia(
    src: &str,
    objets: &Objets,
    cote: Option<u32>,
) -> Result<(Json, Vec<u8>, &'static str), String> {
    let porte = crate::renderer::atelier::admission::de_la_machine();
    let pyramide = crate::renderer::atelier::decoder(src, Some(objets), porte).ok_or(
        "cette photo ne se lit pas : son fichier manque, ou Glucose ne sait pas la décoder",
    )?;
    let (nl, nh) = pyramide.dimensions_natives();
    let (l, h) = taille_pour((nl, nh), cote);
    let reduite = reduire(&pyramide, (l, h)).ok_or("cette photo ne se réduit pas")?;
    let (octets, sorte) =
        encoder(&reduite, pyramide.opaque()).ok_or("cette photo ne s'encode pas")?;
    let n = |x: u32| Json::Nombre(f64::from(x));
    let dit = Json::objet([
        ("largeur", n(l)),
        ("hauteur", n(h)),
        ("native", Json::Liste(vec![n(nl), n(nh)])),
        ("type", Json::from(sorte)),
    ]);
    Ok((dit, octets, sorte))
}

/// Réduite à `(l, h)` depuis le plus petit niveau de la pyramide qui la couvre : moins de pixels à
/// lire, et pas de crénelage.
fn reduire(pyramide: &Pyramide, (l, h): (u32, u32)) -> Option<Pixmap> {
    let (_, source) = pyramide.meilleur_pour(l as f32)?;
    let mut cible = Pixmap::new(l, h)?;
    let (sl, sh) = (source.width() as f32, source.height() as f32);
    let echelle = Transform::from_scale(l as f32 / sl, h as f32 / sh);
    let peinture = PixmapPaint {
        quality: FilterQuality::Bicubic,
        ..PixmapPaint::default()
    };
    cible.draw_pixmap(0, 0, source, &peinture, echelle, None);
    Some(cible)
}

/// JPEG pour une photo opaque, PNG pour une qui a de la transparence.
fn encoder(pixmap: &Pixmap, opaque: bool) -> Option<(Vec<u8>, &'static str)> {
    if !opaque {
        return Some((pixmap.encode_png().ok()?, "image/png"));
    }
    // Opaque : ses pixels prémultipliés valent ses couleurs.
    let rgb: Vec<u8> = (pixmap.pixels().iter())
        .flat_map(|p| [p.red(), p.green(), p.blue()])
        .collect();
    let mut jpeg = Vec::new();
    let (l, h) = (pixmap.width(), pixmap.height());
    image::codecs::jpeg::JpegEncoder::new(&mut jpeg)
        .encode(&rgb, l, h, image::ExtendedColorType::Rgb8)
        .ok()?;
    Some((jpeg, "image/jpeg"))
}

/// **Voit à part** : lit, réduit et encode sur un fil, qui répond lui-même à l'agent. Sans fil
/// possible, l'agent l'apprend tout de suite.
pub fn voir_a_part(
    appel: Appel,
    reponse: mpsc::Sender<String>,
    (id, src): (String, String),
    objets: Arc<Objets>,
    cote: Option<u32>,
) {
    let (appel_du_fil, reponse_du_fil) = (appel.clone(), reponse.clone());
    let fil = std::thread::Builder::new()
        .name("glucose-voir".into())
        .spawn(move || {
            let texte = match image_pour_une_ia(&src, &objets, cote) {
                Ok((mut dit, octets, sorte)) => {
                    if let Json::Objet(champs) = &mut dit {
                        champs.insert(0, ("id".to_string(), Json::from(id.as_str())));
                    }
                    protocole::reponse_d_image(&appel_du_fil, &dit, &octets, sorte)
                }
                Err(e) => protocole::reponse_d_outil(&appel_du_fil, Err(e)),
            };
            let _ = reponse_du_fil.send(texte);
        });
    if let Err(e) = fil {
        let issue = Err(format!(
            "Glucose n'a pas pu ouvrir un fil pour lire la photo : {e}"
        ));
        let _ = reponse.send(protocole::reponse_d_outil(&appel, issue));
    }
}

impl crate::app::GlucoseApp {
    /// **Une IA demande à voir une photo** : le nœud se trouve ici, dans la boucle ; la photo se
    /// lit sur un fil, qui répond.
    pub(crate) fn voir_pour_un_agent(&mut self, d: crate::agents::Demande) {
        let a = &d.appel.arguments;
        let texte = |cle| a.get(cle).and_then(Json::as_str);
        let board = texte("tableau").unwrap_or(self.store.active_board_id());
        let Some(id) = texte("id") else {
            let issue = Err("l'argument « id » nomme la photo à voir".to_string());
            return drop(d.reponse.send(protocole::reponse_d_outil(&d.appel, issue)));
        };
        let photo = self.store.image(board, id);
        let Some(src) = photo.and_then(|p| p.src.clone()).filter(|s| !s.is_empty()) else {
            let issue = Err(format!("« {id} » n'est pas une photo de ce tableau"));
            return drop(d.reponse.send(protocole::reponse_d_outil(&d.appel, issue)));
        };
        let cote = (a.get("cote").and_then(Json::as_f64))
            .filter(|c| c.is_finite() && *c >= 1.0)
            .map(|c| c as u32);
        let (photo, objets) = ((id.to_string(), src), Arc::clone(&self.disque.objets));
        voir_a_part(d.appel, d.reponse, photo, objets, cote);
    }
}

#[cfg(test)]
mod tests;
