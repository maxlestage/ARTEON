//! Les langues d'ARTEON : le français (la langue d'origine, à `/`), l'anglais (`/en/`) et
//! l'espagnol (`/es/`).
//!
//! La langue de la page en cours de rendu est gardée par fil d'exécution : le serveur la
//! fixe au début de chaque page (les pages se rendent d'un trait, sans attente), et, dans
//! le navigateur, les îlots la lisent dans `<html lang>`. Les textes de l'interface
//! s'écrivent dans les trois langues côte à côte : `t("Accueil", "Home", "Inicio")`.

use std::cell::Cell;

use crate::artworks::{EXHIBITIONS, Exhibition};

/// Une langue du site.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Fr,
    En,
    Es,
}

impl Lang {
    /// Toutes les langues, le français d'abord.
    pub const ALL: [Lang; 3] = [Lang::Fr, Lang::En, Lang::Es];

    /// Le code de la langue (`fr`, `en`, `es`).
    pub fn code(self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
            Lang::Es => "es",
        }
    }

    /// Le début des adresses : rien pour le français, `/en` et `/es` sinon.
    pub fn prefix(self) -> &'static str {
        match self {
            Lang::Fr => "",
            Lang::En => "/en",
            Lang::Es => "/es",
        }
    }

    /// Le nom de la langue, dans cette langue.
    pub fn name(self) -> &'static str {
        match self {
            Lang::Fr => "Français",
            Lang::En => "English",
            Lang::Es => "Español",
        }
    }

    /// La langue d'un code (`en`, `es-MX`…), le français par défaut.
    pub fn from_code(code: &str) -> Lang {
        match code.get(..2).unwrap_or("") {
            "en" => Lang::En,
            "es" => Lang::Es,
            _ => Lang::Fr,
        }
    }

    /// Les expositions dans cette langue (traduites dans `expositions/traductions/`).
    pub fn exhibitions(self) -> &'static [Exhibition] {
        match self {
            Lang::Fr => EXHIBITIONS,
            Lang::En => crate::artworks::EXHIBITIONS_EN,
            Lang::Es => crate::artworks::EXHIBITIONS_ES,
        }
    }

    /// Le nom des fichiers de textes d'une exposition dans cette langue
    /// (`donnees/monet.json`, `donnees/monet.en.json`).
    pub fn texts_file(self, slug: &str) -> String {
        match self {
            Lang::Fr => format!("{slug}.json"),
            other => format!("{slug}.{}.json", other.code()),
        }
    }
}

thread_local! {
    static LANG: Cell<Lang> = const { Cell::new(Lang::Fr) };
}

/// La langue de la page en cours.
pub fn lang() -> Lang {
    LANG.with(Cell::get)
}

/// Fixe la langue de la page en cours.
pub fn set_lang(lang: Lang) {
    LANG.with(|l| l.set(lang));
}

/// Un texte de l'interface, dans la langue de la page.
pub fn t(fr: &'static str, en: &'static str, es: &'static str) -> &'static str {
    match lang() {
        Lang::Fr => fr,
        Lang::En => en,
        Lang::Es => es,
    }
}

/// Une adresse du site dans la langue de la page (`/credits` → `/en/credits`).
pub fn u(path: &str) -> String {
    let prefix = lang().prefix();
    if prefix.is_empty() || !path.starts_with('/') {
        path.to_owned()
    } else if path == "/" {
        format!("{prefix}/")
    } else if let Some(rest) = path.strip_prefix("/#") {
        format!("{prefix}/#{rest}")
    } else {
        format!("{prefix}{path}")
    }
}

/// La même exposition dans la langue de la page.
pub fn localized(ex: &'static Exhibition) -> &'static Exhibition {
    lang().exhibitions().iter().find(|e| e.slug == ex.slug).unwrap_or(ex)
}

/// Un texte formaté dans la langue de la page : `tf!("{} œuvres", "{} works", "{} obras", n)`.
#[macro_export]
macro_rules! tf {
    ($fr:literal, $en:literal, $es:literal $(, $arg:expr)* $(,)?) => {
        match $crate::langue::lang() {
            $crate::langue::Lang::Fr => format!($fr $(, $arg)*),
            $crate::langue::Lang::En => format!($en $(, $arg)*),
            $crate::langue::Lang::Es => format!($es $(, $arg)*),
        }
    };
}
