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
/// La racine du site, sans la langue (`/ARTEON/en/monet/galerie` → `/ARTEON`) : là où
/// sont les fichiers communs (données, images).
pub(crate) fn files_path(slug: &str) -> String {
    let base = base_path(slug);
    for l in crate::langue::Lang::ALL {
        if !l.prefix().is_empty() && let Some(root) = base.strip_suffix(l.prefix()) {
            return root.to_owned();
        }
    }
    base
}

/// La racine des pages, avec la langue (`/ARTEON/en/monet/galerie` → `/ARTEON/en`).
pub(crate) fn base_path(slug: &str) -> String {
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

/// L'œuvre nommée par l'ancre de l'adresse (`…/galerie#nuit-etoilee`), depuis sa page.
pub(crate) fn focus_from_hash(ex: &'static Exhibition) -> Option<crate::tour::Focus> {
    let hash = web_sys::window()?.location().hash().ok()?;
    let slug = hash.trim_start_matches('#');
    if let Some(i) = ex.artworks.iter().position(|a| a.slug == slug) {
        return Some(crate::tour::Focus::Art(i));
    }
    let i = ex.sculptures.iter().position(|s| s.slug == slug)?;
    Some(crate::tour::Focus::Sculpture(i))
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
        let url = format!("{}/donnees/{}.bin", files_path(ex.slug), ex.slug);
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
    /// L'interface egui, arrêtée avec l'îlot.
    runner: eframe::WebRunner,
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
        self.runner.destroy();
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

/// Branche l'interface autour de `canvas` (dans `.viewer-shell`) : egui dans son canvas
/// transparent, posé au-dessus, et les gestes qui vont à la scène.
pub fn attach(
    canvas: &Element,
    ex: &'static Exhibition,
    control: SharedControl,
) -> Option<Commands> {
    let shell = canvas.closest(".viewer-shell").ok().flatten()?;
    let document = web_sys::window()?.document()?;
    let base = files_path(ex.slug);
    // L'API Fullscreen a-t-elle été utilisée (pour savoir si sa sortie doit tout fermer) ?
    let native = Rc::new(Cell::new(false));
    // Un glissé vient-il d'avoir lieu (le clic qui suit ne compte pas) ?
    let dragged = Rc::new(Cell::new(false));
    // egui a-t-il le pointeur (au-dessus d'un panneau) ? La scène ne réagit alors pas.
    let busy = Rc::new(Cell::new(false));
    // Les gestes arrivent sur la couche egui, au-dessus de la scène.
    let layer: Element = find(&shell, "egui-layer").unwrap_or_else(|| canvas.clone());

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
        Rc::new(move || shell.class_list().contains("is-fullscreen"))
    };
    let is_free = {
        let control = control.clone();
        move || control.borrow().mode == Mode::Free
    };
    let open_hd = {
        let (shell, control, base) = (shell.clone(), control.clone(), base.clone());
        Rc::new(move || {
            let Some(Focus::Art(i)) = control.borrow().current else {
                return;
            };
            let art = &ex.artworks[i];
            if let Some(image) = find(&shell, "hd-img") {
                let src = if art.is_extra() { art.image_url() } else { format!("{base}{}", art.image_url()) };
                let _ = image.set_attribute("src", &src);
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
    let toggle_pause = {
        let control = control.clone();
        move || {
            let mut c = control.borrow_mut();
            c.paused = !c.paused;
        }
    };
    let toggle_mode = {
        let control = control.clone();
        move || {
            let mut c = control.borrow_mut();
            c.mode = if c.mode == Mode::Free {
                Mode::Guided
            } else {
                Mode::Free
            };
            (c.forward, c.strafe, c.spin) = (0.0, 0.0, 0.0);
        }
    };
    let jump = {
        let control = control.clone();
        move |step: i32| control.borrow_mut().jump += step
    };

    // L'interface egui, dans son canvas.
    let runner = eframe::WebRunner::new();
    if let Some(layer_canvas) = layer
        .clone()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .ok()
        .filter(|c| c.class_list().contains("egui-layer"))
    {
        let actions = crate::ui::Actions {
            enter: enter.clone(),
            exit: exit.clone(),
            is_full: is_full.clone(),
            open_hd: open_hd.clone(),
            download: Rc::new(download),
            copy: Rc::new(|text: &str| {
                if let Some(w) = web_sys::window() {
                    let _ = w.navigator().clipboard().write_text(text);
                }
            }),
            open: Rc::new(|url: &str| {
                if let Some(w) = web_sys::window() {
                    let _ = w.open_with_url_and_target(url, "_blank");
                }
            }),
        };
        // Les textes longs (notices, biographies, bâtiment, fichier source), téléchargés
        // à part : ils ne sont pas dans le WebAssembly.
        let texts: crate::texts::SharedTexts = Default::default();
        let filling = texts.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let url = format!("{}/donnees/{}", files_path(ex.slug), crate::langue::lang().texts_file(ex.slug));
            if let Ok(bytes) = fetch_bytes(&url).await
                && let Some(loaded) = crate::texts::Texts::parse(&String::from_utf8_lossy(&bytes))
            {
                *filling.borrow_mut() = Some(loaded);
            }
        });
        let ui = crate::ui::MuseumUi::new(ex, control.clone(), actions, busy.clone(), texts);
        let options = eframe::WebOptions {
            // Les événements continuent vers la page : la scène reçoit les gestes et le
            // clavier quand egui n'en a pas l'usage.
            should_stop_propagation: Box::new(|_| false),
            should_prevent_default: Box::new(|_| false),
            ..Default::default()
        };
        let starting = runner.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = starting
                .start(
                    layer_canvas,
                    options,
                    Box::new(move |cc| {
                        // Le thème sombre, lisible sur la scène.
                        cc.egui_ctx.set_theme(eframe::egui::Theme::Dark);
                        cc.egui_ctx.set_fonts(fonts());
                        Ok(Box::new(ui))
                    }),
                )
                .await;
        });
    }

    // Toucher la scène (hors des panneaux) : plein écran ; en plein écran (visite guidée),
    // pause ou reprise.
    {
        let (enter, is_full, is_free, dragged, busy, control) = (
            enter.clone(),
            is_full.clone(),
            is_free.clone(),
            dragged.clone(),
            busy.clone(),
            control.clone(),
        );
        listen(&layer, "click", move |_: Event| {
            if dragged.replace(false) || busy.get() {
                return;
            }
            if !is_full() {
                enter();
            } else if !is_free() {
                let mut c = control.borrow_mut();
                c.paused = !c.paused;
            }
        });
    }

    // La visite libre à la souris et au doigt : glisser pour regarder ; sur un écran
    // tactile, la moitié gauche est un joystick pour avancer.
    {
        let touches: Rc<RefCell<HashMap<i32, Touch>>> = Rc::default();
        let (touches_down, is_free_down, layer_down, busy_down) = (
            touches.clone(),
            is_free.clone(),
            layer.clone(),
            busy.clone(),
        );
        listen(&layer, "pointerdown", move |e: PointerEvent| {
            if !is_free_down() || busy_down.get() {
                return;
            }
            let (x, y) = (e.client_x() as f32, e.client_y() as f32);
            let rect = layer_down.get_bounding_client_rect();
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
        });
        let (control_move, touches_move, dragged_move) =
            (control.clone(), touches.clone(), dragged.clone());
        listen(&layer, "pointermove", move |e: PointerEvent| {
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
        for kind in ["pointerup", "pointercancel", "pointerleave"] {
            let (control_up, touches_up) = (control.clone(), touches.clone());
            listen(&layer, kind, move |e: PointerEvent| {
                if let Some(t) = touches_up.borrow_mut().remove(&e.pointer_id())
                    && t.moves
                {
                    let mut c = control_up.borrow_mut();
                    (c.forward, c.strafe) = (0.0, 0.0);
                }
            });
        }
    }

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
        let (is_full, is_free, active) = (is_full.clone(), is_free.clone(), active.clone());
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
        runner,
    })
}

/// Fait télécharger `text` sous le nom `name`.
fn download(name: &str, text: &str) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let parts = js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(text));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("text/plain;charset=utf-8");
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options) else {
        return;
    };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return;
    };
    if let Ok(link) = document.create_element("a") {
        let _ = link.set_attribute("href", &url);
        let _ = link.set_attribute("download", name);
        if let Some(link) = link.dyn_ref::<HtmlElement>() {
            link.click();
        }
    }
    let _ = web_sys::Url::revoke_object_url(&url);
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

