//! **Les documents de Glucose Tauri** : notre lecteur contre la bibliothèque de référence.
//!
//! # Pourquoi des documents fabriqués, alors que les vrais passent
//!
//! Les quatre documents Automerge de l'utilisateur sortent **identiques** à la référence, arbre
//! pour arbre (`examples/oracle_tauri.rs`). Mais deux sabotages sur trois passaient aussi : un
//! document écrit par une seule main, d'un trait, n'a ni **frères** dans l'arbre d'un texte
//! (deux insertions au même endroit) ni **conflit** (deux écritures concurrentes du même champ).
//! Les règles qui les tranchent n'étaient donc prouvées par rien.
//!
//! Chaque épreuve ci-dessous fabrique, avec la bibliothèque de référence, le cas qu'une règle
//! tranche, et exige la même valeur des deux lecteurs. Le sabotage de chaque règle fait tomber
//! au moins une épreuve.

#[path = "commun/oracle_automerge.rs"]
mod oracle;

use automerge::transaction::Transactable;
use automerge::{ActorId, AutoCommit, ObjType, ReadDoc, ROOT};
use glucose_core::persist::tauri::{self, Valeur};
use glucose_desktop::tauri as disque;

fn auteur(octet: u8) -> ActorId {
    ActorId::from(vec![octet; 16])
}

/// Notre lecture et celle de la référence doivent coïncider, arbre pour arbre.
fn meme_valeur(octets: &[u8]) -> Valeur {
    let (nous, fin) = tauri::lire(octets).expect("notre lecteur doit lire ce document");
    assert_eq!(fin, 0, "un document intact n'a pas de fin ignorée");
    let reference = oracle::reference(octets).expect("la référence doit lire ce document");
    if let Some(d) = oracle::premiere_difference(&nous, &reference, "racine") {
        panic!("notre lecteur diffère de la référence : {d}");
    }
    nous
}

/// **Frères RGA** : insérer au milieu d'un texte place deux insertions après le même
/// caractère ; la plus récente passe devant.
#[test]
fn test_une_insertion_au_milieu_d_un_texte_se_lit_a_sa_place() {
    let mut doc = AutoCommit::new().with_actor(auteur(1));
    let t = doc.put_object(ROOT, "texte", ObjType::Text).unwrap();
    doc.splice_text(&t, 0, 0, "ac").unwrap();
    doc.commit();
    doc.splice_text(&t, 1, 0, "b").unwrap();
    doc.splice_text(&t, 0, 0, "<").unwrap();
    doc.splice_text(&t, 4, 0, ">").unwrap();
    doc.commit();
    let v = meme_valeur(&doc.save());
    assert_eq!(v.texte("texte"), Some("<abc>"));
}

/// **Frères concurrents** : deux auteurs insèrent au même endroit en même temps ; l'ordre ne
/// dépend que de leurs horloges, identique chez tous.
#[test]
fn test_deux_insertions_concurrentes_au_meme_endroit_se_rangent_comme_chez_la_reference() {
    let mut a = AutoCommit::new().with_actor(auteur(0x10));
    let liste = a.put_object(ROOT, "liste", ObjType::List).unwrap();
    a.insert(&liste, 0, "début").unwrap();
    a.commit();
    let mut b = a.fork().with_actor(auteur(0x20));
    a.insert(&liste, 1, "de A").unwrap();
    a.insert(&liste, 2, "encore A").unwrap();
    b.insert(&liste, 1, "de B").unwrap();
    a.commit();
    b.commit();
    a.merge(&mut b).unwrap();
    meme_valeur(&a.save());
}

