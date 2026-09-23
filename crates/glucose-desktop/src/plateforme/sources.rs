//! **D'où rapatrier l'image qu'on a déposée**, dans sa meilleure qualité (DEPOT-WEB-4).
//!
//! # Pourquoi ce module existe
//!
//! Glisser une épingle depuis Pinterest n'apporte presque jamais l'image : dix dépôts sur dix
//! de la session du 23/09 n'en portaient aucune. Ils portaient, au mieux, **des adresses** —
//! le lien de l'épingle, un fragment HTML où la vignette paraît, des données que la page a
//! posées elle-même. L'utilisateur a tranché : *« l'objectif ultime de Glucose, c'est qu'il
//! puisse complètement télécharger depuis Internet de lui-même »*. Et il veut l'image **dans
//! sa qualité optimale**.
//!
//! # Ce que Pinterest sert, et ce qu'il garde
//!
//! Pinterest ne montre que des copies réduites — 236, 474, 564 ou 736 pixels de large — mais
//! garde l'original sur le même serveur, sous le même chemin, avec `originals` à la place de
//! la taille : `i.pinimg.com/236x/aa/bb/cc/h.jpg` → `i.pinimg.com/originals/aa/bb/cc/h.jpg`.
//! Les outils libres qui récupèrent la pleine résolution font exactement cela, et **vérifient**
//! que l'original répond : il manque parfois, et l'extension de l'original peut différer. On
//! essaie donc, dans l'ordre, l'original puis la plus grande copie puis celle qu'on a reçue.
//!
//! # Pur, et c'est voulu
//!
//! Tout ce qui **décide** est ici : quelle adresse essayer d'abord, ce qu'une page annonce, si
//! des octets sont une image. Sans une ligne de Windows ni de réseau, testé partout. Le
//! téléchargement lui-même est une mécanique, et elle vit ailleurs.

/// Une adresse web, découpée comme un client HTTP en a besoin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adresse {
    /// `https` plutôt que `http`.
    pub securise: bool,
    /// Le nom du serveur, sans port.
    pub hote: String,
    /// Le port, explicite ou celui du schéma.
    pub port: u16,
    /// Le chemin et la requête, `/` au moins.
    pub chemin: String,
}

/// **Découpe une adresse `http` ou `https`**, et refuse tout le reste.
///
/// Un dépôt vient d'une page : `file://`, `javascript:` ou un schéma inconnu ne se téléchargent
/// pas, pour la même raison que [`crate::interactions::links`] refuse de les ouvrir.
pub fn decouper(url: &str) -> Option<Adresse> {
    let url = url.trim();
    let (securise, reste) = match url.strip_prefix("https://") {
        Some(r) => (true, r),
        None => (false, url.strip_prefix("http://")?),
    };
    let fin_hote = reste.find(['/', '?', '#']).unwrap_or(reste.len());
    let (autorite, chemin) = reste.split_at(fin_hote);
    // Un identifiant dans l'adresse (`utilisateur@hote`) n'a rien à faire dans un dépôt.
    if autorite.is_empty() || autorite.contains('@') {
        return None;
    }
    let (hote, port) = match autorite.rsplit_once(':') {
        Some((h, p)) => (h, p.parse().ok()?),
        None => (autorite, if securise { 443 } else { 80 }),
    };
    let chemin = chemin.split('#').next().unwrap_or("");
    Some(Adresse {
        securise,
        hote: hote.to_ascii_lowercase(),
        port,
        chemin: if chemin.is_empty() || chemin.starts_with('?') {
            format!("/{chemin}")
        } else {
            chemin.to_string()
        },
    })
}

/// Ce qu'on essaie de télécharger, et ce qu'on en attend.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Candidat {
    /// Une adresse qui devrait rendre les octets d'une image.
    Image(String),
    /// Une page, qui annonce son image quelque part dans son HTML.
    Page(String),
}

/// **Les adresses à essayer, de la meilleure qualité à la moins bonne**, sans doublon.
///
/// D'abord les images de Pinterest, chacune déclinée de l'original à la copie reçue ; puis les
/// autres adresses qui désignent une image ; puis les pages. Une page coûte un aller-retour de
/// plus et n'annonce souvent qu'une copie : elle vient en dernier recours.
pub fn candidats(adresses: &[String]) -> Vec<Candidat> {
    let mut images = Vec::new();
    let mut autres_images = Vec::new();
    let mut pages = Vec::new();
    for a in adresses {
        if decouper(a).is_none() {
            continue;
        }
        if let Some(variantes) = variantes_pinterest(a) {
            images.extend(variantes);
        } else if designe_une_image(a) {
            autres_images.push(a.clone());
        } else {
            pages.push(a.clone());
        }
    }
    let mut vus = std::collections::HashSet::new();
    images
        .into_iter()
        .chain(autres_images)
        .map(Candidat::Image)
        .chain(pages.into_iter().map(Candidat::Page))
        .filter(|c| {
            vus.insert(match c {
                Candidat::Image(u) | Candidat::Page(u) => u.clone(),
            })
        })
        .collect()
}

