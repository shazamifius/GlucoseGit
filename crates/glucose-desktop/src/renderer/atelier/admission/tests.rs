//! La porte d'admission, éprouvée sur des nombres fixés : la même règle dit la même chose sur
//! un téléphone et sur un serveur.
//!
//! Chaque décodage simulé tient la porte dans un fil à lui, et toute attente a un délai : une
//! règle cassée fait **tomber** l'épreuve, elle ne la bloque jamais.

use super::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

/// Une machine qui laisse mille octets : ce qui est en vol en prend cinq cents.
fn mille() -> Option<Memoire> {
    Some(Memoire {
        totale: 4_000,
        disponible: 1_000,
    })
}

/// Le temps qu'une règle juste laisse à un décodage pour entrer ; une règle cassée ne
/// l'atteindrait jamais — l'épreuve tombe alors au lieu d'attendre sans fin.
const ENTRER: Duration = Duration::from_secs(30);
/// Le temps pendant lequel un décodage qui ne tient pas doit rester dehors.
const RESTER_DEHORS: Duration = Duration::from_millis(500);

/// Un décodage simulé : il entre par la porte, le dit, et tient jusqu'à ce qu'on le lâche.
struct Decodage {
    entre: Receiver<u64>,
    lacher: Sender<()>,
    fil: JoinHandle<()>,
}

fn decoder(porte: &'static Admission, octets: u64) -> Decodage {
    let (dire, entre) = channel();
    let (lacher, attendre) = channel::<()>();
    let fil = std::thread::spawn(move || {
        porte.pendant(octets, || {
            let _ = dire.send(porte.en_vol());
            let _ = attendre.recv();
        });
    });
    Decodage { entre, lacher, fil }
}

impl Decodage {
    fn finir(self) {
        let _ = self.lacher.send(());
        self.fil.join().expect("le décodage a fini");
    }
}

fn porte(machine: fn() -> Option<Memoire>) -> &'static Admission {
    Box::leak(Box::new(Admission::avec(machine)))
}

/// **Deux décodages qui ne tiennent pas ensemble passent l'un après l'autre** : le second
/// attend la fin du premier, puis entre seul ; à la fin, tout est rendu.
#[test]
fn test_memoire_3_ce_qui_ne_tient_pas_attend_la_fin_d_un_autre() {
    let porte = porte(mille);
    let premier = decoder(porte, 400);
    assert_eq!(premier.entre.recv_timeout(ENTRER), Ok(400));
    let second = decoder(porte, 200);
    assert!(
        second.entre.recv_timeout(RESTER_DEHORS).is_err(),
        "400 + 200 dépasse la moitié de 1 000 : il attend"
    );
    premier.finir();
    assert_eq!(
        second.entre.recv_timeout(ENTRER),
        Ok(200),
        "le premier fini, il entre seul"
    );
    second.finir();
    assert_eq!(porte.en_vol(), 0, "tout est rendu");
}

/// **Ce qui tient ensemble passe ensemble**, sans attendre.
#[test]
fn test_memoire_3_ce_qui_tient_ensemble_passe_ensemble() {
    let porte = porte(mille);
    let a = decoder(porte, 300);
    assert_eq!(a.entre.recv_timeout(ENTRER), Ok(300));
    let b = decoder(porte, 200);
    assert_eq!(b.entre.recv_timeout(ENTRER), Ok(500), "pile la moitié");
    a.finir();
    b.finir();
}

/// **Un décodage seul passe toujours**, même plus grand que la moitié de ce qui reste — sinon
/// une grande photo ne viendrait jamais ; et sur une machine qui ne dit pas sa mémoire, tout
/// passe, comme avant.
#[test]
fn test_memoire_3_seul_ou_sans_mesure_il_passe() {
    let seul = decoder(porte(mille), 10_000);
    assert_eq!(seul.entre.recv_timeout(ENTRER), Ok(10_000));
    seul.finir();
    let aveugle = porte(|| None);
    let a = decoder(aveugle, 10_000);
    assert_eq!(a.entre.recv_timeout(ENTRER), Ok(10_000));
    let b = decoder(aveugle, 10_000);
    assert_eq!(b.entre.recv_timeout(ENTRER), Ok(20_000));
    a.finir();
    b.finir();
}

/// **Le pic se somme exactement** : une image de 4 × 4 a trois niveaux — 16, 4 et 1 pixels —,
/// soit 84 octets de pyramide ; plus quatre octets par pixel natif pour le décodeur (le pire
/// mesuré), et ses octets encodés.
#[test]
fn test_memoire_3_le_pic_se_somme_niveau_par_niveau() {
    assert_eq!(pic(10, (4, 4)), 10 + (16 + 4 + 1) * 4 + 16 * 4);
    // Un côté impair s'arrondit vers le haut à chaque étage : 3 × 1, puis 2 × 1, puis 1 × 1.
    assert_eq!(pic(0, (3, 1)), (3 + 2 + 1) * 4 + 3 * 4);
}
