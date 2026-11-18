//! ARTEON, partagé par le serveur et le navigateur : le catalogue, le plan des musées,
//! les scènes 3D et les îlots qui les font tourner dans un `<canvas>`.
//!
//! Le serveur rend les îlots en HTML (src/main.rs) avec, pour paramètre, l'adresse de
//! l'exposition (`van-gogh`, `monet`…) ; `paquets build` compile ce fichier en
//! WebAssembly : le navigateur télécharge les données de l'exposition, lance la scène et
//! branche son interface (src/viewer.rs).

use paquets::prelude::*;

pub mod aile;
pub mod artworks;
pub mod data;
pub mod editor;
pub mod gallery;
pub mod langue;
pub mod museum;
pub mod surfaces;
pub mod tour;

#[cfg(target_arch = "wasm32")]
mod texts;
#[cfg(target_arch = "wasm32")]
mod ui;
#[cfg(target_arch = "wasm32")]
mod viewer;

use artworks::Exhibition;
use langue::t;

/// L'exposition nommée par les paramètres de l'îlot (la première si elle est inconnue).
fn exhibition(props: &str) -> &'static Exhibition {
    // `monet` ou `monet@en` : l'exposition, et la langue de la page.
    let (slug, code) = props.trim().split_once('@').unwrap_or((props.trim(), "fr"));
    let lang = langue::Lang::from_code(code);
    langue::set_lang(lang);
    lang.exhibitions().iter().find(|e| e.slug == slug).unwrap_or(&lang.exhibitions()[0])
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
    let mb = ex.data_size as f32 / 1_000_000.0;
    match langue::lang() {
        langue::Lang::En => format!("{mb:.1} MB"),
        _ => format!("{mb:.1} Mo").replace('.', ","),
    }
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
                    crate::tf!(
                        "Le musée de l'exposition {} en 3D : toucher pour le plein écran",
                        "The museum of the exhibition {} in 3D: touch for full screen",
                        "El museo de la exposición {} en 3D: toque para pantalla completa",
                        ex.title
                    ),
                )
                .use_action(move |canvas| play_gallery(canvas, ex)),
        )
        // L'interface (egui), dessinée au-dessus de la scène.
        .child(
            canvas()
                .class("egui-layer")
                .attr("aria-label", t("Commandes de la visite", "Tour controls", "Controles de la visita")),
        )
        .child(
            p().class("viewer-loading")
                .text(crate::tf!("Ouverture du musée… ({})", "Opening the museum… ({})", "Abriendo el museo… ({})", megabytes(ex))),
        )
        .child(
            div()
                .class("viewer-sticks")
                .attr("aria-hidden", "true")
                .child(span().class("stick stick-move").text(t("avancer", "move", "avanzar")))
                .child(span().class("stick stick-look").text(t("regarder", "look", "mirar"))),
        )
        .child(
            div()
                .class("viewer-hd")
                .attr("role", "dialog")
                .attr("aria-label", t("La toile en haute définition", "The painting in high definition", "El cuadro en alta definición"))
                .child(img().class("hd-img").attr("alt", ""))
                .child(p().class("hd-title"))
                .child(control("hd-close", t("Fermer", "Close", "Cerrar"), t("✕ Fermer", "✕ Close", "✕ Cerrar"))),
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
            crate::tf!(
                "Une toile de l'exposition {} sur un chevalet tournant, en 3D",
                "A painting from the exhibition {} on a turning easel, in 3D",
                "Un cuadro de la exposición {} en un caballete giratorio, en 3D",
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
    // La visite est dynamique : le musée du fichier, agrandi d'une aile où sont accrochées
    // les œuvres liées des collections du monde (src/aile.rs), autant que le visiteur en veut.
    let visit = viewer::start_visit(canvas, ex, control);
    move || drop(visit) // arrête la visite quand l'îlot s'en va
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
