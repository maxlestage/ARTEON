//! Le serveur d'ARTEON : la page de présentation, la galerie en 3D et le catalogue.
//!
//!     paquets dev      # http://localhost:3000

use paquets::prelude::*;

use arteon::artworks::{ARTWORKS, Artwork, ROOMS, Wall, css_rgb};
use arteon::gallery::tour_duration;
use arteon::{gallery_scene, showcase_scene};

mod export;
mod style;

// ─── Mise en page commune ────────────────────────────────────────────────────

fn nav_link(href: &'static str, label: &'static str, current: &str) -> Element {
    let link = a().href(href).text(label);
    if href == current {
        link.class("current").attr("aria-current", "page")
    } else {
        link
    }
}

/// Le logo : un petit carré façon Mondrian, puis le nom.
fn logo() -> Element {
    a().href("/")
        .class("logo")
        .attr("aria-label", "ARTEON, accueil")
        .child(
            span()
                .class("logo-mark")
                .child(span().class("m-r"))
                .child(span().class("m-b"))
                .child(span().class("m-y")),
        )
        .child(strong().text("ARTEON"))
}

/// Les textes passés à `Page` doivent être `&'static str` ou `String` (pas un `&str`
/// emprunté) : toutes les pages passent ici des littéraux.
fn layout(
    title: &'static str,
    description: &'static str,
    current: &str,
    content: impl Into<Node>,
) -> Page {
    Page::new(title)
        .lang("fr")
        .description(description)
        .seo(Seo::new().site_name("ARTEON").title_template("%s · ARTEON"))
        .view_transitions()
        .prefetch(Prefetch::Hover)
        .style(style::CSS)
        .body(
            header().class("site-header").id("top").child(logo()).child(
                nav()
                    .child(nav_link("/", "Présentation", current))
                    .child(nav_link("/galerie", "Galerie 3D", current))
                    .child(nav_link("/oeuvres", "Œuvres", current)),
            ),
        )
        .body(html::main().class("page").child(content))
        .body(site_footer())
}

/// Les artistes exposés, sans doublon, dans l'ordre du catalogue.
fn artists() -> Vec<(&'static str, &'static str)> {
    let mut list: Vec<(&'static str, &'static str)> = Vec::new();
    for art in &ARTWORKS {
        if !list.iter().any(|(name, _)| *name == art.artist) {
            list.push((art.artist, art.artist_dates));
        }
    }
    list
}

/// Le grand pied de page : une bande Mondrian animée, un défilé des artistes, quatre
/// colonnes de liens et les mentions sur le domaine public.
fn site_footer() -> Element {
    // Le défilé est écrit deux fois de suite : l'animation glisse d'une moitié, en boucle.
    let marquee_items = || {
        ARTWORKS.iter().map(|art| {
            span()
                .class("marquee-item")
                .child(strong().text(art.artist))
                .child(span().text(format!(" · {} ({})", art.title, art.year)))
        })
    };
    footer()
        .class("site-footer")
        .child(
            div()
                .class("footer-band")
                .attr("aria-hidden", "true")
                .children(
                    ["#c82026", "#f2c828", "#1c3a8c", "#161616", "#f2efe6", "#c82026", "#1c3a8c", "#f2c828"]
                        .into_iter()
                        .enumerate()
                        .map(|(i, color)| {
                            span().style(format!("background:{color};animation-delay:{}ms", i * 140))
                        }),
                ),
        )
        .child(
            div()
                .class("marquee")
                .attr("aria-label", "Les œuvres exposées")
                .child(div().class("marquee-track").children(marquee_items()).children(marquee_items())),
        )
        .child(
            div()
                .class("footer-grid")
                .child(
                    div()
                        .class("footer-brand")
                        .child(logo())
                        .child(p().text(
                            "Une galerie d'art en 3D, écrite uniquement en Rust : du serveur à la \
                             scène WebGL, compilée en WebAssembly.",
                        ))
                        .child(a().class("button footer-cta").href("/galerie").text("Entrer dans la galerie →")),
                )
                .child(
                    div()
                        .class("footer-col")
                        .child(h3().text("Visiter"))
                        .child(
                            ul().children(ROOMS.iter().map(|room| li().child(a().href("/galerie").text(room.name)))),
                        ),
                )
                .child(
                    div()
                        .class("footer-col")
                        .child(h3().text("Artistes"))
                        .child(ul().children(artists().into_iter().map(|(name, dates)| {
                            li().child(a().href("/oeuvres").text(name)).child(span().class("muted").text(format!(" {dates}")))
                        }))),
                )
                .child(
                    div()
                        .class("footer-col")
                        .child(h3().text("Le projet"))
                        .child(
                            ul().child(li().child(a().href("https://github.com/maxlestage/ARTEON").text("Code source")))
                                .child(li().child(a().href("https://github.com/maxlestage/Paquets").text("Construit avec Paquets")))
                                .child(li().child(a().href("/oeuvres").text("Catalogue et crédits")))
                                .child(li().child(a().href("#top").text("Retour en haut ↑"))),
                        ),
                ),
        )
        .child(
            div()
                .class("footer-legal")
                .child(p().text(
                    "Toutes les œuvres présentées sont dans le domaine public : leurs auteurs sont \
                     morts depuis plus de 70 ans. Elles sont réinterprétées en volumes géométriques, \
                     ce ne sont pas des reproductions exactes.",
                ))
                .child(p().class("muted").text("© ARTEON · 100 % Rust · 0 ligne de JavaScript écrite à la main")),
        )
}

