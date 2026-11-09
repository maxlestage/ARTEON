//! L'interface du musée, dans le navigateur : télécharger les données de l'exposition,
//! lancer la scène, et brancher les commandes (plein écran, cartel, visite guidée ou
//! libre, œuvre précédente ou suivante, pause, toile en haute définition, clavier,
//! souris et joysticks tactiles).
//!
//! Le plein écran passe par l'API Fullscreen quand le navigateur la propose pour une
//! page (ordinateurs, Android, iPad) ; sur iPhone, qui ne la propose pas, la galerie
//! occupe tout l'écran avec la classe CSS `is-fullscreen`. Les deux vont ensemble.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use paquets::ecs::App;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, Element, Event, HtmlElement, KeyboardEvent, PointerEvent};

use crate::artworks::Exhibition;
use crate::data::ExhibitionData;
use crate::gallery::{Mode, SharedControl};
use crate::tour::Focus;

// ─── Le chargement ────────────────────────────────────────────────────────────

/// Le début des adresses du site (`/ARTEON` sur GitHub Pages, rien sur un domaine) : ce
/// qui précède `/<exposition>` dans l'adresse de la page.
fn base_path(slug: &str) -> String {
    let path = web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default();
    let needle = format!("/{slug}");
    let mut from = 0;
    while let Some(i) = path[from..].find(&needle).map(|i| i + from) {
        let after = &path[i + needle.len()..];
        if after.is_empty() || after.starts_with('/') {
            return path[..i].to_owned();
        }
        from = i + 1;
    }
    String::new()
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    let window = web_sys::window().ok_or("pas de fenêtre")?;
    let response: web_sys::Response = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|_| "le réseau ne répond pas".to_owned())?
        .dyn_into()
        .map_err(|_| "réponse inattendue".to_owned())?;
    if !response.ok() {
        return Err(format!("{url} : erreur {}", response.status()));
    }
    let buffer = JsFuture::from(
        response
            .array_buffer()
            .map_err(|_| "réponse illisible".to_owned())?,
    )
    .await
    .map_err(|_| "téléchargement interrompu".to_owned())?;
    Ok(js_sys::Uint8Array::new(&buffer).to_vec())
}

/// La scène en cours (ou en chargement) : l'arrêter quand l'îlot s'en va.
pub struct Running {
    scene: Rc<RefCell<Option<Box<dyn std::any::Any>>>>,
    cancelled: Rc<Cell<bool>>,
}

impl Drop for Running {
    fn drop(&mut self) {
        self.cancelled.set(true);
        self.scene.borrow_mut().take();
    }
}

/// Télécharge les données de l'exposition, puis lance la scène `make` dans `canvas`. La
/// galerie qui l'entoure (`.viewer-shell`) perd sa classe `is-loading`.
pub fn start_when_loaded(
    canvas: &Element,
    ex: &'static Exhibition,
    make: impl FnOnce(Rc<ExhibitionData>) -> App + 'static,
) -> Running {
    let scene: Rc<RefCell<Option<Box<dyn std::any::Any>>>> = Rc::default();
    let cancelled = Rc::new(Cell::new(false));
    let (canvas, slot, stop) = (canvas.clone(), scene.clone(), cancelled.clone());
    wasm_bindgen_futures::spawn_local(async move {
        let url = format!("{}/donnees/{}.bin", base_path(ex.slug), ex.slug);
        let loaded = fetch_bytes(&url)
            .await
            .and_then(|bytes| ExhibitionData::new(ex, bytes));
        if stop.get() {
            return;
        }
        let shell = canvas.closest(".viewer-shell").ok().flatten();
        match loaded {
            Ok(data) => {
                let running = make(Rc::new(data)).run_in(&canvas).ok();
                *slot.borrow_mut() = Some(Box::new(running));
                if let Some(shell) = shell {
                    let _ = shell.class_list().remove_1("is-loading");
                }
            }
            Err(message) => {
                if let Some(text) = shell.as_ref().and_then(|s| find(s, "viewer-loading")) {
                    text.set_text_content(Some(&format!(
                        "Le musée n'a pas pu s'ouvrir : {message}"
                    )));
                }
            }
        }
    });
    Running { scene, cancelled }
}

