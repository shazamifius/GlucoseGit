//! Le PDF des planches (fiche 68) : les blocs `IDAT` d'un PNG sont l'image entière, et le
//! fichier a la forme que tout lecteur attend — chaque objet là où la table le dit.

use super::*;

/// Une image `l` × `h` dont chaque pixel est différent de ses voisins : un filtre PNG qui se
/// tromperait de rangée se verrait.
fn pixels(l: u32, h: u32) -> Vec<u8> {
    (0..l * h)
        .flat_map(|i| {
            let (x, y) = (i % l, i / l);
            [(x * 7) as u8, (y * 13) as u8, ((x ^ y) * 29) as u8]
        })
        .collect()
}

/// Un PNG RVB 8 bits, ses blocs `IDAT` coupés à `bloc` octets — plusieurs, à dessein.
fn png_rvb(l: u32, h: u32, bloc: usize) -> Vec<u8> {
    let mut octets = Vec::new();
    {
        let mut encodeur = png::Encoder::new(&mut octets, l, h);
        encodeur.set_color(png::ColorType::Rgb);
        encodeur.set_depth(png::BitDepth::Eight);
        let mut ecrivain = encodeur.write_header().expect("l'en-tête");
        let mut flux = ecrivain.stream_writer_with_size(bloc).expect("le flux");
        flux.write_all(&pixels(l, h)).expect("les pixels");
        flux.finish().expect("la fin");
    }
    octets
}

/// Le CRC-32 d'un bloc de PNG (ISO 3309), bit à bit — de quoi refaire un PNG sans dépendance.
fn crc(octets: &[u8]) -> u32 {
    let mut c = !0u32;
    for &o in octets {
        c ^= u32::from(o);
        for _ in 0..8 {
            c = if c & 1 == 1 {
                (c >> 1) ^ 0xEDB8_8320
            } else {
                c >> 1
            };
        }
    }
    !c
}

fn bloc(genre: &[u8; 4], donnees: &[u8]) -> Vec<u8> {
    let mut b = (donnees.len() as u32).to_be_bytes().to_vec();
    b.extend_from_slice(genre);
    b.extend_from_slice(donnees);
    let mut somme = genre.to_vec();
    somme.extend_from_slice(donnees);
    b.extend_from_slice(&crc(&somme).to_be_bytes());
    b
}

/// **Les blocs `IDAT` mis bout à bout sont l'image entière** : un PNG refait de leur seule
/// concaténation, en un bloc, se relit pixel pour pixel — ce que le PDF décodera.
#[test]
fn test_les_blocs_idat_bout_a_bout_sont_l_image() {
    let (l, h) = (37, 23);
    let png = png_rvb(l, h, 64);
    let blocs = png.windows(4).filter(|w| *w == b"IDAT").count();
    assert!(blocs > 3, "plusieurs blocs : {blocs}");
    let p = planche(&png).expect("un PNG RVB");
    assert_eq!((p.largeur, p.hauteur), (l, h));
    let mut ihdr = l.to_be_bytes().to_vec();
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut refait = SIGNATURE.to_vec();
    refait.extend(bloc(b"IHDR", &ihdr));
    refait.extend(bloc(b"IDAT", &p.idat));
    refait.extend(bloc(b"IEND", &[]));
    let mut lecteur = png::Decoder::new(std::io::Cursor::new(refait))
        .read_info()
        .expect("le PNG refait se lit");
    let mut lu = vec![0; lecteur.output_buffer_size().expect("une taille")];
    lecteur.next_frame(&mut lu).expect("ses pixels");
    assert_eq!(lu, pixels(l, h));
}

/// **Une autre forme ne passe pas** : avec alpha, ou entrelacé — le PDF les lirait faux.
#[test]
fn test_seul_le_rvb_8_bits_non_entrelace_passe() {
    let mut avec_alpha = Vec::new();
    let mut e = png::Encoder::new(&mut avec_alpha, 2, 2);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header()
        .expect("l'en-tête")
        .write_image_data(&[0; 16])
        .expect("les pixels");
    assert_eq!(planche(&avec_alpha), None);
    let mut entrelace = png_rvb(4, 4, 1024);
    // L'octet d'entrelacement de l'en-tête : signature (8), longueur et genre (8), puis 12.
    entrelace[8 + 8 + 12] = 1;
    assert_eq!(planche(&entrelace), None);
    assert!(planche(&png_rvb(4, 4, 1024)).is_some());
}

/// **Chaque objet est là où la table le dit**, la table là où la fin le dit ; deux pages, à la
/// taille de leurs planches en points ; l'image de chacune, ses octets tels quels.
#[test]
fn test_la_table_dit_ou_est_chaque_objet() {
    let planches = [
        planche(&png_rvb(40, 30, 1024)).expect("une"),
        planche(&png_rvb(8, 4, 1024)).expect("deux"),
    ];
    let mut pdf = Vec::new();
    let n = ecrire(&mut pdf, &planches).expect("écrit");
    assert_eq!(n, pdf.len() as u64);
    assert!(pdf.starts_with(b"%PDF-1.4\n"));
    // Chercher dans les octets : les flux sont binaires, et une lecture en texte « avec
    // pertes » décalerait chaque position.
    let trouver = |motif: &[u8]| pdf.windows(motif.len()).position(|w| w == motif);
    let fin = pdf
        .windows(10)
        .rposition(|w| w == b"startxref\n")
        .expect("la fin");
    let table: usize = std::str::from_utf8(&pdf[fin + 10..])
        .expect("la fin est du texte")
        .lines()
        .next()
        .and_then(|l| l.parse().ok())
        .expect("la position de la table");
    assert!(
        pdf[table..].starts_with(b"xref\n0 9\n"),
        "neuf objets et le zéro"
    );
    let lignes = &pdf[table + b"xref\n0 9\n".len()..];
    for numero in 1..=8 {
        let ligne = &lignes[numero * 20..numero * 20 + 20];
        assert!(ligne.ends_with(b" 00000 n \n"), "vingt octets : {ligne:?}");
        let position: usize = std::str::from_utf8(&ligne[..10])
            .expect("des chiffres")
            .parse()
            .expect("une position");
        let attendu = format!("{numero} 0 obj\n");
        assert!(
            pdf[position..].starts_with(attendu.as_bytes()),
            "l'objet {numero}"
        );
    }
    assert!(trouver(b"/Kids [3 0 R 6 0 R] /Count 2").is_some());
    // La forme que MuPDF et PDFium ont rendue au pixel près (fiche 68) : rien n'y change.
    let image = b"/Width 40 /Height 30 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /DecodeParms << /Predictor 15 /Colors 3 /BitsPerComponent 8 /Columns 40 >>";
    assert!(trouver(image).is_some(), "le dictionnaire de l'image");
    assert!(
        trouver(b"/MediaBox [0 0 30 22.5]").is_some(),
        "40 × 30 pixels"
    );
    assert!(trouver(b"/MediaBox [0 0 6 3]").is_some(), "8 × 4 pixels");
    for p in &planches {
        let tete = format!("/Length {} >>\nstream\n", p.idat.len());
        let debut = trouver(tete.as_bytes()).expect("le flux de l'image") + tete.len();
        assert_eq!(&pdf[debut..debut + p.idat.len()], &p.idat[..]);
    }
}
