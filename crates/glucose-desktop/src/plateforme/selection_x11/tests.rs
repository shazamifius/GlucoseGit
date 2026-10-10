//! COPIER-1 sous X11 : la sélection, demandée comme les autres applications la demandent.
//!
//! **Jamais sur l'écran de quelqu'un** : ces épreuves prennent la sélection `CLIPBOARD` d'un
//! serveur X ; elles ne visent que celui qu'on leur nomme (`GLUCOSE_ECRAN_X_D_EPREUVE`, un Xvfb),
//! jamais celui de `DISPLAY`. Sans lui elles se sautent — sauf si `GLUCOSE_EXIGER_UN_SERVEUR_X`
//! dit qu'il doit y en avoir un (sur GitHub), et alors elles tombent au lieu de mentir en vert.
//! Le demandeur est `xclip`, une implémentation qui n'est pas la nôtre.

use super::lecture::lire_le_lot_de;
use super::offre::{fichiers_gnome, liste_d_uri, uri_de};
use super::service::pas_avant;
use super::*;
use crate::interactions::clipboard::{Fichier, ImagePosee};
use crate::plateforme::copies::dossier_des_copies;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{MutexGuard, PoisonError};
use std::time::Instant;
use x11rb::connection::Connection as _;
use x11rb::protocol::xproto::{
    Atom, ConnectionExt as _, CreateWindowAux, EventMask, GetPropertyReply, SelectionNotifyEvent,
    SelectionRequestEvent, Window, WindowClass, SELECTION_NOTIFY_EVENT,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::{COPY_FROM_PARENT, CURRENT_TIME, NONE};

/// Les épreuves prennent toutes la même sélection du même serveur : une à la fois.
static UNE_A_LA_FOIS: Mutex<()> = Mutex::new(());

/// Le serveur X d'épreuve, et le droit de prendre sa sélection.
struct Banc {
    ecran: String,
    _seul: MutexGuard<'static, ()>,
}

/// **Le banc, s'il y en a un** : un serveur X nommé, et `xclip`.
fn banc() -> Option<Banc> {
    let exige = std::env::var_os("GLUCOSE_EXIGER_UN_SERVEUR_X").is_some();
    let Ok(ecran) = std::env::var("GLUCOSE_ECRAN_X_D_EPREUVE") else {
        assert!(
            !exige,
            "GLUCOSE_ECRAN_X_D_EPREUVE manque : aucun serveur X d'épreuve"
        );
        return None;
    };
    if Command::new("xclip").arg("-version").output().is_err() {
        assert!(!exige, "xclip manque");
        return None;
    }
    Some(Banc {
        ecran,
        _seul: UNE_A_LA_FOIS.lock().unwrap_or_else(PoisonError::into_inner),
    })
}

impl Banc {
    fn proprietaire(&self) -> Proprietaire {
        Proprietaire::ouvrir(Some(&self.ecran)).expect("un propriétaire sur le banc")
    }

    /// Ce que `xclip` reçoit pour cette cible, ou rien s'il est refusé.
    fn xclip(&self, cible: &str) -> Option<Vec<u8>> {
        let sortie = Command::new("xclip")
            .args(["-display", &self.ecran, "-selection", "clipboard"])
            .args(["-o", "-t", cible])
            .output()
            .expect("xclip");
        sortie.status.success().then_some(sortie.stdout)
    }

    fn cibles(&self) -> Vec<String> {
        let liste = self.xclip("TARGETS").expect("TARGETS");
        String::from_utf8_lossy(&liste)
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// Une connexion à soi, et une fenêtre : un autre programme.
    fn client(&self) -> (RustConnection, Window, Atomes) {
        let (conn, numero) = RustConnection::connect(Some(&self.ecran)).expect("connecté");
        let atomes = Atomes::new(&conn).expect("atomes").reply().expect("atomes");
        let racine = conn.setup().roots[numero].root;
        let fenetre = conn.generate_id().expect("un numéro");
        let attributs = CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE);
        conn.create_window(
            0,
            fenetre,
            racine,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_ONLY,
            COPY_FROM_PARENT,
            &attributs,
        )
        .expect("une fenêtre");
        conn.flush().expect("envoyé");
        (conn, fenetre, atomes)
    }
}

/// Une date neuve à chaque appel : chaque épreuve a son dossier.
fn date() -> i64 {
    static N: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
    let base = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as i64);
    base + N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn lot(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i % 251) as u8).collect()
}

