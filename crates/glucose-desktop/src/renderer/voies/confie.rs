//! **Ce que le processeur confie à la carte** pour une image, et de quoi en tirer les pixels.
//!
//! Extrait de [`super`] quand le niveau des photos (NIVEAU-GPU-1) y est entré : six cent
//! quarante lignes là où la fiche 05 en admet six cents. La coupure tombe là où la question
//! change — là-bas *qu'est-ce que l'écran montre*, ici *qu'est-ce qui voyage vers la carte*.

use crate::present::scene_gpu::Pose;

/// **Ce que le processeur confie à la carte** pour une image.
///
/// Quatre listes, et rien d'autre : ce sont les seules choses que la voie graphique sait
/// produire aujourd'hui. Elles voyagent ensemble parce qu'elles viennent du **même** cadrage
/// — même vue, même culling — et que les séparer laisserait croire qu'on peut les calculer
/// à des instants différents.
#[derive(Debug, Default, Clone)]
pub struct Confie {
    /// Le fond, ou `None` quand la couche du dessous le porte encore.
    pub fond: Option<crate::present::fond_gpu::Fond>,
    /// Les lueurs des cartes visibles, dans l'ordre où le processeur les peindrait.
    pub lueurs: Vec<crate::present::lueurs_gpu::Lueur>,
    /// La forme des membranes visibles, dans l'ordre du modèle (MEMB-FORME-1) : leurs titres,
    /// poignées et réglettes restent dans la couche du dessous.
    pub membranes: Vec<glucose_core::membrane_forme::Membrane>,
    /// Le champ des flèches visibles, dans l'ordre du modèle (FLECHE-2) : leurs étiquettes,
    /// badges et poignées restent dans la couche du dessus.
    pub fleches: Vec<glucose_core::arrow::champ::Champ>,
    /// Où chaque photo visible se pose, à son rang — décodée ou **en chemin**.
    pub photos: Vec<(String, Pose)>,
    /// **Le niveau de chaque photo décodée**, par clé : son fichier, et combien de fois le
    /// niveau que la carte reçoit est réduit (NIVEAU-GPU-1).
    pub niveaux: std::collections::HashMap<String, (String, u32)>,
    /// **Ce qui remplace une photo dont le niveau voulu n'est pas tenu**, par la clé de la
    /// photo : la clé du meilleur niveau tenu, et sa pose (ETAGES-1). La carte ne le pose
    /// que si elle ne détient rien d'autre pour cette photo.
    pub replis: std::collections::HashMap<String, (String, Pose)>,
    /// Où chaque carte de texte visible se pose — **après** les photos, puisque les
    /// annotations passent au-dessus (COMPOSANT-1).
    pub cartes: Vec<(String, Pose)>,
    /// Ce qui se rend **à la demande**, quand la carte graphique ne connaît pas la clé : les
    /// cartes de texte, les photos en chemin.
    pub composants: Vec<crate::renderer::composants::Composant>,
    /// **Les lignes que la couche du dessus porte**, relevées après que tout y a été dessiné.
    ///
    /// Elles décident de ce qui s'efface et de ce qui part sur le bus (BANDE-1) : la chrome
    /// et les ornements n'occupent qu'un huitième de l'écran, et le reste n'a aucune raison
    /// d'être touché. Le relevé appartient à la peinture et non au renderer, parce que la
    /// chrome se dessine **après** lui — un relevé pris ici manquerait les docks.
    pub bandes_du_dessus: crate::present::bandes::Bandes,
    /// **Les lignes que la couche du dessous porte**, relevées de même (BANDE-2).
    ///
    /// Depuis que la forme des membranes est sur la carte, il n'y reste que leurs titres,
    /// poignées et réglettes, et les dossiers : quelques bandes, là où elle partait entière
    /// dès qu'une membrane était visible — quinze mébioctets effacés et envoyés à chaque image.
    pub bandes_du_dessous: crate::present::bandes::Bandes,
    /// Le liseré du passé, quand la Time Machine en montre un : la carte le pose par-dessus
    /// tout.
    pub lisere: Option<crate::present::lisere_gpu::Lisere>,
    /// La couche du dessous a-t-elle reçu de l'encre ?
    ///
    /// Quand elle n'en a pas — ni membrane, ni dossier, le fond étant sur la carte — elle est
    /// entièrement transparente, et **rien ne part** : ni ses quinze mébioctets, ni le dessin
    /// qui composerait du vide. C'est la passe elle-même qui répond, en comptant ce qu'elle
    /// dessine ; balayer les pixels coûterait un écran entier pour la même réponse.
    pub dessous_porte_quelque_chose: bool,
}

