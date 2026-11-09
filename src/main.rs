//! Le serveur d'ARTEON : l'accueil du musée, et pour chaque exposition sa
//! présentation, sa galerie en 3D et son catalogue ; plus le guide pour ajouter une
//! œuvre. Tout le contenu vient de `expositions/*.toml` (voir `src/artworks.rs`).
//!
//!     paquets dev                         # http://localhost:3000
//!     cargo run --release -- export dist  # le site en fichiers statiques

use paquets::prelude::*;

use arteon::artworks::{Artwork, EXHIBITIONS, Exhibition, Wall};
use arteon::gallery::tour_duration;
use arteon::{gallery_scene, showcase_scene};

mod export;
mod style;

/// Le dépôt du projet, pour les liens « modifier sur GitHub ».
const REPOSITORY: &str = "https://github.com/maxlestage/ARTEON";

// ─── Mise en page commune ────────────────────────────────────────────────────

/// Un lien du menu ; `current` est le début de l'adresse de la page affichée.
fn nav_link(href: String, label: &'static str, current: &str) -> Element {
    let active = if href == "/" {
        current == "/"
    } else {
        current == href || current.starts_with(&format!("{href}/"))
    };
    let link = a().href(href).text(label);
    if active {
        link.class("current").attr("aria-current", "page")
    } else {
        link
    }
}

/// Le menu déroulant des expositions (`<details>` : il s'ouvre sans JavaScript).
fn exhibitions_menu(current: &str) -> Element {
    let inside = EXHIBITIONS.iter().any(|ex| {
        let href = format!("/{}", ex.slug);
        current == href || current.starts_with(&format!("{href}/"))
    });
    let summary = summary().text("Expositions");
    details()
        .class("nav-menu")
        .child(if inside {
            summary.class("current")
        } else {
            summary
        })
        .child(ul().children(EXHIBITIONS.iter().map(|ex| {
            li().child(nav_link(format!("/{}", ex.slug), ex.short_name, current))
                .child(span().class("nav-artist").text(ex.artist_dates))
        })))
}

/// Le logo : un petit soleil sur un ciel bleu, puis le nom.
fn logo() -> Element {
    a().href("/")
        .class("logo")
        .attr("aria-label", "ARTEON, accueil")
        .child(
            span()
                .class("logo-mark")
                .child(span().class("m-sun"))
                .child(span().class("m-star")),
        )
        .child(strong().text("ARTEON"))
}

fn rgb([r, g, b]: [u8; 3]) -> String {
    format!("rgb({r},{g},{b})")
}

/// La page complète. `current` est l'adresse de la page (pour le menu) ; `accent`, la
/// couleur de l'exposition affichée.
fn layout(
    title: impl Into<String>,
    description: impl Into<String>,
    current: &str,
    accent: Option<[u8; 3]>,
    content: impl Into<Node>,
) -> Page {
    let page = html::main().class("page").child(content);
    let page = match accent {
        Some(color) => page.style(format!("--accent:{}", rgb(color))),
        None => page,
    };
    Page::new(title)
        .lang("fr")
        .description(description.into())
        .seo(Seo::new().site_name("ARTEON").title_template("%s · ARTEON"))
        .view_transitions()
        .prefetch(Prefetch::Hover)
        .style(style::CSS)
        .body(
            header().class("site-header").id("top").child(logo()).child(
                nav()
                    .child(nav_link("/".into(), "Accueil", current))
                    .child(exhibitions_menu(current))
                    .child(
                        nav_link("/ajouter".into(), "Ajouter", current)
                            .child(span().class("nav-long").text(" une œuvre")),
                    ),
            ),
        )
        .body(page)
        .body(site_footer())
}

/// Les artistes exposés, avec leurs dates : « Vincent van Gogh (1853–1890) et … ».
fn artists_sentence() -> String {
    let names: Vec<String> = EXHIBITIONS
        .iter()
        .map(|e| format!("{} ({})", e.artist, e.artist_dates))
        .collect();
    match names.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} et {last}", rest.join(", ")),
        Some((last, _)) => last.clone(),
        None => String::new(),
    }
}