// ─── La visite dynamique ─────────────────────────────────────────────────────

/// La visite en cours (ou en chargement) : la scène et ses commandes, arrêtées quand
/// l'îlot s'en va.
pub struct Visit {
    scene: Rc<RefCell<Option<Box<dyn std::any::Any>>>>,
    commands: Rc<RefCell<Option<Commands>>>,
    cancelled: Rc<Cell<bool>>,
}

impl Drop for Visit {
    fn drop(&mut self) {
        self.cancelled.set(true);
        self.scene.borrow_mut().take();
        self.commands.borrow_mut().take();
    }
}

/// Le nombre d'œuvres des collections à ajouter à la visite : `?oeuvres=48` dans
/// l'adresse (0 : la visite du fichier, telle quelle), 24 sinon ; 120 au plus.
fn wanted_extras() -> usize {
    let search = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .unwrap_or_default();
    search
        .trim_start_matches('?')
        .split('&')
        .find_map(|pair| pair.strip_prefix("oeuvres="))
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(24)
        .min(120)
}

/// Une image chargée par le navigateur (ou rien si elle ne vient pas en dix secondes) :
/// une promesse qui ne échoue jamais.
fn load_image(src: &str) -> js_sys::Promise {
    let src = src.to_owned();
    js_sys::Promise::new(&mut |resolve, _reject| {
        let Ok(image) = web_sys::HtmlImageElement::new() else {
            let _ = resolve.call1(&wasm_bindgen::JsValue::NULL, &wasm_bindgen::JsValue::NULL);
            return;
        };
        // Les pixels doivent pouvoir être lus : l'image est demandée avec CORS.
        image.set_cross_origin(Some("anonymous"));
        let loaded = {
            let (resolve, image) = (resolve.clone(), image.clone());
            Closure::once_into_js(move || {
                let _ = resolve.call1(&wasm_bindgen::JsValue::NULL, &image);
            })
        };
        let failed = || {
            let resolve = resolve.clone();
            Closure::once_into_js(move || {
                let _ = resolve.call1(&wasm_bindgen::JsValue::NULL, &wasm_bindgen::JsValue::NULL);
            })
        };
        image.set_onload(Some(loaded.unchecked_ref()));
        image.set_onerror(Some(failed().unchecked_ref()));
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(failed().unchecked_ref(), 10_000);
        }
        image.set_src(&src);
    })
}

