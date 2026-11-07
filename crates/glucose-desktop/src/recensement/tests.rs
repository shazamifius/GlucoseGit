//! Le fil, sur de vrais fichiers écrits par le vrai dépôt.

use super::*;
use crate::epreuve::Dossier;
use glucose_core::store::Store;
use glucose_core::types::Annotation;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn allure() -> (String, String) {
    ("#60a5fa".to_string(), "SCI".to_string())
}

/// Écrit un vrai `.glucose` : un document dont un nœud porte ce domaine, sous cette identité.
fn ecrire(chemin: &Path, noeud: &str, domaine: &glucose_core::types::Domain) {
    let mut store = Store::new("P");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::text(noeud, 0.0, 0.0, "Un texte."));
    let id = store.adopter_un_domaine(domaine).expect("adopté");
    store
        .try_assign_domain_to_node(&board, noeud, &id, 1.0)
        .expect("porté");
    let octets = glucose_core::persist::encode(
        &store.project,
        &glucose_core::types::AssetStore::default(),
        0,
    );
    std::fs::write(chemin, octets).expect("écrit");
}

fn eau() -> glucose_core::types::Domain {
    let mut s = Store::new("P");
    let id = s.creer_un_domaine("Eau", allure(), 5).expect("créé");
    s.domain(&id).expect("là").clone()
}