/// Le grand pied de page : une bande de couleurs animée, un défilé des œuvres, trois
/// colonnes de liens et les mentions sur le domaine public.
fn site_footer() -> Element {
    // Le défilé est écrit deux fois de suite : l'animation glisse d'une moitié, en boucle.
    let marquee_items = || {
        EXHIBITIONS.iter().flat_map(|ex| {
            ex.artworks.iter().map(move |art| {
                span()
                    .class("marquee-item")
                    .child(strong().text(art.title))
                    .child(span().text(format!(
                        " · {}, {}",
                        art.artist.unwrap_or(ex.short_name),
                        art.year
                    )))
            })
        })
    };
    footer()
        .class("site-footer")
        .child(div().class("footer-band").attr("aria-hidden", "true").children(
            ["#f2b705", "#1d3f8f", "#d4572a", "#2f6b4f", "#c9b6e4", "#0f1a3a", "#e8c547", "#7fb3a6"]
                .into_iter()
                .enumerate()
                .map(|(i, color)| span().style(format!("background:{color};animation-delay:{}ms", i * 140))),
        ))
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
                            "Un musée virtuel en 3D, écrit uniquement en Rust : du serveur à la scène WebGL, \
                             compilée en WebAssembly.",
                        ))
                        .child(a().class("button footer-cta").href("/#expositions").text("Voir les expositions →")),
                )
                .child(
                    div().class("footer-col").child(h3().text("Expositions")).child(ul().children(EXHIBITIONS.iter().map(
                        |ex| li().child(a().href(format!("/{}", ex.slug)).text(ex.short_name)),
                    ))),
                )
                .child(
                    div().class("footer-col").child(h3().text("Visiter")).child(ul().children(EXHIBITIONS.iter().map(
                        |ex| li().child(a().href(format!("/{}/galerie", ex.slug)).text(format!("Galerie 3D · {}", ex.short_name))),
                    ))),
                )
                .child(
                    div()
                        .class("footer-col")
                        .child(h3().text("Le projet"))
                        .child(
                            ul().child(li().child(a().href("/ajouter").text("Ajouter une œuvre")))
                                .child(li().child(a().href(REPOSITORY).text("Code source")))
                                .child(li().child(a().href("https://github.com/maxlestage/Paquets").text("Construit avec Paquets")))
                                .child(li().child(a().href("#top").text("Retour en haut ↑"))),
                        ),
                ),
        )
        .child(
            div()
                .class("footer-legal")
                .child(p().text(format!(
                    "{} : toutes les œuvres présentées sont dans le domaine public. Les reproductions viennent de \
                     Wikimedia Commons ; dans la galerie 3D, chaque toile est réinterprétée en mosaïque de touches \
                     de couleur, ce n'est pas une reproduction exacte.",
                    artists_sentence()
                )))
                .child(p().class("muted").text("© ARTEON · 100 % Rust · 0 ligne de JavaScript écrite à la main")),
        )
}

// ─── Composants ──────────────────────────────────────────────────────────────

/// La reproduction d'une œuvre (l'image de `public/tableaux/`).
fn picture(ex: &Exhibition, art: &Artwork, lazy: bool) -> Element {
    let image = img()
        .attr("src", art.image_url())
        .attr(
            "alt",
            format!("{}, {} ({})", art.title, art.artist_in(ex), art.year),
        )
        .attr("decoding", "async")
        .style(format!("aspect-ratio:{:.4}", art.width / art.height));
    if lazy {
        image.attr("loading", "lazy")
    } else {
        image
    }
}

fn wall_name(wall: Wall) -> &'static str {
    match wall {
        Wall::Left => "mur gauche",
        Wall::Right => "mur droit",
        Wall::Back => "mur du fond",
    }
}

/// Où se trouve l'œuvre dans la galerie, en clair.
fn location(ex: &Exhibition, art: &Artwork) -> String {
    format!(
        "{} · {} · place {}",
        ex.rooms[art.hook.room].name,
        wall_name(art.hook.wall),
        art.hook.place
    )
}

/// Le cartel d'une œuvre.
fn artwork_card(ex: &Exhibition, art: &Artwork, index: usize) -> Element {
    article()
        .class("art-card reveal")
        .style(format!("animation-delay:{}ms", 60 * index))
        .child(div().class("frame").child(picture(ex, art, true)))
        .child(h3().text(art.title))
        .child(
            p().child(strong().text(art.artist_in(ex)))
                .child(span().class("muted").text(format!(" · {}", art.year))),
        )
        .child(
            p().class("muted small")
                .text(format!("{} — {}", art.technique, art.collection)),
        )
        .child(p().text(art.note))
        .child(p().class("tag").text(location(ex, art)))
}