/// **Conflit** : deux écritures concurrentes du même champ — la plus grande horloge gagne, et
/// à compteur égal, le plus grand auteur. Les deux ordres d'auteurs sont essayés : un lecteur
/// qui prendrait la plus petite horloge, ou la première lue, tombe sur l'un des deux.
#[test]
fn test_un_conflit_se_tranche_comme_chez_la_reference_quel_que_soit_l_ordre_des_auteurs() {
    for (premier, second) in [(0x01, 0xf0), (0xf0, 0x01)] {
        let mut a = AutoCommit::new().with_actor(auteur(premier));
        a.put(ROOT, "x", 0).unwrap();
        a.commit();
        let mut b = a.fork().with_actor(auteur(second));
        a.put(ROOT, "x", "de A").unwrap();
        b.put(ROOT, "x", "de B").unwrap();
        a.commit();
        b.commit();
        a.merge(&mut b).unwrap();
        let v = meme_valeur(&a.save());
        let attendu = if second > premier { "de B" } else { "de A" };
        assert_eq!(
            v.texte("x"),
            Some(attendu),
            "auteurs {premier:#x} puis {second:#x}"
        );
    }
}

/// **Deltas ajoutés** : c'est ainsi que Tauri enregistrait après la première fois — un
/// document, puis des morceaux « delta » à la suite. Leurs liens désignent des
/// prédécesseurs, que la reconstruction doit retourner en successeurs.
#[test]
fn test_des_deltas_ajoutes_apres_le_document_se_lisent() {
    let mut doc = AutoCommit::new().with_actor(auteur(7));
    let images = doc.put_object(ROOT, "images", ObjType::List).unwrap();
    for i in 0..5 {
        let m = doc.insert_object(&images, i, ObjType::Map).unwrap();
        doc.put(&m, "id", format!("img-{i}")).unwrap();
        doc.put(&m, "x", i as f64 * 10.5).unwrap();
    }
    let mut fichier = doc.save();
    // Trois enregistrements incrémentaux : un déplacement, une suppression, un ajout.
    let m0 = doc.get(&images, 0).unwrap().unwrap().1;
    doc.put(&m0, "x", -3.25).unwrap();
    doc.commit();
    fichier.extend(doc.save_incremental());
    doc.delete(&images, 2).unwrap();
    doc.commit();
    fichier.extend(doc.save_incremental());
    let m = doc.insert_object(&images, 1, ObjType::Map).unwrap();
    doc.put(&m, "id", "neuve").unwrap();
    doc.commit();
    fichier.extend(doc.save_incremental());
    let v = meme_valeur(&fichier);
    assert_eq!(v.liste("images").len(), 5, "cinq, moins une, plus une");
}

/// **Delta compressé** : un gros changement s'écrit en morceau DEFLATE, dont la somme de
/// contrôle se calcule comme s'il ne l'était pas.
#[test]
fn test_un_delta_compresse_se_lit() {
    let mut doc = AutoCommit::new().with_actor(auteur(9));
    let t = doc.put_object(ROOT, "long", ObjType::Text).unwrap();
    doc.commit();
    let mut fichier = doc.save();
    let avant = doc.get_heads();
    doc.splice_text(&t, 0, 0, &"une idée qui se répète ".repeat(200))
        .unwrap();
    doc.commit();
    let mut compresse = false;
    for mut change in doc.get_changes(&avant) {
        let octets = change.bytes().into_owned();
        compresse |= octets[8] == 2;
        fichier.extend(octets);
    }
    assert!(
        compresse,
        "l'épreuve doit produire un morceau compressé, sinon elle ne prouve rien"
    );
    meme_valeur(&fichier);
}

/// **Tous les types de valeur**, dont le compteur et ses incréments, et une liste dont un
/// élément est réécrit sur place.
#[test]
fn test_chaque_type_de_valeur_se_lit_comme_chez_la_reference() {
    let mut doc = AutoCommit::new().with_actor(auteur(3));
    doc.put(ROOT, "nul", ()).unwrap();
    doc.put(ROOT, "vrai", true).unwrap();
    doc.put(ROOT, "entier", -42_i64).unwrap();
    doc.put(ROOT, "naturel", 42_u64).unwrap();
    doc.put(ROOT, "flottant", 0.1_f64).unwrap();
    doc.put(ROOT, "octets", vec![0u8, 255, 7]).unwrap();
    doc.put(ROOT, "compteur", automerge::ScalarValue::counter(10))
        .unwrap();
    doc.put(
        ROOT,
        "instant",
        automerge::ScalarValue::Timestamp(1_782_222_588_346),
    )
    .unwrap();
    let l = doc.put_object(ROOT, "liste", ObjType::List).unwrap();
    doc.insert(&l, 0, "a").unwrap();
    doc.insert(&l, 1, "b").unwrap();
    doc.commit();
    doc.increment(ROOT, "compteur", 5).unwrap();
    doc.increment(ROOT, "compteur", -2).unwrap();
    doc.put(&l, 1, "B").unwrap();
    doc.commit();
    let v = meme_valeur(&doc.save());
    assert_eq!(v.entier("compteur"), Some(13));
}

