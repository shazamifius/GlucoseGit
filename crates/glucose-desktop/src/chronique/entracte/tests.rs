use super::*;

/// Une horloge synthétique : des instants exacts, sans rien attendre.
///
/// Un test qui dormirait sept dixièmes de seconde pour mesurer sept dixièmes de seconde
/// mesurerait surtout l'ordonnanceur, et la fiche 25 § 4.3 dit ce que coûte un test dont la
/// preuve dépend de la charge de la machine.
struct Horloge {
    origine: Instant,
    ms: u64,
}

impl Horloge {
    fn neuve() -> Self {
        Self {
            origine: Instant::now(),
            ms: 0,
        }
    }

    /// L'instant courant, après avoir avancé de `ms` millisecondes.
    fn apres(&mut self, ms: u64) -> Instant {
        self.ms += ms;
        self.origine + Duration::from_millis(self.ms)
    }
}

/// Le tour de boucle du terrain : l'image part, Windows attend, la main parle, l'entretien
/// passe, et le rendu reprend.
fn un_entracte_ordinaire(e: &mut Entracte, h: &mut Horloge) {
    let ouverte = h.apres(0);
    e.ouvrir(ouverte);
    e.imputer(h.apres(8), Poste::Main);
    e.imputer(h.apres(1), Poste::Depot);
    e.imputer(h.apres(0), Poste::Entretien);
    e.imputer(h.apres(1), Poste::Systeme);
    e.fermer(h.apres(0), Some(ouverte));
}

#[test]
fn test_les_parts_se_somment_a_l_entracte_entier() {
    let mut e = Entracte::nouveau();
    let mut h = Horloge::neuve();
    un_entracte_ordinaire(&mut e, &mut h);

    let (_, _, pire_total) = e.total();
    let somme: u32 = Poste::TOUS.iter().map(|p| e.poste(*p).2).sum();
    // **C'est la garantie centrale du module**, et elle interdit qu'un poste absorbe ce qui le
    // précède : les quatre marques mal posées de ce dépôt — `occlusion`, `recolte`, `blit`,
    // `minimap` — auraient toutes été attrapées par cette égalité.
    assert_eq!(
        somme, pire_total,
        "les cinq parts doivent valoir l'entracte entier, sinon du temps se perd ou se compte deux fois"
    );
    assert_eq!(pire_total, 10_000, "8 + 1 + 1 millisecondes");
}

#[test]
fn test_le_temps_d_un_poste_ne_tombe_pas_dans_celui_d_a_cote() {
    let mut e = Entracte::nouveau();
    let mut h = Horloge::neuve();
    // Un événement qui tient le fil sept dixièmes de seconde — ce que faisait la bascule de
    // carte du 22/09 au soir, 138 ms pour lâcher l'ancienne et 606 pour ouvrir la nouvelle,
    // avant qu'ARBITRE-4 ne la retire de la session.
    let ouverte = h.apres(0);
    e.ouvrir(ouverte);
    e.imputer(h.apres(4), Poste::Main);
    e.imputer(h.apres(744), Poste::Entretien);
    e.fermer(h.apres(1), Some(ouverte));

    assert_eq!(
        e.poste(Poste::Main).2,
        744_000,
        "la main porte ce qu'elle a tenu"
    );
    // **La preuve à l'envers.** Sans l'imputation au poste de la main, ces 744 ms seraient
    // restées dans `systeme` — l'entracte entier aurait la même durée, et le rapport aurait
    // dit « attendre Windows 748 ms » sur un gel que Glucose s'inflige lui-même. C'est
    // exactement la forme des quatre marques mal posées, et c'est ce que ce test interdit.
    assert_eq!(
        e.poste(Poste::Systeme).2,
        4_000,
        "le systeme ne porte que ce qu'il a vraiment tenu"
    );
    assert_eq!(e.poste(Poste::Entretien).2, 1_000);
}