/// Les œuvres liées de l'exposition à accrocher dans l'aile : `wanted` au plus, réparties
/// sur toute la liste (de la plus ancienne à la plus récente), leurs vignettes chargées et
/// transformées en mosaïques. Celles dont l'image ne se laisse pas lire sont sautées.
async fn load_extras(ex: &'static Exhibition, wanted: usize) -> Vec<crate::aile::Extra> {
    // Une exposition éphémère n'a pas d'œuvres liées (pas de fichier à demander).
    if ex.ephemeral || wanted == 0 {
        return Vec::new();
    }
    let files = files_path(ex.slug);
    let url = format!("{files}/monde/donnees/expo-{}.json", ex.slug);
    let Ok(bytes) = fetch_bytes(&url).await else {
        return Vec::new();
    };
    let Ok(root) = js_sys::JSON::parse(&String::from_utf8_lossy(&bytes)) else {
        return Vec::new();
    };
    if !js_sys::Array::is_array(&root) {
        return Vec::new();
    }
    // (rang dans la liste, identifiant, titre, artiste, date, adresse de la vignette)
    let all: Vec<(usize, String, String, String, String, Option<String>)> = js_sys::Array::from(&root)
        .iter()
        .enumerate()
        .filter_map(|(i, row)| {
            let row = js_sys::Array::from(&row);
            let text = |k: u32| row.get(k).as_string().unwrap_or_default();
            let id = text(0);
            (!id.is_empty()).then(|| (i, id, text(1), text(2), text(3), row.get(4).as_string()))
        })
        .collect();
    // D'abord les vignettes hébergées par le site (toujours lisibles), puis celles des musées.
    let spread = |list: Vec<&(usize, String, String, String, String, Option<String>)>, n: usize| {
        let len = list.len();
        (0..n.min(len)).map(|i| list[i * len / n.min(len)].clone()).collect::<Vec<_>>()
    };
    let hosted: Vec<_> = all.iter().filter(|w| w.5.is_none()).collect();
    let remote: Vec<_> = all.iter().filter(|w| w.5.is_some()).collect();
    let mut chosen = spread(hosted, wanted);
    if chosen.len() < wanted {
        // Le double : certains musées ne laissent pas lire leurs images (pas de CORS), elles
        // sont sautées ; le surplus est retiré à la fin.
        let missing = wanted - chosen.len();
        chosen.extend(spread(remote, missing * 2));
    }
    chosen.sort_by_key(|w| w.0);
    let sources: Vec<String> = chosen
        .iter()
        .map(|w| w.5.clone().unwrap_or_else(|| format!("{files}/monde/v/{}.webp", w.1)))
        .collect();
    let promises = js_sys::Array::new();
    for src in &sources {
        promises.push(&load_image(src));
    }
    let Ok(images) = JsFuture::from(js_sys::Promise::all(&promises)).await else {
        return Vec::new();
    };
    let images = js_sys::Array::from(&images);
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return Vec::new();
    };
    let Some(canvas) = document
        .create_element("canvas")
        .ok()
        .and_then(|c| c.dyn_into::<web_sys::HtmlCanvasElement>().ok())
    else {
        return Vec::new();
    };
    let mut extras = Vec::new();
    let mut from_museum = Vec::new();
    for (k, (_, id, title, artist, year, remote)) in chosen.into_iter().enumerate() {
        let Ok(image) = images.get(k as u32).dyn_into::<web_sys::HtmlImageElement>() else {
            continue;
        };
        let (w, h) = (image.natural_width(), image.natural_height());
        if w < 8 || h < 8 {
            continue;
        }
        let ratio = w as f32 / h as f32;
        let (cols, rows) = crate::aile::grid(ratio);
        canvas.set_width(u32::from(cols));
        canvas.set_height(u32::from(rows));
        let Some(context) = canvas
            .get_context("2d")
            .ok()
            .flatten()
            .and_then(|c| c.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
        else {
            continue;
        };
        if context
            .draw_image_with_html_image_element_and_dw_and_dh(&image, 0.0, 0.0, f64::from(cols), f64::from(rows))
            .is_err()
        {
            continue;
        }
        // Une image venue d'un musée sans CORS ne se laisse pas lire : elle est sautée.
        let Ok(pixels) = context.get_image_data(0.0, 0.0, f64::from(cols), f64::from(rows)) else {
            continue;
        };
        let (palette, cells) = crate::aile::mosaic(&pixels.data());
        from_museum.push(remote.is_some());
        extras.push(crate::aile::Extra {
            id,
            title: if title.is_empty() { crate::langue::t("Sans titre", "Untitled", "Sin título").to_owned() } else { title },
            artist: if artist == "Artiste inconnu" {
                crate::langue::t("Artiste inconnu", "Unknown artist", "Artista desconocido").to_owned()
            } else {
                artist
            },
            year,
            image: sources[k].clone(),
            ratio,
            cols,
            rows,
            palette,
            cells,
        });
    }
    // Trop d'œuvres des musées ont répondu : le surplus est retiré, réparti sur la liste.
    let surplus = extras.len().saturating_sub(wanted);
    if surplus > 0 {
        let places: Vec<usize> = from_museum.iter().enumerate().filter(|(_, m)| **m).map(|(i, _)| i).collect();
        let n = surplus.min(places.len());
        let dropped: std::collections::HashSet<usize> = (0..n).map(|i| places[i * places.len() / n]).collect();
        let mut i = 0;
        extras.retain(|_| {
            i += 1;
            !dropped.contains(&(i - 1))
        });
    }
    extras
}