/// **Une fin tronquée** — un enregistrement interrompu chez Tauri — laisse lire tout ce qui
/// la précède, et se dit.
#[test]
fn test_une_fin_tronquee_se_dit_et_laisse_lire_ce_qui_precede() {
    let mut doc = AutoCommit::new().with_actor(auteur(5));
    doc.put(ROOT, "nom", "sain").unwrap();
    doc.commit();
    let sain = doc.save();
    doc.put(ROOT, "nom", "perdu").unwrap();
    doc.commit();
    let delta = doc.save_incremental();
    let mut fichier = sain.clone();
    fichier.extend_from_slice(&delta[..delta.len() - 3]);
    let (v, fin) = tauri::lire(&fichier).expect("le début sain doit se lire");
    assert_eq!(
        fin,
        delta.len() - 3,
        "la fin ignorée est exactement le delta tronqué"
    );
    assert_eq!(v.texte("nom"), Some("sain"));
}

/// Un premier morceau abîmé n'est pas une fin tronquée : le document est refusé.
#[test]
fn test_un_document_abime_des_son_debut_est_refuse() {
    let mut doc = AutoCommit::new();
    doc.put(ROOT, "nom", "x").unwrap();
    let mut octets = doc.save();
    let milieu = octets.len() / 2;
    octets[milieu] ^= 0xff;
    assert!(tauri::lire(&octets).is_err());
}

// ── Le JSON d'avant ─────────────────────────────────────────────────────────

#[test]
fn test_le_json_lit_echappements_paires_de_substitution_et_nombres() {
    let v = tauri::json::lire(
        r#" { "a": "é\u00e9\ud83d\ude00\n", "n": -12, "f": 1.5e2, "l": [true, null, {}], "v": [] } "#,
    )
    .expect("JSON valide");
    assert_eq!(v.texte("a"), Some("éé😀\n"));
    assert_eq!(v.champ("n"), Some(&Valeur::Entier(-12)));
    assert_eq!(v.nombre("f"), Some(150.0));
    assert_eq!(v.liste("l").len(), 3);
    assert!(v.liste("v").is_empty());
}

/// Cent mille crochets imbriqués ne font pas déborder la pile : le lecteur n'est pas récursif.
#[test]
fn test_le_json_ne_deborde_pas_sur_une_imbrication_demesuree() {
    let profond = format!("{}{}", "[".repeat(100_000), "]".repeat(100_000));
    assert!(tauri::json::lire(&profond).is_ok());
    for faux in ["", "{", "[1,]", "{\"a\" 1}", "[1] 2", "\"\\x\"", "tru"] {
        assert!(
            tauri::json::lire(faux).is_err(),
            "« {faux} » n'est pas du JSON"
        );
    }
}

// ── La traduction en projet, et les images ──────────────────────────────────

