use super::*;

const OCTETS: usize = 3 * 1024 * 1024 + 123;

fn motif(i: usize) -> u8 {
    (i as u8).wrapping_mul(31).wrapping_add(7)
}

#[test]
fn test_une_region_neuve_est_nulle_et_porte_ce_qu_on_a_demande() {
    let r = Tenue::nouvelle(OCTETS).expect("trois mégaoctets se trouvent toujours");
    assert_eq!(r.octets().len(), OCTETS);
    assert!(r.octets().iter().all(|&o| o == 0));
    assert!(Tenue::nouvelle(0).is_none(), "une région vide ne se demande pas");
}

/// **Ce qui revient est ce qui est parti, ou rien** : jamais un contenu altéré.
///
/// Le système peut jeter une région offerte ; il ne l'a pas fait ici, sur une machine qui
/// ne manque pas de place — si ce test tombe sur `None`, c'est la machine qui était pleine,
/// et le code qui l'a dit, ce qui est juste.
#[test]
fn test_une_region_offerte_puis_reprise_rend_ses_octets() {
    let mut r = Tenue::nouvelle(OCTETS).expect("trois mégaoctets se trouvent toujours");
    for (i, o) in r.octets_mut().iter_mut().enumerate() {
        *o = motif(i);
    }
    let offerte = r.offrir();
    assert_eq!(offerte.longueur(), OCTETS);
    let reprise = offerte
        .reprendre()
        .expect("la machine avait de la place : la région devait revenir intacte");
    assert!(
        reprise.octets().iter().enumerate().all(|(i, &o)| o == motif(i)),
        "une région reprise a rendu d'autres octets que ceux qui étaient partis"
    );
}

/// **Offrir retire les pages de la mémoire de travail**, et c'est tout l'objet du geste : le
/// gestionnaire des tâches ne les compte plus, et le système peut s'en servir.
///
/// On demande au système, page par page, lesquelles sont présentes : c'est déterministe, là où
/// la mémoire de travail du processus entier bougerait avec les autres épreuves qui tournent en
/// même temps.
#[cfg(windows)]
#[test]
fn test_une_region_offerte_quitte_la_memoire_de_travail() {
    let mut r = Tenue::nouvelle(OCTETS).expect("trois mégaoctets se trouvent toujours");
    r.octets_mut().fill(1);
    let offerte = r.offrir();
    let (debut, longueur) = offerte.adresse();
    let presentes = pages_presentes(debut, longueur);
    assert_eq!(
        presentes, 0,
        "{presentes} pages offertes sont encore dans la mémoire de travail"
    );
    let reprise = offerte.reprendre().expect("la machine avait de la place");
    assert!(reprise.octets().iter().all(|&o| o == 1));
    let (debut, longueur) = (reprise.octets().as_ptr(), reprise.octets().len());
    assert!(
        pages_presentes(debut, longueur) > 0,
        "une région relue doit être revenue en mémoire — sinon l'épreuve ne voit rien"
    );
}

#[cfg(windows)]
fn pages_presentes(debut: *const u8, longueur: usize) -> usize {
    use windows::Win32::System::ProcessStatus::{
        QueryWorkingSetEx, PSAPI_WORKING_SET_EX_INFORMATION,
    };
    use windows::Win32::System::Threading::GetCurrentProcess;
    const PAGE: usize = 4096;
    let mut pages: Vec<PSAPI_WORKING_SET_EX_INFORMATION> = (0..longueur.div_ceil(PAGE))
        .map(|k| PSAPI_WORKING_SET_EX_INFORMATION {
            VirtualAddress: debut.wrapping_add(k * PAGE).cast_mut().cast(),
            ..Default::default()
        })
        .collect();
    let taille = std::mem::size_of_val(pages.as_slice()) as u32;
    // Sûr : le tableau est local et dimensionné par sa propre taille.
    unsafe { QueryWorkingSetEx(GetCurrentProcess(), pages.as_mut_ptr().cast(), taille) }
        .expect("le système dit toujours quelles pages il tient");
    // Le bit 0 est `Valid` : la page est dans la mémoire de travail.
    pages
        .iter()
        .filter(|p| unsafe { p.VirtualAttributes.Flags } & 1 == 1)
        .count()
}

/// **Moins d'une page ne s'offre pas, et revient toujours** : le système n'offre que des
/// pages entières, et en consacrer une à quelques octets gaspillerait le reste.
#[test]
fn test_une_region_plus_petite_qu_une_page_revient_toujours() {
    let mut r = Tenue::nouvelle(100).expect("cent octets se trouvent toujours");
    r.octets_mut().fill(9);
    let reprise = r.offrir().reprendre().expect("une petite région ne se perd jamais");
    assert_eq!(reprise.octets(), &[9u8; 100][..]);
}
