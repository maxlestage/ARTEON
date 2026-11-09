//! Le serveur d'ARTEON : la présentation de l'exposition, la galerie en 3D, le
//! catalogue et le guide pour ajouter une œuvre. Tout le contenu vient de
//! `exposition.toml` (voir `src/artworks.rs`).
//!
//!     paquets dev                       # http://localhost:3000
//!     cargo run --release -- export dist  # le site en fichiers statiques

use paquets::prelude::*;

use arteon::artworks::{ARTWORKS, Artwork, EXHIBITION, ROOMS, Wall, works_in_room};
use arteon::gallery::tour_duration;
use arteon::{gallery_scene, showcase_scene};

mod export;
mod style;

/// Le dépôt du projet, pour les liens « modifier sur GitHub ».
const REPOSITORY: &str = "https://github.com/maxlestage/ARTEON";

// ─── Mise en page commune ────────────────────────────────────────────────────

fn nav_link(href: &'static str, label: &'static str, current: &str) -> Element {
    let link = a().href(href).text(label);
    if href == current {
        link.class("current").attr("aria-current", "page")
    } else {
        link
    }
}

/// Le logo : un petit soleil tourbillonnant sur un ciel bleu, puis le nom.
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

/// Les textes passés à `Page` doivent être `&'static str` ou `String` (pas un `&str`
/// emprunté) : toutes les pages passent ici des littéraux ou des données statiques.
fn layout(
    title: impl Into<String>,
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
                    .child(nav_link("/oeuvres", "Œuvres", current))
                    .child(
                        nav_link("/ajouter", "Ajouter", current)
                            .child(span().class("nav-long").text(" une œuvre")),
                    ),
            ),
        )
        .body(html::main().class("page").child(content))
        .body(site_footer())
}

/// Le grand pied de page : une bande de couleurs animée, un défilé des œuvres, trois
/// colonnes de liens et les mentions sur le domaine public.
fn site_footer() -> Element {
    // Le défilé est écrit deux fois de suite : l'animation glisse d'une moitié, en boucle.
    let marquee_items = || {
        ARTWORKS.iter().map(|art| {
            span()
                .class("marquee-item")
                .child(strong().text(art.title))
                .child(span().text(format!(" · {}", art.year)))
        })
    };
    footer()
        .class("site-footer")
        .child(div().class("footer-band").attr("aria-hidden", "true").children(
            ["#f2b705", "#1d3f8f", "#d4572a", "#2f6b4f", "#f4ecd8", "#0f1a3a", "#e8c547", "#5b8bd0"]
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
                        .child(p().text(format!(
                            "{} : une exposition virtuelle en 3D, écrite uniquement en Rust, du serveur à la scène \
                             WebGL compilée en WebAssembly.",
                            EXHIBITION.title
                        )))
                        .child(a().class("button footer-cta").href("/galerie").text("Entrer dans la galerie →")),
                )
                .child(
                    div()
                        .class("footer-col")
                        .child(h3().text("Visiter"))
                        .child(ul().children(ROOMS.iter().map(|room| li().child(a().href("/galerie").text(room.name))))),
                )
                .child(
                    div().class("footer-col").child(h3().text("Œuvres")).child(
                        ul().children(ARTWORKS.iter().take(6).map(|art| li().child(a().href("/oeuvres").text(art.title)))),
                    ),
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
                    "{} ({}) : toutes les œuvres présentées sont dans le domaine public. Les reproductions \
                     viennent de Wikimedia Commons ; dans la galerie 3D, chaque toile est réinterprétée en \
                     mosaïque de touches en relief, ce n'est pas une reproduction exacte.",
                    EXHIBITION.artist, EXHIBITION.artist_dates
                )))
                .child(p().class("muted").text("© ARTEON · 100 % Rust · 0 ligne de JavaScript écrite à la main")),
        )
}

// ─── Composants ──────────────────────────────────────────────────────────────

