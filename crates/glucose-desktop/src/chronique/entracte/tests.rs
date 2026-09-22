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
    e.ouvrir(h.apres(0));
    e.imputer(h.apres(8), Poste::Main);
    e.imputer(h.apres(1), Poste::Depot);
    e.imputer(h.apres(0), Poste::Carte);
    e.imputer(h.apres(0), Poste::Entretien);
    e.imputer(h.apres(1), Poste::Systeme);
    e.fermer(h.apres(0), true);
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
    // La bascule de carte du terrain : 138 ms pour lâcher l'ancienne, 606 pour ouvrir la
    // nouvelle. La bannière du 22/09 au soir donne ces deux nombres, et la chronique de la
    // même session donne « 748,7 ms à ne pas dessiner » sur son pire gel.
    e.ouvrir(h.apres(0));
    e.imputer(h.apres(4), Poste::Carte);
    e.imputer(h.apres(744), Poste::Entretien);
    e.fermer(h.apres(1), true);

    assert_eq!(
        e.poste(Poste::Carte).2,
        744_000,
        "la carte porte sa bascule"
    );
    // **La preuve à l'envers.** Sans l'imputation au poste `carte`, ces 744 ms seraient
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
    e.ouvrir(h.apres(2_000));
    e.imputer(h.apres(4), Poste::Carte);
    e.imputer(h.apres(744), Poste::Entretien);
    e.fermer(h.apres(1), true);
    un_entracte_ordinaire(&mut e, &mut h);

    let (quand, total, parts) = e.pire();
    assert_eq!(total, Duration::from_millis(749));
    let carte = parts
        .iter()
        .find(|(p, _)| *p == Poste::Carte)
        .map(|(_, d)| *d)
        .expect("le poste de la carte");
    // **La ligne qui nomme un gel** : 744 des 749 millisecondes, et on sait quoi corriger.
    assert_eq!(carte, Duration::from_millis(744));
    assert!(
        quand >= Duration::from_millis(2_000),
        "le pire est date, parce qu'un gel a la premiere seconde est une initialisation : {quand:?}"
    );
}

#[test]
fn test_un_sommeil_ne_se_range_pas_comme_un_gel() {
    let mut e = Entracte::nouveau();
    let mut h = Horloge::neuve();
    un_entracte_ordinaire(&mut e, &mut h);
    // Personne ne touche à rien pendant deux minutes : l'application dort, et c'est sain.
    e.ouvrir(h.apres(0));
    e.imputer(h.apres(120_000), Poste::Entretien);
    e.fermer(h.apres(1), false);

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
    e.ouvrir(h.apres(0));
    // L'utilisateur choisit un fichier : la boucle est tenue ailleurs, et l'intervalle qui
    // suit n'est ni un gel ni un mouvement.
    e.oublier();
    e.imputer(h.apres(19_800), Poste::Entretien);
    e.fermer(h.apres(1), true);

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
    e.imputer(h.apres(50), Poste::Carte);
    e.fermer(h.apres(50), true);
    assert_eq!(
        e.comptes(),
        0,
        "aucun intervalle a decouper, donc rien a ranger"
    );
    assert_eq!(e.poste(Poste::Carte).2, 0);
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