#[test]
fn test_le_pire_entracte_garde_sa_decomposition() {
    let mut e = Entracte::nouveau();
    let mut h = Horloge::neuve();
    un_entracte_ordinaire(&mut e, &mut h);
    // Puis la bascule, bien plus tard dans la session.
    let ouverte = h.apres(2_000);
    e.ouvrir(ouverte);
    e.imputer(h.apres(4), Poste::Main);
    e.imputer(h.apres(744), Poste::Entretien);
    e.fermer(h.apres(1), Some(ouverte));
    un_entracte_ordinaire(&mut e, &mut h);

    let (gels, tus) = e.gels();
    assert_eq!(tus, 0);
    assert_eq!(
        gels.len(),
        1,
        "les entractes de dix millisecondes ne sont pas des gels"
    );
    assert_eq!(gels[0].total, Duration::from_millis(749));
    // **La ligne qui nomme un gel** : 744 des 749 millisecondes, et on sait quoi corriger.
    assert_eq!(
        gels[0].parts().first().copied(),
        Some((Poste::Main, Duration::from_millis(744)))
    );
    assert!(
        gels[0].a >= Duration::from_millis(2_000),
        "le gel est date, parce qu'un gel a la premiere seconde est une initialisation : {:?}",
        gels[0].a
    );
}

/// **ENTRACTE-2** — le gel du démarrage ne cache plus les autres.
///
/// C'est la première session réelle qui l'a montré : le pire entracte était le démarrage,
/// 607 ms à la 0,6ᵉ seconde, et la section ne décomposait que lui pendant que le verdict
/// comptait 2 314 ms perdues. Chaque attente qui a mangé au moins une image est gardée, et la
/// plus longue vient en tête.
#[test]
fn test_le_gel_du_demarrage_ne_cache_pas_les_autres() {
    let mut e = Entracte::nouveau();
    let mut h = Horloge::neuve();
    let ouverte = h.apres(0);
    e.ouvrir(ouverte);
    e.fermer(h.apres(607), Some(ouverte));
    un_entracte_ordinaire(&mut e, &mut h);
    let ouverte = h.apres(4_000);
    e.ouvrir(ouverte);
    e.imputer(h.apres(0), Poste::Main);
    e.imputer(h.apres(163), Poste::Systeme);
    e.fermer(h.apres(1), Some(ouverte));

    let (gels, _) = e.gels();
    assert_eq!(
        gels.len(),
        2,
        "le demarrage ET l'evenement de 163 ms : {gels:?}"
    );
    assert_eq!(gels[0].total, Duration::from_millis(607));
    assert_eq!(
        gels[1].parts().first().copied(),
        Some((Poste::Main, Duration::from_millis(163))),
        "le second gel est nomme, lui aussi"
    );
}

#[test]
fn test_un_sommeil_ne_se_range_pas_comme_un_gel() {
    let mut e = Entracte::nouveau();
    let mut h = Horloge::neuve();
    un_entracte_ordinaire(&mut e, &mut h);
    // Personne ne touche à rien pendant deux minutes : l'application dort, et c'est sain.
    let ouverte = h.apres(0);
    e.ouvrir(ouverte);
    e.imputer(h.apres(120_000), Poste::Entretien);
    e.fermer(h.apres(1), None);

    assert_eq!(e.comptes(), 1, "le sommeil n'est pas un entracte de plus");
    assert_eq!(
        e.total().2,
        10_000,
        "deux minutes de repos ne deviennent pas le pire gel de la session"
    );
}

#[test]
fn test_un_dialogue_natif_s_oublie() {
    let mut e = Entracte::nouveau();
    let mut h = Horloge::neuve();
    un_entracte_ordinaire(&mut e, &mut h);
    let ouverte = h.apres(0);
    e.ouvrir(ouverte);
    // L'utilisateur choisit un fichier : la boucle est tenue ailleurs, et l'intervalle qui
    // suit n'est ni un gel ni un mouvement.
    e.oublier();
    e.imputer(h.apres(19_800), Poste::Entretien);
    e.fermer(h.apres(1), Some(ouverte));

    assert_eq!(e.comptes(), 1);
    assert_eq!(
        e.total().2,
        10_000,
        "l'ouverture de fichier ne fait pas un gel"
    );
}

