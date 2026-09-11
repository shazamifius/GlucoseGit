//! Une saisie de texte d'une ligne, sûre en UTF-8.
//!
//! # ENTRY-1 — le curseur est un indice d'octet, toujours sur une frontière de caractère
//!
//! `String` s'indexe en octets, et `é` en occupe deux. Un curseur avancé d'un octet tombe donc
//! au milieu d'un caractère, et `insert` ou `drain` y **paniquent**. Chaque déplacement de ce
//! type saute donc jusqu'à la prochaine frontière, et `debug_assert` vérifie l'invariant à la
//! sortie de chaque opération.
//!
//! Ce type existe parce que le panneau DOMAINES a besoin d'une saisie et que
//! [`crate::renderer::TextEditSession`] n'en est pas une : cette dernière est une session
//! d'édition d'**annotation**, avec son identifiant de nœud, son clignotement et sa validation
//! dans le document. Recopier sa logique de curseur aurait fait deux implémentations du même
//! parcours d'octets, donc deux occasions de paniquer.

/// Un tampon de texte et la position de son curseur, en octets.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextEntry {
    buffer: String,
    cursor: usize,
}

impl TextEntry {
    /// Une saisie amorcée par `initial`, curseur à la fin.
    pub fn new(initial: impl Into<String>) -> Self {
        let buffer = initial.into();
        let cursor = buffer.len();
        Self { buffer, cursor }
    }

    pub fn text(&self) -> &str {
        &self.buffer
    }

    /// Le texte qui précède le curseur — ce qu'il faut mesurer pour poser le caret.
    pub fn before_cursor(&self) -> &str {
        &self.buffer[..self.cursor]
    }

    pub fn into_text(self) -> String {
        self.buffer
    }

    pub fn insert(&mut self, text: &str) {
        self.buffer.insert_str(self.cursor, text);
        self.cursor += text.len();
        self.debug_check();
    }

    /// Efface le caractère à gauche. Rend `false` si le curseur est au début.
    pub fn backspace(&mut self) -> bool {
        let Some(previous) = self.previous_boundary() else {
            return false;
        };
        self.buffer.drain(previous..self.cursor);
        self.cursor = previous;
        self.debug_check();
        true
    }

    /// Efface le caractère à droite. Rend `false` si le curseur est à la fin.
    pub fn delete(&mut self) -> bool {
        let Some(next) = self.next_boundary() else {
            return false;
        };
        self.buffer.drain(self.cursor..next);
        self.debug_check();
        true
    }

    pub fn move_left(&mut self) -> bool {
        let Some(previous) = self.previous_boundary() else {
            return false;
        };
        self.cursor = previous;
        self.debug_check();
        true
    }

    pub fn move_right(&mut self) -> bool {
        let Some(next) = self.next_boundary() else {
            return false;
        };
        self.cursor = next;
        self.debug_check();
        true
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.buffer.len();
    }

    /// Frontière de caractère immédiatement à gauche du curseur.
    fn previous_boundary(&self) -> Option<usize> {
        if self.cursor == 0 {
            return None;
        }
        let mut index = self.cursor - 1;
        while index > 0 && !self.buffer.is_char_boundary(index) {
            index -= 1;
        }
        Some(index)
    }

    /// Frontière de caractère immédiatement à droite du curseur.
    fn next_boundary(&self) -> Option<usize> {
        if self.cursor >= self.buffer.len() {
            return None;
        }
        let mut index = self.cursor + 1;
        while index < self.buffer.len() && !self.buffer.is_char_boundary(index) {
            index += 1;
        }
        Some(index)
    }

    /// ENTRY-1 — le curseur ne quitte jamais une frontière de caractère.
    fn debug_check(&self) {
        debug_assert!(
            self.buffer.is_char_boundary(self.cursor),
            "curseur {} au milieu d'un caractère de « {} »",
            self.cursor,
            self.buffer
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_new_entry_places_the_cursor_at_the_end() {
        let entry = TextEntry::new("Science");
        assert_eq!(entry.text(), "Science");
        assert_eq!(entry.before_cursor(), "Science");
    }

    #[test]
    fn test_entry_1_a_cursor_steps_over_a_whole_character_not_a_byte() {
        // « é » pèse deux octets, « 🜁 » quatre : un curseur qui avance d'un octet paniquerait
        // à l'insertion suivante.
        let mut entry = TextEntry::new("Théorie");
        entry.home();
        for expected in ["T", "Th", "Thé", "Théo"] {
            assert!(entry.move_right());
            assert_eq!(entry.before_cursor(), expected);
        }
        entry.end();
        assert!(entry.backspace());
        assert_eq!(entry.text(), "Théori");
    }

    #[test]
    fn test_an_entry_refuses_to_step_past_its_own_ends() {
        let mut entry = TextEntry::new("ab");
        entry.home();
        assert!(!entry.move_left());
        assert!(!entry.backspace());
        entry.end();
        assert!(!entry.move_right());
        assert!(!entry.delete());
        assert_eq!(entry.text(), "ab");
    }

    #[test]
    fn test_typing_and_erasing_around_a_multibyte_character() {
        let mut entry = TextEntry::new("");
        entry.insert("Con");
        entry.insert("œ");
        entry.insert("ur");
        assert_eq!(entry.text(), "Conœur");

        entry.home();
        for _ in 0..3 {
            assert!(entry.move_right());
        }
        assert!(entry.delete(), "le œ doit partir d'un coup");
        assert_eq!(entry.text(), "Conur");
        assert_eq!(entry.before_cursor(), "Con");
    }

    #[test]
    fn test_an_entry_hands_back_exactly_what_was_typed() {
        let mut entry = TextEntry::new("Jeu");
        entry.insert(" vidéo");
        assert_eq!(entry.into_text(), "Jeu vidéo");
    }
}