// ─── Composants ──────────────────────────────────────────────────────────────

/// Une vignette du tableau, dessinée avec les mêmes aplats que la scène 3D.
fn thumbnail(art: &Artwork) -> Element {
    let ratio = art.width / art.height;
    div()
        .class("thumb")
        .attr("role", "img")
        .attr("aria-label", format!("{}, {}", art.title, art.artist))
        .style(format!(
            "aspect-ratio:{ratio:.3};background:{}",
            css_rgb(art.background)
        ))
        .children(art.shapes.iter().map(|s| {
            span().style(format!(
                "left:{:.1}%;bottom:{:.1}%;width:{:.1}%;height:{:.1}%;background:{}",
                s.x * 100.0,
                s.y * 100.0,
                s.w * 100.0,
                s.h * 100.0,
                css_rgb(s.rgb)
            ))
        }))
}

fn wall_name(wall: Wall) -> &'static str {
    match wall {
        Wall::Left => "mur gauche",
        Wall::Right => "mur droit",
        Wall::Back => "mur du fond",
    }
}

/// Le cartel d'une œuvre.
fn artwork_card(art: &Artwork, index: usize) -> Element {
    article()
        .class("art-card reveal")
        .style(format!("animation-delay:{}ms", 80 * index))
        .child(div().class("frame").child(thumbnail(art)))
        .child(h3().text(art.title))
        .child(
            p().child(strong().text(art.artist)).child(
                span()
                    .class("muted")
                    .text(format!(" ({}) · {}", art.artist_dates, art.year)),
            ),
        )
        .child(p().class("muted").text(art.note))
        .child(p().class("tag").text(format!(
            "{} · {} · {}",
            ROOMS[art.hook.room].name,
            wall_name(art.hook.wall),
            art.collection
        )))
}

/// Un cube en CSS 3D, aux couleurs de De Stijl, qui tourne dans l'en-tête.
fn css_cube() -> Element {
    div().class("cube-stage").attr("aria-hidden", "true").child(
        div().class("cube").children(
            ["front", "back", "right", "left", "top", "bottom"]
                .into_iter()
                .map(|face| span().class(format!("face {face}"))),
        ),
    )
}

/// Le titre, lettre par lettre, chaque lettre arrivant un peu après la précédente.
fn animated_title(text: &'static str) -> Element {
    h1().class("hero-title")
        .attr("aria-label", text)
        .children(text.chars().enumerate().map(|(i, c)| {
            span()
                .attr("aria-hidden", "true")
                .style(format!("animation-delay:{}ms", 90 * i))
                .text(c.to_string())
        }))
}

