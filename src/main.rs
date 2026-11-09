//! Le serveur d'ARTEON : l'accueil du musée, et pour chaque exposition sa
//! présentation, sa galerie en 3D et son catalogue ; plus le guide pour ajouter une
//! œuvre. Tout le contenu vient de `expositions/*.toml` (voir `src/artworks.rs`).
//!
//!     paquets dev                         # http://localhost:3000
//!     cargo run --release -- export dist  # le site en fichiers statiques

use paquets::prelude::*;

use arteon::artworks::{Artist, Artwork, EXHIBITIONS, Exhibition, Notice, Sculpture, Side, Work};
use arteon::tour::tour_duration;
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
                .child(span().class("nav-artist").text(movement_or_dates(ex)))
        })))
}

/// Le style de l'exposition, ou ses dates s'il n'est pas donné.
fn movement_or_dates(ex: &Exhibition) -> &'static str {
    if ex.movement.is_empty() {
        ex.artist_dates
    } else {
        ex.movement
    }
}

/// Le nombre d'artistes différents exposés dans tout le musée.
fn artist_count() -> usize {
    let mut names: Vec<&str> = EXHIBITIONS
        .iter()
        .flat_map(|ex| ex.works().map(move |w| w.artist(ex)))
        .collect();
    names.sort_unstable();
    names.dedup();
    names.len()
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
    format!("{} artistes, morts il y a plus de 70 ans", artist_count())
}

