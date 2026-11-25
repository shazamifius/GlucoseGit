//! **Les photos HEIC**, celles de l'iPhone (HEIC-2, fiche 161 § 6).
//!
//! `image` ne lit pas le HEIC ; `heifer` (`decisions/20`) le décode en Rust. Il entre par les
//! **crochets** de `image` (`image::hooks`) : enregistré une fois au démarrage, il sert toutes les
//! portes qui reconnaissent un format par `with_guessed_format` — l'ouvrier de l'atelier, le dépôt
//! d'une image venue d'une page, le presse-papiers, la sélection Wayland.
//!
//! **Paresseux**, à la différence du décodeur que `heifer` offre (`image_crate::HeifDecoder`, qui
//! décode dès sa construction) : se construire ne lit que le conteneur, quelques microsecondes.
//! Le dépôt ne demande que les dimensions, sur le fil de l'interface ; l'ouvrier les demande
//! **avant** l'admission de la mémoire (MEMOIRE-3) — décoder là bloquerait l'image 430 ms par
//! photo, ou contournerait l'admission. Le décodage n'a lieu que dans `read_image`, sur huit bits
//! et converti en sRGB (le Display P3 de l'iPhone, sinon terne), sur un seul fil : l'atelier en
//! décode déjà plusieurs à la fois.

use heifer::heifer_isobmff::{boxes::Property, HeifFile};
use image::error::{DecodingError, ImageFormatHint};
use image::{ColorType, ImageDecoder, ImageError, ImageResult};
use std::io::Read;

/// Les marques majeures d'un HEIF codé en HEVC, que la signature reconnaît (octets 8 à 11) ;
/// `mif1`, la marque générique, en est souvent une (des Android l'écrivent) — un `mif1` qui
/// n'est pas du HEVC est refusé au décodage, comme aujourd'hui.
const MARQUES: [[u8; 4]; 5] = [*b"heic", *b"heix", *b"heim", *b"heis", *b"mif1"];

/// La taille de la boîte (octets 0 à 3) varie ; `ftyp` et la marque, non.
const MASQUE: &[u8] = b"\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff";

/// Les signatures, une par marque : quatre octets libres, `ftyp`, la marque.
static SIGNATURES: [[u8; 12]; 5] = {
    let mut s = [[0u8; 12]; 5];
    let mut i = 0;
    while i < 5 {
        let mut j = 0;
        while j < 4 {
            s[i][4 + j] = b"ftyp"[j];
            s[i][8 + j] = MARQUES[i][j];
            j += 1;
        }
        i += 1;
    }
    s
};

/// **Apprend le HEIC à `image`**, une fois pour tout le processus (les crochets sont globaux).
pub fn enregistrer() {
    static UNE_FOIS: std::sync::Once = std::sync::Once::new();
    UNE_FOIS.call_once(|| {
        image::hooks::register_decoding_hook(
            "heic".into(),
            Box::new(|lecteur| Ok(Box::new(Heic::lire(lecteur)?))),
        );
        for signature in &SIGNATURES {
            image::hooks::register_format_detection_hook("heic".into(), signature, Some(MASQUE));
        }
    });
}

/// Un fichier HEIC lu, pas encore décodé : ses octets et les dimensions de son image première.
pub struct Heic {
    octets: Vec<u8>,
    dimensions: (u32, u32),
}

impl Heic {
    fn lire(mut lecteur: impl Read) -> ImageResult<Self> {
        let mut octets = Vec::new();
        lecteur
            .read_to_end(&mut octets)
            .map_err(ImageError::IoError)?;
        let dimensions = dimensions(&octets).ok_or_else(|| erreur("conteneur illisible"))?;
        Ok(Self { octets, dimensions })
    }
}

/// **Les dimensions de l'image montrée, sans la décoder** : l'`ispe` de l'image première, puis
/// ses transformations dans l'ordre du fichier, comme `heifer` les applique (`decode_item_with`) —
/// le recadrage (`clap`) la rétrécit, un quart de tour impair (`irot`) échange largeur et
/// hauteur, le miroir (`imir`) ne change rien.
fn dimensions(octets: &[u8]) -> Option<(u32, u32)> {
    let fichier = HeifFile::parse(octets).ok()?;
    let taille = fichier.image_size(fichier.primary_id).ok()??;
    let taille = transformer(taille, fichier.item_properties(fichier.primary_id).ok()?);
    taille
}

/// La taille codée, transformée par les propriétés de l'image, dans leur ordre.
fn transformer<'a>(
    mut taille: (u32, u32),
    proprietes: impl IntoIterator<Item = &'a Property>,
) -> Option<(u32, u32)> {
    for propriete in proprietes {
        match *propriete {
            Property::CleanAperture {
                width,
                height,
                horiz_offset,
                vert_offset,
            } => {
                taille = (
                    recadrer(taille.0, width, horiz_offset)?,
                    recadrer(taille.1, height, vert_offset)?,
                );
            }
            Property::Rotation(quarts) if quarts % 2 == 1 => taille = (taille.1, taille.0),
            _ => {}
        }
    }
    Some(taille)
}

/// Un côté recadré par le `clap`, au pixel près comme `heifer::clean_aperture` puis
/// `Image::crop` : le côté arrondi, centré sur le milieu décalé, borné par l'image.
fn recadrer(cote: u32, (num, den): (u32, u32), (dec_num, dec_den): (i32, u32)) -> Option<u32> {
    if den == 0 || dec_den == 0 {
        return None;
    }
    let voulu = f64::from(num) / f64::from(den);
    let milieu = f64::from(dec_num) / f64::from(dec_den) + (f64::from(cote) - 1.0) / 2.0;
    let debut = ((milieu - (voulu - 1.0) / 2.0).round().max(0.0) as u32).min(cote);
    Some((voulu.round() as u32).min(cote - debut))
}

fn erreur(raison: &str) -> ImageError {
    ImageError::Decoding(DecodingError::new(
        ImageFormatHint::Name("HEIC".into()),
        raison.to_string(),
    ))
}

impl ImageDecoder for Heic {
    fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    fn color_type(&self) -> ColorType {
        ColorType::Rgba8
    }

    fn set_limits(&mut self, limites: image::Limits) -> ImageResult<()> {
        limites.check_support(&image::LimitSupport::default())?;
        limites.check_dimensions(self.dimensions.0, self.dimensions.1)
    }

    /// Le décodage, enfin : refusé par une erreur — jamais un tampon faux — si l'image ne fait
    /// pas les dimensions annoncées.
    fn read_image(self, tampon: &mut [u8]) -> ImageResult<()> {
        let un_fil = heifer::Options::new().with_max_threads(1);
        let image =
            heifer::decode_srgb_rgba8(&self.octets, &un_fil).map_err(|e| erreur(&e.to_string()))?;
        if (image.width, image.height) != self.dimensions || image.data.len() != tampon.len() {
            return Err(erreur("dimensions autres que celles du conteneur"));
        }
        tampon.copy_from_slice(&image.data);
        Ok(())
    }

    fn read_image_boxed(self: Box<Self>, tampon: &mut [u8]) -> ImageResult<()> {
        (*self).read_image(tampon)
    }
}

#[cfg(test)]
mod tests;
