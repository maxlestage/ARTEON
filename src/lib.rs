//! ARTEON, partagé par le serveur et le navigateur : le catalogue, les scènes 3D et
//! les îlots qui les font tourner dans un `<canvas>`.
//!
//! Le serveur rend les îlots en HTML (src/main.rs) ; `paquets build` compile ce
//! fichier en WebAssembly pour que le navigateur lance les scènes.

use paquets::prelude::*;

pub mod artworks;
pub mod gallery;

/// L'îlot de la galerie : trois salles, deux couloirs et une visite guidée en 3D.
pub fn gallery_scene(_props: &str) -> Node {
    canvas()
        .class("scene scene-gallery")
        .attr("aria-label", "Visite guidée en 3D de la galerie ARTEON")
        .use_action(play_gallery)
        .into()
}

/// L'îlot de la page de présentation : une sculpture aux couleurs de Mondrian.
pub fn showcase_scene(_props: &str) -> Node {
    canvas()
        .class("scene scene-showcase")
        .attr("aria-label", "Sculpture 3D animée aux couleurs de De Stijl")
        .use_action(play_showcase)
        .into()
}

#[cfg(target_arch = "wasm32")]
fn play_gallery(canvas: &paquets::active::Dom) -> impl FnOnce() + use<> {
    let running = gallery::tour_app().run_in(canvas).ok();
    move || drop(running) // arrête la visite quand l'îlot s'en va
}

#[cfg(not(target_arch = "wasm32"))]
fn play_gallery(_canvas: &paquets::active::Dom) -> impl FnOnce() + use<> {
    || {}
}

#[cfg(target_arch = "wasm32")]
fn play_showcase(canvas: &paquets::active::Dom) -> impl FnOnce() + use<> {
    let running = gallery::showcase_app().run_in(canvas).ok();
    move || drop(running)
}

#[cfg(not(target_arch = "wasm32"))]
fn play_showcase(_canvas: &paquets::active::Dom) -> impl FnOnce() + use<> {
    || {}
}

/// Le point d'entrée du navigateur : rend interactifs tous les îlots de la page.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&[("gallery", gallery_scene), ("showcase", showcase_scene)]);
}
