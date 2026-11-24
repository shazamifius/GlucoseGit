//! **Ce que le processeur confie à la carte** pour une image, et de quoi en tirer les pixels.
//!
//! Extrait de [`super`] quand le niveau des photos (NIVEAU-GPU-1) y est entré : six cent
//! quarante lignes là où la fiche 05 en admet six cents. La coupure tombe là où la question
//! change — là-bas *qu'est-ce que l'écran montre*, ici *qu'est-ce qui voyage vers la carte*.

use crate::present::scene_gpu::{Cle, Pose};

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
    /// **Le corps des zones visibles** (PRESET-1) : les mêmes rectangles que le processeur
    /// remplit, posés après le sol des régions et avant leurs frontières (REGION-1) — leurs textes
    /// restent dans la couche du dessous.
    pub zones: Vec<crate::renderer::zone::Rectangle>,
    /// **Les champs des régions** (REGION-1, MEMB-FORME-1) : leur sol, puis leurs frontières, de la
    /// plus grande à la plus petite, ce qui ne se voit pas écarté ; leurs poignées et réglettes
    /// restent dans la couche du dessous, leurs titres sont des textures ([`Self::titres`]).
    pub membranes: glucose_core::membrane_forme::Regions,
    /// Le champ des flèches visibles, dans l'ordre du modèle (FLECHE-2) : leurs étiquettes,
    /// badges et poignées restent dans la couche du dessus.
    pub fleches: Vec<glucose_core::arrow::champ::Champ>,
    /// Où chaque photo visible se pose, à son rang — décodée ou **en chemin**. Une photo décodée
    /// porte son niveau dans sa clé (NIVEAU-GPU-1), et, si ce niveau n'est pas tenu, le meilleur
    /// qui l'est en repli (ETAGES-1).
    pub photos: Vec<APoser>,
    /// Où chaque carte de texte visible se pose — **après** les photos, puisque les
    /// annotations passent au-dessus (COMPOSANT-1).
    pub cartes: Vec<APoser>,
    /// **Où chaque titre de membrane se pose** (TITRE-3, fiche 108) : sous les photos, juste
    /// après les frontières des régions. Ils viennent après les cartes dans la liste des
    /// textures — les photos doivent en rester les premières (LOT-1) —, et se posent à leur rang
    /// par [`Self::rangs_des_titres`].
    pub titres: Vec<APoser>,
    /// **Les photos de cette image sont celles que la carte tient déjà** (LOT-1) : `photos` est
    /// alors vide, et elles se posent à la similitude près.
    pub lot: Option<super::LotEnPlace>,
    /// **Cette image complète propose ses photos comme lot**, sous ce numéro : la carte le retient
    /// si elle les a toutes, à jour.
    pub lot_propose: Option<u64>,
    /// **La chrome permanente** — la bande, le fil d'Ariane, la minimap, le rail —, posée par la
    /// carte **après** la couche du dessus et avant les panneaux (CHROME-1) : des textures
    /// qu'elle ne reçoit qu'à leur changement, et le rectangle de la vue en filets.
    pub chrome: crate::ui::chrome::ChromeConfiee,
    /// **Les panneaux du dock**, posés par la carte **après** la couche du dessus (PANNEAUX-1) :
    /// leur clé change à chaque redessin, et leurs pixels viennent du cache des panneaux.
    pub panneaux: Vec<APoser>,
    /// Ce qui se rend **à la demande**, quand la carte graphique ne connaît pas la clé : les
    /// cartes de texte, les photos en chemin.
    pub composants: Vec<crate::renderer::composants::Composant>,
    /// **Ce que la carte dessine en formes** (fiche 73) : ce qui est plus grand que l'écran, une
    /// suite par composant, à son rang parmi les textures.
    pub formes: crate::present::formes_gpu::Formes,
    /// **Ce qui se prépare dans le temps qui reste, sans se poser** : la texture de chaque
    /// composant dessiné en formes, prête pour l'instant où, dézoomé, il retient dans l'écran.
    pub a_preparer: Vec<APoser>,
    /// **Les lignes que la couche du dessus porte**, relevées après que tout y a été dessiné.
    ///
    /// Elles décident de ce qui s'efface et de ce qui part sur le bus (BANDE-1) : depuis que la
    /// carte pose la chrome (CHROME-1), il n'y reste que les ornements de la scène — rien du
    /// tout sans sélection ni flèche —, et le reste n'a aucune raison d'être touché. Le relevé
    /// appartient à la peinture et non au renderer : un panneau qui ne tient pas dans sa
    /// texture s'y dessine **après** lui, et un relevé pris ici le manquerait.
    pub bandes_du_dessus: crate::present::bandes::Bandes,
    /// **Les lignes que la couche du dessous porte**, relevées de même (BANDE-2).
    ///
    /// Depuis que la forme des membranes est sur la carte, il n'y reste que leurs poignées et
    /// réglettes, les titres que la carte ne pose pas (TITRE-3), et les dossiers : quelques
    /// bandes, là où elle partait entière dès qu'une membrane était visible — quinze mébioctets
    /// effacés et envoyés à chaque image.
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
    pub cle: Cle,
    /// Ce que le composant **est** : stable d'un palier à l'autre, d'une frappe à l'autre.
    pub identite: Cle,
    /// Où la poser, à l'échelle de la vue.
    pub pose: Pose,
    /// **Ce que la carte pose à la place, tant qu'elle ne détient pas celle-ci** : pour la
    /// tuile d'un composant plus grand que l'écran, le même rectangle lu dans sa texture
    /// entière à un palier plus bas (DE-PRES-1). Un repli n'a pas de repli.
    pub repli: Option<Box<APoser>>,
}