fn fichier(nom: &str, octet: u8) -> Fichier {
    Fichier {
        nom: nom.into(),
        octets: Arc::new(vec![octet; 64]),
    }
}

fn formes(lot: Vec<u8>, texte: Option<&str>, image: bool, fichiers: Vec<Fichier>) -> Formes {
    Formes {
        lot,
        texte: texte.map(str::to_string),
        image: image.then(|| ImagePosee {
            png: b"\x89PNG faux".to_vec(),
            largeur: 1,
            hauteur: 1,
            rgba: vec![1, 2, 3, 255],
        }),
        fichiers,
        date: date(),
    }
}

/// `%XX` décodé, comme Chromium et GLib décodent une URI `file://`.
fn chemin_de(uri: &str) -> PathBuf {
    let code = uri
        .strip_prefix("file://")
        .expect("une URI de fichier")
        .as_bytes();
    let mut octets = Vec::new();
    let mut i = 0;
    while i < code.len() {
        if code[i] == b'%' {
            let hexa = std::str::from_utf8(&code[i + 1..i + 3]).expect("hexa");
            octets.push(u8::from_str_radix(hexa, 16).expect("hexa"));
            i += 3;
        } else {
            octets.push(code[i]);
            i += 1;
        }
    }
    PathBuf::from(String::from_utf8(octets).expect("UTF-8"))
}

/// **Un chemin devient une URI que chacun relit** : espaces, accents, `#` et `?` codés ; la
/// liste finit chaque ligne par `\r\n` — Chromium ne coupe que là ; celle de GNOME dit `copy`.
#[test]
fn test_un_chemin_devient_une_uri_que_chacun_relit() {
    let chemin = Path::new("/tmp/glucose-copies/12/été #1?.png");
    let uri = uri_de(chemin);
    assert_eq!(
        uri,
        "file:///tmp/glucose-copies/12/%C3%A9t%C3%A9%20%231%3F.png"
    );
    assert_eq!(chemin_de(&uri), chemin, "et se relit");
    let chemins = [PathBuf::from("/a b/x.png"), PathBuf::from("/c.png")];
    assert_eq!(
        liste_d_uri(&chemins),
        "file:///a%20b/x.png\r\nfile:///c.png\r\n"
    );
    assert_eq!(
        fichiers_gnome(&chemins),
        "copy\nfile:///a%20b/x.png\nfile:///c.png"
    );
}

/// **Une demande d'avant la prise est refusée** (ICCCM 2.2), `CurrentTime` jamais ; et l'horloge
/// du serveur qui boucle ne fait pas d'une demande récente une demande ancienne.
#[test]
fn test_une_demande_d_avant_la_prise_est_refusee() {
    assert!(!pas_avant(5, 10), "d'avant la prise");
    assert!(pas_avant(10, 10) && pas_avant(11, 10));
    assert!(pas_avant(CURRENT_TIME, 10), "CurrentTime");
    assert!(pas_avant(3, u32::MAX - 2), "après le tour de l'horloge");
}

/// **Une image seule part en image** — et pas en fichier, que Discord ferait seconde pièce
/// jointe ; le lot, le texte, `TARGETS`, `TIMESTAMP` et `MULTIPLE` sont là.
#[test]
fn test_une_image_seule_part_en_image() {
    let Some(b) = banc() else { return };
    let p = b.proprietaire();
    let le_lot = lot(3000);
    let f = formes(
        le_lot.clone(),
        Some("une note"),
        true,
        vec![fichier("a.png", 1)],
    );
    p.confier(f).expect("la sélection est prise");
    let cibles = b.cibles();
    for c in [
        "TARGETS",
        "TIMESTAMP",
        "MULTIPLE",
        "SAVE_TARGETS",
        "application/x-glucose-lot",
        "UTF8_STRING",
        "text/plain;charset=utf-8",
        "image/png",
    ] {
        assert!(cibles.iter().any(|x| x == c), "{c} manque : {cibles:?}");
    }
    assert!(
        !cibles.iter().any(|x| x == "text/uri-list"),
        "pas de fichier"
    );
    assert_eq!(b.xclip("image/png").as_deref(), Some(&b"\x89PNG faux"[..]));
    assert_eq!(b.xclip("UTF8_STRING").as_deref(), Some(&b"une note"[..]));
    assert_eq!(b.xclip("application/x-glucose-lot"), Some(le_lot));
    assert_eq!(b.xclip("text/uri-list"), None, "refusée");
}