/// Un cube en CSS 3D, aux couleurs du musée, qui tourne dans l'en-tête.
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
/// Les mots ne se coupent pas : la ligne passe à la ligne entre deux mots.
fn animated_title(text: &'static str) -> Element {
    let mut index = 0;
    h1().class("hero-title")
        .attr("aria-label", text)
        .children(text.split(' ').map(|word| {
            span().class("word").attr("aria-hidden", "true").children(
                word.chars()
                    .map(|c| {
                        index += 1;
                        span()
                            .class("letter")
                            .style(format!("animation-delay:{}ms", 70 * index))
                            .text(c.to_string())
                    })
                    .collect::<Vec<_>>(),
            )
        }))
}

/// Les onglets d'une exposition : présentation, galerie 3D, œuvres.
fn tabs(ex: &Exhibition, current: &str) -> Element {
    let tab = |href: String, label: &'static str| {
        let link = a().href(href.clone()).text(label);
        if href == current {
            link.class("current").attr("aria-current", "page")
        } else {
            link
        }
    };
    nav()
        .class("ex-tabs")
        .attr("aria-label", format!("Exposition {}", ex.short_name))
        .child(tab(format!("/{}", ex.slug), "Présentation"))
        .child(tab(format!("/{}/galerie", ex.slug), "Galerie 3D"))
        .child(tab(format!("/{}/oeuvres", ex.slug), "Œuvres"))
}

fn features() -> Element {
    section()
        .class("features")
        .child(h2().class("section-title").text("Comment c'est fait"))
        .child(
            div()
                .class("grid")
                .child(div().class("card feature reveal").child(span().class("dot yellow")).child(h3().text("Touche par touche")).child(p().class("muted").text("Le moteur 3D n'affiche pas d'images : chaque toile devient une mosaïque de touches de couleur de 7 mm, légèrement en relief.")))
                .child(div().class("card feature reveal").child(span().class("dot blue")).child(h3().text("Une scène 3D en Rust")).child(p().class("muted").text("Murs, sols, tableaux et lumières sont des entités du moteur de jeu de Paquets, compilé en WebAssembly et rendu en WebGL 2.")))
                .child(div().class("card feature reveal").child(span().class("dot red")).child(h3().text("Le domaine public")).child(p().class("muted").text("Uniquement des peintres morts depuis plus de 70 ans. Les reproductions viennent de Wikimedia Commons.")))
                .child(div().class("card feature reveal").child(span().class("dot green")).child(h3().text("Votre accrochage")).child(p().class("muted").child(span().text("Une image et quelques lignes dans un fichier suffisent pour accrocher une toile, ou créer une exposition. ")).child(a().href("/ajouter").text("Ajouter une œuvre →")))),
        )
}

// ─── L'accueil du musée ──────────────────────────────────────────────────────