/// **Une image de Pinterest, de l'original à la copie reçue** — ou rien si ce n'en est pas une.
///
/// Le premier segment du chemin est la taille (`236x`, `736x`, `originals`, `75x75_RS`…) ; le
/// reste désigne l'image. L'original peut porter une autre extension que la copie — un PNG
/// réduit en JPEG —, d'où les deux essais avant la plus grande copie.
pub fn variantes_pinterest(url: &str) -> Option<Vec<String>> {
    let a = decouper(url)?;
    if a.hote != "i.pinimg.com" {
        return None;
    }
    let chemin = a.chemin.trim_start_matches('/');
    let (_taille, reste) = chemin.split_once('/')?;
    let base = "https://i.pinimg.com";
    let mut v = vec![format!("{base}/originals/{reste}")];
    if let Some((sans_ext, ext)) = reste.rsplit_once('.') {
        for autre in ["png", "jpg", "webp", "gif"] {
            if !ext.eq_ignore_ascii_case(autre) {
                v.push(format!("{base}/originals/{sans_ext}.{autre}"));
            }
        }
    }
    v.push(format!("{base}/736x/{reste}"));
    v.push(url.trim().to_string());
    Some(v)
}

/// Cette adresse désigne-t-elle une image, à en croire son chemin ?
fn designe_une_image(url: &str) -> bool {
    let Some(a) = decouper(url) else {
        return false;
    };
    let chemin = a
        .chemin
        .split('?')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    [".jpg", ".jpeg", ".png", ".webp", ".gif", ".avif", ".bmp"]
        .iter()
        .any(|e| chemin.ends_with(e))
}

/// **L'image qu'une page déclare comme la sienne** : `og:image`, puis `twitter:image`.
///
/// # Ce que la première version faisait, et l'image qu'elle a rapportée
///
/// Elle faisait passer devant **toutes** les adresses de `i.pinimg.com` trouvées dans la page,
/// en raisonnant qu'elles menaient à l'original. Sur la première épingle réelle essayée, elle a
/// rapporté une icône en dégradé : une page d'épingle en porte des dizaines — icônes, avatars,
/// épingles voisines —, et rien ne dit laquelle est la bonne. Le journal disait « image
/// rapatriée » ; c'est en la **regardant** qu'on a vu que ce n'était pas elle.
///
/// Seule la page sait laquelle est la sienne, et elle le déclare dans ses balises d'aperçu. Sur
/// Pinterest c'est une copie de 736 pixels, que [`candidats`] décline ensuite jusqu'à
/// l'original — sous le même chemin, donc la même image.
pub fn images_de_la_page(html: &str) -> Vec<String> {
    let mut trouvees = Vec::new();
    for propriete in ["og:image", "twitter:image"] {
        for (i, _) in html.match_indices(propriete) {
            let debut = html[..i].rfind('<').unwrap_or(0);
            let fin = html[i..].find('>').map_or(html.len(), |f| i + f);
            // `og:image:width` porte aussi le nom : seul un contenu qui est une adresse compte.
            if let Some(contenu) = attribut(&html[debut..fin], "content") {
                if decouper(&contenu).is_some() && !trouvees.contains(&contenu) {
                    trouvees.push(contenu);
                }
            }
        }
    }
    trouvees
}

/// La valeur d'un attribut dans une balise, entre guillemets doubles ou simples.
fn attribut(balise: &str, nom: &str) -> Option<String> {
    let i = balise.find(&format!("{nom}="))? + nom.len() + 1;
    let guillemet = balise[i..].chars().next()?;
    if guillemet != '"' && guillemet != '\'' {
        return None;
    }
    let debut = i + 1;
    let fin = balise[debut..].find(guillemet)? + debut;
    Some(balise[debut..fin].replace("&amp;", "&"))
}

/// **Ces octets sont-ils une image**, à en croire leur signature ?
///
/// Un serveur qui refuse répond souvent `200` avec une page d'erreur en HTML : l'écrire en
/// `.jpg` poserait une image cassée. On ne garde que ce qui commence comme une image — PNG,
/// JPEG, GIF, WebP, BMP, ou la famille HEIF qui porte l'AVIF.
pub fn est_une_image(octets: &[u8]) -> bool {
    octets.starts_with(b"\x89PNG\r\n\x1a\n")
        || octets.starts_with(&[0xFF, 0xD8, 0xFF])
        || octets.starts_with(b"GIF87a")
        || octets.starts_with(b"GIF89a")
        || (octets.len() >= 12 && &octets[..4] == b"RIFF" && &octets[8..12] == b"WEBP")
        || octets.starts_with(b"BM")
        || (octets.len() >= 12 && &octets[4..8] == b"ftyp")
}

/// Le nom à donner au fichier rapatrié : le dernier segment de son chemin.
pub fn nom_pour(url: &str) -> String {
    decouper(url)
        .and_then(|a| {
            a.chemin
                .split('?')
                .next()
                .and_then(|c| c.rsplit('/').next())
                .filter(|n| !n.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "image".to_string())
}

#[cfg(test)]
mod tests;