#[test]
fn test_hors_d_un_entracte_rien_ne_se_mesure() {
    let mut e = Entracte::nouveau();
    let mut h = Horloge::neuve();
    // Pendant le rendu, `imputer` peut être appelé par un chemin qui ne sait pas où il est.
    let jamais_ouverte = h.apres(0);
    e.imputer(h.apres(50), Poste::Main);
    e.fermer(h.apres(50), Some(jamais_ouverte));
    assert_eq!(
        e.comptes(),
        0,
        "aucun intervalle a decouper, donc rien a ranger"
    );
    assert_eq!(e.poste(Poste::Main).2, 0);
}

#[test]
fn test_chaque_poste_a_un_nom_et_un_indice_distincts() {
    // Un poste ajouté sans son indice écraserait silencieusement un autre : les deux
    // tableaux seraient justes, et la mesure fausse.
    let mut indices: Vec<usize> = Poste::TOUS.iter().map(|p| p.indice()).collect();
    indices.sort_unstable();
    indices.dedup();
    assert_eq!(indices.len(), Poste::COMBIEN);
    let mut noms: Vec<&str> = Poste::TOUS.iter().map(|p| p.nom()).collect();
    noms.sort_unstable();
    noms.dedup();
    assert_eq!(noms.len(), Poste::COMBIEN);
}

/// **GEL-1** — une pause suivie d'un dépôt n'est pas un gel.
///
/// C'est la session économe du 23/09 : l'utilisateur quitte Glucose pour son navigateur, y
/// reste onze secondes, glisse une image. La première version rangeait les onze secondes en
/// « attendre Windows » — premier du verdict à x310. L'image n'est devenue nécessaire qu'au
/// dépôt ; seul ce qui suit l'échéance se range.
#[test]
fn test_une_pause_avant_un_depot_n_est_pas_un_gel() {
    let mut e = Entracte::nouveau();
    let mut h = Horloge::neuve();
    let ouverte = h.apres(0);
    e.ouvrir(ouverte);
    let depot = h.apres(11_000);
    e.imputer(depot, Poste::Depot);
    e.imputer(h.apres(1), Poste::Systeme);
    e.fermer(h.apres(1), Some(depot));

    assert!(
        e.gels().0.is_empty(),
        "deux millisecondes apres le depot ne sont pas un gel : {:?}",
        e.gels().0
    );
    assert_eq!(e.total().2, 2_000);
    // **La preuve a l'envers** : avec l'echeance posee a l'ouverture -- ce que la premiere
    // version faisait en rangeant tout l'entracte --, les onze secondes redeviennent un gel.
    let mut ancienne = Entracte::nouveau();
    let mut h = Horloge::neuve();
    let ouverte = h.apres(0);
    ancienne.ouvrir(ouverte);
    ancienne.imputer(h.apres(11_000), Poste::Depot);
    ancienne.fermer(h.apres(2), Some(ouverte));
    assert_eq!(
        ancienne.gels().0.len(),
        1,
        "l'ancienne lecture comptait un gel"
    );
}

/// **GEL-1** — une image due que Windows ne laisse pas rendre, elle, est un gel.
///
/// Le cas inverse, sans lequel le précédent ne prouverait rien : une échéance qui ne
/// compterait jamais rien passerait le test de la pause.
#[test]
fn test_une_image_due_et_retenue_par_windows_est_un_gel() {
    let mut e = Entracte::nouveau();
    let mut h = Horloge::neuve();
    let ouverte = h.apres(0);
    e.ouvrir(ouverte);
    let due = h.apres(1);
    e.fermer(h.apres(700), Some(due));

    let (gels, _) = e.gels();
    assert_eq!(gels.len(), 1);
    assert_eq!(
        gels[0].parts().first().copied(),
        Some((Poste::Systeme, Duration::from_millis(700))),
        "l'image etait due, et Windows a tenu le fil sept dixiemes de seconde"
    );
}