/// La reproduction d'une œuvre (l'image de `public/tableaux/`).
fn picture(art: &Artwork, lazy: bool) -> Element {
    let image = img()
        .attr("src", art.image_url())
        .attr(
            "alt",
            format!("{}, {} ({})", art.title, EXHIBITION.artist, art.year),
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
fn location(art: &Artwork) -> String {
    format!(
        "{} · {} · place {}",
        ROOMS[art.hook.room].name,
        wall_name(art.hook.wall),
        art.hook.place
    )
}

/// Le cartel d'une œuvre.
fn artwork_card(art: &Artwork, index: usize) -> Element {
    article()
        .class("art-card reveal")
        .style(format!("animation-delay:{}ms", 60 * index))
        .child(div().class("frame").child(picture(art, true)))
        .child(h3().text(art.title))
        .child(
            p().child(strong().text(EXHIBITION.artist))
                .child(span().class("muted").text(format!(" · {}", art.year))),
        )
        .child(
            p().class("muted small")
                .text(format!("{} — {}", art.technique, art.collection)),
        )
        .child(p().text(art.note))
        .child(p().class("tag").text(location(art)))
}

/// Un cube en CSS 3D, aux couleurs de Van Gogh, qui tourne dans l'en-tête.
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

/// Les années couvertes par l'exposition (« 1885–1890 »).
fn years() -> String {
    let numbers: Vec<u32> = ARTWORKS
        .iter()
        .filter_map(|a| a.year.get(..4)?.parse().ok())
        .collect();
    match (numbers.iter().min(), numbers.iter().max()) {
        (Some(first), Some(last)) if first != last => format!("{first}–{last}"),
        (Some(year), _) => year.to_string(),
        _ => "—".into(),
    }
}

// ─── Pages ───────────────────────────────────────────────────────────────────

async fn home(_req: Request) -> Page {
    let minutes = (tour_duration() / 60.0).round().max(1.0) as u32;
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
                        .child(p().class("kicker reveal").text(EXHIBITION.kicker))
                        .child(animated_title(EXHIBITION.artist))
                        .child(p().class("hero-sub reveal").text(EXHIBITION.title))
                        .child(p().class("lead reveal").text(EXHIBITION.introduction))
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
                .child(div().class("stat reveal").child(strong().text(ARTWORKS.len().to_string())).child(span().text("toiles exposées")))
                .child(div().class("stat reveal").child(strong().text(ROOMS.len().to_string())).child(span().text("salles d'exposition")))
                .child(div().class("stat reveal").child(strong().text(years())).child(span().text("années de peinture")))
                .child(div().class("stat reveal").child(strong().text(format!("{minutes} min"))).child(span().text("de visite guidée"))),
        )
        .child(
            section()
                .class("rooms")
                .child(h2().class("section-title").text("Le parcours"))
                .child(div().class("timeline").children(ROOMS.iter().enumerate().map(|(i, room)| {
                    let works = works_in_room(i);
                    div()
                        .class("step reveal")
                        .style(format!("animation-delay:{}ms", 120 * i))
                        .child(span().class("step-number").text(format!("{:02}", i + 1)))
                        .child(h3().text(room.name))
                        .child(p().class("muted").text(room.theme))
                        .child(div().class("step-works").children(works.iter().map(|art| {
                            a().href("/oeuvres").attr("title", art.title).child(picture(art, true))
                        })))
                        .child(p().class("tag").text(format!("{} œuvre(s)", works.len())))
                }))),
        )
        .child(
            section()
                .class("preview")
                .child(h2().class("section-title").text("Quelques chefs-d'œuvre"))
                .child(div().class("preview-strip").children(ARTWORKS.iter().take(8).enumerate().map(|(i, art)| {
                    a().href("/oeuvres")
                        .class("tilt reveal")
                        .style(format!("animation-delay:{}ms", 90 * i))
                        .child(picture(art, true))
                        .child(span().class("tilt-caption").text(format!("{} · {}", art.title, art.year)))
                }))),
        )
        .child(
            section()
                .class("features")
                .child(h2().class("section-title").text("Comment c'est fait"))
                .child(
                    div()
                        .class("grid")
                        .child(div().class("card feature reveal").child(span().class("dot yellow")).child(h3().text("Touche par touche")).child(p().class("muted").text("Chaque toile devient une mosaïque de petites touches en relief, plus épaisses dans les tons clairs, comme les empâtements du peintre.")))
                        .child(div().class("card feature reveal").child(span().class("dot blue")).child(h3().text("Une scène 3D en Rust")).child(p().class("muted").text("Murs, sols, tableaux et lumières sont des entités du moteur de jeu de Paquets, compilé en WebAssembly et rendu en WebGL 2.")))
                        .child(div().class("card feature reveal").child(span().class("dot red")).child(h3().text("Le domaine public")).child(p().class("muted").text("Van Gogh est mort en 1890 : ses toiles appartiennent à tous. Les reproductions viennent de Wikimedia Commons.")))
                        .child(div().class("card feature reveal").child(span().class("dot green")).child(h3().text("Votre accrochage")).child(p().class("muted").child(span().text("Une image et quelques lignes dans un fichier suffisent pour accrocher une nouvelle toile. ")).child(a().href("/ajouter").text("Ajouter une œuvre →")))),
                ),
        );
    layout(
        EXHIBITION.title,
        "Une exposition virtuelle en 3D consacrée à Vincent van Gogh : ses toiles, reconstruites touche par touche dans un musée écrit en Rust.",
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
                    "La caméra traverse les salles et leurs couloirs, et s'arrête devant chaque toile. \
                     La visite recommence en boucle.",
                )),
        )
        .child(div().class("viewer").child(island("gallery", "", gallery_scene)))
        .child(div().class("grid room-legend").children(ROOMS.iter().enumerate().map(|(i, room)| {
            div()
                .class("card")
                .child(h3().text(room.name))
                .child(p().class("muted").text(room.theme))
                .child(ul().children(works_in_room(i).into_iter().map(|a| {
                    li().child(strong().text(a.title)).child(span().class("muted").text(format!(" · {}", a.year)))
                })))
        })));
    layout(
        "La galerie en 3D",
        "Visite guidée en 3D de l'exposition Van Gogh d'ARTEON.",
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
                .child(p().class("lead").text(format!(
                    "{} toiles de {} ({}), dans l'ordre de l'accrochage, avec leur cartel et leur place dans la galerie.",
                    ARTWORKS.len(),
                    EXHIBITION.artist,
                    EXHIBITION.artist_dates
                ))),
        )
        .child(div().class("art-grid").children(ARTWORKS.iter().enumerate().map(|(i, art)| artwork_card(art, i))));
    layout(
        "Les œuvres",
        "Le catalogue de l'exposition Van Gogh d'ARTEON : chaque toile, son cartel et sa place dans la galerie.",
        "/oeuvres",
        content,
    )
}