const PROJET_TAURI: &str = r##"{
  "version": "1.0.0", "name": "Essai", "activeBoardId": "b1", "createdAt": 1, "updatedAt": 2,
  "presets": [], "domains": [{"id": "d1", "name": "Art", "color": "#f00", "icon": "A", "createdAt": 3}],
  "boards": [{
    "id": "b1", "name": "Principal", "createdAt": 4, "updatedAt": 5,
    "viewport": {"x": 10, "y": -20.5, "scale": 0.25},
    "bookmarks": {"1": {"x": 1, "y": 2, "scale": 3}},
    "presetId": "p1", "panels": [], "zones": [], "folders": [],
    "images": [
      {"id": "i1", "x": 5, "y": 6, "width": 280, "height": 200, "rotation": 0.5, "locked": true,
       "tags": ["mer"], "originalWidth": 1000, "originalHeight": 700,
       "asset": {"mode": "link", "href": "asset:0123456789abcdef.png", "sha256": "0123456789abcdef00"}},
      {"id": "i2", "x": 0, "y": 0, "width": 1, "height": 1, "rotation": 0, "locked": false, "tags": [],
       "originalWidth": 1, "originalHeight": 1, "src": "data:image/png;base64,iVBORw0KGgo="},
      {"id": "i3", "x": 0, "y": 0, "width": 1, "height": 1, "rotation": 0, "locked": false, "tags": [],
       "originalWidth": 1, "originalHeight": 1, "src": "asset:../../secret.txt"}
    ],
    "annotations": [
      {"type": "text", "id": "t1", "x": 1, "y": 2, "text": "# titre", "fontSize": 14, "width": 240,
       "domains": [{"domainId": "d1", "weight": 0.5}]},
      {"type": "sticky", "id": "s1", "x": 0, "y": 0, "text": "ET", "operator": "AND", "bgColor": "#fde047"},
      {"type": "arrow", "id": "a1", "x": 0, "y": 0, "x2": 10, "y2": 20, "predicate": "inspire",
       "sourceId": "t1", "targetId": "s1", "sourceTextSel": [{"start": 2, "end": 7, "quote": "titre"}],
       "waypoints": [{"x": 5, "y": 5}]},
      {"type": "membrane", "id": "m1", "x": 0, "y": 0, "width": 300, "height": 200, "mode": "stretched",
       "curtains": [{"id": "c1", "ownerId": "u", "ownerName": "Moi", "ownerColor": "#0f0",
                     "visibility": "shared", "editable": "everyone", "notes": [], "createdAt": 9}]},
      {"type": "cercle", "id": "z"}
    ]
  }]
}"##;

#[test]
fn test_la_traduction_garde_chaque_champ_et_dit_ce_qu_elle_omet() {
    use glucose_core::types::{Annotation, MembraneMode, StickyOperator};
    let v = tauri::json::lire(PROJET_TAURI).unwrap();
    let (p, r) = tauri::projet::traduire(&v);
    assert_eq!(
        (p.name.as_str(), p.active_board_id.as_str()),
        ("Essai", "b1")
    );
    assert_eq!(p.domains.len(), 1);
    let b = &p.boards[0];
    assert_eq!(
        (b.viewport.x, b.viewport.y, b.viewport.scale),
        (10.0, -20.5, 0.25)
    );
    assert_eq!(b.bookmarks["1"].scale, 3.0);
    let i1 = &b.images[0];
    assert!(i1.locked && i1.rotation == 0.5 && i1.tags == ["mer"]);
    assert_eq!((i1.original_width, i1.original_height), (1000.0, 700.0));
    assert!(matches!(
        &b.annotations[1],
        Annotation::Sticky {
            operator: Some(StickyOperator::And),
            ..
        }
    ));
    assert!(
        matches!(&b.annotations[3], Annotation::Membrane { mode: MembraneMode::Stretched, curtains, .. } if curtains.len() == 1)
    );
    assert_eq!(b.annotations.len(), 4, "le type inconnu est omis");
    assert_eq!(r.omis.get("annotation d'un type inconnu"), Some(&1));
    assert_eq!(r.omis.get("preset appliqué à un tableau"), Some(&1));
    assert_eq!(r.provenances.len(), 3);
    assert_eq!(
        r.provenances[1].1,
        tauri::projet::Provenance::Octets(vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a])
    );
    assert_eq!(
        r.provenances[2].1,
        tauri::projet::Provenance::Inconnue,
        "un nom de magasin qui sort du magasin est refusé"
    );
}