async fn home(_req: Request) -> Page {
    let works: usize = EXHIBITIONS.iter().map(|e| e.artworks.len()).sum();
    let rooms: usize = EXHIBITIONS.iter().map(|e| e.rooms.len()).sum();
    let covers: Vec<(&Exhibition, &Artwork)> = EXHIBITIONS
        .iter()
        .filter_map(|e| e.cover().map(|a| (e, a)))
        .take(3)
        .collect();
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
                        .child(p().class("kicker reveal").text("Musée virtuel · 100 % Rust"))
                        .child(animated_title("ARTEON"))
                        .child(p().class("hero-sub reveal").text("Les grands peintres, en 3D, dans votre navigateur"))
                        .child(p().class("lead reveal").text(
                            "ARTEON est un musée qui n'existe pas. Chaque exposition est consacrée à un peintre : \
                             ses toiles sont accrochées dans des salles reliées par des couloirs, et une caméra \
                             vous fait la visite guidée, de tableau en tableau.",
                        ))
                        .child(
                            div()
                                .class("row reveal")
                                .child(a().class("button").href("#expositions").text("Voir les expositions"))
                                .child(a().class("button ghost").href("/ajouter").text("Ajouter une œuvre")),
                        ),
                )
                .child(
                    div()
                        .class("hero-visual hero-covers")
                        .attr("aria-hidden", "true")
                        .children(covers.iter().enumerate().map(|(i, (ex, art))| {
                            div().class(format!("cover-frame c{i}")).child(picture(ex, art, false))
                        }))
                        .child(css_cube()),
                ),
        )
        .child(
            section()
                .class("stats")
                .child(div().class("stat reveal").child(strong().text(EXHIBITIONS.len().to_string())).child(span().text("expositions")))
                .child(div().class("stat reveal").child(strong().text(works.to_string())).child(span().text("toiles du domaine public")))
                .child(div().class("stat reveal").child(strong().text(rooms.to_string())).child(span().text("salles en 3D")))
                .child(div().class("stat reveal").child(strong().text("0")).child(span().text("ligne de JavaScript écrite à la main"))),
        )
        .child(
            section()
                .class("exhibitions")
                .id("expositions")
                .child(h2().class("section-title").text("Les expositions"))
                .child(div().class("ex-grid").children(EXHIBITIONS.iter().enumerate().map(|(i, ex)| {
                    let card = article()
                        .class("ex-card reveal")
                        .style(format!("--accent:{};animation-delay:{}ms", rgb(ex.accent), 120 * i));
                    let card = match ex.cover() {
                        Some(art) => card.child(a().href(format!("/{}", ex.slug)).class("ex-cover").child(picture(ex, art, true))),
                        None => card,
                    };
                    card.child(
                        div()
                            .class("ex-body")
                            .child(p().class("kicker").text(format!("{} · {}", ex.artist, ex.artist_dates)))
                            .child(h3().child(a().href(format!("/{}", ex.slug)).text(ex.title)))
                            .child(p().class("muted").text(ex.introduction))
                            .child(p().class("tag").text(format!(
                                "{} toiles · {} salles · {}",
                                ex.artworks.len(),
                                ex.rooms.len(),
                                ex.years()
                            )))
                            .child(
                                div()
                                    .class("row")
                                    .child(a().class("button").href(format!("/{}/galerie", ex.slug)).text("Visite 3D"))
                                    .child(a().class("button ghost").href(format!("/{}", ex.slug)).text("Découvrir")),
                            ),
                    )
                }))),
        )
        .child(features());
    layout(
        "Un musée virtuel en 3D",
        "ARTEON, un musée virtuel en 3D écrit en Rust : des expositions consacrées aux grands peintres du domaine public, à visiter dans le navigateur.",
        "/",
        None,
        content,
    )
}

// ─── Les pages d'une exposition ──────────────────────────────────────────────