/// Le plan des emplacements : pour chaque salle, chaque mur et chaque place, l'œuvre
/// qui l'occupe ou « libre ».
fn slots_table() -> Element {
    let rows = ROOMS.iter().enumerate().flat_map(|(r, room)| {
        let last = r == ROOMS.len() - 1;
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
                let taken = ARTWORKS
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
titre = "Le Semeur"
annee = "1888"
image = "semeur.jpg"
technique = "Huile sur toile, 64 × 80,5 cm"
collection = "Musée Kröller-Müller, Otterlo"
note = "Un soleil immense derrière le semeur, dans un champ violet."
salle = 3
mur = "droite"
place = 3
largeur = 1.3"#;
    let step = |n: &'static str, title: &'static str, body: Element| {
        div()
            .class("card how-step reveal")
            .child(span().class("step-number").text(n))
            .child(h3().text(title))
            .child(body)
    };
    let content = div()
        .child(
            section()
                .class("page-intro")
                .child(p().class("kicker").text("Modifier l'exposition"))
                .child(h1().text("Ajouter ou modifier une œuvre"))
                .child(p().class("lead").text(
                    "Tout le musée est décrit dans un seul fichier, exposition.toml, et les images sont dans le \
                     dossier public/tableaux/. Pas une ligne de code à écrire : le site se reconstruit et se \
                     republie tout seul après chaque modification.",
                )),
        )
        .child(
            div()
                .class("grid how")
                .child(step(
                    "01",
                    "Déposer l'image",
                    p().child(span().text("Une reproduction JPEG ou PNG de la toile (900 pixels de large suffisent), dans le dossier "))
                        .child(code().text("public/tableaux/"))
                        .child(span().text(". Sur GitHub : "))
                        .child(a().href(format!("{REPOSITORY}/upload/master/public/tableaux")).text("Add file → Upload files"))
                        .child(span().text(".")),
                ))
                .child(step(
                    "02",
                    "Décrire l'œuvre",
                    p().child(span().text("Copier un bloc "))
                        .child(code().text("[[oeuvres]]"))
                        .child(span().text(" dans "))
                        .child(a().href(format!("{REPOSITORY}/edit/master/exposition.toml")).text("exposition.toml"))
                        .child(span().text(" et changer ses valeurs : le titre, l'image, la salle, le mur, la place et la largeur en mètres.")),
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
                .child(li().child(strong().text("Une erreur ?")).child(span().text(" La compilation s'arrête avec un message en français qui dit quoi corriger (image introuvable, place déjà prise, toiles qui se chevauchent…), et le site en ligne reste tel quel."))),
        )
        .child(h2().class("section-title").text("Les emplacements"))
        .child(slots_table());
    layout(
        "Ajouter une œuvre",
        "Comment ajouter, déplacer ou modifier une œuvre de l'exposition ARTEON : une image et quelques lignes dans exposition.toml.",
        "/ajouter",
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

/// Les pages du site.
pub(crate) const ROUTES: [&str; 4] = ["/", "/galerie", "/oeuvres", "/ajouter"];

/// L'application : le même serveur pour `paquets dev` et pour l'export statique.
fn app() -> App {
    let mut app = App::new();
    app.get(ROUTES[0], home);
    app.get(ROUTES[1], gallery_page);
    app.get(ROUTES[2], catalogue);
    app.get(ROUTES[3], add_artwork);
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
