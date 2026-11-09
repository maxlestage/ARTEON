//! ARTEON, partagé par le serveur et le navigateur : le catalogue, les scènes 3D et
//! les îlots qui les font tourner dans un `<canvas>`.
//!
//! Le serveur rend les îlots en HTML (src/main.rs) avec, pour paramètre, l'adresse de
//! l'exposition (`van-gogh`, `monet`…) ; `paquets build` compile ce fichier en
//! WebAssembly pour que le navigateur lance les scènes.

use paquets::prelude::*;

pub mod artworks;
pub mod gallery;

use artworks::{EXHIBITIONS, Exhibition};

/// L'exposition nommée par les paramètres de l'îlot (la première si elle est inconnue).
fn exhibition(props: &str) -> &'static Exhibition {
    Exhibition::find(props.trim()).unwrap_or(&EXHIBITIONS[0])
}

/// L'îlot de la galerie : les salles d'une exposition, leurs couloirs et la visite guidée.
pub fn gallery_scene(props: &str) -> Node {
    let ex = exhibition(props);
    canvas()
        .class("scene scene-gallery")
        .attr(
            "aria-label",
            format!("Visite guidée en 3D de l'exposition {}", ex.title),
        )
        .use_action(move |canvas| play_gallery(canvas, ex))
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
    let running = gallery::tour_app(ex).run_in(canvas).ok();
    move || drop(running) // arrête la visite quand l'îlot s'en va
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