async fn exhibition_home(_req: Request, ex: &'static Exhibition) -> Page {
    let minutes = (tour_duration(ex) / 60.0).round().max(1.0) as u32;
    let here = format!("/{}", ex.slug);
    let content = div()
        .child(tabs(ex, &here))
        .child(
            section()
                .class("hero")
                .child(div().class("floating f1").attr("aria-hidden", "true"))
                .child(div().class("floating f2").attr("aria-hidden", "true"))
                .child(
                    div()
                        .class("hero-text")
                        .child(p().class("kicker reveal").text(ex.kicker))
                        .child(animated_title(ex.artist))
                        .child(p().class("hero-sub reveal").text(ex.title))
                        .child(p().class("lead reveal").text(ex.introduction))
                        .child(
                            div()
                                .class("row reveal")
                                .child(
                                    a().class("button")
                                        .href(format!("/{}/galerie", ex.slug))
                                        .text("Lancer la visite 3D"),
                                )
                                .child(
                                    a().class("button ghost")
                                        .href(format!("/{}/oeuvres", ex.slug))
                                        .text("Voir les œuvres"),
                                ),
                        ),
                )
                .child(div().class("hero-visual").child(island(
                    "showcase",
                    ex.slug,
                    showcase_scene,
                ))),
        )
        .child(
            section()
                .class("stats")
                .child(
                    div()
                        .class("stat reveal")
                        .child(strong().text(ex.artworks.len().to_string()))
                        .child(span().text("toiles exposées")),
                )
                .child(
                    div()
                        .class("stat reveal")
                        .child(strong().text(ex.rooms.len().to_string()))
                        .child(span().text("salles d'exposition")),
                )
                .child(
                    div()
                        .class("stat reveal")
                        .child(strong().text(ex.years()))
                        .child(span().text("années de peinture")),
                )
                .child(
                    div()
                        .class("stat reveal")
                        .child(strong().text(format!("{minutes} min")))
                        .child(span().text("de visite guidée")),
                ),
        )
        .child(
            section()
                .class("rooms")
                .child(h2().class("section-title").text("Le parcours"))
                .child(
                    div()
                        .class("timeline")
                        .children(ex.rooms.iter().enumerate().map(|(i, room)| {
                            let works = ex.works_in_room(i);
                            div()
                                .class("step reveal")
                                .style(format!("animation-delay:{}ms", 120 * i))
                                .child(span().class("step-number").text(format!("{:02}", i + 1)))
                                .child(h3().text(room.name))
                                .child(p().class("muted").text(room.theme))
                                .child(div().class("step-works").children(works.iter().map(
                                    |art| {
                                        a().href(format!("/{}/oeuvres", ex.slug))
                                            .attr("title", art.title)
                                            .child(picture(ex, art, true))
                                    },
                                )))
                                .child(p().class("tag").text(format!("{} œuvre(s)", works.len())))
                        })),
                ),
        )
        .child(
            section()
                .class("preview")
                .child(h2().class("section-title").text("Quelques chefs-d'œuvre"))
                .child(div().class("preview-strip").children(
                    ex.artworks.iter().take(8).enumerate().map(|(i, art)| {
                        a().href(format!("/{}/oeuvres", ex.slug))
                            .class("tilt reveal")
                            .style(format!("animation-delay:{}ms", 90 * i))
                            .child(picture(ex, art, true))
                            .child(
                                span()
                                    .class("tilt-caption")
                                    .text(format!("{} · {}", art.title, art.year)),
                            )
                    }),
                )),
        )
        .child(others(ex));
    layout(ex.title, ex.introduction, &here, Some(ex.accent), content)
}

/// Les autres expositions du musée, en bas de la présentation d'une exposition.
fn others(ex: &Exhibition) -> Element {
    let others: Vec<&Exhibition> = EXHIBITIONS.iter().filter(|e| e.slug != ex.slug).collect();
    if others.is_empty() {
        return features();
    }
    section()
        .class("others")
        .child(h2().class("section-title").text("Les autres expositions"))
        .child(
            div()
                .class("others-grid")
                .children(others.into_iter().map(|o| {
                    let link = a()
                        .href(format!("/{}", o.slug))
                        .class("other reveal")
                        .style(format!("--accent:{}", rgb(o.accent)));
                    let link = match o.cover() {
                        Some(art) => link.child(picture(o, art, true)),
                        None => link,
                    };
                    link.child(
                        span()
                            .class("other-text")
                            .child(strong().text(o.short_name))
                            .child(span().text(o.title)),
                    )
                })),
        )
}

async fn gallery_page(_req: Request, ex: &'static Exhibition) -> Page {
    let here = format!("/{}/galerie", ex.slug);
    let content = div()
        .child(tabs(ex, &here))
        .child(
            section()
                .class("page-intro")
                .child(p().class("kicker").text(format!("Visite guidée · {}", ex.short_name)))
                .child(h1().text("La galerie en 3D"))
                .child(p().class("lead").text(
                    "La caméra traverse les salles et leurs couloirs, et s'arrête devant chaque toile. \
                     La visite recommence en boucle.",
                )),
        )
        .child(div().class("viewer").child(island("gallery", ex.slug, gallery_scene)))
        .child(div().class("grid room-legend").children(ex.rooms.iter().enumerate().map(|(i, room)| {
            div()
                .class("card")
                .child(h3().text(room.name))
                .child(p().class("muted").text(room.theme))
                .child(ul().children(ex.works_in_room(i).into_iter().map(|a| {
                    li().child(strong().text(a.title)).child(span().class("muted").text(format!(" · {}", a.year)))
                })))
        })));
    layout(
        format!("Galerie 3D · {}", ex.short_name),
        format!("Visite guidée en 3D de l'exposition {}.", ex.title),
        &here,
        Some(ex.accent),
        content,
    )
}