/// Une texture que la carte doit poser : ce qu'elle est, ce qu'elle montre, et où.
///
/// La **clé** change dès qu'un pixel change ; l'**identité** ne change jamais tant que c'est
/// le même composant. Les séparer est ce qui permet de poser l'ancien palier d'une carte
/// pendant que le nouveau se rend (CASCADE-2).
#[derive(Debug, Clone)]
pub struct APoser {
    /// Ce que la texture montre : une empreinte nouvelle est une texture nouvelle.
    pub cle: String,
    /// Ce que le composant **est** : stable d'un palier à l'autre, d'une frappe à l'autre.
    pub identite: String,
    /// Où la poser, à l'échelle de la vue.
    pub pose: Pose,
    /// **Ce que la carte pose à la place, tant qu'elle ne détient pas celle-ci** : pour la
    /// tuile d'un composant plus grand que l'écran, le même rectangle lu dans sa texture
    /// entière à un palier plus bas (DE-PRES-1). Un repli n'a pas de repli.
    pub repli: Option<Box<APoser>>,
}

impl Confie {
    /// **Tout ce que la carte pose comme texture**, dans l'ordre du modèle : les photos, puis
    /// les cartes de texte par-dessus.
    ///
    /// Une photo est sa propre identité : ses octets ne changent pas, donc sa clé non plus.
    /// Une carte de texte porte les deux, et elles diffèrent dès qu'elle change de palier.
    pub fn textures(&self) -> Vec<APoser> {
        // **Les identités se relèvent une fois, et se lisent ensuite.**
        //
        // La première version de CASCADE-2 cherchait l'identité de chaque clé par un parcours
        // linéaire des composants. Sur le document de l'utilisateur — quatre cent
        // quatre-vingt-deux cartes — cela fait deux cent trente-deux mille comparaisons de
        // chaînes par image, et la chronique du terrain les a chiffrées : le poste `textures`
        // restait à **treize millisecondes** sur les images de zoom alors que son budget en
        // vaut moins de deux, et que le rendu des textures, lui, était bien borné.
        //
        // C'est exactement ce que la fiche 05 interdit — la géométrie calculée deux fois —
        // sous une autre forme : une correspondance recalculée à chaque élément.
        let par_cle: std::collections::HashMap<&str, &crate::renderer::composants::Composant> =
            self.composants
                .iter()
                .map(|c| (c.cle.as_str(), c))
                .collect();
        self.photos
            .iter()
            .chain(self.cartes.iter())
            .map(|(cle, pose)| {
                let composant = par_cle.get(cle.as_str());
                // Une photo a pour identité son fichier : ses niveaux sont le même objet à
                // deux finesses, et l'ancien se pose pendant que le nouveau se téléverse.
                let identite = match (composant, self.niveaux.get(cle.as_str())) {
                    (Some(c), _) => c.identite.clone(),
                    (None, Some((src, _))) => src.clone(),
                    (None, None) => cle.clone(),
                };
                // Le repli d'une photo a son identité à lui : sous celle de la photo, il
                // remplacerait la texture nette que la carte a peut-être gardée (ETAGES-1).
                let repli = match composant {
                    Some(c) => c.repli.clone(),
                    None => self
                        .replis
                        .get(cle.as_str())
                        .map(|(cle_du_repli, pose)| APoser {
                            identite: format!("{identite}#repli"),
                            cle: cle_du_repli.clone(),
                            pose: *pose,
                            repli: None,
                        }),
                };
                APoser {
                    identite,
                    cle: cle.clone(),
                    pose: *pose,
                    repli: repli.map(Box::new),
                }
            })
            .collect()
    }

    /// Le composant qui porte cette clé, s'il y en a un : c'est lui qui sait se rendre.
    pub fn composant(&self, cle: &str) -> Option<&crate::renderer::composants::Composant> {
        self.composants.iter().find(|c| c.cle == cle)
    }

    /// **Les pixels d'une texture que la carte ne connaît pas** : un composant se rend, une
    /// photo prête le niveau que sa clé désigne — sans copie, puisqu'il est déjà là.
    ///
    /// Une seule définition, que la présentation, les bancs et les épreuves lisent : deux
    /// copies de cette règle finiraient par ne plus poser la même chose.
    pub fn pixels<'a>(
        &'a self,
        renderer: &'a crate::renderer::Renderer,
        cle: &str,
    ) -> Option<crate::present::scene_gpu::Pixels<'a>> {
        use crate::present::scene_gpu::Pixels;
        if let Some(composant) = self.composant(cle) {
            return composant.rendre(renderer.kit()).map(Pixels::Rendues);
        }
        // Un niveau offert au système ne se prête pas (ETAGES-1) : la texture attend qu'il
        // soit repris, et l'ancienne se pose en attendant.
        let (src, facteur) = self.niveaux.get(cle)?;
        let entree = renderer.magasin.cache.get(src)?;
        entree.pyramide.niveau(*facteur).map(Pixels::Pretes)
    }
}
