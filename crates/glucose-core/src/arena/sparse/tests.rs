//! Les lois des tables creuses, dont celle qui décide de leur emploi.

use super::*;

fn id(i: usize) -> NodeId {
    NodeId::from_index(i)
}

/// La formule qui décide entre dense et creux, vérifiée sur les quatre tailles de valeur du
/// tableau de l'en-tête. Elle ne comporte aucun seuil réglable : c'est une égalité de coûts.
#[test]
fn test_la_formule_decide_sans_aucun_seuil_a_regler() {
    // Le seuil exact vaut s / (4 + s) : c'est le point où les deux dispositions coûtent pareil.
    for (taille, seuil) in [(1usize, 0.2), (4, 0.5), (8, 8.0 / 12.0), (40, 40.0 / 44.0)] {
        let n = 1_000_000usize;
        let k_sous = (n as f64 * seuil).floor() as usize - 1;
        let k_sur = (n as f64 * seuil).ceil() as usize + 1;
        assert!(prefer_sparse(k_sous, n, taille), "s={taille}, k/n juste sous {seuil}");
        assert!(!prefer_sparse(k_sur, n, taille), "s={taille}, k/n juste au-dessus de {seuil}");
    }

    // Les cas dégénérés : aucun porteur gagne toujours, tous les porteurs perdent toujours.
    assert!(prefer_sparse(0, 1_000_000, 8));
    assert!(!prefer_sparse(1_000_000, 1_000_000, 8));
    assert!(!prefer_sparse(5, 0, 8), "sur un document vide, rien ne vaut mieux que rien");
}

/// Une table qui a trop grossi le dit elle-même : c'est la même formule, posée sur son propre
/// contenu, plutôt qu'une vérification qu'il faudrait penser à faire.
#[test]
fn test_une_table_trop_grosse_le_dit_elle_meme() {
    let mut t: Sparse<u32> = Sparse::new();
    for i in 0..400 {
        t.set(id(i), i as u32);
    }
    // 400 porteurs sur 1 000 nœuds, valeur de 4 octets : sous le seuil de 50 %, le creux gagne.
    assert!(!t.would_fit_denser(1_000));
    // Les mêmes 400 porteurs sur 700 nœuds : au-dessus, le dense gagne.
    assert!(t.would_fit_denser(700));
    assert_eq!(t.bytes(), 400 * 8);
}

/// Ce qu'on pose est ce qu'on relit, quel que soit l'ordre d'écriture, et les identifiants
/// restent triés — c'est ce qui rend la lecture binaire.
#[test]
fn test_la_table_reste_triee_quel_que_soit_l_ordre_d_ecriture() {
    let mut t: Sparse<&str> = Sparse::new();
    for (i, v) in [(9usize, "neuf"), (0, "zéro"), (4, "quatre"), (1, "un"), (7, "sept")] {
        assert_eq!(t.set(id(i), v), None, "premier passage sur {i}");
    }
    t.check().unwrap();
    assert_eq!(t.len(), 5);

    for (i, v) in [(0usize, "zéro"), (1, "un"), (4, "quatre"), (7, "sept"), (9, "neuf")] {
        assert_eq!(t.get(id(i)), Some(&v));
        assert!(t.has(id(i)));
    }
    for absent in [2usize, 3, 5, 6, 8, 10, 1_000] {
        assert_eq!(t.get(id(absent)), None);
        assert!(!t.has(id(absent)));
    }
    assert_eq!(
        t.iter().map(|(i, v)| (i.index(), *v)).collect::<Vec<_>>(),
        vec![(0, "zéro"), (1, "un"), (4, "quatre"), (7, "sept"), (9, "neuf")]
    );
}

/// Poser deux fois le même nœud remplace la valeur et rend l'ancienne, sans créer de doublon.
#[test]
fn test_reposer_un_noeud_remplace_sans_dupliquer() {
    let mut t: Sparse<u32> = Sparse::new();
    assert_eq!(t.set(id(3), 10), None);
    assert_eq!(t.set(id(3), 20), Some(10));
    assert_eq!(t.len(), 1);
    assert_eq!(t.get(id(3)), Some(&20));
    *t.get_mut(id(3)).unwrap() += 1;
    assert_eq!(t.get(id(3)), Some(&21));
    t.check().unwrap();
}

/// Retirer un attribut rend sa valeur et laisse la table triée ; retirer un absent ne fait rien.
#[test]
fn test_retirer_laisse_la_table_triee() {
    let mut t: Sparse<u32> = Sparse::new();
    for i in 0..10 {
        t.set(id(i), i as u32 * 100);
    }
    assert_eq!(t.remove(id(5)), Some(500));
    assert_eq!(t.remove(id(5)), None, "retirer deux fois ne rend rien");
    assert_eq!(t.remove(id(0)), Some(0), "le premier");
    assert_eq!(t.remove(id(9)), Some(900), "le dernier");
    assert_eq!(t.len(), 7);
    assert_eq!(
        t.iter().map(|(i, _)| i.index()).collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 6, 7, 8]
    );
    t.check().unwrap();
}

/// Charger un document pose les attributs dans l'ordre des nœuds : ce n'est alors qu'une suite
/// d'ajouts en queue, sans aucun décalage.
#[test]
fn test_charger_dans_l_ordre_n_est_qu_une_suite_d_ajouts() {
    let mut t: Sparse<u64> = Sparse::with_capacity(10_000);
    for i in 0..10_000 {
        assert_eq!(t.set(id(i), i as u64), None);
    }
    t.check().unwrap();
    assert_eq!(t.len(), 10_000);
    assert_eq!(t.get(id(9_999)), Some(&9_999));
    assert_eq!(t.bytes(), 10_000 * 12);
}

/// L'absence de nœud ne porte pas d'attribut : `NodeId::NONE` est refusé plutôt que rangé au
/// bout de la table, où il ferait mentir tout le reste.
#[test]
fn test_l_absence_de_noeud_ne_porte_pas_d_attribut() {
    let mut t: Sparse<u32> = Sparse::new();
    t.set(id(1), 1);
    assert_eq!(t.set(NodeId::NONE, 42), None);
    assert_eq!(t.get(NodeId::NONE), None);
    assert!(!t.has(NodeId::NONE));
    assert_eq!(t.len(), 1);
    t.check().unwrap();
}

/// Une table vide se comporte comme une table, pas comme un cas particulier.
#[test]
fn test_une_table_vide_se_comporte_comme_une_table() {
    let t: Sparse<u32> = Sparse::new();
    assert!(t.is_empty());
    assert_eq!(t.len(), 0);
    assert_eq!(t.bytes(), 0);
    assert_eq!(t.get(id(0)), None);
    assert_eq!(t.iter().count(), 0);
    t.check().unwrap();
}

/// La vérification attrape une incohérence plutôt que de la laisser passer.
#[test]
fn test_la_verification_attrape_une_incoherence() {
    let mut t: Sparse<u32> = Sparse::new();
    t.set(id(1), 1);
    t.set(id(2), 2);
    t.check().unwrap();

    t.vals.push(3); // une valeur sans identifiant
    assert!(t.check().unwrap_err().contains("pour 3 valeurs"));
    t.vals.pop();

    t.ids[1] = id(0); // ordre rompu
    assert!(t.check().unwrap_err().contains("croissants"));
    t.ids[1] = id(1); // doublon
    assert!(t.check().unwrap_err().contains("croissants"));
}