/// Lance la visite du musée dans `canvas` : télécharge les données de l'exposition, ajoute
/// l'aile des collections avec le nombre d'œuvres demandé (voir [`wanted_extras`]), puis
/// branche les commandes et démarre la scène.
pub fn start_visit(canvas: &Element, ex: &'static Exhibition, control: SharedControl) -> Visit {
    let scene: Rc<RefCell<Option<Box<dyn std::any::Any>>>> = Rc::default();
    let commands: Rc<RefCell<Option<Commands>>> = Rc::default();
    let cancelled = Rc::new(Cell::new(false));
    let (canvas, slot, hands, stop) = (canvas.clone(), scene.clone(), commands.clone(), cancelled.clone());
    wasm_bindgen_futures::spawn_local(async move {
        let shell = canvas.closest(".viewer-shell").ok().flatten();
        let say = |message: &str| {
            if let Some(text) = shell.as_ref().and_then(|s| find(s, "viewer-loading")) {
                text.set_text_content(Some(message));
            }
        };
        let url = format!("{}/donnees/{}.bin", files_path(ex.slug), ex.slug);
        let loaded = fetch_bytes(&url).await.and_then(|bytes| ExhibitionData::new(ex, bytes));
        if stop.get() {
            return;
        }
        let data = match loaded {
            Ok(data) => data,
            Err(message) => {
                say(&crate::tf!(
                    "Le musée n'a pas pu s'ouvrir : {}",
                    "The museum could not open: {}",
                    "El museo no pudo abrirse: {}",
                    message
                ));
                return;
            }
        };
        // Le choix affiché sous la visite : la taille en cours est marquée.
        let wanted = wanted_extras();
        if let Some(document) = web_sys::window().and_then(|w| w.document())
            && let Ok(Some(chip)) = document.query_selector(&format!(".visit-size a[data-oeuvres=\"{wanted}\"]"))
        {
            let _ = chip.class_list().add_1("current");
        }
        let (ex, data) = if wanted == 0 {
            (ex, data)
        } else {
            say(crate::langue::t(
                "Accrochage des œuvres des collections…",
                "Hanging the works from the collections…",
                "Colgando las obras de las colecciones…",
            ));
            let extras = load_extras(ex, wanted).await;
            if stop.get() {
                return;
            }
            match crate::aile::extend(ex, &data, extras) {
                Some((bigger, more)) => (bigger, more),
                None => (ex, data),
            }
        };
        // Arrivé avec l'adresse d'une œuvre de l'aile (`#chicago-27992`) : la visite s'y arrête.
        if control.borrow().go_to.is_none()
            && let Some(focus) = focus_from_hash(ex)
        {
            let mut c = control.borrow_mut();
            c.go_to = Some(focus);
            c.paused = true;
        }
        *hands.borrow_mut() = attach(&canvas, ex, control.clone());
        let running = crate::gallery::tour_app_with(ex, Rc::new(data), control).run_in(&canvas).ok();
        *slot.borrow_mut() = Some(Box::new(running));
        if let Some(shell) = shell {
            let _ = shell.class_list().remove_1("is-loading");
        }
    });
    Visit { scene, commands, cancelled }
}

/// Les polices de l'interface : Ubuntu Light (la police d'egui, licence UFL, assets/fonts/)
/// pour tout le texte. Seule elle est embarquée : les emoji des boutons viennent des polices
/// du navigateur, et la police à chasse fixe d'egui ne sert jamais.
fn fonts() -> eframe::egui::FontDefinitions {
    use eframe::egui::{FontData, FontDefinitions, FontFamily};
    let mut fonts = FontDefinitions::empty();
    fonts.font_data.insert(
        "Ubuntu-Light".to_owned(),
        std::sync::Arc::new(FontData::from_static(include_bytes!("../assets/fonts/Ubuntu-Light.ttf"))),
    );
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts.families.insert(family, vec!["Ubuntu-Light".to_owned()]);
    }
    fonts
}