// ─── Pages ───────────────────────────────────────────────────────────────────

async fn home(_req: Request) -> Page {
    let minutes = (tour_duration() / 60.0).round() as u32;
    let content = div()
        .child(
            section()
                .class("hero")
                .child(div().class("floating f1").attr("aria-hidden", "true"))
                .child(div().class("floating f2").attr("aria-hidden", "true"))
                .child(div().class("floating f3").attr("aria-hidden", "true"))
                .child(
                    div()
                        .class("hero-text")
                        .child(p().class("kicker reveal").text("Galerie d'art en 3D · 100 % Rust"))
                        .child(animated_title("ARTEON"))
                        .child(p().class("lead reveal").text(
                            "Parcourez trois salles d'exposition reliées par des couloirs, au milieu \
                             d'œuvres du domaine public de Mondrian, Malevitch, Klee, van Doesburg et \
                             Taeuber-Arp, réinventées en volumes.",
                        ))
                        .child(
                            div()
                                .class("row reveal")
                                .child(a().class("button").href("/galerie").text("Lancer la visite 3D"))
                                .child(a().class("button ghost").href("/oeuvres").text("Voir les œuvres")),
                        ),
                )
                .child(div().class("hero-visual").child(island("showcase", "", showcase_scene)).child(css_cube())),
        )
        .child(
            section()
                .class("stats")
                .child(div().class("stat reveal").child(strong().text(ROOMS.len().to_string())).child(span().text("salles d'exposition")))
                .child(div().class("stat reveal").child(strong().text((ROOMS.len() - 1).to_string())).child(span().text("couloirs")))
                .child(div().class("stat reveal").child(strong().text(ARTWORKS.len().to_string())).child(span().text("œuvres du domaine public")))
                .child(div().class("stat reveal").child(strong().text(format!("{minutes} min"))).child(span().text("de visite guidée"))),
        )
        .child(
            section()
                .class("features")
                .child(h2().class("section-title").text("Comment c'est fait"))
                .child(
                    div()
                        .class("grid")
                        .child(div().class("card feature reveal").child(span().class("dot red")).child(h3().text("Une scène 3D en Rust")).child(p().class("muted").text("Murs, sols, tableaux et lumières sont des entités du moteur de jeu de Paquets, rendues en WebGL 2.")))
                        .child(div().class("card feature reveal").child(span().class("dot yellow")).child(h3().text("Compilée en WebAssembly")).child(p().class("muted").text("Le même code Rust tourne sur le serveur pour le HTML et dans le navigateur pour la 3D.")))
                        .child(div().class("card feature reveal").child(span().class("dot blue")).child(h3().text("Des œuvres libres de droits")).child(p().class("muted").text("Uniquement des artistes morts depuis plus de 70 ans, réinterprétés en aplats et en reliefs.")))
                        .child(div().class("card feature reveal").child(span().class("dot black")).child(h3().text("Une visite guidée")).child(p().class("muted").text("La caméra marche de tableau en tableau, traverse les couloirs et s'arrête devant chaque œuvre."))),
                ),
        )
        .child(
            section()
                .class("rooms")
                .child(h2().class("section-title").text("Le parcours"))
                .child(div().class("timeline").children(ROOMS.iter().enumerate().map(|(i, room)| {
                    let works = ARTWORKS.iter().filter(|a| a.hook.room == i).count();
                    div()
                        .class("step reveal")
                        .style(format!("animation-delay:{}ms", 120 * i))
                        .child(span().class("step-number").text(format!("0{}", i + 1)))
                        .child(h3().text(room.name))
                        .child(p().class("muted").text(room.theme))
                        .child(p().class("tag").text(format!("{works} œuvre(s)")))
                }))),
        )
        .child(
            section()
                .class("preview")
                .child(h2().class("section-title").text("Quelques œuvres"))
                .child(div().class("preview-strip").children(ARTWORKS.iter().take(5).enumerate().map(|(i, art)| {
                    a().href("/oeuvres")
                        .class("tilt reveal")
                        .style(format!("animation-delay:{}ms", 90 * i))
                        .child(thumbnail(art))
                }))),
        );
    layout(
        "Une galerie d'art en 3D, en Rust",
        "ARTEON : trois salles d'exposition et des couloirs en 3D, des œuvres du domaine public, écrits uniquement en Rust.",
        "/",
        content,
    )
}