/// **Plusieurs images partent en fichiers, écrits au premier collage qui les veut, une fois** :
/// rien sur le disque avant ; puis une URI par original, que `xclip` reçoit et qui mène à ses
/// octets ; un second collage ne réécrit rien.
#[test]
fn test_plusieurs_images_partent_en_fichiers_ecrits_a_la_demande() {
    let Some(b) = banc() else { return };
    let p = b.proprietaire();
    let f = formes(
        lot(10),
        None,
        false,
        vec![fichier("a.png", 1), fichier("a.png", 2)],
    );
    let dossier = dossier_des_copies().join(f.date.to_string());
    p.confier(f).expect("la sélection est prise");
    let cibles = b.cibles();
    assert!(cibles.iter().any(|x| x == "text/uri-list"));
    assert!(cibles.iter().any(|x| x == "x-special/gnome-copied-files"));
    assert!(
        !cibles.iter().any(|x| x == "image/png"),
        "pas d'image seule"
    );
    assert!(!dossier.exists(), "rien d'écrit avant qu'on le demande");
    let liste = String::from_utf8(b.xclip("text/uri-list").expect("la liste")).expect("UTF-8");
    let chemins: Vec<PathBuf> = liste
        .split("\r\n")
        .filter(|l| !l.is_empty())
        .map(chemin_de)
        .collect();
    assert_eq!(chemins.len(), 2, "deux fichiers : {liste:?}");
    assert_eq!(std::fs::read(&chemins[1]).expect("écrit"), vec![2u8; 64]);
    std::fs::write(&chemins[1], b"retouche").expect("modifié");
    assert_eq!(b.xclip("text/uri-list").as_deref(), Some(liste.as_bytes()));
    assert_eq!(
        std::fs::read(&chemins[1]).expect("lu"),
        b"retouche",
        "une seule écriture"
    );
    let gnome = b
        .xclip("x-special/gnome-copied-files")
        .expect("celle de GNOME");
    assert!(gnome.starts_with(b"copy\nfile://"));
    let _ = std::fs::remove_dir_all(dossier);
}

/// **Un lot plus gros qu'une requête part par tranches** (`INCR`), et revient entier — à `xclip`
/// comme à un autre Glucose.
#[test]
fn test_un_lot_plus_gros_qu_une_requete_part_par_tranches() {
    let Some(b) = banc() else { return };
    let p = b.proprietaire();
    let gros = lot(2 * p.serveur.tranche + 12_345);
    p.confier(formes(gros.clone(), None, false, Vec::new()))
        .expect("la sélection est prise");
    assert!(
        b.xclip("application/x-glucose-lot") == Some(gros.clone()),
        "xclip le relit entier"
    );
    let relu = lire_le_lot_de(Some(&b.ecran), p.fenetre());
    assert!(relu == Some(gros), "un autre Glucose aussi");
}

/// **`MULTIPLE`** : chaque paire convertie ; celle qu'on ne sait pas convertir revient sans
/// propriété — c'est ainsi que le gestionnaire de GNOME prend tout d'un coup.
#[test]
fn test_multiple_convertit_chaque_paire() {
    let Some(b) = banc() else { return };
    let p = b.proprietaire();
    p.confier(formes(lot(10), Some("une note"), false, Vec::new()))
        .expect("la sélection est prise");
    let (conn, w, a) = b.client();
    let nom = |n: &[u8]| {
        conn.intern_atom(false, n)
            .expect("atome")
            .reply()
            .expect("atome")
            .atom
    };
    let (un, deux, toutes) = (nom(b"_UN"), nom(b"_DEUX"), nom(b"_TOUTES"));
    let paires = [a.UTF8_STRING, un, a.PNG, deux];
    conn.change_property32(PropMode::REPLACE, w, toutes, a.ATOM_PAIR, &paires)
        .expect("les paires");
    conn.convert_selection(w, a.CLIPBOARD, a.MULTIPLE, toutes, CURRENT_TIME)
        .expect("demandé");
    conn.flush().expect("envoyé");
    let reponse = loop {
        if let Event::SelectionNotify(e) = conn.wait_for_event().expect("un évènement") {
            break e;
        }
    };
    assert_eq!(reponse.property, toutes, "acceptée");
    let resultats: Vec<Atom> = lire(&conn, w, toutes, false)
        .value32()
        .expect("des atomes")
        .collect();
    assert_eq!(
        resultats,
        [a.UTF8_STRING, un, a.PNG, NONE],
        "la seconde refusée"
    );
    assert_eq!(lire(&conn, w, un, false).value, b"une note");
}