/// Attend que la passe en route soit finie, puis rend ce qu'elle a trouvé. Une passe qui ne finit
/// pas fait tomber l'épreuve, elle ne la fige pas.
fn attendre_la_fin(r: &mut Recenseur) {
    let debut = Instant::now();
    while r.en_cours {
        assert!(
            debut.elapsed() < Duration::from_secs(20),
            "le fil ne finit pas"
        );
        r.relever();
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn recenseur(dossier: &Path) -> (Recenseur, Arc<AtomicUsize>) {
    let reveils = Arc::new(AtomicUsize::new(0));
    let compte = Arc::clone(&reveils);
    let reveil: Reveil = Arc::new(move || {
        compte.fetch_add(1, Ordering::SeqCst);
    });
    let mut r = Recenseur::default();
    r.demarrer(dossier, Vec::new(), reveil);
    attendre_la_fin(&mut r);
    (r, reveils)
}

/// **Deux fichiers rangés dans deux dossiers, un seul domaine**, et ce qu'on connaît survit au
/// relancement, parce que la liste est écrite.
#[test]
fn deux_fichiers_connus_portent_le_meme_domaine() {
    let glucose = Dossier::nouveau("recense-glucose");
    let (ici, la) = (Dossier::nouveau("recense-a"), Dossier::nouveau("recense-b"));
    let (a, b) = (ici.join("a.glucose"), la.join("b.glucose"));
    let d = eau();
    ecrire(&a, "mouille", &d);
    ecrire(&b, "jeu", &d);

    let (mut r, reveils) = recenseur(&glucose);
    assert!(r.etat.vue(None).is_empty(), "rien de connu : rien de lu");
    assert_eq!(
        reveils.load(Ordering::SeqCst),
        0,
        "rien de neuf : aucun réveil"
    );

    r.connaitre(&a);
    r.connaitre(&b);
    attendre_la_fin(&mut r);
    let vue = r.etat.vue(None);
    assert_eq!(vue.len(), 1, "{vue:?}");
    assert_eq!(vue[0].noeuds(), 2);
    assert_eq!(vue[0].fichiers.len(), 2);
    assert!(
        reveils.load(Ordering::SeqCst) > 0,
        "le neuf réveille la boucle"
    );

    let (r2, _) = recenseur(&glucose);
    drop(r2);
    let (mut r3, _) = recenseur(&glucose);
    r3.relancer();
    attendre_la_fin(&mut r3);
    assert_eq!(r3.etat.vue(None).len(), 1, "la liste survit au relancement");
}

/// **Un fichier déplacé emporte ses domaines** : dans un dossier connu, il se retrouve seul ; hors de
/// tout dossier connu, il sort de la vue, et revient quand on l'ouvre.
#[test]
fn un_fichier_deplace_garde_ses_domaines() {
    let glucose = Dossier::nouveau("deplace-glucose");
    let (ici, loin) = (
        Dossier::nouveau("deplace-a"),
        Dossier::nouveau("deplace-loin"),
    );
    let a = ici.join("a.glucose");
    ecrire(&a, "mouille", &eau());
    let (mut r, _) = recenseur(&glucose);
    r.connaitre(&a);
    attendre_la_fin(&mut r);
    assert_eq!(
        r.etat.vue(None)[0].fichiers[0].chemin,
        a.display().to_string()
    );

    // Dans le même dossier : renommé, il se retrouve sans qu'on l'ouvre.
    let renomme = ici.join("renomme.glucose");
    std::fs::rename(&a, &renomme).expect("renommé");
    r.relancer();
    attendre_la_fin(&mut r);
    let vue = r.etat.vue(None);
    assert_eq!(vue[0].fichiers.len(), 1, "pas de fantôme");
    assert_eq!(vue[0].fichiers[0].chemin, renomme.display().to_string());

    // Hors de tout dossier connu : il sort de la vue.
    let lointain = loin.join("a.glucose");
    std::fs::rename(&renomme, &lointain).expect("déplacé");
    r.relancer();
    attendre_la_fin(&mut r);
    assert!(r.etat.vue(None).is_empty());

    // L'ouvrir d'où il est : ses domaines sont là, rien ne s'est perdu.
    r.connaitre(&lointain);
    attendre_la_fin(&mut r);
    assert_eq!(r.etat.vue(None)[0].noeuds(), 1);
}

/// **RECENSE-2** : une passe qui ne trouve rien de neuf ne relit rien, ne réveille personne ; un
/// fichier qui change est relu. **Et lire n'écrit jamais** : l'octet près, le fichier est le même.
#[test]
fn une_passe_ne_relit_que_ce_qui_a_change_et_n_ecrit_rien() {
    let glucose = Dossier::nouveau("relire-glucose");
    let ici = Dossier::nouveau("relire-a");
    let a = ici.join("a.glucose");
    ecrire(&a, "mouille", &eau());
    let (mut r, reveils) = recenseur(&glucose);
    r.connaitre(&a);
    attendre_la_fin(&mut r);
    let avant = reveils.load(Ordering::SeqCst);
    let octets = std::fs::read(&a).expect("lu");

    r.relancer();
    attendre_la_fin(&mut r);
    assert_eq!(
        reveils.load(Ordering::SeqCst),
        avant,
        "rien de neuf : aucun réveil"
    );
    assert_eq!(std::fs::read(&a).expect("lu"), octets, "lire n'écrit rien");

    let mut autre = eau();
    autre.name = "Eau pure".into();
    ecrire(&a, "autre-noeud-plus-long", &autre);
    r.relancer();
    attendre_la_fin(&mut r);
    assert!(reveils.load(Ordering::SeqCst) > avant, "il a changé : relu");
    assert_eq!(r.etat.vue(None)[0].definition.name, "Eau pure");
}

/// **Un fichier qui n'est pas un document ne dit rien**, et ne casse pas la passe : les autres sont
/// lus. Un voisin d'une autre extension n'est même pas ouvert.
#[test]
fn un_fichier_illisible_ne_casse_pas_la_passe() {
    let glucose = Dossier::nouveau("illisible-glucose");
    let ici = Dossier::nouveau("illisible-a");
    let (a, faux) = (ici.join("a.glucose"), ici.join("faux.glucose"));
    ecrire(&a, "mouille", &eau());
    std::fs::write(&faux, b"ceci n'est pas un document").expect("écrit");
    std::fs::write(ici.join("note.txt"), b"rien").expect("écrit");
    let (mut r, _) = recenseur(&glucose);
    r.connaitre(&a);
    attendre_la_fin(&mut r);
    let vue = r.etat.vue(None);
    assert_eq!(vue.len(), 1);
    assert_eq!(vue[0].fichiers.len(), 1, "seul le vrai parle : {vue:?}");
}

/// **Sans démarrage, rien ne se lit** : une épreuve ou un banc n'ouvre aucun fichier d'une personne.
#[test]
fn sans_demarrage_rien_ne_se_lit() {
    let ici = Dossier::nouveau("sans-demarrage");
    let a = ici.join("a.glucose");
    ecrire(&a, "mouille", &eau());
    let mut r = Recenseur::default();
    r.connaitre(&a);
    r.relancer();
    assert!(!r.relever());
    assert!(!r.en_cours);
    assert!(r.etat.vue(None).is_empty());
}

#[test]
fn la_liste_garde_le_plus_recent_en_tete_sans_doublon() {
    let liste = lire_les_connus("/a\n\n/b\n");
    assert_eq!(liste, [PathBuf::from("/a"), PathBuf::from("/b")]);
    let liste = avec_en_tete(liste, Path::new("/b"));
    assert_eq!(liste, [PathBuf::from("/b"), PathBuf::from("/a")]);
}

/// **Le fichier ouvert ne se lit pas** : vivant dans Glucose, c'est de là que la vue le prend. Quand on
/// en ouvre un autre, il se lit tel qu'on l'a laissé. La preuve à l'envers : sans le dire ouvert, il
/// est lu.
#[test]
fn le_fichier_ouvert_ne_se_lit_pas() {
    let glucose = Dossier::nouveau("ouvert-glucose");
    let ici = Dossier::nouveau("ouvert-a");
    let (a, b) = (ici.join("a.glucose"), ici.join("b.glucose"));
    ecrire(&a, "mouille", &eau());
    ecrire(&b, "autre", &eau());
    let (mut r, _) = recenseur(&glucose);
    r.suivre_le_fichier_ouvert(Some(a.clone()));
    r.connaitre(&a);
    attendre_la_fin(&mut r);
    let vue = r.etat.vue(None);
    let chemins: Vec<_> = vue[0].fichiers.iter().map(|f| f.chemin.clone()).collect();
    assert_eq!(chemins, [b.display().to_string()], "seul son voisin est lu");

    r.suivre_le_fichier_ouvert(Some(b.clone()));
    attendre_la_fin(&mut r);
    let vue = r.etat.vue(None);
    assert_eq!(vue[0].fichiers.len(), 2, "celui qu'on a quitté se lit");
    r.suivre_le_fichier_ouvert(Some(b.clone()));
    assert!(!r.en_cours, "le même : aucune passe");
}