async fn gallery_page(_req: Request) -> Page {
    let content = div()
        .child(
            section()
                .class("page-intro")
                .child(p().class("kicker").text("Visite guidée"))
                .child(h1().text("La galerie en 3D"))
                .child(p().class("lead").text(
                    "La caméra traverse les trois salles et leurs couloirs, et s'arrête devant chaque \
                     œuvre. La visite recommence en boucle.",
                )),
        )
        .child(div().class("viewer").child(island("gallery", "", gallery_scene)))
        .child(div().class("grid room-legend").children(ROOMS.iter().enumerate().map(|(i, room)| {
            div()
                .class("card")
                .child(h3().text(room.name))
                .child(p().class("muted").text(room.theme))
                .child(ul().children(
                    ARTWORKS
                        .iter()
                        .filter(|a| a.hook.room == i)
                        .map(|a| li().child(strong().text(a.title)).child(span().class("muted").text(format!(" · {}", a.artist)))),
                ))
        })));
    layout(
        "La galerie en 3D",
        "Visite guidée en 3D des trois salles d'ARTEON, reliées par des couloirs.",
        "/galerie",
        content,
    )
}

async fn catalogue(_req: Request) -> Page {
    let content = div()
        .child(
            section()
                .class("page-intro")
                .child(p().class("kicker").text("Catalogue"))
                .child(h1().text("Les œuvres"))
                .child(p().class("lead").text(
                    "Neuf œuvres du domaine public. Chaque vignette est dessinée à partir des mêmes \
                     aplats que la scène 3D.",
                )),
        )
        .child(div().class("art-grid").children(ARTWORKS.iter().enumerate().map(|(i, art)| artwork_card(art, i))));
    layout(
        "Les œuvres",
        "Le catalogue d'ARTEON : des œuvres du domaine public de Mondrian, Malevitch, Klee, van Doesburg et Taeuber-Arp.",
        "/oeuvres",
        content,
    )
}

async fn not_found(_req: Request) -> Page {
    let content = section()
        .class("page-intro")
        .child(p().class("kicker").text("Erreur 404"))
        .child(h1().text("Cette salle n'existe pas"))
        .child(
            p().class("lead")
                .text("La page demandée est introuvable. Le reste du musée vous attend."),
        )
        .child(
            div()
                .class("row")
                .child(a().class("button").href("/").text("Retour à l'accueil"))
                .child(
                    a().class("button ghost")
                        .href("/galerie")
                        .text("Entrer dans la galerie"),
                ),
        );
    layout(
        "Page introuvable",
        "Cette page d'ARTEON n'existe pas.",
        "",
        content,
    )
    .status(404)
}

/// Les pages du site : la présentation, la galerie 3D et le catalogue.
pub(crate) const ROUTES: [&str; 3] = ["/", "/galerie", "/oeuvres"];

/// L'application : le même serveur pour `paquets dev` et pour l'export statique.
fn app() -> App {
    let mut app = App::new();
    app.get(ROUTES[0], home);
    app.get(ROUTES[1], gallery_page);
    app.get(ROUTES[2], catalogue);
    app.fallback(not_found);
    app
}

/// `cargo run` : le serveur, sur http://localhost:3000.
/// `cargo run -- export dist` : tout le site en fichiers statiques (GitHub Pages).
fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("export") => {
            let dir = args.next().unwrap_or_else(|| "dist".into());
            let base = std::env::var("BASE_PATH").unwrap_or_default();
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            let files =
                runtime.block_on(export::export(app(), std::path::Path::new(&dir), &base))?;
            println!("📦 {files} fichiers écrits dans {dir}");
            Ok(())
        }
        Some(other) => {
            eprintln!("commande inconnue {other:?} : lancer sans argument, ou `export <dossier>`");
            std::process::exit(2);
        }
        None => app().run(),
    }
}