async fn catalogue(_req: Request, ex: &'static Exhibition) -> Page {
    let here = format!("/{}/oeuvres", ex.slug);
    let content = div()
        .child(tabs(ex, &here))
        .child(
            section()
                .class("page-intro")
                .child(p().class("kicker").text(format!("Catalogue · {}", ex.short_name)))
                .child(h1().text("Les œuvres"))
                .child(p().class("lead").text(format!(
                    "{} toiles de {} ({}), dans l'ordre de l'accrochage, avec leur cartel et leur place dans la galerie.",
                    ex.artworks.len(),
                    ex.artist,
                    ex.artist_dates
                ))),
        )
        .child(div().class("art-grid").children(ex.artworks.iter().enumerate().map(|(i, art)| artwork_card(ex, art, i))));
    layout(
        format!("Œuvres · {}", ex.short_name),
        format!(
            "Le catalogue de l'exposition {} : chaque toile, son cartel et sa place dans la galerie.",
            ex.title
        ),
        &here,
        Some(ex.accent),
        content,
    )
}

// ─── Ajouter une œuvre ───────────────────────────────────────────────────────

/// Le plan des emplacements d'une exposition : pour chaque salle, chaque mur et chaque
/// place, l'œuvre qui l'occupe ou « libre ».
fn slots_table(ex: &'static Exhibition) -> Element {
    let rows = ex.rooms.iter().enumerate().flat_map(move |(r, room)| {
        let last = r == ex.rooms.len() - 1;
        [
            ("gauche", Wall::Left),
            ("droite", Wall::Right),
            ("fond", Wall::Back),
        ]
        .into_iter()
        .map(move |(label, wall)| {
            tr().child(
                th().attr("scope", "row")
                    .text(format!("{} · {label}", room.name)),
            )
            .children((1..=3).map(move |place| {
                let taken = ex
                    .artworks
                    .iter()
                    .find(|a| a.hook.room == r && a.hook.wall == wall && a.hook.place == place);
                match taken {
                    Some(art) => td().class("taken").text(art.title),
                    None if wall == Wall::Back && place == 2 && !last => {
                        td().class("door").text("porte")
                    }
                    None => td().class("free").text("libre"),
                }
            }))
        })
    });
    div().class("table-wrap").child(
        table()
            .class("slots")
            .child(
                thead().child(
                    tr().child(th().text("Salle · mur"))
                        .child(th().text("Place 1"))
                        .child(th().text("Place 2"))
                        .child(th().text("Place 3")),
                ),
            )
            .child(tbody().children(rows)),
    )
}

