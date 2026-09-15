//! Ce qu'un fichier déposé sur le canevas devient, et sous quelle forme il s'y lit.
//!
//! # Ce que le noyau tranche, et ce qu'il laisse au décodeur
//!
//! Le noyau dit **ce qui se lit comme du texte**. C'est une question de domaine, que
//! l'export, la persistance et un futur `glucose-brain` posent toutes.
//!
//! Il ne dit pas ce qui est une image : il n'a pas de décodeur, et n'en veut pas. Glucose
//! Tauri tenait pour cela une liste d'extensions — `png|jpg|jpeg|gif|webp|avif|svg|bmp|tiff?`
//! —, mais une telle liste **diverge** de ce que le décodeur sait vraiment lire, et la
//! divergence se voit à l'écran : un `.svg` classé image refuse de se charger, alors qu'il
//! aurait fait un lanceur utile. La question se mesure au lieu de se deviner, en tentant
//! l'en-tête du fichier, et la liste disparaît avec elle.

/// Combien d'octets d'un fichier se posent sur une carte.
///
/// La valeur de Glucose Tauri. Au-delà, un journal de plusieurs mégaoctets déposé par
/// mégarde emporterait le document avec lui.
pub const INLINE_MAX_BYTES: usize = 100_000;

/// Comment le contenu d'un fichier se pose sur le canevas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readable {
    /// Du Markdown : il se rend tel quel, titres et formules comprises.
    Markdown,
    /// Autre chose de lisible : son contenu se pose dans un bloc marqué de ce langage.
    Fenced(&'static str),
}

/// Les extensions qui se lisent, et le langage dans lequel elles se posent.
///
/// **Triée par extension** — [`readable`] y cherche par dichotomie, et un test vérifie le
/// tri à chaque compilation. Ce n'est pas une optimisation : c'est ce qui fait qu'un
/// doublon ou une faute de frappe ne peut pas entrer sans qu'un test le dise.
const READABLE: &[(&str, Readable)] = &[
    ("asm", Readable::Fenced("asm")),
    ("bash", Readable::Fenced("bash")),
    ("bib", Readable::Fenced("bibtex")),
    ("c", Readable::Fenced("c")),
    ("cc", Readable::Fenced("cpp")),
    ("cfg", Readable::Fenced("text")),
    ("cjs", Readable::Fenced("javascript")),
    ("clj", Readable::Fenced("clojure")),
    ("conf", Readable::Fenced("text")),
    ("cpp", Readable::Fenced("cpp")),
    ("cs", Readable::Fenced("csharp")),
    ("csv", Readable::Fenced("csv")),
    ("dart", Readable::Fenced("dart")),
    ("env", Readable::Fenced("text")),
    ("erl", Readable::Fenced("erlang")),
    ("ex", Readable::Fenced("elixir")),
    ("exs", Readable::Fenced("elixir")),
    ("fish", Readable::Fenced("fish")),
    ("fs", Readable::Fenced("fsharp")),
    ("gitattributes", Readable::Fenced("text")),
    ("gitignore", Readable::Fenced("text")),
    ("go", Readable::Fenced("go")),
    ("h", Readable::Fenced("c")),
    ("hpp", Readable::Fenced("cpp")),
    ("hs", Readable::Fenced("haskell")),
    ("htm", Readable::Fenced("html")),
    ("html", Readable::Fenced("html")),
    ("ini", Readable::Fenced("text")),
    ("java", Readable::Fenced("java")),
    ("jl", Readable::Fenced("julia")),
    ("js", Readable::Fenced("javascript")),
    ("json", Readable::Fenced("json")),
    ("jsonl", Readable::Fenced("json")),
    ("jsx", Readable::Fenced("jsx")),
    ("kt", Readable::Fenced("kotlin")),
    ("log", Readable::Fenced("text")),
    ("lua", Readable::Fenced("lua")),
    ("markdown", Readable::Markdown),
    ("md", Readable::Markdown),
    ("mjs", Readable::Fenced("javascript")),
    ("nim", Readable::Fenced("nim")),
    ("php", Readable::Fenced("php")),
    ("ps1", Readable::Fenced("powershell")),
    ("py", Readable::Fenced("python")),
    ("r", Readable::Fenced("r")),
    ("rb", Readable::Fenced("ruby")),
    ("rs", Readable::Fenced("rust")),
    ("s", Readable::Fenced("asm")),
    ("scala", Readable::Fenced("scala")),
    ("sh", Readable::Fenced("bash")),
    ("sql", Readable::Fenced("sql")),
    ("swift", Readable::Fenced("swift")),
    ("tex", Readable::Fenced("latex")),
    ("toml", Readable::Fenced("toml")),
    ("ts", Readable::Fenced("typescript")),
    ("tsv", Readable::Fenced("csv")),
    ("tsx", Readable::Fenced("tsx")),
    ("txt", Readable::Fenced("text")),
    ("vim", Readable::Fenced("vim")),
    ("xml", Readable::Fenced("xml")),
    ("yaml", Readable::Fenced("yaml")),
    ("yml", Readable::Fenced("yaml")),
    ("zig", Readable::Fenced("zig")),
    ("zsh", Readable::Fenced("bash")),
];

/// L'extension d'un nom de fichier, en minuscules, sans son point.
///
/// Un nom qui commence par un point et n'en a pas d'autre — `.gitignore` — rend ce qui
/// suit ce point : c'est bien la façon dont on le désigne.
pub fn extension(name: &str) -> String {
    name.rsplit(['/', '\\'])
        .next()
        .unwrap_or(name)
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_default()
}

/// Ce fichier se lit-il comme du texte, et sous quelle forme ?
pub fn readable(name: &str) -> Option<Readable> {
    let ext = extension(name);
    READABLE
        .binary_search_by_key(&ext.as_str(), |(key, _)| key)
        .ok()
        .map(|index| READABLE[index].1)
}

/// Le début de `text` qui tient dans `limit` octets, coupé sur une frontière de caractère.
///
/// Trancher au milieu d'un caractère accentué rendrait la chaîne invalide — et c'est
/// exactement ce qui arrive dès qu'un journal français dépasse la limite.
pub fn truncate_on_char_boundary(text: &str, limit: usize) -> &str {
    if text.len() <= limit {
        return text;
    }
    let mut end = limit;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// Le contenu d'un fichier, mis en Markdown pour une carte de Glucose.
///
/// Le titre porte le nom du fichier, le corps son contenu — tel quel s'il est déjà du
/// Markdown, dans un bloc marqué de son langage sinon. Un pied signale la coupure quand
/// le fichier dépassait [`INLINE_MAX_BYTES`], pour qu'une carte tronquée ne se fasse
/// jamais passer pour un fichier entier.
///
/// Glucose Tauri préfixe son titre d'un emoji ; il est laissé de côté tant que le rendu
/// de texte n'a pas de police de couleur — un glyphe absent ne dit rien, il fait un carré.
pub fn as_markdown(name: &str, content: &str, truncated: bool) -> String {
    let body = match readable(name) {
        Some(Readable::Markdown) | None => content.to_string(),
        Some(Readable::Fenced(lang)) => format!("```{lang}\n{content}\n```"),
    };
    let footer = if truncated {
        format!("\n\n_(tronqué à {INLINE_MAX_BYTES} octets — le fichier est plus grand)_")
    } else {
        String::new()
    };
    format!("### {name}\n\n{body}{footer}")
}

#[cfg(test)]
mod tests;
