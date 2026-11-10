//! Le pont de Wayland : ce qui s'éprouve sans compositeur. Le reste — copier vers une autre
//! application, coller, déposer, glisser dehors — se joue dans le vrai Glucose, avec de vrais
//! programmes de l'autre côté (`outils/pilote/essai_wayland.sh`, fiche 100).

use super::*;

/// **Un glisser parti de ce Glucose se reconnaît à sa marque**, et seulement le sien : celui d'un
/// autre Glucose (un autre processus) se dépose ici comme n'importe quel autre.
#[test]
fn test_un_glisser_parti_d_ici_se_reconnait_a_sa_marque() {
    let ici = format!("{MARQUE};n={}.7", std::process::id());
    let ailleurs = format!("{MARQUE};n={}.7", std::process::id() + 1);
    let types = |t: &[&str]| t.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert!(vient_d_ici(&types(&[LOT, &ici])));
    assert!(!vient_d_ici(&types(&[LOT, &ailleurs])), "un autre Glucose");
    assert!(!vient_d_ici(&types(&[LOT, PNG])), "sans marque");
    // Le numéro du processus est un préfixe : 12 n'est pas 123.
    let voisin = format!("{MARQUE};n={}1.7", std::process::id());
    assert!(!vient_d_ici(&types(&[&voisin])));
}