/// **Un aller-retour avec le serveur** : quand la réponse revient, il a traité tout ce que ce
/// client a demandé avant. Entre deux clients, rien d'autre ne garantit l'ordre — sans lui, la
/// question d'un autre programme peut passer avant une prise déjà envoyée.
fn aller_retour(conn: &RustConnection, a: &Atomes) {
    conn.get_selection_owner(a.CLIPBOARD)
        .expect("demandé")
        .reply()
        .expect("répondu");
}

/// **Dit au demandeur que sa demande est faite**, et attend que le serveur l'ait traité : un
/// client qui ferme sa connexion juste après peut voir ses dernières requêtes jetées — le
/// gestionnaire d'épreuve s'arrête aussitôt, et `xclip` restait sans réponse une fois sur deux.
fn notifier(conn: &RustConnection, a: &Atomes, e: &SelectionRequestEvent) {
    let fait = SelectionNotifyEvent {
        response_type: SELECTION_NOTIFY_EVENT,
        sequence: 0,
        time: e.time,
        requestor: e.requestor,
        selection: e.selection,
        target: e.target,
        property: e.property,
    };
    conn.send_event(false, e.requestor, EventMask::NO_EVENT, fait)
        .expect("notifié");
    aller_retour(conn, a);
}

/// Une propriété entière, lue.
fn lire(conn: &RustConnection, w: Window, prop: Atom, effacer: bool) -> GetPropertyReply {
    let r = conn.get_property(effacer, w, prop, AtomEnum::ANY, 0, 1 << 20);
    r.expect("lue").reply().expect("lue")
}

/// Un gestionnaire du presse-papiers réduit à `SAVE_TARGETS`, comme la spécification de
/// freedesktop le décrit : il lit la liste des cibles, prend le texte, devient propriétaire, et
/// le sert à qui le demande ensuite. Il rend la liste qu'on lui a confiée.
fn gestionnaire(b: &Banc, pret: mpsc::Sender<()>) -> std::thread::JoinHandle<Vec<Atom>> {
    let (conn, w, a) = b.client();
    std::thread::spawn(move || {
        conn.set_selection_owner(w, a.CLIPBOARD_MANAGER, CURRENT_TIME)
            .expect("gestionnaire");
        aller_retour(&conn, &a);
        let _ = pret.send(());
        let (mut liste, mut remise) = (Vec::new(), None);
        loop {
            match conn.wait_for_event().expect("un évènement") {
                Event::SelectionRequest(e) if e.target == a.SAVE_TARGETS => {
                    let cibles = lire(&conn, e.requestor, e.property, false);
                    liste = cibles.value32().expect("des atomes").collect();
                    conn.convert_selection(w, a.CLIPBOARD, a.UTF8_STRING, a.RECU, CURRENT_TIME)
                        .expect("demandé");
                    conn.flush().expect("envoyé");
                    remise = Some(e);
                }
                Event::SelectionNotify(e) if e.requestor == w => {
                    let texte = lire(&conn, w, a.RECU, true).value;
                    conn.change_property8(PropMode::REPLACE, w, a.GARDER, a.UTF8_STRING, &texte)
                        .expect("gardé");
                    conn.set_selection_owner(w, a.CLIPBOARD, CURRENT_TIME)
                        .expect("propriétaire");
                    notifier(&conn, &a, &remise.take().expect("la remise"));
                }
                Event::SelectionRequest(e) if e.target == a.UTF8_STRING => {
                    let texte = lire(&conn, w, a.GARDER, false).value;
                    let utf8 = a.UTF8_STRING;
                    conn.change_property8(PropMode::REPLACE, e.requestor, e.property, utf8, &texte)
                        .expect("servi");
                    notifier(&conn, &a, &e);
                    return liste;
                }
                _ => {}
            }
        }
    })
}

