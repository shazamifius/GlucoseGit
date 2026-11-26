//! MCP-CHANGE — ce qui a changé dans un texte (fiche 180).

use super::*;

fn change(ligne: usize, avant: Option<&str>, apres: Option<&str>) -> Changement {
    let o = |t: Option<&str>| t.map(str::to_string);
    Changement {
        ligne,
        avant: o(avant),
        apres: o(apres),
    }
}

/// **Rejoue les changements sur `avant`** : ce qu'une IA qui le connaissait en déduit.
fn rejouer(avant: &str, changements: &[Changement]) -> String {
    let a: Vec<&str> = avant.split('\n').collect();
    let compte = |t: &Option<String>| t.as_ref().map_or(0, |t| t.split('\n').count());
    let (mut rendu, mut i) = (Vec::<String>::new(), 0);
    for c in changements {
        let communes = c.ligne - 1 - rendu.len();
        rendu.extend(a[i..i + communes].iter().map(|l| l.to_string()));
        i += communes + compte(&c.avant);
        rendu.extend(
            c.apres
                .iter()
                .flat_map(|t| t.split('\n').map(str::to_string)),
        );
    }
    rendu.extend(a[i..].iter().map(|l| l.to_string()));
    rendu.join("\n")
}

/// **Sa réponse sous une question** : seulement elle.
#[test]
fn mcp_change_une_reponse_ajoutee_seule() {
    let avant = "**Une question**\n\nTon avis ?\n\nTa réponse :";
    let apres = format!("{avant}\noui, vraiment");
    let c = changements(avant, &apres).expect("plus court que le texte");
    assert_eq!(c, vec![change(6, None, Some("oui, vraiment"))]);
}

/// **Deux retouches loin l'une de l'autre** : deux changements, chacun à sa ligne — pas tout ce
/// qui les sépare.
#[test]
fn mcp_change_deux_retouches_eloignees() {
    let lignes: Vec<String> = (1..=20).map(|k| format!("ligne {k}")).collect();
    let avant = lignes.join("\n");
    let mut l = lignes.clone();
    l[1] = "ligne deux, récrite".into();
    l.insert(17, "une ligne neuve".into());
    let apres = l.join("\n");
    let c = changements(&avant, &apres).expect("plus court");
    assert_eq!(
        c,
        vec![
            change(2, Some("ligne 2"), Some("ligne deux, récrite")),
            change(18, None, Some("une ligne neuve")),
        ]
    );
}

/// **Les lignes vides se répètent** : un paragraphe inséré entre deux autres reste un seul
/// changement ; une ligne vide retirée se dit `Some("")`.
#[test]
fn mcp_change_les_lignes_vides_ne_brouillent_rien() {
    let avant = "Premier.\n\nDeuxième.\n\nTroisième.";
    let apres = "Premier.\n\nDeuxième.\n\nInséré.\n\nTroisième.";
    let c = changements(avant, apres).expect("plus court");
    assert_eq!(c, vec![change(5, None, Some("Inséré.\n"))]);
    assert_eq!(rejouer(avant, &c), apres);
    let c = changements("a\n\nb\nc\nd\ne", "a\nb\nc\nd\ne").expect("plus court");
    assert_eq!(c, vec![change(2, Some(""), None)]);
    // Des débuts communs qui ne sont pas uniques s'apparient quand même.
    let c = changements("-\n-\n-\na", "-\n-\n-\nb").expect("plus court");
    assert_eq!(c, vec![change(4, Some("a"), Some("b"))]);
}

/// **Seules les lignes uniques ancrent, et chaque région se reprend** : une ligne répétée ne tire
/// pas l'appariement de travers ; après la dernière ancre, les lignes communes s'apparient encore.
#[test]
fn mcp_change_seules_les_lignes_uniques_ancrent() {
    let c = tous("y\nx\nz\nx", "x\nz");
    assert_eq!(
        c,
        vec![change(1, Some("y"), None), change(3, Some("x"), None)]
    );
    let c = tous("p\nU\n-\n-\nq", "r\nU\n-\n-\ns");
    assert_eq!(
        c,
        vec![
            change(1, Some("p"), Some("r")),
            change(5, Some("q"), Some("s"))
        ]
    );
}

/// **Un texte récrit** : rien de plus court que lui, il se relit entier.
#[test]
fn mcp_change_un_texte_recrit_se_relit_entier() {
    assert_eq!(changements("un\ndeux\ntrois", "quatre\ncinq\nsix"), None);
    assert_eq!(changements("", "x"), None);
}

/// **Rejoués, les changements rendent le texte d'après** : sur des textes tirés au hasard (une
/// suite fixe), lignes répétées, déplacées, retirées et ajoutées.
#[test]
fn mcp_change_rejoues_ils_rendent_le_texte() {
    let mut graine: u64 = 0x2611_2026;
    let mut tirer = |n: u64| {
        graine = graine
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (graine >> 33) % n
    };
    for _ in 0..300 {
        let mots = ["", "a", "b", "c", "d", "e", "f", "g"];
        let mut a: Vec<String> = Vec::new();
        for _ in 0..tirer(30) {
            a.push(format!("{}{}", mots[tirer(8) as usize], tirer(6)));
        }
        let mut b = a.clone();
        for _ in 0..tirer(6) {
            let k = tirer(b.len() as u64 + 1) as usize;
            match tirer(3) {
                0 if k < b.len() => drop(b.remove(k)),
                1 => b.insert(k, format!("neuf{}", tirer(4))),
                _ if k < b.len() => b[k] = mots[tirer(8) as usize].to_string(),
                _ => {}
            }
        }
        let (avant, apres) = (a.join("\n"), b.join("\n"));
        let c = tous(&avant, &apres);
        assert_eq!(rejouer(&avant, &c), apres, "{avant:?} → {apres:?} : {c:?}");
    }
}

/// **Le plus long ordre commun des ancres** : celles qui ont changé de place n'en sont plus.
#[test]
fn mcp_change_la_plus_longue_suite_croissante() {
    let p = [(0, 3), (1, 0), (2, 1), (3, 4), (4, 2), (5, 5)];
    assert_eq!(
        plus_longue_croissante(&p),
        vec![(1, 0), (2, 1), (4, 2), (5, 5)]
    );
    assert!(plus_longue_croissante(&[]).is_empty());
}
