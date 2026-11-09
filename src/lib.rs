//! ARTEON, partagé par le serveur et le navigateur : le catalogue, les scènes 3D et
//! les îlots qui les font tourner dans un `<canvas>`.
//!
//! Le serveur rend les îlots en HTML (src/main.rs) avec, pour paramètre, l'adresse de
//! l'exposition (`van-gogh`, `monet`…) ; `paquets build` compile ce fichier en
//! WebAssembly pour que le navigateur lance les scènes et branche les commandes de la
//! galerie (plein écran, cartel, œuvre précédente ou suivante, pause, image en HD).

use paquets::prelude::*;

pub mod artworks;
pub mod gallery;

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

/// L'îlot de la galerie : les salles d'une exposition, la visite guidée, et autour de la
/// scène, les commandes. Toucher la scène la met en plein écran.
pub fn gallery_scene(props: &str) -> Node {
    let ex = exhibition(props);
    div()
        .class("viewer-shell")
        .child(
            canvas()
                .class("scene scene-gallery")
                .attr(
                    "aria-label",
                    format!(
                        "Visite guidée en 3D de l'exposition {} : toucher pour le plein écran",
                        ex.title
                    ),
                )
                .use_action(move |canvas| play_gallery(canvas, ex)),
        )
        .child(control("viewer-full", "Plein écran", "⛶ Plein écran"))
        .child(
            div()
                .class("viewer-caption")
                .attr("aria-live", "polite")
                .child(p().class("cap-title"))
                .child(p().class("cap-meta"))
                .child(control(
                    "cap-hd",
                    "Voir la toile en haute définition",
                    "Voir en HD",
                )),
        )
        .child(
            div()
                .class("viewer-controls")
                .child(control("ctl-prev", "Œuvre précédente", "‹"))
                .child(control("ctl-play", "Pause", "❚❚"))
                .child(control("ctl-next", "Œuvre suivante", "›"))
                .child(control("ctl-exit", "Quitter le plein écran", "✕")),
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
    let commands = viewer::attach(canvas, ex, control.clone());
    let running = gallery::tour_app_with(ex, control).run_in(canvas).ok();
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
    let running = gallery::showcase_app(ex).run_in(canvas).ok();
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