impl Confie {
    /// **Tout ce que la carte pose comme texture** : les photos, puis les cartes de texte
    /// par-dessus, puis les titres des membranes (qui se posent, eux, sous les photos :
    /// [`Self::rangs_des_titres`]) — puis la chrome et les panneaux, qui passent après la couche
    /// du dessus.
    pub fn textures(&self) -> Vec<APoser> {
        self.photos
            .iter()
            .chain(&self.cartes)
            .chain(&self.titres)
            .chain(self.chrome.sous_le_cadre())
            .chain(self.chrome.sur_le_cadre())
            .chain(&self.panneaux)
            .cloned()
            .collect()
    }

    /// Combien des textures demandées passent **sous** la couche du dessus : les photos, les
    /// cartes et les titres. La chrome et les panneaux viennent après.
    pub fn sous_le_dessus(&self) -> usize {
        self.rangs_des_titres().end
    }

    /// **Les rangs des titres des membranes** parmi les textures demandées (TITRE-3) : après
    /// les photos et les cartes dans la liste, posés avant les photos à l'image.
    pub fn rangs_des_titres(&self) -> std::ops::Range<usize> {
        let debut = self.nombre_de_photos() + self.cartes.len();
        debut..debut + self.titres.len()
    }

    /// **Confie les titres des membranes** (TITRE-3) : leurs poses, et leurs composants, qui
    /// savent se rendre quand la carte ne connaît pas leur clé.
    pub fn confier_les_titres(&mut self, titres: Vec<crate::renderer::composants::Composant>) {
        for c in titres {
            self.titres.push(APoser {
                cle: (&c.cle).into(),
                identite: (&c.identite).into(),
                pose: c.pose,
                repli: None,
            });
            self.composants.push(c);
        }
    }

    /// Combien de photos cette image pose, qu'elle les dise ou que la carte les tienne (LOT-1).
    pub fn nombre_de_photos(&self) -> usize {
        self.lot.map_or(self.photos.len(), |lot| lot.nombre)
    }

    /// Combien des textures demandées passent **sous le rectangle de la vue** de la minimap :
    /// celles d'avant la couche du dessus, puis la bande, le fil et le fond de la minimap. Le
    /// rail, les panneaux et la décision viennent après (CHROME-1).
    pub fn sous_le_cadre(&self) -> usize {
        self.sous_le_dessus() + self.chrome.sous_le_cadre().len()
    }

    /// Le composant qui porte cette clé, s'il y en a un : c'est lui qui sait se rendre.
    pub fn composant(&self, cle: &Cle) -> Option<&crate::renderer::composants::Composant> {
        let nom = cle.nom()?;
        self.composants.iter().find(|c| c.cle == nom)
    }

    /// **Les pixels d'une texture que la carte ne connaît pas** : un composant se rend, une
    /// photo prête le niveau que sa clé désigne — sans copie, puisqu'il est déjà là.
    ///
    /// Une seule définition, que la présentation, les bancs et les épreuves lisent : deux
    /// copies de cette règle finiraient par ne plus poser la même chose.
    pub fn pixels<'a>(
        &'a self,
        renderer: &'a crate::renderer::Renderer,
        cle: &Cle,
    ) -> Option<crate::present::scene_gpu::Pixels<'a>> {
        use crate::present::scene_gpu::Pixels;
        // Un niveau offert au système ne se prête pas (ETAGES-1) : la texture attend qu'il
        // soit repris, et l'ancienne se pose en attendant.
        if let Cle::Niveau(numero, facteur) = cle {
            let entree = renderer.magasin.cache.par_numero(*numero)?;
            return entree.pyramide.niveau(*facteur).map(Pixels::Pretes);
        }
        if let Some(chrome) = cle.nom().and_then(|nom| self.chrome.pixels(nom)) {
            return Some(Pixels::Pretes(chrome));
        }
        self.composant(cle)?
            .rendre(renderer.kit())
            .map(Pixels::Rendues)
    }
}
