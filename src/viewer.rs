//! Les commandes de la galerie, dans le navigateur : plein écran, cartel de l'œuvre
//! regardée, œuvre précédente ou suivante, pause, et la toile en haute définition.
//!
//! Le plein écran passe par l'API Fullscreen quand le navigateur la propose pour une
//! page (ordinateurs, Android, iPad) ; sur iPhone, qui ne la propose pas, la galerie
//! occupe tout l'écran avec la classe CSS `is-fullscreen`. Les deux vont ensemble.

use std::cell::Cell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Document, Element, Event, KeyboardEvent};

use crate::artworks::Exhibition;
use crate::gallery::SharedControl;

/// Ce qui doit être retiré quand l'îlot s'en va : l'écoute du clavier sur la page.
pub struct Commands {
    document: Document,
    keys: Closure<dyn FnMut(KeyboardEvent)>,
}

impl Drop for Commands {
    fn drop(&mut self) {
        let _ = self
            .document
            .remove_event_listener_with_callback("keydown", self.keys.as_ref().unchecked_ref());
        if let Some(body) = self.document.body() {
            let _ = body.class_list().remove_1("viewer-open");
        }
    }
}

/// Le début des adresses du site (`/ARTEON` sur GitHub Pages, rien sur un domaine) : ce
/// qui précède `/<exposition>/galerie` dans l'adresse de la page.
fn base_path(slug: &str) -> String {
    let path = web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default();
    match path.find(&format!("/{slug}/galerie")) {
        Some(i) => path[..i].to_owned(),
        None => String::new(),
    }
}

fn find(shell: &Element, class: &str) -> Option<Element> {
    shell.query_selector(&format!(".{class}")).ok().flatten()
}

/// Appelle `action` à chaque clic sur l'élément de classe `class` de la galerie.
fn on_click(shell: &Element, class: &str, action: impl FnMut(Event) + 'static) {
    if let Some(element) = find(shell, class) {
        let closure = Closure::<dyn FnMut(Event)>::new(action);
        let _ = element.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        closure.forget(); // l'élément et l'écoute disparaissent avec la page
    }
}