// ─── Les commandes ────────────────────────────────────────────────────────────

/// Une écoute du clavier sur la page : l'événement (`keydown`, `keyup`) et la fonction.
type KeyListener = (&'static str, Closure<dyn FnMut(KeyboardEvent)>);

/// Ce qui doit être retiré quand l'îlot s'en va : l'écoute du clavier sur la page.
pub struct Commands {
    document: Document,
    listeners: Vec<KeyListener>,
}

impl Drop for Commands {
    fn drop(&mut self) {
        for (kind, listener) in &self.listeners {
            let _ = self
                .document
                .remove_event_listener_with_callback(kind, listener.as_ref().unchecked_ref());
        }
        if let Some(body) = self.document.body() {
            let _ = body.class_list().remove_1("viewer-open");
        }
    }
}

fn find(shell: &Element, class: &str) -> Option<Element> {
    shell.query_selector(&format!(".{class}")).ok().flatten()
}

fn set_button(shell: &Element, class: &str, text: &str, label: &str) {
    if let Some(button) = find(shell, class) {
        button.set_text_content(Some(text));
        let _ = button.set_attribute("aria-label", label);
        let _ = button.set_attribute("title", label);
    }
}

/// Appelle `action` à chaque clic sur l'élément de classe `class` de la galerie.
fn on_click(shell: &Element, class: &str, action: impl FnMut(Event) + 'static) {
    if let Some(element) = find(shell, class) {
        let closure = Closure::<dyn FnMut(Event)>::new(action);
        let _ = element.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        closure.forget(); // l'élément et l'écoute disparaissent avec la page
    }
}

fn listen<E: wasm_bindgen::convert::FromWasmAbi + 'static>(
    target: &Element,
    kind: &str,
    action: impl FnMut(E) + 'static,
) {
    let closure = Closure::<dyn FnMut(E)>::new(action);
    let _ = target.add_event_listener_with_callback(kind, closure.as_ref().unchecked_ref());
    closure.forget();
}

