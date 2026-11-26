//! Ce qui ne tient qu'au lecteur de GStreamer ; les épreuves de tout lecteur du système vivent
//! dans `renderer/video/epreuves.rs`.

use super::*;

#[test]
fn video_1_les_rangees_coupent_ce_qui_depasse_et_refusent_ce_qui_manque() {
    assert_eq!(rangees(&[1; 40], 2, 4).map(|v| v.len()), Some(32));
    assert_eq!(rangees(&[1; 31], 2, 4), None);
}