/// Branche les commandes autour de `canvas` (dans `.viewer-shell`).
pub fn attach(
    canvas: &Element,
    ex: &'static Exhibition,
    control: SharedControl,
) -> Option<Commands> {
    let shell = canvas.closest(".viewer-shell").ok().flatten()?;
    let document = web_sys::window()?.document()?;
    let base = base_path(ex.slug);
    // L'API Fullscreen a-t-elle été utilisée (pour savoir si sa sortie doit tout fermer) ?
    let native = Rc::new(Cell::new(false));

    let enter = {
        let (shell, document, native) = (shell.clone(), document.clone(), native.clone());
        Rc::new(move || {
            let _ = shell.class_list().add_1("is-fullscreen");
            if let Some(body) = document.body() {
                let _ = body.class_list().add_1("viewer-open");
            }
            native.set(shell.request_fullscreen().is_ok());
        })
    };
    let exit = {
        let (shell, document, native) = (shell.clone(), document.clone(), native.clone());
        Rc::new(move || {
            let _ = shell.class_list().remove_1("is-fullscreen");
            let _ = shell.class_list().remove_1("hd-open");
            if let Some(body) = document.body() {
                let _ = body.class_list().remove_1("viewer-open");
            }
            if native.replace(false) && document.fullscreen_element().is_some() {
                document.exit_fullscreen();
            }
        })
    };
    let is_full = {
        let shell = shell.clone();
        move || shell.class_list().contains("is-fullscreen")
    };
    let toggle_pause = {
        let (shell, control) = (shell.clone(), control.clone());
        Rc::new(move || {
            let paused = {
                let mut c = control.borrow_mut();
                c.paused = !c.paused;
                c.paused
            };
            if let Some(button) = find(&shell, "ctl-play") {
                button.set_text_content(Some(if paused { "▶" } else { "❚❚" }));
                let label = if paused {
                    "Reprendre la visite"
                } else {
                    "Pause"
                };
                let _ = button.set_attribute("aria-label", label);
                let _ = button.set_attribute("title", label);
            }
            let _ = shell.class_list().toggle_with_force("is-paused", paused);
        })
    };
    let jump = {
        let control = control.clone();
        move |step: i32| control.borrow_mut().jump += step
    };
    let open_hd = {
        let (shell, control, base) = (shell.clone(), control.clone(), base.clone());
        Rc::new(move || {
            let Some(art) = control.borrow().current.map(|i| &ex.artworks[i]) else {
                return;
            };
            if let Some(image) = find(&shell, "hd-img") {
                let _ = image.set_attribute("src", &format!("{base}{}", art.image_url()));
                let _ = image.set_attribute(
                    "alt",
                    &format!("{}, {} ({})", art.title, ex.artist, art.year),
                );
            }
            if let Some(title) = find(&shell, "hd-title") {
                title.set_text_content(Some(&format!(
                    "{} · {} · {}",
                    art.title, art.year, art.collection
                )));
            }
            let _ = shell.class_list().add_1("hd-open");
        })
    };
    let close_hd = {
        let shell = shell.clone();
        Rc::new(move || {
            let _ = shell.class_list().remove_1("hd-open");
        })
    };

    // Le cartel suit l'œuvre regardée.
    {
        let shell = shell.clone();
        control.borrow_mut().on_change = Some(Rc::new(move |current: Option<usize>| {
            let _ = shell
                .class_list()
                .toggle_with_force("has-art", current.is_some());
            let Some(art) = current.map(|i| &ex.artworks[i]) else {
                return;
            };
            if let Some(title) = find(&shell, "cap-title") {
                title.set_text_content(Some(art.title));
            }
            if let Some(meta) = find(&shell, "cap-meta") {
                meta.set_text_content(Some(&format!(
                    "{} · {} · {}",
                    ex.artist, art.year, art.collection
                )));
            }
        }));
    }

    // Toucher la scène : plein écran ; en plein écran, pause ou reprise.
    {
        let (enter, toggle_pause, is_full) = (enter.clone(), toggle_pause.clone(), is_full.clone());
        let closure =
            Closure::<dyn FnMut(Event)>::new(
                move |_| if is_full() { toggle_pause() } else { enter() },
            );
        let _ = canvas.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        closure.forget();
    }
    on_click(&shell, "viewer-full", {
        let enter = enter.clone();
        move |_| enter()
    });
    on_click(&shell, "ctl-exit", {
        let exit = exit.clone();
        move |_| exit()
    });
    on_click(&shell, "ctl-play", {
        let toggle_pause = toggle_pause.clone();
        move |_| toggle_pause()
    });
    on_click(&shell, "ctl-prev", {
        let jump = jump.clone();
        move |_| jump(-1)
    });
    on_click(&shell, "ctl-next", {
        let jump = jump.clone();
        move |_| jump(1)
    });
    on_click(&shell, "cap-hd", {
        let open_hd = open_hd.clone();
        move |_| open_hd()
    });
    on_click(&shell, "hd-close", {
        let close_hd = close_hd.clone();
        move |_| close_hd()
    });

    // Sortie du plein écran natif (touche Échap du navigateur, geste du système).
    {
        let (exit, document_ref, native) = (exit.clone(), document.clone(), native.clone());
        let closure = Closure::<dyn FnMut(Event)>::new(move |_| {
            if native.get() && document_ref.fullscreen_element().is_none() {
                exit();
            }
        });
        let _ = document
            .add_event_listener_with_callback("fullscreenchange", closure.as_ref().unchecked_ref());
        closure.forget();
    }

    // Le clavier, quand la galerie est en plein écran : flèches, espace, Échap, F.
    let keys = {
        let shell = shell.clone();
        Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
            let full = is_full();
            match event.key().as_str() {
                "f" | "F" if !full => enter(),
                "Escape" if shell.class_list().contains("hd-open") => close_hd(),
                "Escape" if full => exit(),
                "ArrowRight" if full => jump(1),
                "ArrowLeft" if full => jump(-1),
                " " if full => {
                    event.prevent_default();
                    toggle_pause();
                }
                _ => {}
            }
        })
    };
    let _ = document.add_event_listener_with_callback("keydown", keys.as_ref().unchecked_ref());
    Some(Commands { document, keys })
}