/// Une image du magasin se lit dans `objects/` d'un document portable, puis dans le magasin
/// global — et **seulement si** son contenu porte l'empreinte que son nom annonce.
#[test]
fn test_une_image_du_magasin_se_verifie_contre_son_empreinte() {
    use glucose_core::hash::{hex_of, sha256};
    let racine = std::env::temp_dir().join("glucose-tests-tauri-magasin");
    let objets = racine.join("portable").join("objects");
    let magasin = racine.join("global");
    std::fs::create_dir_all(&objets).unwrap();
    std::fs::create_dir_all(&magasin).unwrap();
    let octets = b"les pixels d'une photo".to_vec();
    let nom = format!("{}.png", &hex_of(&sha256(&octets))[..16]);
    std::fs::write(magasin.join(&nom), &octets).unwrap();
    let p = tauri::projet::Provenance::Magasin(nom.clone());
    let lu = disque::resoudre(&p, Some(&racine.join("portable")), Some(&magasin));
    assert_eq!(
        lu.as_deref(),
        Ok(&octets[..]),
        "trouvée dans le magasin global"
    );

    std::fs::write(objets.join(&nom), b"autre chose").unwrap();
    let lu = disque::resoudre(&p, Some(&racine.join("portable")), Some(&magasin));
    assert_eq!(
        lu.as_deref(),
        Ok(&octets[..]),
        "une copie abîmée dans objects/ ne masque pas la bonne du magasin global"
    );
    let lu = disque::resoudre(&p, Some(&racine.join("portable")), None);
    assert!(
        lu.is_err_and(|e| e.contains("abîmé")),
        "seule une copie abîmée : refusée et dite, jamais posée à la place"
    );
    std::fs::remove_dir_all(&racine).unwrap();
}

/// Vingt mille cartes imbriquées — un fichier forgé — se lisent et se libèrent sans faire
/// déborder la pile : la reconstruction n'est pas récursive, la destruction non plus. (La
/// référence, elle, n'est pas interrogée ici : sa propre lecture est récursive.)
#[test]
fn test_un_document_automerge_demesurement_imbrique_ne_fait_pas_tomber_l_application() {
    let mut doc = AutoCommit::new().with_actor(auteur(4));
    let mut objet = doc.put_object(ROOT, "n", ObjType::Map).unwrap();
    for _ in 0..20_000 {
        objet = doc.put_object(&objet, "n", ObjType::Map).unwrap();
    }
    doc.put(&objet, "fond", "atteint").unwrap();
    doc.commit();
    let (v, _) = tauri::lire(&doc.save()).expect("un document profond reste un document");
    let mut niveau = &v;
    let mut profondeur = 0;
    while let Some(suivant) = niveau.champ("n") {
        niveau = suivant;
        profondeur += 1;
    }
    assert_eq!(profondeur, 20_001);
    assert_eq!(niveau.texte("fond"), Some("atteint"));
}

// ── Notre DEFLATE contre celui de l'application ─────────────────────────────

/// Ce que `miniz_oxide` compresse, à chaque niveau — du bloc brut (0) au plus serré (10) —,
/// notre lecteur le rend à l'octet près. Les données mêlent du texte qui se répète, du bruit
/// qui ne se compresse pas, et des longues plages d'un même octet (recopies chevauchantes).
#[test]
fn test_notre_deflate_rend_ce_que_miniz_oxide_compresse_a_chaque_niveau() {
    let mut donnees = Vec::new();
    let mut graine = 0x2545_f491_4f6c_dd1du64;
    for i in 0..40_000u32 {
        graine ^= graine << 13;
        graine ^= graine >> 7;
        graine ^= graine << 17;
        donnees.push(match i % 3000 {
            0..=999 => b"une idee revient "[i as usize % 17],
            1000..=1999 => graine as u8,
            _ => 0x41,
        });
    }
    for niveau in 0..=10u8 {
        for taille in [0, 1, 17, 300, 5_000, donnees.len()] {
            let clair = &donnees[..taille];
            let serre = miniz_oxide::deflate::compress_to_vec(clair, niveau);
            let relu = tauri::inflate::inflate(&serre)
                .unwrap_or_else(|e| panic!("niveau {niveau}, {taille} octets : {e}"));
            assert!(
                relu == clair,
                "niveau {niveau}, {taille} octets : contenu différent"
            );
        }
    }
}
