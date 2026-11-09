//! Les textes longs d'une exposition, téléchargés par le musée 3D : les notices, les
//! biographies, la présentation du bâtiment et le fichier source (pour l'éditeur). Ils ne
//! sont pas compilés dans le WebAssembly (voir `text!` dans src/artworks.rs) : build.rs
//! les écrit dans `donnees/<exposition>.json`, lus ici avec le JSON du navigateur.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};

/// La notice d'une œuvre.
#[derive(Default)]
pub struct WorkText {
    pub description: String,
    pub evocation: String,
    pub story: String,
    pub manner: String,
    pub details: Vec<String>,
    pub anecdote: String,
}

impl WorkText {
    /// Les rubriques non vides, avec leur titre.
    pub fn sections(&self) -> Vec<(&'static str, &str)> {
        [
            ("Ce que l'on voit", self.description.as_str()),
            ("Ce qu'elle évoque", self.evocation.as_str()),
            ("Son histoire", self.story.as_str()),
            ("La manière", self.manner.as_str()),
        ]
        .into_iter()
        .filter(|(_, t)| !t.is_empty())
        .collect()
    }
}

/// La vie et la manière d'un artiste.
#[derive(Default)]
pub struct ArtistText {
    pub life: String,
    pub biography: String,
    pub style: String,
}

#[derive(Default)]
pub struct Texts {
    pub works: HashMap<String, WorkText>,
    pub artists: HashMap<String, ArtistText>,
    pub presentation: String,
    pub here: String,
    pub source: String,
}

/// Les textes, partagés entre la page et l'interface : `None` tant qu'ils se chargent.
pub type SharedTexts = Rc<RefCell<Option<Texts>>>;

fn field(object: &JsValue, key: &str) -> JsValue {
    js_sys::Reflect::get(object, &JsValue::from_str(key)).unwrap_or(JsValue::UNDEFINED)
}

fn string(object: &JsValue, key: &str) -> String {
    field(object, key).as_string().unwrap_or_default()
}

/// Les clés d'un objet JSON, avec leur valeur.
fn entries(object: &JsValue) -> Vec<(String, JsValue)> {
    let Some(object) = object.dyn_ref::<js_sys::Object>() else {
        return Vec::new();
    };
    js_sys::Object::keys(object)
        .iter()
        .filter_map(|k| k.as_string())
        .map(|k| {
            let v = field(object, &k);
            (k, v)
        })
        .collect()
}

impl Texts {
    pub fn parse(json: &str) -> Option<Texts> {
        let root = js_sys::JSON::parse(json).ok()?;
        let works = entries(&field(&root, "works"))
            .into_iter()
            .map(|(slug, w)| {
                let list = field(&w, "details");
                let details = if js_sys::Array::is_array(&list) {
                    js_sys::Array::from(&list)
                        .iter()
                        .filter_map(|d| d.as_string())
                        .collect()
                } else {
                    Vec::new()
                };
                let text = WorkText {
                    description: string(&w, "description"),
                    evocation: string(&w, "evocation"),
                    story: string(&w, "story"),
                    manner: string(&w, "manner"),
                    details,
                    anecdote: string(&w, "anecdote"),
                };
                (slug, text)
            })
            .collect();
        let artists = entries(&field(&root, "artists"))
            .into_iter()
            .map(|(name, a)| {
                let text = ArtistText {
                    life: string(&a, "life"),
                    biography: string(&a, "biography"),
                    style: string(&a, "style"),
                };
                (name, text)
            })
            .collect();
        let museum = field(&root, "museum");
        Some(Texts {
            works,
            artists,
            presentation: string(&museum, "presentation"),
            here: string(&museum, "here"),
            source: string(&root, "source"),
        })
    }
}