/// Le grand pied de page : une bande de couleurs animée, un défilé des œuvres, trois
/// colonnes de liens et les mentions sur le domaine public.
fn site_footer() -> Element {
    // Le défilé est écrit deux fois de suite : l'animation glisse d'une moitié, en boucle.
    let marquee_items = || {
        EXHIBITIONS.iter().flat_map(|ex| {
            ex.artworks.iter().map(move |art| {
                a().href(work_url(ex, art.slug))
                    .class("marquee-item")
                    .attr("tabindex", "-1")
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
                .child(
                    div()
                        .class("marquee-track")
                        // Une vitesse de lecture : environ 6 secondes par œuvre, quel que soit leur nombre.
                        .style(format!(
                            "animation-duration:{}s",
                            6 * EXHIBITIONS.iter().map(|e| e.artworks.len()).sum::<usize>()
                        ))
                        .children(marquee_items())
                        .children(marquee_items()),
                ),
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
                    div().class("footer-col footer-wide").child(h3().text("Expositions")).child(ul().children(EXHIBITIONS.iter().map(
                        |ex| {
                            li().child(a().href(format!("/{}", ex.slug)).text(ex.short_name))
                                .child(span().text(" · "))
                                .child(
                                    a().href(format!("/{}/galerie", ex.slug))
                                        .class("footer-3d")
                                        .attr("aria-label", format!("Visite 3D · {}", ex.short_name))
                                        .text("3D"),
                                )
                        },
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

fn wall_name(side: Side) -> String {
    format!("mur {}", side.french())
}

/// Où se trouve l'œuvre dans la galerie, en clair.
fn location(ex: &Exhibition, art: &Artwork) -> String {
    let at = if art.hook.place > 0 {
        format!("place {}", art.hook.place)
    } else {
        format!("à {:.1} m du milieu", art.hook.position).replace('.', ",")
    };
    format!(
        "{} · {} · {at}",
        ex.rooms[art.hook.room].name,
        wall_name(art.hook.side)
    )
}

/// La vignette d'une sculpture (dessinée à la compilation).
fn sculpture_picture(s: &Sculpture, lazy: bool) -> Element {
    let image = img()
        .attr("src", s.image_url())
        .attr("alt", format!("{}, {} ({})", s.title, s.artist, s.year))
        .attr("decoding", "async")
        .class("sculpture-img");
    if lazy {
        image.attr("loading", "lazy")
    } else {
        image
    }
}

/// Le cartel d'une sculpture.
fn sculpture_card(ex: &Exhibition, s: &Sculpture, index: usize) -> Element {
    article()
        .class("art-card sculpture-card reveal")
        .style(format!("animation-delay:{}ms", 60 * index))
        .child(
            a().href(work_url(ex, s.slug))
                .class("plinth")
                .child(sculpture_picture(s, true)),
        )
        .child(h3().child(a().href(work_url(ex, s.slug)).text(s.title)))
        .child(
            p().child(strong().text(s.artist))
                .child(span().class("muted").text(format!(" · {}", s.year))),
        )
        .child(
            p().class("muted small")
                .text(format!("{} — {}", s.technique, s.collection)),
        )
        .child(p().text(s.note))
        .child(p().class("tag").text(
            format!("{} · sculpture, {:.2} m", ex.rooms[s.room].name, s.height).replace('.', ","),
        ))
        .child(more_link(ex, s.slug, &s.notice))
}

/// L'adresse de la page d'une œuvre.
fn work_url(ex: &Exhibition, slug: &str) -> String {
    format!("/{}/oeuvres/{slug}", ex.slug)
}

/// Le lien vers la notice complète d'une œuvre.
fn more_link(ex: &Exhibition, slug: &str, notice: &Notice) -> Element {
    let label = if notice.is_empty() {
        "Sa page →"
    } else {
        "Tout savoir sur l'œuvre →"
    };
    a().class("more").href(work_url(ex, slug)).text(label)
}

/// Le cartel d'une œuvre.
fn artwork_card(ex: &Exhibition, art: &Artwork, index: usize) -> Element {
    article()
        .class("art-card reveal")
        .style(format!("animation-delay:{}ms", 60 * index))
        .child(
            a().href(work_url(ex, art.slug))
                .class("frame")
                .child(picture(ex, art, true)),
        )
        .child(h3().child(a().href(work_url(ex, art.slug)).text(art.title)))
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
        .child(more_link(ex, art.slug, &art.notice))
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
    let works: usize = EXHIBITIONS
        .iter()
        .map(|e| e.artworks.len() + e.sculptures.len())
        .sum();
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
                        .child(p().class("hero-sub reveal").text("Toute l'histoire de l'art, en 3D, dans votre navigateur"))
                        .child(p().class("lead reveal").text(
                            "ARTEON est un musée qui n'existe pas. Du Moyen Âge à l'abstraction, de Florence à Edo, \
                             chaque exposition raconte un style dans un bâtiment inspiré d'un vrai musée. On s'y \
                             promène librement ou en visite guidée, et chaque œuvre a sa notice complète. Toutes \
                             sont dans le domaine public.",
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
                .child(div().class("stat reveal").child(strong().text(works.to_string())).child(span().text("œuvres du domaine public")))
                .child(div().class("stat reveal").child(strong().text(artist_count().to_string())).child(span().text("artistes")))
                .child(div().class("stat reveal").child(strong().text(rooms.to_string())).child(span().text("salles en 3D"))),
        )
        .child(
            section()
                .class("exhibitions")
                .id("expositions")
                .child(h2().class("section-title").text("Toute l'histoire de l'art"))
                .child(p().class("lead").text(
                    "Les expositions, dans l'ordre de l'histoire : chacune est un style, un musée et ses artistes.",
                ))
                .child(nav().class("styles").attr("aria-label", "Les styles").children(EXHIBITIONS.iter().map(|ex| {
                    a().class("chip")
                        .href(format!("#{}", ex.slug))
                        .style(format!("--accent:{}", rgb(ex.accent)))
                        .text(movement_or_dates(ex))
                })))
                .child(div().class("ex-grid").children(EXHIBITIONS.iter().enumerate().map(|(i, ex)| {
                    let card = article()
                        .class("ex-card reveal")
                        .id(ex.slug)
                        .style(format!("--accent:{};animation-delay:{}ms", rgb(ex.accent), 40 * (i % 6)));
                    let card = match ex.cover() {
                        Some(art) => card.child(a().href(format!("/{}", ex.slug)).class("ex-cover").child(picture(ex, art, true))),
                        None => card,
                    };
                    card.child(
                        div()
                            .class("ex-body")
                            .child(p().class("kicker").text(format!("{} · {}", movement_or_dates(ex), ex.artist_dates)))
                            .child(h3().child(a().href(format!("/{}", ex.slug)).text(ex.title)))
                            .child(p().class("ex-artists").text(ex.artist))
                            .child(p().class("muted ex-intro").text(ex.introduction))
                            .child(p().class("tag").text(counts(ex)))
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
        "ARTEON, un musée virtuel en 3D écrit en Rust : toute l'histoire de l'art du domaine public, du Moyen Âge à l'abstraction, à visiter dans le navigateur.",
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
                        .child(strong().text((ex.artworks.len() + ex.sculptures.len()).to_string()))
                        .child(span().text(if ex.sculptures.is_empty() {
                            "toiles exposées"
                        } else {
                            "œuvres exposées"
                        })),
                )
                .child(
                    div()
                        .class("stat reveal")
                        .child(strong().text(ex.galleries().count().to_string()))
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
                        .children(ex.galleries().enumerate().map(|(n, (i, room))| {
                            let works = ex.works_in_room(i);
                            let statues = ex.sculptures_in_room(i);
                            div()
                                .class("step reveal")
                                .style(format!("animation-delay:{}ms", 120 * n))
                                .child(span().class("step-number").text(format!("{:02}", n + 1)))
                                .child(h3().text(room.name))
                                .child(p().class("muted").text(room.theme))
                                .child(
                                    div()
                                        .class("step-works")
                                        .children(works.iter().map(|art| {
                                            a().href(work_url(ex, art.slug))
                                                .attr("title", art.title)
                                                .child(picture(ex, art, true))
                                        }))
                                        .children(statues.iter().map(|s| {
                                            a().href(work_url(ex, s.slug))
                                                .attr("title", s.title)
                                                .child(sculpture_picture(s, true))
                                        })),
                                )
                                .child(
                                    p().class("tag")
                                        .text(format!("{} œuvre(s)", works.len() + statues.len())),
                                )
                        })),
                ),
        )
        .child(
            section()
                .class("preview")
                .child(h2().class("section-title").text("Quelques chefs-d'œuvre"))
                .child(div().class("preview-strip").children(
                    ex.artworks.iter().take(8).enumerate().map(|(i, art)| {
                        a().href(work_url(ex, art.slug))
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
        .child(if ex.artists.is_empty() {
            div()
        } else {
            section()
                .class("artists")
                .id("artistes")
                .child(h2().class("section-title").text(if ex.artists.len() > 1 {
                    "Les artistes"
                } else {
                    "L'artiste"
                }))
                .child(
                    div()
                        .class("artist-list")
                        .children(ex.artists.iter().map(artist_card)),
                )
        })
        .child(others(ex));
    layout(ex.title, ex.introduction, &here, Some(ex.accent), content)
}

/// « 12 toiles · 2 sculptures · 3 salles · 1885–1890 ».
fn counts(ex: &Exhibition) -> String {
    let mut parts = vec![format!("{} toiles", ex.artworks.len())];
    if !ex.sculptures.is_empty() {
        parts.push(format!(
            "{} sculpture{}",
            ex.sculptures.len(),
            if ex.sculptures.len() > 1 { "s" } else { "" }
        ));
    }
    parts.push(format!("{} salles", ex.galleries().count()));
    parts.push(ex.years());
    parts.join(" · ")
}

/// Les autres expositions du musée, en bas de la présentation d'une exposition.
fn others(ex: &Exhibition) -> Element {
    // Les voisines dans l'histoire de l'art : trois avant, trois après.
    let here = EXHIBITIONS
        .iter()
        .position(|e| e.slug == ex.slug)
        .unwrap_or(0);
    let others: Vec<&Exhibition> = EXHIBITIONS
        .iter()
        .enumerate()
        .filter(|(i, e)| e.slug != ex.slug && i.abs_diff(here) <= 3)
        .map(|(_, e)| e)
        .collect();
    if others.is_empty() {
        return features();
    }
    section()
        .class("others")
        .child(h2().class("section-title").text("Avant, après"))
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
                .child(p().class("kicker").text(format!("Visite · {}", ex.short_name)))
                .child(h1().text("Le musée en 3D"))
                .child(p().class("lead").text(
                    "En visite guidée, la caméra traverse les salles et s'arrête devant chaque œuvre, en boucle. \
                     En visite libre, on marche où l'on veut : ZQSD ou WASD et la souris sur ordinateur, les deux \
                     joysticks sur téléphone. Le plan permet de se rendre d'un clic dans une salle ou devant une œuvre, \
                     et l'éditeur de déplacer les œuvres.",
                )),
        )
        .child(div().class("viewer").child(island("gallery", ex.slug, gallery_scene)))
        .child(div().class("grid room-legend").children(ex.galleries().map(|(i, room)| {
            div()
                .class("card")
                .child(h3().text(room.name))
                .child(p().class("muted").text(room.theme))
                .child(
                    ul().children(ex.works_in_room(i).into_iter().map(|a| {
                        li().child(strong().text(a.title)).child(span().class("muted").text(format!(" · {}", a.year)))
                    }))
                    .children(ex.sculptures_in_room(i).into_iter().map(|s| {
                        li().child(strong().text(s.title)).child(span().class("muted").text(format!(" · {}, sculpture", s.year)))
                    })),
                )
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
                    "{} toiles de {} ({}), dans l'ordre de l'accrochage, avec leur cartel et leur place dans la galerie. \
                     Chaque œuvre a sa page : ce que l'on voit, ce qu'elle évoque, son histoire, la manière de l'artiste.",
                    ex.artworks.len(),
                    ex.artist,
                    ex.artist_dates
                ))),
        )
        .child(div().class("art-grid").children(ex.artworks.iter().enumerate().map(|(i, art)| artwork_card(ex, art, i))))
        .child(if ex.sculptures.is_empty() {
            div()
        } else {
            div()
                .child(h2().class("section-title").text("Les sculptures"))
                .child(div().class("art-grid").children(ex.sculptures.iter().enumerate().map(|(i, s)| sculpture_card(ex, s, i))))
        });
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

/// La vie et la manière d'un artiste.
fn artist_card(artist: &Artist) -> Element {
    let block = article()
        .class("card artist-card reveal")
        .child(h3().text(artist.name));
    let block = if artist.life.is_empty() {
        block
    } else {
        block.child(p().class("muted small").text(artist.life))
    };
    let block = if artist.biography.is_empty() {
        block
    } else {
        block.child(p().text(artist.biography))
    };
    if artist.style.is_empty() {
        block
    } else {
        block
            .child(h4().text("Sa manière"))
            .child(p().text(artist.style))
    }
}

/// La page d'une œuvre : sa reproduction, son cartel, et tout ce qu'il faut savoir.
async fn work_page(_req: Request, ex: &'static Exhibition, work: Work) -> Page {
    let catalogue = format!("/{}/oeuvres", ex.slug);
    let artist = work.artist(ex);
    let notice = work.notice();
    let works: Vec<Work> = ex.works().collect();
    let index = works
        .iter()
        .position(|w| w.slug() == work.slug())
        .unwrap_or(0);
    let (year, technique, collection, note, place) = match work {
        Work::Art(art) => (
            art.year,
            art.technique,
            art.collection,
            art.note,
            location(ex, art),
        ),
        Work::Sculpture(s) => (
            s.year,
            s.technique,
            s.collection,
            s.note,
            format!(
                "{} · sculpture de {:.2} m, sur un socle",
                ex.rooms[s.room].name, s.height
            )
            .replace('.', ","),
        ),
    };
    let image = match work {
        Work::Art(art) => a()
            .href(art.image_url())
            .class("frame work-frame")
            .attr("title", "Ouvrir la reproduction en grand")
            .child(picture(ex, art, false)),
        Work::Sculpture(s) => div()
            .class("plinth work-frame")
            .child(sculpture_picture(s, false)),
    };
    let fact = |label: &'static str, value: String| {
        div()
            .class("fact")
            .child(dt().text(label))
            .child(dd().text(value))
    };
    let mut facts = dl()
        .class("facts")
        .child(fact("Artiste", artist.to_owned()))
        .child(fact("Date", year.to_owned()));
    if !technique.is_empty() {
        facts = facts.child(fact("Technique", technique.to_owned()));
    }
    if !collection.is_empty() {
        facts = facts.child(fact("Collection", collection.to_owned()));
    }
    facts = facts.child(fact("Dans le musée", place));
    let cartel = div()
        .class("work-cartel")
        .child(p().class("lead").text(note))
        .child(facts)
        .child(
            div()
                .class("row")
                .child(
                    a().class("button")
                        .href(format!("/{}/galerie#{}", ex.slug, work.slug()))
                        .text("La voir dans le musée 3D"),
                )
                .child(
                    a().class("button ghost")
                        .href(catalogue.clone())
                        .text("Toutes les œuvres"),
                ),
        );
    let cartel = if notice.themes.is_empty() {
        cartel
    } else {
        cartel.child(
            div()
                .class("themes")
                .children(notice.themes.iter().map(|t| span().class("chip").text(*t))),
        )
    };
    let mut body = div().class("notice");
    for (title, text) in notice.sections() {
        body = body.child(
            section()
                .class("notice-part reveal")
                .child(h2().text(title))
                .child(p().text(text)),
        );
    }
    if !notice.details.is_empty() {
        body = body.child(
            section()
                .class("notice-part reveal")
                .child(h2().text("À regarder de près"))
                .child(
                    ul().class("details")
                        .children(notice.details.iter().map(|d| li().text(*d))),
                ),
        );
    }
    if !notice.anecdote.is_empty() {
        body = body.child(
            aside()
                .class("anecdote reveal")
                .child(strong().text("Le saviez-vous ? "))
                .child(span().text(notice.anecdote)),
        );
    }
    let neighbour = |w: Work, label: &'static str, class: &'static str| {
        a().class(class)
            .href(work_url(ex, w.slug()))
            .child(span().class("muted small").text(label))
            .child(strong().text(w.title()))
    };
    let previous = works[(index + works.len() - 1) % works.len()];
    let next = works[(index + 1) % works.len()];
    let content = div()
        .child(tabs(ex, &catalogue))
        .child(
            section()
                .class("page-intro")
                .child(
                    p().class("kicker")
                        .child(
                            a().href(catalogue.clone())
                                .text(format!("Œuvres · {}", ex.short_name)),
                        )
                        .child(span().text(format!(" · {} / {}", index + 1, works.len()))),
                )
                .child(h1().text(work.title()))
                .child(p().class("work-byline").text(format!("{artist} · {year}"))),
        )
        .child(div().class("work-head").child(image).child(cartel))
        .child(body)
        .child(match ex.artist_named(artist) {
            Some(about) => section()
                .class("artists")
                .child(h2().class("section-title").text("L'artiste"))
                .child(artist_card(about)),
            None => section(),
        })
        .child(
            nav()
                .class("work-nav")
                .attr("aria-label", "Œuvre précédente et suivante")
                .child(neighbour(previous, "← Précédente", "prev"))
                .child(neighbour(next, "Suivante →", "next")),
        );
    let summary = if notice.description.is_empty() {
        note
    } else {
        notice.description
    };
    layout(
        format!("{} · {}", work.title(), artist),
        summary.chars().take(155).collect::<String>(),
        &catalogue,
        Some(ex.accent),
        content,
    )
}

// ─── Ajouter une œuvre ───────────────────────────────────────────────────────

/// Le plan des emplacements d'une exposition : pour chaque salle, chaque mur et chaque
/// place, l'œuvre qui l'occupe ou « libre ».
fn slots_table(ex: &'static Exhibition) -> Element {
    let rows = ex.galleries().flat_map(move |(r, room)| {
        Side::ALL.into_iter().map(move |side| {
            let length = room.wall_length(side);
            let places = room.places(side);
            let cells = (1..=places).map(move |place| {
                let at = -length / 2.0 + length * (f32::from(place) - 0.5) / f32::from(places);
                let taken = ex.artworks.iter().find(|a| {
                    a.hook.room == r && a.hook.side == side && (a.hook.position - at).abs() < 1.2
                });
                let door = room
                    .doors
                    .iter()
                    .any(|d| d.side == side && (d.position - at).abs() < d.width / 2.0 + 0.6);
                let chip = span().class("chip");
                match taken {
                    Some(art) => chip.class("taken").text(format!("{place} · {}", art.title)),
                    None if door => chip.class("door").text(format!("{place} · porte")),
                    None => chip.class("free").text(format!("{place} · libre")),
                }
            });
            tr().child(
                th().attr("scope", "row")
                    .text(format!("{} · {}", room.name, side.french())),
            )
            .child(
                td().child(
                    span()
                        .class("muted small")
                        .text(format!("{:.1} m", length).replace('.', ",")),
                ),
            )
            .child(td().children(cells))
        })
    });
    div().class("table-wrap").child(
        table()
            .class("slots")
            .child(
                thead().child(
                    tr().child(th().text("Salle · mur"))
                        .child(th().text("Longueur"))
                        .child(th().text("Places, de gauche à droite face au mur")),
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
    const PLAN: &str = r##"[[salles]]
nom = "La grande galerie"
theme = "Une longue galerie haute, éclairée par le haut."
murs = "#e9e3d6"
x = 0            # le centre de la salle, en mètres
z = 0
largeur = 9      # est-ouest
profondeur = 30  # nord-sud
hauteur = 7
portes = ["sud", "nord", { mur = "est", position = -5 }]

[[salles]]
nom = "Le cabinet"
x = 8.5          # contre le mur est de la galerie
z = -5
largeur = 8
profondeur = 8
hauteur = 4.5    # sa porte vers la galerie est ajoutée d'elle-même"##;
    const SCULPTURE: &str = r##"[[sculptures]]
titre = "The Greek Slave"
artiste = "Hiram Powers"
annee = "1843"
modele = "greek-slave.stl"   # dans modeles/
technique = "Plâtre"
collection = "Smithsonian American Art Museum"
note = "La sculpture la plus célèbre de son temps."
salle = 1
x = 0                        # depuis le centre de la salle
z = 4
hauteur = 1.66               # la sculpture, sans le socle
rotation = 0                 # 0 : elle regarde le sud (l'entrée)
matiere = "marbre"           # marbre, platre, pierre, bronze, or ou #rrggbb
socle = 0.6"##;
    const NOTICE: &str = r##"# Dans le bloc [[oeuvres]] (ou [[sculptures]]), après note :
description = """
Ce que l'on voit : la scène, les personnages, la composition…"""
evocation = """
Ce qu'elle évoque : le sens, les émotions, les symboles…"""
histoire = """
Son histoire : pourquoi, pour qui, l'accueil, le parcours jusqu'au musée…"""
facture = """
La manière : la touche, les couleurs, la lumière, le cadrage…"""
details = [
  "Un détail précis à repérer dans l'œuvre.",
  "Un autre.",
]
themes = ["nuit", "paysage", "japonisme"]
anecdote = "Le saviez-vous ? Un fait marquant."

# À la fin du fichier, un bloc par artiste (le nom exact des œuvres) :
[[artistes]]
nom = "Vincent van Gogh"
vie = "Groot-Zundert, 1853 – Auvers-sur-Oise, 1890"
biographie = """
Sa vie…"""
style = """
Sa manière…""""##;
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
        .child(h2().class("section-title").text("Déplacer une œuvre depuis le musée"))
        .child(p().class("lead").text(
            "Dans la visite 3D, le bouton « Éditer » ouvre l'éditeur du musée : choisir une toile ou une sculpture \
             (ou « L'œuvre regardée »), puis sa salle, son mur et sa position avec les curseurs. La scène se met à jour \
             aussitôt, et l'éditeur dit si la place est libre. « Télécharger » donne le fichier de l'exposition modifié \
             (seules les lignes de placement changent), « Copier » le met dans le presse-papiers, et « Publier sur GitHub » \
             ouvre l'éditeur du fichier dans le dépôt : il reste à coller et à valider.",
        ))
        .child(h2().class("section-title").text("Un exemple"))
        .child(pre().class("example").child(code().text(EXAMPLE)))
        .child(
            ul().class("rules")
                .child(li().child(strong().text("mur")).child(span().text(" : \"nord\", \"sud\", \"est\" ou \"ouest\" (le visiteur entre par le sud et avance vers le nord) ; \"gauche\", \"droite\" et \"fond\" marchent aussi.")))
                .child(li().child(strong().text("place")).child(span().text(" : 1, 2, 3… numérotées de gauche à droite quand on regarde le mur (une place tous les 3 m environ, voir le tableau ci-dessous). Ou bien position = 2.5 : le centre du tableau à 2,5 m du milieu du mur, vers la droite (négatif : vers la gauche).")))
                .child(li().child(strong().text("largeur")).child(span().text(" : en mètres ; la hauteur suit les proportions de l'image. Un grand tableau peut garder sa taille réelle si la salle est assez haute (la Ronde de nuit fait 4,5 m).")))
                .child(li().child(strong().text("Modifier ou retirer")).child(span().text(" : changer les valeurs d'un bloc, ou le supprimer.")))
                .child(li().child(strong().text("Une nouvelle exposition")).child(span().text(" : copier un fichier de expositions/ sous un autre nom (par exemple renoir.toml : l'exposition sera à l'adresse /renoir), créer le dossier public/tableaux/renoir/ pour ses images, et changer les textes et les œuvres. Elle apparaît d'elle-même sur l'accueil et dans le menu.")))
                .child(li().child(strong().text("Une erreur ?")).child(span().text(" La compilation s'arrête avec un message en français qui dit quoi corriger (image introuvable, place déjà prise, toiles qui se chevauchent, tableau sur une porte, salles qui se chevauchent, sculpture contre un mur…), et le site en ligne reste tel quel."))),
        )
        .child(h2().class("section-title").text("Écrire la notice complète"))
        .child(p().class("lead").text(
            "Chaque œuvre a sa page (/<exposition>/oeuvres/<nom de l'image>) et son panneau « Tout savoir » dans le musée 3D. \
             Ils affichent les champs facultatifs ci-dessous : ce que l'on voit, ce qu'elle évoque, son histoire, la manière, \
             les détails à regarder, les thèmes et une anecdote, ainsi que la biographie de l'artiste.",
        ))
        .child(pre().class("example").child(code().text(NOTICE)))
        .child(h2().class("section-title").text("Dessiner le plan du musée"))
        .child(p().class("lead").text(
            "Sans plan, les salles s'enchaînent en enfilade, reliées par des couloirs. Avec un plan, chaque salle a sa place \
             (x vers l'est, z vers le sud, en mètres depuis le centre du musée), sa taille et ses portes : on dessine une \
             galerie d'honneur, une nef, un atrium… Une porte se déclare d'un seul côté : la salle voisine reçoit la sienne.",
        ))
        .child(pre().class("example").child(code().text(PLAN)))
        .child(h2().class("section-title").text("Ajouter une sculpture"))
        .child(p().class("lead").text(
            "Déposer le modèle 3D (STL, OBJ ou glTF) dans le dossier modeles/ : il est simplifié à la compilation pour \
             rester léger, posé sur un socle, et une vignette est dessinée pour le catalogue. Des scans libres de droits \
             se trouvent par exemple au Smithsonian (3d.si.edu, licence CC0).",
        ))
        .child(pre().class("example").child(code().text(SCULPTURE)))
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
        for work in ex.works() {
            list.push(format!("/{}/oeuvres/{}", ex.slug, work.slug()));
        }
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
        for work in ex.works() {
            app.get(
                &format!("/{}/oeuvres/{}", ex.slug, work.slug()),
                move |req: Request| work_page(req, ex, work),
            );
        }
    }
    // Les fichiers de données des expositions (mosaïques, sculptures, vignettes).
    app.static_dir("/donnees", arteon::artworks::DATA_DIR);
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