/// Un doigt ou la souris posé sur la scène en visite libre.
#[derive(Clone, Copy)]
struct Touch {
    /// Le joystick de déplacement (moitié gauche d'un écran tactile), sinon le regard.
    moves: bool,
    start: (f32, f32),
    last: (f32, f32),
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
    // Un glissé vient-il d'avoir lieu (le clic qui suit ne compte pas) ?
    let dragged = Rc::new(Cell::new(false));

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
    let is_free = {
        let control = control.clone();
        move || control.borrow().mode == Mode::Free
    };
    let toggle_pause = {
        let (shell, control) = (shell.clone(), control.clone());
        Rc::new(move || {
            let paused = {
                let mut c = control.borrow_mut();
                c.paused = !c.paused;
                c.paused
            };
            set_button(
                &shell,
                "ctl-play",
                if paused { "▶" } else { "❚❚" },
                if paused {
                    "Reprendre la visite"
                } else {
                    "Pause"
                },
            );
            let _ = shell.class_list().toggle_with_force("is-paused", paused);
        })
    };
    let toggle_mode = {
        let (shell, control) = (shell.clone(), control.clone());
        Rc::new(move || {
            let free = {
                let mut c = control.borrow_mut();
                c.mode = if c.mode == Mode::Free {
                    Mode::Guided
                } else {
                    Mode::Free
                };
                (c.forward, c.strafe, c.spin) = (0.0, 0.0, 0.0);
                c.mode == Mode::Free
            };
            let _ = shell.class_list().toggle_with_force("is-free", free);
            if free {
                set_button(
                    &shell,
                    "ctl-mode",
                    "🎬",
                    "Visite guidée : la caméra vous emmène",
                );
            } else {
                set_button(
                    &shell,
                    "ctl-mode",
                    "🚶",
                    "Visite libre : se promener soi-même",
                );
            }
        })
    };
    let jump = {
        let control = control.clone();
        move |step: i32| {
            let mut c = control.borrow_mut();
            if c.mode == Mode::Free {
                // En visite libre, « suivante » emmène devant l'œuvre suivante de la visite.
                let order = crate::tour::stops(ex)
                    .iter()
                    .filter_map(|s| s.focus)
                    .collect::<Vec<_>>();
                if !order.is_empty() {
                    let at = c.current.and_then(|f| order.iter().position(|o| *o == f));
                    let k = at.map_or(0, |i| {
                        (i as i32 + step).rem_euclid(order.len() as i32) as usize
                    });
                    c.go_to = Some(order[k]);
                }
            } else {
                c.jump += step;
            }
        }
    };
    let open_hd = {
        let (shell, control, base) = (shell.clone(), control.clone(), base.clone());
        Rc::new(move || {
            let Some(Focus::Art(i)) = control.borrow().current else {
                return;
            };
            let art = &ex.artworks[i];
            if let Some(image) = find(&shell, "hd-img") {
                let _ = image.set_attribute("src", &format!("{base}{}", art.image_url()));
                let _ = image.set_attribute(
                    "alt",
                    &format!("{}, {} ({})", art.title, art.artist_in(ex), art.year),
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

    // Le cartel suit l'œuvre regardée : un tableau (avec « Voir en HD ») ou une sculpture.
    {
        let shell = shell.clone();
        control.borrow_mut().on_change = Some(Rc::new(move |current: Option<Focus>| {
            let _ = shell
                .class_list()
                .toggle_with_force("has-art", current.is_some());
            let _ = shell
                .class_list()
                .toggle_with_force("has-hd", matches!(current, Some(Focus::Art(_))));
            let (title, meta) = match current {
                Some(Focus::Art(i)) => {
                    let a = &ex.artworks[i];
                    (
                        a.title,
                        format!("{} · {} · {}", a.artist_in(ex), a.year, a.collection),
                    )
                }
                Some(Focus::Sculpture(i)) => {
                    let s = &ex.sculptures[i];
                    (
                        s.title,
                        format!("{} · {} · {}", s.artist, s.year, s.collection),
                    )
                }
                None => return,
            };
            if let Some(t) = find(&shell, "cap-title") {
                t.set_text_content(Some(title));
            }
            if let Some(m) = find(&shell, "cap-meta") {
                m.set_text_content(Some(&meta));
            }
        }));
    }

    // Toucher la scène : plein écran ; en plein écran (visite guidée), pause ou reprise.
    {
        let (enter, toggle_pause, is_full, is_free, dragged) = (
            enter.clone(),
            toggle_pause.clone(),
            is_full.clone(),
            is_free.clone(),
            dragged.clone(),
        );
        listen(canvas, "click", move |_: Event| {
            if dragged.replace(false) {
                return;
            }
            if !is_full() {
                enter();
            } else if !is_free() {
                toggle_pause();
            }
        });
    }

    // La visite libre à la souris et au doigt : glisser pour regarder ; sur un écran
    // tactile, la moitié gauche est un joystick pour avancer.
    {
        let touches: Rc<RefCell<HashMap<i32, Touch>>> = Rc::default();
        let (control_down, touches_down, is_free_down, canvas_down) = (
            control.clone(),
            touches.clone(),
            is_free.clone(),
            canvas.clone(),
        );
        listen(canvas, "pointerdown", move |e: PointerEvent| {
            if !is_free_down() {
                return;
            }
            let (x, y) = (e.client_x() as f32, e.client_y() as f32);
            let rect = canvas_down.get_bounding_client_rect();
            let moves =
                e.pointer_type() == "touch" && (x - rect.left() as f32) < rect.width() as f32 / 2.0;
            touches_down.borrow_mut().insert(
                e.pointer_id(),
                Touch {
                    moves,
                    start: (x, y),
                    last: (x, y),
                },
            );
            let _ = canvas_down.set_pointer_capture(e.pointer_id());
            let _ = &control_down;
        });
        let (control_move, touches_move, dragged_move) =
            (control.clone(), touches.clone(), dragged.clone());
        listen(canvas, "pointermove", move |e: PointerEvent| {
            let mut all = touches_move.borrow_mut();
            let Some(t) = all.get_mut(&e.pointer_id()) else {
                return;
            };
            let (x, y) = (e.client_x() as f32, e.client_y() as f32);
            if (x - t.start.0).hypot(y - t.start.1) > 6.0 {
                dragged_move.set(true);
            }
            let mut c = control_move.borrow_mut();
            if t.moves {
                c.forward = (-(y - t.start.1) / 55.0).clamp(-1.0, 1.0);
                c.strafe = ((x - t.start.0) / 55.0).clamp(-1.0, 1.0);
            } else {
                c.turn += (x - t.last.0) * 0.0045;
                c.tilt += (y - t.last.1) * 0.0035;
            }
            t.last = (x, y);
        });
        for kind in ["pointerup", "pointercancel"] {
            let (control_up, touches_up) = (control.clone(), touches.clone());
            listen(canvas, kind, move |e: PointerEvent| {
                if let Some(t) = touches_up.borrow_mut().remove(&e.pointer_id())
                    && t.moves
                {
                    let mut c = control_up.borrow_mut();
                    (c.forward, c.strafe) = (0.0, 0.0);
                }
            });
        }
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
    on_click(&shell, "ctl-mode", {
        let toggle_mode = toggle_mode.clone();
        move |_| toggle_mode()
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

    // Le clavier, quand la galerie est en plein écran ou en visite libre.
    let active = {
        let (is_full, is_free) = (is_full.clone(), is_free.clone());
        move || is_full() || is_free()
    };
    let typing = |e: &KeyboardEvent| {
        e.target()
            .and_then(|t| t.dyn_into::<HtmlElement>().ok())
            .is_some_and(|t| matches!(t.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT"))
    };
    let keydown = {
        let (shell, control) = (shell.clone(), control.clone());
        let (enter, exit, close_hd, toggle_pause, toggle_mode, jump) = (
            enter,
            exit,
            close_hd,
            toggle_pause,
            toggle_mode.clone(),
            jump,
        );
        let (is_full, is_free, active) = (is_full, is_free.clone(), active.clone());
        Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
            if typing(&e) {
                return;
            }
            let free = is_free();
            if free && active() && held(&control, &e.code(), true, e.shift_key()) {
                e.prevent_default();
                return;
            }
            match e.key().as_str() {
                "f" | "F" if !is_full() => enter(),
                "v" | "V" if active() => toggle_mode(),
                "Escape" if shell.class_list().contains("hd-open") => close_hd(),
                "Escape" if is_full() => exit(),
                "ArrowRight" if is_full() && !free => jump(1),
                "ArrowLeft" if is_full() && !free => jump(-1),
                " " if is_full() && !free => {
                    e.prevent_default();
                    toggle_pause();
                }
                _ => {}
            }
        })
    };
    let keyup = {
        let control = control.clone();
        Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
            held(&control, &e.code(), false, e.shift_key());
        })
    };
    let _ = document.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref());
    let _ = document.add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref());
    Some(Commands {
        document,
        listeners: vec![("keydown", keydown), ("keyup", keyup)],
    })
}

/// Les touches de la visite libre, par leur position sur le clavier (`code`) : ZQSD sur
/// un clavier AZERTY, WASD sur un QWERTY ; les flèches tournent ; Maj fait courir.
/// Renvoie si la touche a servi.
fn held(control: &SharedControl, code: &str, down: bool, shift: bool) -> bool {
    let mut c = control.borrow_mut();
    let value = if down { 1.0 } else { 0.0 };
    c.run = shift;
    match code {
        "KeyW" | "ArrowUp" => c.forward = value,
        "KeyS" | "ArrowDown" => c.forward = -value,
        "KeyA" => c.strafe = -value,
        "KeyD" => c.strafe = value,
        "ArrowLeft" => c.spin = -value * 1.6,
        "ArrowRight" => c.spin = value * 1.6,
        "ShiftLeft" | "ShiftRight" => {}
        _ => return false,
    }
    true
}