/// **À la fermeture, la sélection va au gestionnaire du presse-papiers**, avec la liste de ce
/// qu'elle offre ; la remise finit quand il l'a prise — pas à l'échéance —, et un collage après
/// coup trouve encore le texte.
#[test]
fn test_a_la_fermeture_la_selection_va_au_gestionnaire() {
    let Some(b) = banc() else { return };
    let (pret, attendre) = mpsc::channel();
    let fil = gestionnaire(&b, pret);
    attendre.recv().expect("le gestionnaire est là");
    let p = b.proprietaire();
    p.confier(formes(lot(10), Some("une note"), false, Vec::new()))
        .expect("la sélection est prise");
    let lot_atome = p.serveur.atomes.LOT;
    let debut = Instant::now();
    p.rendre();
    assert!(debut.elapsed() < PATIENCE / 2, "finie quand il l'a prise");
    assert_eq!(
        b.xclip("UTF8_STRING").as_deref(),
        Some(&b"une note"[..]),
        "gardée"
    );
    let liste = fil.join().expect("le gestionnaire");
    assert!(liste.contains(&lot_atome), "le lot dans la liste à garder");
}

/// **Un lot se lit chez soi, chez un autre Glucose, et jamais ailleurs** : une autre application
/// qui tient la sélection n'est pas même sollicitée — elle pourrait ne jamais répondre.
#[test]
fn test_un_lot_se_lit_chez_soi_et_chez_un_autre_glucose_jamais_ailleurs() {
    let Some(b) = banc() else { return };
    let (ici, ailleurs) = (b.proprietaire(), b.proprietaire());
    let le_lot = lot(5000);
    ici.confier(formes(le_lot.clone(), None, false, Vec::new()))
        .expect("la sélection est prise");
    assert_eq!(ici.lot().expect("chez soi")(), Some(le_lot.clone()));
    assert_eq!(
        ailleurs.lot().expect("chez un autre Glucose")(),
        Some(le_lot)
    );
    // Une application muette prend la sélection, et ne lit jamais ce qu'on lui demande.
    let (muette, w, a) = b.client();
    muette
        .set_selection_owner(w, a.CLIPBOARD, CURRENT_TIME)
        .expect("prise");
    aller_retour(&muette, &a);
    let debut = Instant::now();
    assert!(
        ici.lot().is_none(),
        "une autre application n'est pas sollicitée"
    );
    assert!(debut.elapsed() < PATIENCE / 5, "et rien ne l'attend");
}

/// Demande cette cible à l'heure dite, et rend la propriété de la réponse (`NONE` : refusée).
fn demander(conn: &RustConnection, w: Window, a: &Atomes, cible: Atom, temps: u32) -> Atom {
    conn.convert_selection(w, a.CLIPBOARD, cible, a.RECU, temps)
        .expect("demandé");
    conn.flush().expect("envoyé");
    loop {
        if let Event::SelectionNotify(e) = conn.wait_for_event().expect("un évènement") {
            return e.property;
        }
    }
}

/// **Le service refuse une demande d'avant sa prise** (ICCCM 2.2), à l'heure que `TIMESTAMP`
/// dit — et sert celle qui vient à cette heure-là.
#[test]
fn test_le_service_refuse_une_demande_d_avant_sa_prise() {
    let Some(b) = banc() else { return };
    let p = b.proprietaire();
    p.confier(formes(lot(10), Some("une note"), false, Vec::new()))
        .expect("la sélection est prise");
    let (conn, w, a) = b.client();
    assert_eq!(demander(&conn, w, &a, a.TIMESTAMP, CURRENT_TIME), a.RECU);
    let prise = lire(&conn, w, a.RECU, true)
        .value32()
        .and_then(|mut v| v.next())
        .expect("l'heure de la prise");
    let avant = demander(&conn, w, &a, a.UTF8_STRING, prise.wrapping_sub(1));
    assert_eq!(avant, NONE, "d'avant la prise : refusée");
    let a_l_heure = demander(&conn, w, &a, a.UTF8_STRING, prise);
    assert_eq!(a_l_heure, a.RECU, "à l'heure de la prise : servie");
    assert_eq!(lire(&conn, w, a.RECU, true).value, b"une note");
}