async fn add_artwork(_req: Request) -> Page {
    const EXAMPLE: &str = r#"[[oeuvres]]
titre = "Le Déjeuner sur l'herbe"
annee = "1866"
image = "dejeuner-sur-l-herbe.jpg"
technique = "Huile sur toile, 130 × 181 cm"
collection = "Musée d'Orsay, Paris"
note = "Une partie de campagne en forêt de Fontainebleau, peinte en plein air."
salle = 1
mur = "fond"
place = 1
largeur = 1.6"#;
    let step = |n: &'static str, title: &'static str, body: Element| {
        div()
            .class("card how-step reveal")
            .child(span().class("step-number").text(n))
            .child(h3().text(title))
            .child(body)
    };
    let files = EXHIBITIONS
        .iter()
        .map(|e| format!("expositions/{}.toml", e.slug))
        .collect::<Vec<_>>()
        .join(", ");
    let content = div()
        .child(
            section()
                .class("page-intro")
                .child(p().class("kicker").text("Modifier le musée"))
                .child(h1().text("Ajouter ou modifier une œuvre"))
                .child(p().class("lead").text(format!(
                    "Chaque exposition est décrite dans un seul fichier ({files}) et ses images sont dans le dossier \
                     public/tableaux/<nom>/. Pas une ligne de code à écrire : le site se reconstruit et se republie \
                     tout seul après chaque modification."
                ))),
        )
        .child(
            div()
                .class("grid how")
                .child(step(
                    "01",
                    "Déposer l'image",
                    p().child(span().text("Une reproduction JPEG ou PNG de la toile (900 pixels de large suffisent), dans le dossier de l'exposition, par exemple "))
                        .child(code().text("public/tableaux/monet/"))
                        .child(span().text(". Sur GitHub : "))
                        .child(a().href(format!("{REPOSITORY}/tree/master/public/tableaux")).text("ouvrir le dossier, puis Add file → Upload files"))
                        .child(span().text(".")),
                ))
                .child(step(
                    "02",
                    "Décrire l'œuvre",
                    p().child(span().text("Copier un bloc "))
                        .child(code().text("[[oeuvres]]"))
                        .child(span().text(" dans le fichier de l'exposition ("))
                        .child(a().href(format!("{REPOSITORY}/tree/master/expositions")).text("dossier expositions/"))
                        .child(span().text(") et changer ses valeurs : le titre, l'image, la salle, le mur, la place et la largeur en mètres.")),
                ))
                .child(step(
                    "03",
                    "Enregistrer",
                    p().text(
                        "Valider la modification (« Commit changes ») sur la branche master. Quelques minutes plus \
                         tard, la toile est accrochée dans la galerie 3D, dans le catalogue et dans la visite guidée.",
                    ),
                )),
        )
        .child(h2().class("section-title").text("Un exemple"))
        .child(pre().class("example").child(code().text(EXAMPLE)))
        .child(
            ul().class("rules")
                .child(li().child(strong().text("mur")).child(span().text(" : \"gauche\" ou \"droite\" en avançant dans la galerie, ou \"fond\".")))
                .child(li().child(strong().text("place")).child(span().text(" : 1, 2 ou 3. Sur les murs latéraux, 1 est près de l'entrée de la salle ; sur le mur du fond, de gauche à droite. Le milieu du mur du fond est une porte, sauf dans la dernière salle.")))
                .child(li().child(strong().text("largeur")).child(span().text(" : entre 0,6 et 2,6 mètres ; la hauteur suit les proportions de l'image.")))
                .child(li().child(strong().text("Modifier ou retirer")).child(span().text(" : changer les valeurs d'un bloc, ou le supprimer. Ajouter un bloc [[salles]] ajoute une salle et son couloir.")))
                .child(li().child(strong().text("Une nouvelle exposition")).child(span().text(" : copier un fichier de expositions/ sous un autre nom (par exemple renoir.toml : l'exposition sera à l'adresse /renoir), créer le dossier public/tableaux/renoir/ pour ses images, et changer les textes et les œuvres. Elle apparaît d'elle-même sur l'accueil et dans le menu.")))
                .child(li().child(strong().text("Une erreur ?")).child(span().text(" La compilation s'arrête avec un message en français qui dit quoi corriger (image introuvable, place déjà prise, toiles qui se chevauchent…), et le site en ligne reste tel quel."))),
        )
        .child(h2().class("section-title").text("Les emplacements"))
        .children(EXHIBITIONS.iter().map(|ex| {
            div()
                .class("slots-block")
                .child(h3().text(format!("{} · expositions/{}.toml · images dans public/tableaux/{}/", ex.short_name, ex.slug, ex.slug)))
                .child(slots_table(ex))
        }));
    layout(
        "Ajouter une œuvre",
        "Comment ajouter, déplacer ou modifier une œuvre, ou créer une exposition dans ARTEON : une image et quelques lignes dans un fichier.",
        "/ajouter",
        None,
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
                        .href("/#expositions")
                        .text("Voir les expositions"),
                ),
        );
    layout(
        "Page introuvable",
        "Cette page d'ARTEON n'existe pas.",
        "",
        None,
        content,
    )
    .status(404)
}

// ─── L'application ───────────────────────────────────────────────────────────

/// Toutes les pages du site : l'accueil, le guide, et trois pages par exposition.
pub(crate) fn routes() -> Vec<String> {
    let mut list = vec!["/".to_owned(), "/ajouter".to_owned()];
    for ex in EXHIBITIONS {
        list.push(format!("/{}", ex.slug));
        list.push(format!("/{}/galerie", ex.slug));
        list.push(format!("/{}/oeuvres", ex.slug));
    }
    list
}

/// L'application : le même serveur pour `paquets dev` et pour l'export statique.
fn app() -> App {
    let mut app = App::new();
    app.get("/", home);
    app.get("/ajouter", add_artwork);
    for ex in EXHIBITIONS {
        app.get(&format!("/{}", ex.slug), move |req: Request| {
            exhibition_home(req, ex)
        });
        app.get(&format!("/{}/galerie", ex.slug), move |req: Request| {
            gallery_page(req, ex)
        });
        app.get(&format!("/{}/oeuvres", ex.slug), move |req: Request| {
            catalogue(req, ex)
        });
    }
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
