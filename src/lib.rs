//! ARTEON, partagé par le serveur et le navigateur : le catalogue, le plan des musées,
//! les scènes 3D et les îlots qui les font tourner dans un `<canvas>`.
//!
//! Le serveur rend les îlots en HTML (src/main.rs) avec, pour paramètre, l'adresse de
//! l'exposition (`van-gogh`, `monet`…) ; `paquets build` compile ce fichier en
//! WebAssembly : le navigateur télécharge les données de l'exposition, lance la scène et
//! branche son interface (src/viewer.rs).

use paquets::prelude::*;

pub mod artworks;
pub mod data;
pub mod editor;
pub mod gallery;
pub mod museum;
pub mod tour;

#[cfg(target_arch = "wasm32")]
mod texts;
#[cfg(target_arch = "wasm32")]
mod ui;
#[cfg(target_arch = "wasm32")]
mod viewer;

use artworks::{EXHIBITIONS, Exhibition};

/// L'exposition nommée par les paramètres de l'îlot (la première si elle est inconnue).
fn exhibition(props: &str) -> &'static Exhibition {
    Exhibition::find(props.trim()).unwrap_or(&EXHIBITIONS[0])
}

fn control(class: &'static str, label: &'static str, text: &'static str) -> Element {
    button()
        .attr("type", "button")
        .class(class)
        .attr("aria-label", label)
        .attr("title", label)
        .text(text)
}

/// Le poids des données d'une exposition, en clair.
fn megabytes(ex: &Exhibition) -> String {
    format!("{:.1} Mo", ex.data_size as f32 / 1_000_000.0).replace('.', ",")
}

/// L'îlot du musée : la scène, et autour d'elle l'interface de visite. Toucher la scène
/// la met en plein écran.
pub fn gallery_scene(props: &str) -> Node {
    let ex = exhibition(props);
    div()
        .class("viewer-shell is-loading")
        .child(
            canvas()
                .class("scene scene-gallery")
                .attr(
                    "aria-label",
                    format!(
                        "Le musée de l'exposition {} en 3D : toucher pour le plein écran",
                        ex.title
                    ),
                )
                .use_action(move |canvas| play_gallery(canvas, ex)),
        )
        // L'interface (egui), dessinée au-dessus de la scène.
        .child(
            canvas()
                .class("egui-layer")
                .attr("aria-label", "Commandes de la visite"),
        )
        .child(
            p().class("viewer-loading")
                .text(format!("Ouverture du musée… ({})", megabytes(ex))),
        )
        .child(
            div()
                .class("viewer-sticks")
                .attr("aria-hidden", "true")
                .child(span().class("stick stick-move").text("avancer"))
                .child(span().class("stick stick-look").text("regarder")),
        )
        .child(
            div()
                .class("viewer-hd")
                .attr("role", "dialog")
                .attr("aria-label", "La toile en haute définition")
                .child(img().class("hd-img").attr("alt", ""))
                .child(p().class("hd-title"))
                .child(control("hd-close", "Fermer", "✕ Fermer")),
        )
        .into()
}

/// L'îlot de la présentation d'une exposition : l'œuvre de la vitrine sur un chevalet.
pub fn showcase_scene(props: &str) -> Node {
    let ex = exhibition(props);
    canvas()
        .class("scene scene-showcase")
        .attr(
            "aria-label",
            format!(
                "Une toile de l'exposition {} sur un chevalet tournant, en 3D",
                ex.title
            ),
        )
        .use_action(move |canvas| play_showcase(canvas, ex))
        .into()
}

#[cfg(target_arch = "wasm32")]
fn play_gallery(canvas: &paquets::active::Dom, ex: &'static Exhibition) -> impl FnOnce() + use<> {
    let control = gallery::SharedControl::default();
    // Arrivé depuis la page d'une œuvre : la visite s'arrête devant elle.
    if let Some(focus) = viewer::focus_from_hash(ex) {
        let mut c = control.borrow_mut();
        c.go_to = Some(focus);
        c.paused = true;
    }
    let commands = viewer::attach(canvas, ex, control.clone());
    let running = viewer::start_when_loaded(canvas, ex, move |data| {
        gallery::tour_app_with(ex, data, control)
    });
    move || {
        drop(running); // arrête la visite quand l'îlot s'en va
        drop(commands);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn play_gallery(_canvas: &paquets::active::Dom, _ex: &'static Exhibition) -> impl FnOnce() + use<> {
    || {}
}

#[cfg(target_arch = "wasm32")]
fn play_showcase(canvas: &paquets::active::Dom, ex: &'static Exhibition) -> impl FnOnce() + use<> {
    let running =
        viewer::start_when_loaded(canvas, ex, move |data| gallery::showcase_app(ex, data));
    move || drop(running)
}

#[cfg(not(target_arch = "wasm32"))]
fn play_showcase(
    _canvas: &paquets::active::Dom,
    _ex: &'static Exhibition,
) -> impl FnOnce() + use<> {
    || {}
}

/// Le point d'entrée du navigateur : rend interactifs tous les îlots de la page.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&[("gallery", gallery_scene), ("showcase", showcase_scene)]);
}
