//! Les discriminants stables du format : une énumération du modèle ↔ un `u8` sur le disque.
//!
//! # INVARIANT PERSIST-4 — un numéro attribué ne bouge JAMAIS
//!
//! Une variante nouvelle prend le numéro suivant ; aucune n'est renumérotée, aucune n'est
//! réutilisée après suppression. C'est la seule règle qui rende un `.glucose` d'hier lisible
//! demain. Les rassembler dans un unique fichier rend cette règle vérifiable d'un coup d'œil,
//! au lieu de la disperser dans chaque encodeur.
//!
//! Un numéro inconnu n'est jamais deviné ni remplacé par une valeur par défaut : on refuse le
//! fichier avec un message qui dit quoi faire. Accepter silencieusement une membrane
//! « classique » là où le disque disait « étirée » corromprait le document de l'utilisateur
//! sans qu'il le sache.

use crate::error::{CoreError, CoreResult};
use crate::types::{
    ArrowPredicate, CurtainEditable, CurtainVisibility, FolderSortMode, MembraneMode,
    StickyOperator,
};

pub fn unknown(kind: &str, tag: u8) -> CoreError {
    CoreError::DeserializationError(format!(
        "valeur de {kind} inconnue ({tag}) : ce document vient d'une version plus récente de Glucose, mets-la à jour"
    ))
}

pub fn predicate_tag(p: ArrowPredicate) -> u8 {
    match p {
        ArrowPredicate::EstPrecurseur => 0,
        ArrowPredicate::Contredit => 1,
        ArrowPredicate::HeriteDe => 2,
        ArrowPredicate::Inspire => 3,
        ArrowPredicate::DependDe => 4,
        ArrowPredicate::Illustre => 5,
    }
}

pub fn predicate_from_tag(tag: u8) -> CoreResult<ArrowPredicate> {
    match tag {
        0 => Ok(ArrowPredicate::EstPrecurseur),
        1 => Ok(ArrowPredicate::Contredit),
        2 => Ok(ArrowPredicate::HeriteDe),
        3 => Ok(ArrowPredicate::Inspire),
        4 => Ok(ArrowPredicate::DependDe),
        5 => Ok(ArrowPredicate::Illustre),
        other => Err(unknown("prédicat de flèche", other)),
    }
}

pub fn operator_tag(op: StickyOperator) -> u8 {
    match op {
        StickyOperator::And => 0,
        StickyOperator::Or => 1,
        StickyOperator::But => 2,
        StickyOperator::Because => 3,
    }
}

pub fn operator_from_tag(tag: u8) -> CoreResult<StickyOperator> {
    match tag {
        0 => Ok(StickyOperator::And),
        1 => Ok(StickyOperator::Or),
        2 => Ok(StickyOperator::But),
        3 => Ok(StickyOperator::Because),
        other => Err(unknown("opérateur de pense-bête", other)),
    }
}

pub fn membrane_mode_tag(mode: MembraneMode) -> u8 {
    match mode {
        MembraneMode::Classic => 0,
        MembraneMode::Minimized => 1,
        MembraneMode::Stretched => 2,
    }
}

pub fn membrane_mode_from_tag(tag: u8) -> CoreResult<MembraneMode> {
    match tag {
        0 => Ok(MembraneMode::Classic),
        1 => Ok(MembraneMode::Minimized),
        2 => Ok(MembraneMode::Stretched),
        other => Err(unknown("mode de membrane", other)),
    }
}

pub fn visibility_tag(v: CurtainVisibility) -> u8 {
    match v {
        CurtainVisibility::Private => 0,
        CurtainVisibility::Shared => 1,
    }
}

pub fn visibility_from_tag(tag: u8) -> CoreResult<CurtainVisibility> {
    match tag {
        0 => Ok(CurtainVisibility::Private),
        1 => Ok(CurtainVisibility::Shared),
        other => Err(unknown("visibilité de rideau", other)),
    }
}

pub fn editable_tag(e: CurtainEditable) -> u8 {
    match e {
        CurtainEditable::Owner => 0,
        CurtainEditable::Everyone => 1,
    }
}

pub fn editable_from_tag(tag: u8) -> CoreResult<CurtainEditable> {
    match tag {
        0 => Ok(CurtainEditable::Owner),
        1 => Ok(CurtainEditable::Everyone),
        other => Err(unknown("droit d'édition de rideau", other)),
    }
}

pub fn sort_mode_tag(mode: FolderSortMode) -> u8 {
    match mode {
        FolderSortMode::NameAsc => 0,
        FolderSortMode::NameDesc => 1,
        FolderSortMode::Type => 2,
        FolderSortMode::SizeDesc => 3,
        FolderSortMode::SizeAsc => 4,
        FolderSortMode::ModifiedDesc => 5,
        FolderSortMode::ModifiedAsc => 6,
    }
}

pub fn sort_mode_from_tag(tag: u8) -> CoreResult<FolderSortMode> {
    match tag {
        0 => Ok(FolderSortMode::NameAsc),
        1 => Ok(FolderSortMode::NameDesc),
        2 => Ok(FolderSortMode::Type),
        3 => Ok(FolderSortMode::SizeDesc),
        4 => Ok(FolderSortMode::SizeAsc),
        5 => Ok(FolderSortMode::ModifiedDesc),
        6 => Ok(FolderSortMode::ModifiedAsc),
        other => Err(unknown("tri de dossier", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// INVARIANT PERSIST-4 : ce test fige les numéros. Le modifier veut dire qu'on vient de
    /// rendre illisibles tous les `.glucose` déjà écrits — c'est l'alarme, pas un détail.
    #[test]
    fn test_the_discriminants_are_frozen() {
        assert_eq!(predicate_tag(ArrowPredicate::EstPrecurseur), 0);
        assert_eq!(predicate_tag(ArrowPredicate::Illustre), 5);
        assert_eq!(operator_tag(StickyOperator::And), 0);
        assert_eq!(operator_tag(StickyOperator::Because), 3);
        assert_eq!(membrane_mode_tag(MembraneMode::Classic), 0);
        assert_eq!(membrane_mode_tag(MembraneMode::Stretched), 2);
        assert_eq!(visibility_tag(CurtainVisibility::Private), 0);
        assert_eq!(editable_tag(CurtainEditable::Owner), 0);
        assert_eq!(sort_mode_tag(FolderSortMode::NameAsc), 0);
        assert_eq!(sort_mode_tag(FolderSortMode::ModifiedAsc), 6);
    }

    #[test]
    fn test_an_unknown_discriminant_asks_the_user_to_update() {
        for message in [
            predicate_from_tag(200)
                .expect_err("200 n'est pas un prédicat")
                .to_string(),
            operator_from_tag(200)
                .expect_err("200 n'est pas un opérateur")
                .to_string(),
            membrane_mode_from_tag(200)
                .expect_err("200 n'est pas un mode")
                .to_string(),
            visibility_from_tag(200)
                .expect_err("200 n'est pas une visibilité")
                .to_string(),
            editable_from_tag(200)
                .expect_err("200 n'est pas un droit")
                .to_string(),
            sort_mode_from_tag(200)
                .expect_err("200 n'est pas un tri")
                .to_string(),
        ] {
            assert!(message.contains("mets-la à jour"), "message : {message}");
        }
    }
}
