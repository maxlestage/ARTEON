//! Le serveur d'ARTEON : l'accueil du musée, et pour chaque exposition sa
//! présentation, sa galerie en 3D et son catalogue ; plus le guide pour ajouter une
//! œuvre. Tout le contenu vient de `expositions/*.toml` (voir `src/artworks.rs`).
//!
//!     paquets dev                         # http://localhost:3000
//!     cargo run --release -- export dist  # le site en fichiers statiques

use paquets::prelude::*;

use arteon::artworks::{Artist, Artwork, EXHIBITIONS, Exhibition, Notice, Sculpture, Side, Work};
use arteon::langue::{Lang, lang, set_lang, t, u};
use arteon::tf;
use arteon::tour::tour_duration;
use arteon::{gallery_scene, showcase_scene};

mod api;
mod catalogue;
mod export;
mod style;

/// Le dépôt du projet, pour les liens « modifier sur GitHub ».
/// Qui a conçu et développé ARTEON (et Paquets).
const AUTHOR: &str = "Maxime Nathan Lestage";

const REPOSITORY: &str = "https://github.com/maxlestage/ARTEON";

// ─── Mise en page commune ────────────────────────────────────────────────────

/// Les expositions dans la langue de la page.
fn exhibitions() -> &'static [Exhibition] {
    lang().exhibitions()
}

/// Un lien du menu ; `current` est l'adresse de la page affichée (sans la langue).
fn nav_link(href: &str, label: impl Into<String>, current: &str) -> Element {
    let active = if href == "/" {
        current == "/"
    } else {
        let h = href.trim_end_matches('/');
        current == h || current.starts_with(&format!("{h}/"))
    };
    let link = a().href(u(href)).text(label.into());
    if active {
        link.class("current").attr("aria-current", "page")
    } else {
        link
    }
}

/// Les dernières expositions arrivées au musée : en tête du menu, avec la mention « nouveau ».
const NEW_EXHIBITIONS: &[&str] = &["voitures", "velos", "robes", "verreries", "photographie-art"];

/// Une ligne de la liste des expositions du menu.
fn menu_item(ex: &Exhibition, current: &str, new: bool) -> Element {
    let item = li().child(nav_link(&format!("/{}", ex.slug), ex.short_name, current));
    let item = if new || NEW_EXHIBITIONS.contains(&ex.slug) {
        item.child(span().class("nav-new").text(t("nouveau", "new", "nuevo")))
    } else {
        item
    };
    item.child(span().class("nav-artist").text(movement_or_dates(ex)))
}

/// Le menu « burger » : tout le musée derrière un seul bouton (`<details>` : il s'ouvre
/// sans JavaScript), les langues comprises.
fn burger_menu(current: &str) -> Element {
    let exs = exhibitions();
    let inside = exs.iter().any(|ex| {
        let href = format!("/{}", ex.slug);
        current == href || current.starts_with(&format!("{href}/"))
    });
    let group = |title: &'static str| li().class("nav-group").text(title);
    let entry = |href: &str, label: &'static str, detail: &'static str| {
        li().child(nav_link(href, label, current)).child(span().class("nav-artist").text(detail))
    };
    let exhibitions_list = details()
        .class("burger-exhibitions")
        .bool_attr("open", move || inside)
        .child(summary().text(tf!("Les {} expositions", "All {} exhibitions", "Las {} exposiciones", exs.len())))
        .child(
            ul().child(group(t("Nouveautés", "New", "Novedades")))
                .children(
                    NEW_EXHIBITIONS
                        .iter()
                        .filter_map(|slug| exs.iter().find(|ex| ex.slug == *slug))
                        .map(|ex| menu_item(ex, current, true)),
                )
                .child(group(t("Toutes les expositions", "All the exhibitions", "Todas las exposiciones")))
                .children(exs.iter().map(|ex| menu_item(ex, current, false))),
        );
    let langues = Lang::ALL.into_iter().map(|l| {
        let href = format!("{}{}", l.prefix(), if current == "/" { "/" } else { current });
        let link = a().href(href).attr("hreflang", l.code()).attr("lang", l.code()).text(l.name());
        li().child(if l == lang() { link.class("current").attr("aria-current", "true") } else { link })
    });
    details()
        .class("burger")
        .child(
            summary()
                .class("burger-button")
                .attr("aria-label", t("Ouvrir le menu", "Open the menu", "Abrir el menú"))
                .child(span().class("burger-icon").attr("aria-hidden", "true").child(span()).child(span()).child(span()))
                .child(span().class("burger-label").text(t("Menu", "Menu", "Menú"))),
        )
        .child(
            nav()
                .class("burger-panel")
                .attr("aria-label", t("Tout le musée", "The whole museum", "Todo el museo"))
                .child(
                    ul().class("burger-list")
                        .child(group(t("Visiter", "Visit", "Visitar")))
                        .child(entry("/", t("Accueil", "Home", "Inicio"), t("le musée", "the museum", "el museo")))
                        .child(entry("/#expositions", t("Les expositions", "Exhibitions", "Exposiciones"), t("en 3D", "in 3D", "en 3D")))
                        .child(li().class("burger-wide").child(exhibitions_list))
                        .child(entry("/musees/", t("Les musées", "Museums", "Museos"), t("plans et bâtiments", "plans and buildings", "planos y edificios")))
                        .child(entry("/monde/", t("Les collections du monde", "World collections", "Colecciones del mundo"), t("domaine public", "public domain", "dominio público")))
                        .child(group(t("Explorer", "Explore", "Explorar")))
                        .child(entry("/explorer/", t("Les artistes de A à Z", "Artists from A to Z", "Artistas de la A a la Z"), t("tous les artistes", "every artist", "todos los artistas")))
                        .child(entry("/explorer/#/chronologie", t("La chronologie", "Timeline", "Cronología"), t("siècle par siècle", "century by century", "siglo a siglo")))
                        .child(entry("/explorer/#/themes", t("Les thèmes", "Themes", "Temas"), t("sujets et motifs", "subjects and motifs", "temas y motivos")))
                        .child(entry("/explorer/#/jeu", t("Qui l'a fait ?", "Who made it?", "¿Quién lo hizo?"), t("le jeu", "the game", "el juego")))
                        .child(group(t("Le projet", "The project", "El proyecto")))
                        .child(entry("/ajouter", t("Ajouter une œuvre", "Add an artwork", "Añadir una obra"), t("le guide", "the guide", "la guía")))
                        .child(entry("/credits", t("Crédits et licences", "Credits and licences", "Créditos y licencias"), t("vie privée", "privacy", "privacidad")))
                        .child(
                            li().child(a().href(REPOSITORY).attr("target", "_blank").attr("rel", "noopener").text(t("Code source", "Source code", "Código fuente")))
                                .child(span().class("nav-artist").text("GitHub")),
                        )
                        .child(group(t("Langue", "Language", "Idioma")))
                        .child(li().class("burger-wide").child(ul().class("burger-langs").children(langues))),
                ),
        )
}

/// Un texte pour une adresse (« Vincent van Gogh » → « Vincent%20van%20Gogh »).
fn encode_fragment(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
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
    a().href(u("/"))
        .class("logo")
        .attr("aria-label", t("ARTEON, accueil", "ARTEON, home", "ARTEON, inicio"))
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

/// La page complète. `current` est l'adresse de la page sans la langue (pour le menu et
/// les liens vers les autres langues) ; `accent`, la couleur de l'exposition affichée.
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
    // Les autres langues de la même page, pour les moteurs de recherche.
    let alternates: String = Lang::ALL
        .iter()
        .map(|l| {
            format!(
                r#"<link rel="alternate" hreflang="{}" href="{}{}"/>"#,
                l.code(),
                l.prefix(),
                if current == "/" { "/" } else { current }
            )
        })
        .collect();
    Page::new(title)
        .lang(lang().code())
        .description(description.into())
        .seo(Seo::new().site_name("ARTEON").title_template("%s · ARTEON"))
        .view_transitions()
        .prefetch(Prefetch::Hover)
        .stylesheet(style_url())
        .head(r#"<link rel="icon" type="image/png" sizes="32x32" href="/icons/favicon-32.png"/>"#)
        .head(alternates)
        .body(
            header()
                .class("site-header")
                .id("top")
                .child(logo())
                .child(
                    div()
                        .class("header-links")
                        .child(nav_link("/#expositions", t("Expositions", "Exhibitions", "Exposiciones"), current).class("header-quick"))
                        .child(nav_link("/explorer/", t("Explorer", "Explore", "Explorar"), current).class("header-quick"))
                        .child(burger_menu(current)),
                ),
        )
        .body(page)
        .body(site_footer())
}

/// L'adresse de la feuille de style, avec son empreinte (le navigateur la garde, et en
/// reprend une neuve quand elle change). Une seule feuille pour toutes les pages, plutôt
/// que le même style recopié dans chacune : le site est trois fois plus grand en trois langues.
fn style_url() -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    style::CSS.hash(&mut h);
    format!("/style.css?v={:08x}", h.finish() as u32)
}

/// Le nombre d'artistes, en clair.
fn artists_sentence() -> String {
    tf!("{} artistes et ateliers", "{} artists and workshops", "{} artistas y talleres", artist_count())
}

/// Le grand pied de page : une bande de couleurs animée, un défilé des œuvres, trois
/// colonnes de liens et les mentions sur le domaine public.
fn site_footer() -> Element {
    // Le défilé est écrit deux fois de suite : l'animation glisse d'une moitié, en boucle.
    let marquee_items = || {
        // Une œuvre par exposition (celle de sa vitrine) : le défilé reste léger, même avec
        // des centaines d'œuvres.
        exhibitions().iter().flat_map(|ex| {
            ex.cover().into_iter().map(move |art| {
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
    let works: usize = exhibitions()
        .iter()
        .map(|e| e.artworks.len() + e.sculptures.len())
        .sum();
    let external = |link: Element| link.attr("target", "_blank").attr("rel", "noopener");
    let column = |title: &'static str, links: Vec<(&'static str, String, bool)>| {
        div()
            .class("footer-col")
            .child(h3().text(title))
            .child(
                ul().children(links.into_iter().map(|(text, href, outside)| {
                    let link = a().href(if outside { href } else { u(&href) }).text(text);
                    li().child(if outside { external(link) } else { link })
                })),
            )
    };
    let tile =
        |name: &'static str, text: &'static str, href: &'static str, tag: Option<&'static str>| {
            let title = span().class("eco-name").text(name);
            let title = match tag {
                Some(t) => title.child(span().class("eco-tag").text(t)),
                None => title,
            };
            li().child(
                external(a().class("eco").href(if href.starts_with('/') { u(href) } else { href.to_owned() }))
                    .child(title)
                    .child(span().class("eco-text").text(text)),
            )
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
                .attr("aria-label", t("Les œuvres exposées", "The works on show", "Las obras expuestas"))
                .child(
                    div()
                        .class("marquee-track")
                        // Une vitesse de lecture : environ 6 secondes par œuvre, quel que soit leur nombre.
                        .style(format!(
                            "animation-duration:{}s",
                            6 * exhibitions().iter().filter(|e| e.cover().is_some()).count()
                        ))
                        .children(marquee_items())
                        .children(marquee_items()),
                ),
        )
        // L'appel à la visite.
        .child(
            div()
                .class("footer-cta-block")
                .child(
                    a().class("footer-cta-link")
                        .href(u("/#expositions"))
                        .child(span().text(t("Entrer au ", "Enter the ", "Entrar al ")))
                        .child(em().text(t("musée", "museum", "museo")))
                        .child(span().class("arrow").text(" →")),
                )
                .child(p().class("footer-figures").text(tf!(
                    "{} expositions · {} œuvres · {} artistes · de l'Antiquité au XXᵉ siècle",
                    "{} exhibitions · {} works · {} artists · from Antiquity to the 20th century",
                    "{} exposiciones · {} obras · {} artistas · de la Antigüedad al siglo XX",
                    EXHIBITIONS.len(),
                    works,
                    artist_count()
                ))),
        )
        .child(
            div()
                .class("footer-grid")
                .child(
                    div()
                        .class("footer-brand")
                        .child(logo())
                        .child(p().text(t(
                            "Un musée virtuel en 3D, gratuit et ouvert à tous : peinture, sculpture, objets, \
                             photographie, de l'Antiquité aux avant-gardes, chaque œuvre avec sa notice complète.",
                            "A free 3D virtual museum, open to all: painting, sculpture, objects and photography, \
                             from Antiquity to the avant-gardes, each work with its full notice.",
                            "Un museo virtual en 3D, gratuito y abierto a todos: pintura, escultura, objetos y \
                             fotografía, de la Antigüedad a las vanguardias, cada obra con su ficha completa.",
                        )))
                        .child(
                            ul().class("footer-badges")
                                .child(li().text("100 % Rust"))
                                .child(li().text("WebAssembly"))
                                .child(li().text("egui"))
                                .child(li().child(external(a().href(REPOSITORY).text("GitHub")))),
                        ),
                )
                .child(
                    div().class("footer-col footer-wide").child(h3().text(t("Expositions", "Exhibitions", "Exposiciones"))).child(ul().children(exhibitions().iter().map(
                        |ex| {
                            li().child(a().href(u(&format!("/{}", ex.slug))).text(ex.short_name))
                                .child(span().text(" · "))
                                .child(
                                    a().href(u(&format!("/{}/galerie", ex.slug)))
                                        .class("footer-3d")
                                        .attr("aria-label", tf!("Visite 3D · {}", "3D tour · {}", "Visita 3D · {}", ex.short_name))
                                        .text("3D"),
                                )
                        },
                    ))),
                )
                .child(column(
                    t("Le musée", "The museum", "El museo"),
                    vec![
                        (t("Accueil", "Home", "Inicio"), "/".to_owned(), false),
                        (t("Toutes les expositions", "All the exhibitions", "Todas las exposiciones"), "/#expositions".to_owned(), false),
                        (t("Les musées", "Museums", "Museos"), "/musees/".to_owned(), false),
                        (t("Les collections du monde", "World collections", "Colecciones del mundo"), "/monde/".to_owned(), false),
                        (t("Explorer les artistes", "Explore the artists", "Explorar los artistas"), "/explorer/".to_owned(), false),
                        (t("Crédits et licences", "Credits and licences", "Créditos y licencias"), "/credits".to_owned(), false),
                        (t("Ajouter une œuvre", "Add an artwork", "Añadir una obra"), "/ajouter".to_owned(), false),
                        (t("Code source", "Source code", "Código fuente"), REPOSITORY.to_owned(), true),
                    ],
                )),
        )
        // Construit avec : l'écosystème de Paquets, et les sources des œuvres.
        .child(
            nav()
                .class("footer-eco")
                .attr("aria-label", t("Construit avec", "Built with", "Construido con"))
                .child(
                    div()
                        .class("footer-eco-head")
                        .child(h3().text(t("Construit avec", "Built with", "Construido con")))
                        .child(p().text(t(
                            "Les cinq projets Rust de Paquets, du front-end à la base de données, et les collections \
                             ouvertes des musées.",
                            "The five Rust projects of Paquets, from front end to database, and the museums' open collections.",
                            "Los cinco proyectos Rust de Paquets, del front-end a la base de datos, y las colecciones \
                             abiertas de los museos.",
                        ))),
                )
                .child(
                    ul().class("eco-grid")
                        .child(tile("Paquets", t("Le framework full-stack qui les réunit", "The full-stack framework that brings them together", "El framework full-stack que los reúne"), "https://github.com/maxlestage/Paquets", Some(t("le socle", "the base", "la base"))))
                        .child(tile("Active", t("Front-end : composants, SSR, îlots WebAssembly, 3D", "Front end: components, SSR, WebAssembly islands, 3D", "Front-end: componentes, SSR, islas WebAssembly, 3D"), "https://active-c1883a966650.herokuapp.com/", None))
                        .child(tile("Vitesse", t("Serveur web façon Express, HTTP/3", "Express-style web server, HTTP/3", "Servidor web al estilo Express, HTTP/3"), "https://vitesse-f16135654c3d.herokuapp.com/", None))
                        .child(tile("Securite", t("Middlewares de sécurité pour Vitesse et tower", "Security middleware for Vitesse and tower", "Middlewares de seguridad para Vitesse y tower"), "https://securite-2db4c1994be7.herokuapp.com/", None))
                        .child(tile("Memoire", t("ORM asynchrone typé, SQL et NoSQL", "Typed async ORM, SQL and NoSQL", "ORM asíncrono tipado, SQL y NoSQL"), "https://memoire-7f4525f1fdaf.herokuapp.com/", None))
                        .child(tile("egui", t("L'interface de visite, dessinée dans le navigateur", "The tour interface, drawn in the browser", "La interfaz de visita, dibujada en el navegador"), "https://github.com/emilk/egui", None))
                        .child(tile("Wikimedia Commons", t("Les reproductions des œuvres", "The reproductions of the works", "Las reproducciones de las obras"), "https://commons.wikimedia.org/", Some(t("œuvres", "works", "obras"))))
                        .child(tile("Smithsonian Open Access", t("Les sculptures scannées en 3D (CC0)", "The 3D-scanned sculptures (CC0)", "Las esculturas escaneadas en 3D (CC0)"), "https://3d.si.edu/", None))
                        .child(tile(t("Musées en accès libre", "Open-access museums", "Museos de acceso abierto"), "Met, Cleveland, Walters, Rijksmuseum, Art Institute…", "/credits", None)),
                ),
        )
        // La signature.
        .child(
            div()
                .class("footer-credits")
                .child(p().class("footer-by").text(t("Conçu et développé par", "Designed and developed by", "Diseñado y desarrollado por")))
                .child(p().class("footer-name").children(
                    AUTHOR
                        .split(' ')
                        .enumerate()
                        .map(|(i, word)| span().style(format!("--i:{i}")).text(word.to_owned())),
                ))
                .child(p().class("footer-crafted").text(t("Fabriqué en Rust, en France", "Made in Rust, in France", "Hecho en Rust, en Francia"))),
        )
        .child(
            div()
                .class("footer-word")
                .attr("aria-hidden", "true")
                .children("ARTEON".chars().enumerate().map(|(i, c)| span().style(format!("--i:{i}")).text(c.to_string()))),
        )
        .child(
            div()
                .class("footer-legal")
                .child(p().text(tf!(
                    "{} : les œuvres sont dans le domaine public, ou photographiées sous licence libre et créditées. \
                     Dans le musée 3D, chaque image est réinterprétée en mosaïque : ce n'est pas une reproduction exacte. \
                     ARTEON est indépendant : les musées réels qui inspirent ses bâtiments n'y sont pas associés et ne l'ont pas approuvé.",
                    "{}: the works are in the public domain, or photographed under a free licence and credited. \
                     In the 3D museum each image is reinterpreted as a mosaic: it is not an exact reproduction. \
                     ARTEON is independent: the real museums that inspire its buildings are not associated with it and have not endorsed it.",
                    "{}: las obras son de dominio público, o fotografiadas con licencia libre y acreditadas. \
                     En el museo 3D cada imagen se reinterpreta como un mosaico: no es una reproducción exacta. \
                     ARTEON es independiente: los museos reales que inspiran sus edificios no están asociados ni lo han aprobado.",
                    artists_sentence()
                )))
                .child(
                    div()
                        .class("footer-bottom")
                        .child(p().class("muted").text(tf!("© 2026 {} · code sous licence MIT", "© 2026 {} · code under the MIT licence", "© 2026 {} · código con licencia MIT", AUTHOR)))
                        .child(a().href(u("/credits")).text(t("Crédits et licences", "Credits and licences", "Créditos y licencias")))
                        .child(p().class("muted").text(t("📦 Construit avec Paquets — Rust, Active et WebAssembly", "📦 Built with Paquets — Rust, Active and WebAssembly", "📦 Construido con Paquets — Rust, Active y WebAssembly")))
                        .child(a().class("totop").href("#top").attr("aria-label", t("Retour en haut", "Back to top", "Volver arriba")).text("↑")),
                ),
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
    match side {
        Side::North => t("mur nord", "north wall", "muro norte"),
        Side::South => t("mur sud", "south wall", "muro sur"),
        Side::East => t("mur est", "east wall", "muro este"),
        Side::West => t("mur ouest", "west wall", "muro oeste"),
    }
    .to_owned()
}

/// Un nombre décimal à la manière de la langue (« 2,5 » en français et en espagnol).
fn decimal(text: String) -> String {
    if lang() == Lang::En { text } else { text.replace('.', ",") }
}

/// Où se trouve l'œuvre dans la galerie, en clair.
fn location(ex: &Exhibition, art: &Artwork) -> String {
    let at = if art.hook.place > 0 {
        tf!("place {}", "place {}", "lugar {}", art.hook.place)
    } else {
        decimal(tf!("à {:.1} m du milieu", "{:.1} m from the centre", "a {:.1} m del centro", art.hook.position))
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
        .child(p().class("tag").text(decimal(
            tf!("{} · sculpture, {:.2} m", "{} · sculpture, {:.2} m", "{} · escultura, {:.2} m", ex.rooms[s.room].name, s.height),
        )))
        .child(more_link(ex, s.slug, &s.notice))
}

/// D'où vient une reproduction : son auteur, sa licence, et le lien vers sa source.
fn credit_line(credit: &arteon::artworks::Credit) -> Element {
    let mut parts = span().class("credit");
    if !credit.author.is_empty() {
        parts = parts.child(span().text(format!("{} · ", credit.author)));
    }
    parts = parts.child(span().text(if credit.licence.is_empty() {
        t("Domaine public", "Public domain", "Dominio público")
    } else {
        credit.licence_or_default()
    }));
    if credit.source.is_empty() {
        parts.child(span().class("muted").text(" · Wikimedia Commons"))
    } else {
        parts.child(span().text(" · ")).child(
            a().href(credit.source)
                .attr("target", "_blank")
                .attr("rel", "noopener")
                .text("source"),
        )
    }
}

/// L'adresse de la page d'une œuvre.
fn work_url(ex: &Exhibition, slug: &str) -> String {
    u(&format!("/{}/oeuvres/{slug}", ex.slug))
}

/// Le lien vers la notice complète d'une œuvre.
fn more_link(ex: &Exhibition, slug: &str, notice: &Notice) -> Element {
    let label = if notice.is_empty() {
        t("Sa page →", "Its page →", "Su página →")
    } else {
        t("Tout savoir sur l'œuvre →", "All about the work →", "Todo sobre la obra →")
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
        .child(if art.credit.requires_attribution() {
            p().class("muted small")
                .text(format!("{} · {}", art.credit.author, art.credit.licence))
        } else {
            p()
        })
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
        let link = a().href(u(&href)).text(label);
        if href == current {
            link.class("current").attr("aria-current", "page")
        } else {
            link
        }
    };
    nav()
        .class("ex-tabs")
        .attr("aria-label", tf!("Exposition {}", "Exhibition {}", "Exposición {}", ex.short_name))
        .child(tab(format!("/{}", ex.slug), t("Présentation", "Overview", "Presentación")))
        .child(tab(format!("/{}/galerie", ex.slug), t("Galerie 3D", "3D gallery", "Galería 3D")))
        .child(tab(format!("/{}/oeuvres", ex.slug), t("Œuvres", "Works", "Obras")))
}

fn features() -> Element {
    section()
        .class("features")
        .child(h2().class("section-title").text(t("Comment c'est fait", "How it is made", "Cómo está hecho")))
        .child(
            div()
                .class("grid")
                .child(div().class("card feature reveal").child(span().class("dot yellow")).child(h3().text(t("Touche par touche", "Stroke by stroke", "Pincelada a pincelada"))).child(p().class("muted").text(t("Le moteur 3D n'affiche pas d'images : chaque toile devient une mosaïque de touches de couleur de 7 mm, légèrement en relief.", "The 3D engine does not display images: each canvas becomes a mosaic of 7 mm dabs of colour, slightly raised.", "El motor 3D no muestra imágenes: cada lienzo se convierte en un mosaico de toques de color de 7 mm, ligeramente en relieve."))))
                .child(div().class("card feature reveal").child(span().class("dot blue")).child(h3().text(t("Une scène 3D en Rust", "A 3D scene in Rust", "Una escena 3D en Rust"))).child(p().class("muted").text(t("Murs, sols, tableaux et lumières sont des entités du moteur de jeu de Paquets, compilé en WebAssembly et rendu en WebGL 2.", "Walls, floors, paintings and lights are entities of the Paquets game engine, compiled to WebAssembly and rendered with WebGL 2.", "Muros, suelos, cuadros y luces son entidades del motor de juego de Paquets, compilado a WebAssembly y renderizado con WebGL 2."))))
                .child(div().class("card feature reveal").child(span().class("dot red")).child(h3().text(t("Le domaine public", "The public domain", "El dominio público"))).child(p().class("muted").text(t("Uniquement des artistes morts depuis plus de 70 ans. Les reproductions viennent de Wikimedia Commons et des collections ouvertes des musées.", "Only artists who died more than 70 years ago. The reproductions come from Wikimedia Commons and the museums' open collections.", "Solo artistas fallecidos hace más de 70 años. Las reproducciones proceden de Wikimedia Commons y de las colecciones abiertas de los museos."))))
                .child(div().class("card feature reveal").child(span().class("dot green")).child(h3().text(t("Votre accrochage", "Your own hang", "Su propio montaje"))).child(p().class("muted").child(span().text(t("Une image et quelques lignes dans un fichier suffisent pour accrocher une toile, ou créer une exposition. ", "An image and a few lines in a file are enough to hang a painting, or to create an exhibition. ", "Una imagen y unas líneas en un archivo bastan para colgar un cuadro o crear una exposición. "))).child(a().href(u("/ajouter")).text(t("Ajouter une œuvre →", "Add an artwork →", "Añadir una obra →"))))),
        )
}

// ─── L'accueil du musée ──────────────────────────────────────────────────────

async fn home(_req: Request) -> Page {
    let works: usize = exhibitions()
        .iter()
        .map(|e| e.artworks.len() + e.sculptures.len())
        .sum();
    let rooms: usize = exhibitions().iter().map(|e| e.rooms.len()).sum();
    let covers: Vec<(&Exhibition, &Artwork)> = exhibitions()
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
                        .child(p().class("kicker reveal").text(t("Musée virtuel · 100 % Rust", "Virtual museum · 100% Rust", "Museo virtual · 100 % Rust")))
                        .child(animated_title("ARTEON"))
                        .child(p().class("hero-sub reveal").text(t("Toute l'histoire de l'art, en 3D, dans votre navigateur", "The whole history of art, in 3D, in your browser", "Toda la historia del arte, en 3D, en su navegador")))
                        .child(p().class("lead reveal").text(t(
                            "ARTEON est un musée qui n'existe pas. De l'Antiquité aux avant-gardes, de Florence à Edo, \
                             chaque exposition raconte un style dans un bâtiment inspiré d'un vrai musée. On s'y \
                             promène librement ou en visite guidée, et chaque œuvre a sa notice complète. Toutes \
                             sont libres de droits ou photographiées sous licence libre.",
                            "ARTEON is a museum that does not exist. From Antiquity to the avant-gardes, from Florence to Edo, \
                             each exhibition tells the story of a style in a building inspired by a real museum. You can \
                             wander freely or take a guided tour, and every work has its full notice. All of them are \
                             copyright-free or photographed under a free licence.",
                            "ARTEON es un museo que no existe. De la Antigüedad a las vanguardias, de Florencia a Edo, \
                             cada exposición cuenta un estilo en un edificio inspirado en un museo real. Se puede recorrer \
                             libremente o en visita guiada, y cada obra tiene su ficha completa. Todas están libres de \
                             derechos o fotografiadas con licencia libre.",
                        )))
                        .child(
                            div()
                                .class("row reveal")
                                .child(a().class("button").href("#expositions").text(t("Voir les expositions", "See the exhibitions", "Ver las exposiciones")))
                                .child(a().class("button ghost").href(u("/ajouter")).text(t("Ajouter une œuvre", "Add an artwork", "Añadir una obra"))),
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
                .child(div().class("stat reveal").child(strong().text(EXHIBITIONS.len().to_string())).child(span().text(t("expositions", "exhibitions", "exposiciones"))))
                .child(div().class("stat reveal").child(strong().text(works.to_string())).child(span().text(t("œuvres", "works", "obras"))))
                .child(div().class("stat reveal").child(strong().text(artist_count().to_string())).child(span().text(t("artistes", "artists", "artistas"))))
                .child(div().class("stat reveal").child(strong().text(rooms.to_string())).child(span().text(t("salles en 3D", "rooms in 3D", "salas en 3D")))),
        )
        .child({
            let current: Vec<&'static Exhibition> = exhibitions().iter().filter(|e| e.ephemeral).collect();
            if current.is_empty() {
                section()
            } else {
                section()
                    .class("exhibitions ephemeral")
                    .id("en-ce-moment")
                    .child(h2().class("section-title").text(t("En ce moment dans les musées du monde", "Now on view in the world's museums", "Ahora en los museos del mundo")))
                    .child(p().class("lead").text(t(
                        "Des expositions éphémères, refaites chaque semaine : des œuvres tirées au sort parmi celles que \
                         de grands musées exposent en ce moment dans leurs salles. Quand un musée change son accrochage, \
                         l'exposition change avec lui.",
                        "Temporary exhibitions, remade every week: works drawn at random from those that great museums \
                         are showing in their galleries right now. When a museum rehangs, the exhibition changes with it.",
                        "Exposiciones efímeras, renovadas cada semana: obras elegidas al azar entre las que grandes museos \
                         exponen ahora en sus salas. Cuando un museo cambia su montaje, la exposición cambia con él.",
                    )))
                    .child(div().class("ex-grid").children(current.into_iter().enumerate().map(|(i, ex)| ex_card(i, ex))))
            }
        })
        .child(
            section()
                .class("exhibitions")
                .id("expositions")
                .child(h2().class("section-title").text(t("Toute l'histoire de l'art", "The whole history of art", "Toda la historia del arte")))
                .child(p().class("lead").text(t(
                    "Les expositions, dans l'ordre de l'histoire : chacune est un style, un musée et ses artistes.",
                    "The exhibitions, in historical order: each one is a style, a museum and its artists.",
                    "Las exposiciones, en orden histórico: cada una es un estilo, un museo y sus artistas.",
                )))
                .child(nav().class("styles").attr("aria-label", t("Les styles", "Styles", "Estilos")).children(
                    exhibitions().iter().filter(|e| !e.ephemeral).map(|ex| {
                        a().class("chip")
                            .href(format!("#{}", ex.slug))
                            .style(format!("--accent:{}", rgb(ex.accent)))
                            .text(movement_or_dates(ex))
                    }),
                ))
                .child(div().class("ex-grid").children(
                    exhibitions().iter().filter(|e| !e.ephemeral).enumerate().map(|(i, ex)| ex_card(i, ex)),
                )),
        )
        .child(install_card())
        .child(features());
    layout(
        t("Un musée virtuel en 3D", "A 3D virtual museum", "Un museo virtual en 3D"),
        t(
            "ARTEON, un musée virtuel en 3D écrit en Rust : toute l'histoire de l'art, de l'Antiquité aux avant-gardes, peinture, sculpture, objets et photographie, à visiter dans le navigateur.",
            "ARTEON, a 3D virtual museum written in Rust: the whole history of art, from Antiquity to the avant-gardes, painting, sculpture, objects and photography, to visit in your browser.",
            "ARTEON, un museo virtual en 3D escrito en Rust: toda la historia del arte, de la Antigüedad a las vanguardias, pintura, escultura, objetos y fotografía, para visitar en el navegador.",
        ),
        "/",
        None,
        content,
    )
}

/// Installer ARTEON comme une application, sur téléphone ou sur ordinateur.
fn install_card() -> Element {
    let step = |title: &'static str, text: &'static str| {
        div()
            .class("card install-step reveal")
            .child(h3().text(title))
            .child(p().text(text))
    };
    section()
        .class("install")
        .id("installer")
        .child(h2().class("section-title").text(t("Installer ARTEON", "Install ARTEON", "Instalar ARTEON")))
        .child(p().class("lead").text(t(
            "ARTEON s'installe comme une application : une icône sur l'écran d'accueil, le musée en plein écran, \
             et les salles déjà visitées consultables sans réseau.",
            "ARTEON installs like an app: an icon on your home screen, the museum in full screen, \
             and the rooms you have already visited available offline.",
            "ARTEON se instala como una aplicación: un icono en la pantalla de inicio, el museo a pantalla completa \
             y las salas ya visitadas disponibles sin conexión.",
        )))
        .child(
            div()
                .class("grid install-grid")
                .child(step(
                    t("iPhone et iPad", "iPhone and iPad", "iPhone y iPad"),
                    t("Dans Safari, touchez le bouton Partager, puis « Sur l'écran d'accueil ».", "In Safari, tap the Share button, then “Add to Home Screen”.", "En Safari, toque el botón Compartir y luego «Añadir a pantalla de inicio»."),
                ))
                .child(step(
                    "Android",
                    t("Dans Chrome, ouvrez le menu ⋮ puis « Installer l'application » (ou « Ajouter à l'écran d'accueil »).", "In Chrome, open the ⋮ menu, then “Install app” (or “Add to Home screen”).", "En Chrome, abra el menú ⋮ y luego «Instalar aplicación» (o «Añadir a pantalla de inicio»)."),
                ))
                .child(step(
                    t("Ordinateur", "Computer", "Ordenador"),
                    t("Dans Chrome ou Edge, cliquez sur l'icône d'installation au bout de la barre d'adresse.", "In Chrome or Edge, click the install icon at the end of the address bar.", "En Chrome o Edge, haga clic en el icono de instalación al final de la barra de direcciones."),
                )),
        )
}

/// La carte d'une exposition, sur l'accueil.
fn ex_card(i: usize, ex: &'static Exhibition) -> Element {
    let card = article().class("ex-card reveal").id(ex.slug).style(format!(
        "--accent:{};animation-delay:{}ms",
        rgb(ex.accent),
        40 * (i % 6)
    ));
    let card = match ex.cover() {
        Some(art) => card.child(
            a().href(u(&format!("/{}", ex.slug)))
                .class("ex-cover")
                .child(picture(ex, art, true)),
        ),
        None => card,
    };
    card.child(
        div()
            .class("ex-body")
            .child(p().class("kicker").text(format!(
                "{} · {}",
                movement_or_dates(ex),
                ex.artist_dates
            )))
            .child(h3().child(a().href(u(&format!("/{}", ex.slug))).text(ex.title)))
            .child(
                p().class("ex-artists")
                    .text(if ex.ephemeral { ex.kicker } else { ex.artist }),
            )
            .child(p().class("muted ex-intro").text(ex.introduction))
            .child(p().class("tag").text(counts(ex)))
            .child(
                div()
                    .class("row")
                    .child(
                        a().class("button")
                            .href(u(&format!("/{}/galerie", ex.slug)))
                            .text(t("Visite 3D", "3D tour", "Visita 3D")),
                    )
                    .child(
                        a().class("button ghost")
                            .href(u(&format!("/{}", ex.slug)))
                            .text(t("Découvrir", "Discover", "Descubrir")),
                    ),
            ),
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
                                        .href(u(&format!("/{}/galerie", ex.slug)))
                                        .text(t("Lancer la visite 3D", "Start the 3D tour", "Iniciar la visita 3D")),
                                )
                                .child(
                                    a().class("button ghost")
                                        .href(u(&format!("/{}/oeuvres", ex.slug)))
                                        .text(t("Voir les œuvres", "See the works", "Ver las obras")),
                                ),
                        ),
                )
                .child(div().class("hero-visual").child(island(
                    "showcase",
                    island_props(ex),
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
                            t("toiles exposées", "paintings on show", "cuadros expuestos")
                        } else {
                            t("œuvres exposées", "works on show", "obras expuestas")
                        })),
                )
                .child(
                    div()
                        .class("stat reveal")
                        .child(strong().text(ex.galleries().count().to_string()))
                        .child(span().text(t("salles d'exposition", "exhibition rooms", "salas de exposición"))),
                )
                .child(
                    div()
                        .class("stat reveal")
                        .child(strong().text(ex.years()))
                        .child(span().text(t("années de création", "years of creation", "años de creación"))),
                )
                .child(
                    div()
                        .class("stat reveal")
                        .child(strong().text(format!("{minutes} min")))
                        .child(span().text(t("de visite guidée", "of guided tour", "de visita guiada"))),
                ),
        )
        .child(building(ex))
        .child(
            section()
                .class("rooms")
                .child(h2().class("section-title").text(t("Le parcours", "The route", "El recorrido")))
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
                                        .text(match works.len() + statues.len() {
                                            1 => t("1 œuvre", "1 work", "1 obra").to_owned(),
                                            n => tf!("{} œuvres", "{} works", "{} obras", n),
                                        }),
                                )
                        })),
                ),
        )
        .child(
            section()
                .class("preview")
                .child(h2().class("section-title").text(t("Quelques chefs-d'œuvre", "Some masterpieces", "Algunas obras maestras")))
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
                    t("Les artistes", "The artists", "Los artistas")
                } else {
                    t("L'artiste", "The artist", "El artista")
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

/// Les paramètres des îlots 3D : l'exposition, et la langue si ce n'est pas le français
/// (`monet`, `monet@en`).
fn island_props(ex: &Exhibition) -> String {
    match lang() {
        Lang::Fr => ex.slug.to_owned(),
        other => format!("{}@{}", ex.slug, other.code()),
    }
}

/// « 12 œuvres · 2 sculptures · 3 salles · 1885–1890 ».
fn counts(ex: &Exhibition) -> String {
    let mut parts = vec![tf!("{} œuvres", "{} works", "{} obras", ex.artworks.len())];
    if !ex.sculptures.is_empty() {
        parts.push(match ex.sculptures.len() {
            1 => t("1 sculpture", "1 sculpture", "1 escultura").to_owned(),
            n => tf!("{} sculptures", "{} sculptures", "{} esculturas", n),
        });
    }
    parts.push(tf!("{} salles", "{} rooms", "{} salas", ex.galleries().count()));
    parts.push(ex.years());
    parts.join(" · ")
}

/// Les autres expositions du musée, en bas de la présentation d'une exposition.
fn others(ex: &Exhibition) -> Element {
    // Les voisines dans l'histoire de l'art : trois avant, trois après.
    let here = exhibitions()
        .iter()
        .position(|e| e.slug == ex.slug)
        .unwrap_or(0);
    let others: Vec<&Exhibition> = exhibitions()
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
        .child(h2().class("section-title").text(t("Avant, après", "Before, after", "Antes, después")))
        .child(
            div()
                .class("others-grid")
                .children(others.into_iter().map(|o| {
                    let link = a()
                        .href(u(&format!("/{}", o.slug)))
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
                .child(p().class("kicker").text(tf!("Visite · {}", "Tour · {}", "Visita · {}", ex.short_name)))
                .child(h1().text(t("Le musée en 3D", "The museum in 3D", "El museo en 3D")))
                .child(p().class("lead").text(t(
                    "En visite guidée, la caméra traverse les salles et s'arrête devant chaque œuvre, en boucle. \
                     En visite libre, on marche où l'on veut : ZQSD ou WASD et la souris sur ordinateur, les deux \
                     joysticks sur téléphone. Le plan permet de se rendre d'un clic dans une salle ou devant une œuvre, \
                     et l'éditeur de déplacer les œuvres. La visite n'est pas figée : une aile s'ajoute au musée avec \
                     les œuvres liées des collections du monde, autant que vous en choisissez ci-dessous.",
                    "On the guided tour, the camera moves through the rooms and stops in front of each work, in a loop. \
                     On the free tour, you walk wherever you like: WASD and the mouse on a computer, the two joysticks \
                     on a phone. The plan takes you to a room or a work in one click, and the editor lets you move the works. \
                     The tour is not fixed: a wing is added to the museum with related works from the world collections, \
                     as many as you choose below.",
                    "En la visita guiada, la cámara recorre las salas y se detiene ante cada obra, en bucle. \
                     En la visita libre, se camina por donde se quiera: WASD y el ratón en el ordenador, los dos joysticks \
                     en el teléfono. El plano lleva con un clic a una sala o ante una obra, y el editor permite mover las obras. \
                     La visita no es fija: se añade un ala al museo con las obras relacionadas de las colecciones del mundo, \
                     tantas como elija abajo.",
                ))),
        )
        // La visite est dynamique : le visiteur choisit combien d'œuvres liées des collections
        // du monde sont accrochées dans l'aile ajoutée au musée (src/aile.rs).
        .child(if ex.ephemeral {
            nav()
        } else {
            let own = ex.artworks.len() + ex.sculptures.len();
            nav()
                .class("visit-size")
                .attr("aria-label", t("Le nombre d'œuvres de la visite", "Number of works in the tour", "Número de obras de la visita"))
                .child(span().class("visit-size-label").text(t(
                    "Œuvres dans la visite :",
                    "Works in the tour:",
                    "Obras en la visita:",
                )))
                .children([0usize, 24, 48, 96, 120].into_iter().map(|n| {
                    a().class("chip")
                        .href(u(&format!("/{}/galerie?oeuvres={n}", ex.slug)))
                        .attr("data-oeuvres", n.to_string())
                        .text(if n == 0 {
                            tf!("{} (l'accrochage d'origine)", "{} (the original hang)", "{} (el montaje original)", own)
                        } else {
                            tf!("{} + {} des collections", "{} + {} from the collections", "{} + {} de las colecciones", own, n)
                        })
                }))
        })
        .child(div().class("viewer").child(island("gallery", island_props(ex), gallery_scene)))
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
                        li().child(strong().text(s.title)).child(span().class("muted").text(tf!(" · {}, sculpture", " · {}, sculpture", " · {}, escultura", s.year)))
                    })),
                )
        })));
    layout(
        tf!("Galerie 3D · {}", "3D gallery · {}", "Galería 3D · {}", ex.short_name),
        tf!("Visite guidée en 3D de l'exposition {}.", "Guided 3D tour of the exhibition {}.", "Visita guiada en 3D de la exposición {}.", ex.title),
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
                .child(p().class("kicker").text(tf!("Catalogue · {}", "Catalogue · {}", "Catálogo · {}", ex.short_name)))
                .child(h1().text(t("Les œuvres", "The works", "Las obras")))
                .child(p().class("lead").text(tf!(
                    "{} œuvres de {} ({}), dans l'ordre de l'accrochage, avec leur cartel et leur place dans la galerie. \
                     Chaque œuvre a sa page : ce que l'on voit, ce qu'elle évoque, son histoire, la manière de l'artiste.",
                    "{} works by {} ({}), in hanging order, with their labels and their place in the gallery. \
                     Each work has its own page: what you see, what it evokes, its history, the artist's technique.",
                    "{} obras de {} ({}), en el orden del montaje, con su cartela y su lugar en la galería. \
                     Cada obra tiene su página: lo que se ve, lo que evoca, su historia, la técnica del artista.",
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
                .child(h2().class("section-title").text(t("Les sculptures", "The sculptures", "Las esculturas")))
                .child(div().class("art-grid").children(ex.sculptures.iter().enumerate().map(|(i, s)| sculpture_card(ex, s, i))))
        });
    layout(
        tf!("Œuvres · {}", "Works · {}", "Obras · {}", ex.short_name),
        tf!(
            "Le catalogue de l'exposition {} : chaque œuvre, son cartel et sa place dans la galerie.",
            "The catalogue of the exhibition {}: each work, its label and its place in the gallery.",
            "El catálogo de la exposición {}: cada obra, su cartela y su lugar en la galería.",
            ex.title
        ),
        &here,
        Some(ex.accent),
        content,
    )
}

/// Le plan du musée, dessiné en SVG (en mètres) : les salles à la couleur de leurs
/// murs, leurs portes, les tableaux le long des murs, les sculptures, et l'entrée.
fn plan_svg(ex: &'static Exhibition) -> Element {
    let (x0, x1, z0, z1) = ex.rooms.iter().fold(
        (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
        |(a, b, c, d), r| {
            (
                a.min(r.x - r.width / 2.0),
                b.max(r.x + r.width / 2.0),
                c.min(r.z - r.depth / 2.0),
                d.max(r.z + r.depth / 2.0),
            )
        },
    );
    let pad = 2.5;
    let view = format!(
        "{:.2} {:.2} {:.2} {:.2}",
        x0 - pad,
        z0 - pad,
        x1 - x0 + 2.0 * pad,
        z1 - z0 + 2.0 * pad
    );
    let number: std::collections::HashMap<usize, usize> = ex
        .galleries()
        .enumerate()
        .map(|(n, (i, _))| (i, n + 1))
        .collect();
    let mut plan = svg()
        .class("plan")
        .attr("viewBox", view)
        .attr("role", "img")
        .attr(
            "aria-label",
            tf!("Le plan du musée : {} salles", "The museum plan: {} rooms", "El plano del museo: {} salas", number.len()),
        );
    for r in ex.rooms {
        plan = plan.child(
            rect()
                .class(if r.corridor {
                    "plan-room corridor"
                } else {
                    "plan-room"
                })
                .attr("x", format!("{:.2}", r.x - r.width / 2.0))
                .attr("y", format!("{:.2}", r.z - r.depth / 2.0))
                .attr("width", format!("{:.2}", r.width))
                .attr("height", format!("{:.2}", r.depth))
                .attr("fill", rgb(r.walls)),
        );
    }
    // Les portes : une ouverture dans le trait du mur.
    for r in ex.rooms {
        for d in r.doors {
            let (ax, az) = r.point_on(d.side, d.position - d.width / 2.0);
            let (bx, bz) = r.point_on(d.side, d.position + d.width / 2.0);
            plan = plan.child(
                line()
                    .class("plan-door")
                    .attr("x1", format!("{ax:.2}"))
                    .attr("y1", format!("{az:.2}"))
                    .attr("x2", format!("{bx:.2}"))
                    .attr("y2", format!("{bz:.2}")),
            );
        }
    }
    // Les tableaux, le long de leur mur, un peu en retrait.
    for art in ex.artworks {
        let r = &ex.rooms[art.hook.room];
        let (ix, iz) = art.hook.side.inward();
        let (ax, az) = r.point_on(art.hook.side, art.hook.position - art.width / 2.0);
        let (bx, bz) = r.point_on(art.hook.side, art.hook.position + art.width / 2.0);
        plan = plan.child(
            line()
                .class("plan-art")
                .attr("x1", format!("{:.2}", ax + ix * 0.35))
                .attr("y1", format!("{:.2}", az + iz * 0.35))
                .attr("x2", format!("{:.2}", bx + ix * 0.35))
                .attr("y2", format!("{:.2}", bz + iz * 0.35)),
        );
    }
    for s in ex.sculptures {
        plan = plan.child(
            circle()
                .class("plan-sculpture")
                .attr("cx", format!("{:.2}", s.x))
                .attr("cy", format!("{:.2}", s.z))
                .attr("r", format!("{:.2}", s.radius.max(0.5))),
        );
    }
    // Les numéros des salles, ceux du parcours.
    for (i, r) in ex.rooms.iter().enumerate() {
        if let Some(n) = number.get(&i) {
            let size = (r.width.min(r.depth) * 0.45).clamp(1.2, 3.2);
            plan = plan.child(
                text_svg()
                    .class("plan-number")
                    .attr("x", format!("{:.2}", r.x))
                    .attr("y", format!("{:.2}", r.z + size * 0.35))
                    .attr("font-size", format!("{size:.2}"))
                    .text(n.to_string()),
            );
        }
    }
    if let Some((_, x, z, side)) = arteon::museum::entrance(ex) {
        let (ix, iz) = side.inward();
        let (ox, oz) = (x - ix * 1.6, z - iz * 1.6);
        plan = plan.child(
            text_svg()
                .class("plan-entrance")
                .attr("x", format!("{ox:.2}"))
                .attr("y", format!("{:.2}", oz + 0.5))
                .attr("font-size", "1.3")
                .text(t("▲ Entrée", "▲ Entrance", "▲ Entrada")),
        );
    }
    plan
}

/// Le bâtiment de l'exposition : le vrai musée dont il s'inspire, ce qu'il en garde,
/// et le plan des salles.
fn building(ex: &'static Exhibition) -> Element {
    let rooms = ol()
        .class("plan-legend")
        .children(ex.galleries().map(|(i, room)| {
            let works = ex.works_in_room(i).len() + ex.sculptures_in_room(i).len();
            li().child(strong().text(room.name)).child(
                span().class("muted small").text(
                    decimal(tf!(
                        " · {:.0} × {:.0} m, {:.1} m sous plafond · {} œuvre(s)",
                        " · {:.0} × {:.0} m, {:.1} m ceiling · {} work(s)",
                        " · {:.0} × {:.0} m, {:.1} m de altura · {} obra(s)",
                        room.width, room.depth, room.height, works
                    )),
                ),
            )
        }));
    let text = match ex.museum {
        Some(m) => {
            let facts: Vec<&str> = [m.city, m.architect, m.built]
                .into_iter()
                .filter(|t| !t.is_empty())
                .collect();
            let card = div()
                .class("building-text")
                .child(p().class("kicker").text(t("Inspiré de", "Inspired by", "Inspirado en")))
                .child(h3().text(m.name));
            let card = if facts.is_empty() {
                card
            } else {
                card.child(p().class("muted").text(facts.join(" · ")))
            };
            let card = if m.presentation.is_empty() {
                card
            } else {
                card.child(p().text(m.presentation))
            };
            if m.here.is_empty() {
                card
            } else {
                card.child(h4().text(t("Dans ARTEON", "In ARTEON", "En ARTEON"))).child(p().text(m.here))
            }
        }
        None => div().class("building-text").child(p().text(tf!(
            "Un bâtiment de {} salles, dessiné pour cette exposition.",
            "A building of {} rooms, designed for this exhibition.",
            "Un edificio de {} salas, diseñado para esta exposición.",
            ex.galleries().count()
        ))),
    };
    section()
        .class("building")
        .id("batiment")
        .child(h2().class("section-title").text(t("Le bâtiment", "The building", "El edificio")))
        .child(
            div().class("building-grid").child(text).child(
                figure()
                    .class("building-plan reveal")
                    .child(plan_svg(ex))
                    .child(figcaption().child(rooms)),
            ),
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
            .child(h4().text(t("Sa manière", "Technique", "Su estilo")))
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
            decimal(tf!(
                "{} · sculpture de {:.2} m, sur un socle",
                "{} · sculpture, {:.2} m, on a plinth",
                "{} · escultura de {:.2} m, sobre un pedestal",
                ex.rooms[s.room].name, s.height
            )),
        ),
    };
    let image = match work {
        Work::Art(art) => a()
            .href(art.image_url())
            .class("frame work-frame")
            .attr("title", t("Ouvrir la reproduction en grand", "Open the full-size reproduction", "Abrir la reproducción en grande"))
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
        .child(fact(t("Artiste", "Artist", "Artista"), artist.to_owned()))
        .child(fact(t("Date", "Date", "Fecha"), year.to_owned()));
    if !technique.is_empty() {
        facts = facts.child(fact(t("Technique", "Medium", "Técnica"), technique.to_owned()));
    }
    if !collection.is_empty() {
        facts = facts.child(fact(t("Collection", "Collection", "Colección"), collection.to_owned()));
    }
    facts = facts.child(fact(t("Dans le musée", "In the museum", "En el museo"), place));
    if let Work::Art(art) = work {
        facts = facts.child(
            div()
                .class("fact")
                .child(dt().text(t("Reproduction", "Reproduction", "Reproducción")))
                .child(dd().child(credit_line(&art.credit))),
        );
    }
    let cartel = div()
        .class("work-cartel")
        .child(p().class("lead").text(note))
        .child(facts)
        .child(
            div()
                .class("row")
                .child(
                    a().class("button")
                        .href(u(&format!("/{}/galerie#{}", ex.slug, work.slug())))
                        .text(t("La voir dans le musée 3D", "See it in the 3D museum", "Verla en el museo 3D")),
                )
                .child(
                    a().class("button ghost")
                        .href(u(&catalogue))
                        .text(t("Toutes les œuvres", "All the works", "Todas las obras")),
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
                .child(h2().text(t("À regarder de près", "Look closely", "Para mirar de cerca")))
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
                .child(strong().text(t("Le saviez-vous ? ", "Did you know? ", "¿Sabía que…? ")))
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
                            a().href(u(&catalogue))
                                .text(tf!("Œuvres · {}", "Works · {}", "Obras · {}", ex.short_name)),
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
                .child(h2().class("section-title").text(t("L'artiste", "The artist", "El artista")))
                .child(artist_card(about)),
            None => section(),
        })
        // Toutes ses œuvres : celles des expositions et celles des collections du monde.
        .child(
            p().class("work-catalogue").child(
                a().class("button ghost")
                    .href(u(&format!("/explorer/#/artiste/{}", encode_fragment(artist))))
                    .text(tf!("Le catalogue complet : {} →", "The complete catalogue: {} →", "El catálogo completo: {} →", artist)),
            ),
        )
        .child(
            nav()
                .class("work-nav")
                .attr("aria-label", t("Œuvre précédente et suivante", "Previous and next work", "Obra anterior y siguiente"))
                .child(neighbour(previous, t("← Précédente", "← Previous", "← Anterior"), "prev"))
                .child(neighbour(next, t("Suivante →", "Next →", "Siguiente →"), "next")),
        );
    let summary = if notice.description.is_empty() {
        note
    } else {
        notice.description
    };
    layout(
        format!("{} · {}", work.title(), artist),
        summary.chars().take(155).collect::<String>(),
        &format!("{catalogue}/{}", work.slug()),
        Some(ex.accent),
        content,
    )
}

// ─── Les crédits ─────────────────────────────────────────────────────────────

async fn credits(_req: Request) -> Page {
    let tool =
        |name: &'static str, who: &'static str, licence: &'static str, href: &'static str| {
            tr().child(
                td().child(
                    a().href(href)
                        .attr("target", "_blank")
                        .attr("rel", "noopener")
                        .text(name),
                ),
            )
            .child(td().text(who))
            .child(td().text(licence))
        };
    let tools = table()
        .class("credits-table")
        .child(
            thead().child(
                tr().child(th().text(t("Outil", "Tool", "Herramienta")))
                    .child(th().text(t("Auteur", "Author", "Autor")))
                    .child(th().text(t("Licence", "Licence", "Licencia"))),
            ),
        )
        .child(
            tbody()
                .child(tool(
                    "Paquets, Active, Vitesse, Securite, Memoire",
                    AUTHOR,
                    t("Propriétaire, utilisable comme dépendance", "Proprietary, usable as a dependency", "Propietaria, utilizable como dependencia"),
                    "https://github.com/maxlestage/Paquets",
                ))
                .child(tool(
                    "egui et eframe",
                    t("Emil Ernerfeldt et contributeurs", "Emil Ernerfeldt and contributors", "Emil Ernerfeldt y colaboradores"),
                    t("MIT ou Apache 2.0", "MIT or Apache 2.0", "MIT o Apache 2.0"),
                    "https://github.com/emilk/egui",
                ))
                .child(tool(
                    t("Scans 3D des sculptures", "3D scans of the sculptures", "Escaneos 3D de las esculturas"),
                    "Smithsonian Institution",
                    "CC0",
                    "https://3d.si.edu/",
                ))
                .child(tool(
                    t("Reproductions des œuvres", "Reproductions of the works", "Reproducciones de las obras"),
                    t("Wikimedia Commons et les musées en accès libre", "Wikimedia Commons and open-access museums", "Wikimedia Commons y los museos de acceso abierto"),
                    t("Domaine public, CC0 ou licences libres (ci-dessous)", "Public domain, CC0 or free licences (below)", "Dominio público, CC0 o licencias libres (abajo)"),
                    "https://commons.wikimedia.org/",
                )),
        );
    let attributed: usize = EXHIBITIONS
        .iter()
        .flat_map(|e| e.artworks.iter())
        .filter(|a| a.credit.requires_attribution())
        .count();
    let per_exhibition = exhibitions().iter().map(|ex| {
        let rows = ex.artworks.iter().map(move |art| {
            tr().child(td().child(a().href(work_url(ex, art.slug)).text(art.title)))
                .child(td().text(art.artist_in(ex)))
                .child(td().child(credit_line(&art.credit)))
        });
        let rows = rows.chain(ex.sculptures.iter().map(move |s| {
            tr().child(td().child(a().href(work_url(ex, s.slug)).text(s.title)))
                .child(td().text(s.artist))
                .child(td().text(t("Scan 3D · Smithsonian Institution · CC0", "3D scan · Smithsonian Institution · CC0", "Escaneo 3D · Smithsonian Institution · CC0")))
        }));
        details()
            .class("credits-block")
            .child(
                summary()
                    .child(strong().text(ex.title))
                    .child(span().class("muted").text(tf!(
                        " · {} œuvres",
                        " · {} works",
                        " · {} obras",
                        ex.artworks.len() + ex.sculptures.len()
                    ))),
            )
            .child(
                div().class("table-wrap").child(
                    table()
                        .class("credits-table")
                        .child(
                            thead().child(
                                tr().child(th().text(t("Œuvre", "Work", "Obra")))
                                    .child(th().text(t("Artiste", "Artist", "Artista")))
                                    .child(th().text(t("Reproduction", "Reproduction", "Reproducción"))),
                            ),
                        )
                        .child(tbody().children(rows)),
                ),
            )
    });
    let content = div()
        .child(
            section()
                .class("page-intro")
                .child(p().class("kicker").text(t("Crédits", "Credits", "Créditos")))
                .child(h1().text(t("Crédits et licences", "Credits and licences", "Créditos y licencias")))
                .child(p().class("lead").text(tf!(
                    "ARTEON est conçu et développé par {}, en Rust, avec Paquets. Les œuvres exposées sont dans le \
                     domaine public : leurs artistes sont morts en 1955 ou avant, et elles datent d'avant 1931. Quand la \
                     photographie d'une œuvre est sous licence libre (une sculpture ou un objet photographié par un \
                     particulier), son auteur et sa licence sont cités ci-dessous et sur la page de l'œuvre — \
                     {} image(s) sont dans ce cas. Dans le musée 3D, les images sont réinterprétées en mosaïque.",
                    "ARTEON is designed and developed by {}, in Rust, with Paquets. The works on show are in the public \
                     domain: their artists died in 1955 or earlier, and they date from before 1931. When the photograph \
                     of a work is under a free licence (a sculpture or an object photographed by an individual), its \
                     author and licence are credited below and on the work's page — {} image(s) are in this case. In the \
                     3D museum, the images are reinterpreted as mosaics.",
                    "ARTEON está diseñado y desarrollado por {}, en Rust, con Paquets. Las obras expuestas son de dominio \
                     público: sus artistas murieron en 1955 o antes, y son anteriores a 1931. Cuando la fotografía de una \
                     obra tiene licencia libre (una escultura o un objeto fotografiado por un particular), su autor y su \
                     licencia se citan abajo y en la página de la obra — {} imagen(es) están en este caso. En el museo 3D, \
                     las imágenes se reinterpretan como mosaicos.",
                    AUTHOR,
                    attributed
                ))),
        )
        .child(h2().class("section-title").text(t("Mentions légales et vie privée", "Legal notice and privacy", "Aviso legal y privacidad")))
        .child(
            div()
                .class("credits-legal")
                .child(h3().text(t("Les œuvres encore protégées", "Works still under copyright", "Obras aún protegidas")))
                .child(p().text(t(
                    "Les œuvres d'artistes morts après 1955, ou créées après 1930, sont encore protégées par le droit \
                     d'auteur dans une partie du monde : ARTEON ne les reproduit jamais. Elles peuvent être citées dans les \
                     textes, mais aucune image n'en est montrée, ni sur le site, ni dans le musée 3D, ni dans l'application. \
                     Les expositions consacrées à l'art du XXᵉ siècle montrent ainsi les pionniers et les ancêtres, libres \
                     de droits, des mouvements récents.",
                    "Works by artists who died after 1955, or created after 1930, are still protected by copyright in part \
                     of the world: ARTEON never reproduces them. They may be mentioned in the texts, but no image of them is \
                     shown, neither on the site, nor in the 3D museum, nor in the app. The exhibitions devoted to 20th-century \
                     art thus show the copyright-free pioneers and forerunners of recent movements.",
                    "Las obras de artistas fallecidos después de 1955, o creadas después de 1930, siguen protegidas por \
                     derechos de autor en parte del mundo: ARTEON nunca las reproduce. Pueden citarse en los textos, pero \
                     no se muestra ninguna imagen de ellas, ni en el sitio, ni en el museo 3D, ni en la aplicación. Las \
                     exposiciones dedicadas al arte del siglo XX muestran así a los pioneros y precursores, libres de \
                     derechos, de los movimientos recientes.",
                )))
                .child(h3().text(t("Les musées réels", "The real museums", "Los museos reales")))
                .child(p().text(t(
                    "ARTEON est un projet indépendant. Ses bâtiments s'inspirent de musées réels, dont les noms ne sont \
                     cités que pour dire ce qui a inspiré chaque plan : ARTEON n'est ni affilié à ces musées, ni approuvé \
                     ni soutenu par eux, et leurs noms restent la propriété de leurs titulaires. Les plans sont des \
                     interprétations libres, pas des relevés d'architecture.",
                    "ARTEON is an independent project. Its buildings are inspired by real museums, whose names are \
                     mentioned only to say what inspired each plan: ARTEON is neither affiliated with these museums nor \
                     endorsed or supported by them, and their names remain the property of their owners. The plans are \
                     free interpretations, not architectural surveys.",
                    "ARTEON es un proyecto independiente. Sus edificios se inspiran en museos reales, cuyos nombres solo \
                     se citan para indicar qué inspiró cada plano: ARTEON no está afiliado a estos museos, ni aprobado ni \
                     respaldado por ellos, y sus nombres siguen siendo propiedad de sus titulares. Los planos son \
                     interpretaciones libres, no levantamientos arquitectónicos.",
                )))
                .child(h3().text(t("Les licences libres", "Free licences", "Las licencias libres")))
                .child(p().text(t(
                    "Une photographie sous licence CC BY ou CC BY-SA est toujours citée avec son auteur, sa licence et \
                     un lien vers sa source. Sa mosaïque dans le musée 3D est une adaptation de cette photographie : elle \
                     est partagée aux mêmes conditions, sous la même licence et avec le même crédit. Les photographies \
                     publiées en CC0 par les musées et les œuvres du domaine public peuvent être réutilisées librement.",
                    "A photograph under a CC BY or CC BY-SA licence is always credited with its author, its licence and \
                     a link to its source. Its mosaic in the 3D museum is an adaptation of that photograph: it is shared \
                     under the same terms, the same licence and with the same credit. Photographs published as CC0 by \
                     museums and public-domain works may be reused freely.",
                    "Una fotografía con licencia CC BY o CC BY-SA siempre se cita con su autor, su licencia y un enlace a \
                     su fuente. Su mosaico en el museo 3D es una adaptación de esa fotografía: se comparte en las mismas \
                     condiciones, con la misma licencia y el mismo crédito. Las fotografías publicadas en CC0 por los \
                     museos y las obras de dominio público pueden reutilizarse libremente.",
                )))
                .child(h3().text(t("Vie privée", "Privacy", "Privacidad")))
                .child(p().text(t(
                    "ARTEON n'utilise ni cookie, ni traceur, ni publicité, et ne demande aucun compte. Les pages des \
                     collections du monde affichent les images en grand depuis les serveurs des musées (Art Institute of \
                     Chicago, Cleveland Museum of Art, Metropolitan Museum of Art, National Gallery of Art), et le catalogue de \
                     chaque artiste interroge en direct ces musées, le Statens Museum for Kunst, le Minneapolis Institute \
                     of Art, Europeana et Wikidata (images de Wikimedia Commons) : votre \
                     navigateur s'y connecte alors directement, comme pour toute image venue d'un autre site. Le record \
                     du jeu « Qui l'a fait ? » reste dans votre navigateur. Pour aller plus vite, le navigateur garde aussi \
                     une copie compressée des données du musée (IndexedDB) : elle n'est jamais envoyée ailleurs et \
                     s'efface avec les données du site, dans les réglages du navigateur. L'application iOS passe par \
                     l'API ARTEON : elle garde une copie compressée de la visite en cours (SQLite), effacée quand on la quitte.",
                    "ARTEON uses no cookies, no trackers and no advertising, and asks for no account. The world collections \
                     pages show large images from the museums' servers (Art Institute of Chicago, Cleveland Museum of Art, \
                     Metropolitan Museum of Art, National Gallery of Art), and each artist's catalogue queries these museums, \
                     the Statens Museum for Kunst, the Minneapolis Institute of Art, Europeana and Wikidata (images from \
                     Wikimedia Commons) live: your browser then connects to them directly, as for any image from another \
                     site. The high score of the “Who made it?” game stays in your browser. To be faster, the browser also \
                     keeps a compressed copy of the museum's data (IndexedDB): it is never sent anywhere and is erased with \
                     the site's data, in the browser settings. The iOS app goes through the ARTEON API: it keeps a compressed \
                     copy of the current visit (SQLite), erased when you quit it.",
                    "ARTEON no utiliza cookies, ni rastreadores, ni publicidad, y no pide ninguna cuenta. Las páginas de las \
                     colecciones del mundo muestran las imágenes en grande desde los servidores de los museos (Art Institute \
                     of Chicago, Cleveland Museum of Art, Metropolitan Museum of Art, National Gallery of Art), y el catálogo \
                     de cada artista consulta en directo esos museos, el Statens Museum for Kunst, el Minneapolis Institute \
                     of Art, Europeana y Wikidata (imágenes de Wikimedia Commons): su navegador se conecta entonces \
                     directamente a ellos, como con cualquier imagen de otro sitio. El récord del juego «¿Quién lo hizo?» \
                     se queda en su navegador. Para ir más rápido, el navegador guarda también una copia comprimida de los \
                     datos del museo (IndexedDB): nunca se envía a ningún sitio y se borra con los datos del sitio, en los \
                     ajustes del navegador. La aplicación iOS pasa por la API de ARTEON: guarda una copia comprimida de la \
                     visita en curso (SQLite), que se borra al salir.",
                ))),
        )
        .child(h2().class("section-title").text(t("Le code et les outils", "Code and tools", "El código y las herramientas")))
        .child(div().class("table-wrap").child(tools))
        .child(h2().class("section-title").text(t("Les reproductions, exposition par exposition", "Reproductions, exhibition by exhibition", "Las reproducciones, exposición por exposición")))
        .children(per_exhibition);
    layout(
        t("Crédits et licences", "Credits and licences", "Créditos y licencias"),
        t(
            "Les crédits d'ARTEON : qui l'a fait, avec quels outils, et d'où viennent les reproductions des œuvres.",
            "ARTEON's credits: who made it, with which tools, and where the reproductions of the works come from.",
            "Los créditos de ARTEON: quién lo hizo, con qué herramientas y de dónde vienen las reproducciones de las obras.",
        ),
        "/credits",
        None,
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
                    None if door => chip.class("door").text(tf!("{} · porte", "{} · door", "{} · puerta", place)),
                    None => chip.class("free").text(tf!("{} · libre", "{} · free", "{} · libre", place)),
                }
            });
            tr().child(
                th().attr("scope", "row")
                    .text(format!("{} · {}", room.name, wall_name(side))),
            )
            .child(
                td().child(
                    span()
                        .class("muted small")
                        .text(decimal(format!("{:.1} m", length))),
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
                    tr().child(th().text(t("Salle · mur", "Room · wall", "Sala · muro")))
                        .child(th().text(t("Longueur", "Length", "Longitud")))
                        .child(th().text(t("Places, de gauche à droite face au mur", "Places, from left to right facing the wall", "Lugares, de izquierda a derecha frente al muro"))),
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
    let files = exhibitions()
        .iter()
        .map(|e| format!("expositions/{}.toml", e.slug))
        .collect::<Vec<_>>()
        .join(", ");
    let content = div()
        .child(
            section()
                .class("page-intro")
                .child(p().class("kicker").text(t("Modifier le musée", "Edit the museum", "Modificar el museo")))
                .child(h1().text(t("Ajouter ou modifier une œuvre", "Add or edit an artwork", "Añadir o modificar una obra")))
                .child(p().class("lead").text(tf!(
                    "Chaque exposition est décrite dans un seul fichier ({}) et ses images sont dans le dossier \
                     public/tableaux/<nom>/. Pas une ligne de code à écrire : le site se reconstruit et se republie \
                     tout seul après chaque modification.",
                    "Each exhibition is described in a single file ({}) and its images are in the folder \
                     public/tableaux/<name>/. Not a line of code to write: the site rebuilds and republishes \
                     itself after every change.",
                    "Cada exposición se describe en un solo archivo ({}) y sus imágenes están en la carpeta \
                     public/tableaux/<nombre>/. Ni una línea de código que escribir: el sitio se reconstruye y se \
                     vuelve a publicar solo después de cada cambio.",
                    files
                ))),
        )
        .child(
            div()
                .class("grid how")
                .child(step(
                    "01",
                    t("Déposer l'image", "Upload the image", "Subir la imagen"),
                    p().child(span().text(t("Une reproduction JPEG ou PNG de la toile (900 pixels de large suffisent), dans le dossier de l'exposition, par exemple ", "A JPEG or PNG reproduction of the painting (900 pixels wide is enough), in the exhibition's folder, for example ", "Una reproducción JPEG o PNG del cuadro (bastan 900 píxeles de ancho), en la carpeta de la exposición, por ejemplo ")))
                        .child(code().text("public/tableaux/monet/"))
                        .child(span().text(t(". Sur GitHub : ", ". On GitHub: ", ". En GitHub: ")))
                        .child(a().href(format!("{REPOSITORY}/tree/master/public/tableaux")).text(t("ouvrir le dossier, puis Add file → Upload files", "open the folder, then Add file → Upload files", "abrir la carpeta y luego Add file → Upload files")))
                        .child(span().text(".")),
                ))
                .child(step(
                    "02",
                    t("Décrire l'œuvre", "Describe the work", "Describir la obra"),
                    p().child(span().text(t("Copier un bloc ", "Copy an ", "Copiar un bloque ")))
                        .child(code().text("[[oeuvres]]"))
                        .child(span().text(t(" dans le fichier de l'exposition (", " block into the exhibition's file (", " en el archivo de la exposición (")))
                        .child(a().href(format!("{REPOSITORY}/tree/master/expositions")).text(t("dossier expositions/", "expositions/ folder", "carpeta expositions/")))
                        .child(span().text(t(") et changer ses valeurs : le titre, l'image, la salle, le mur, la place et la largeur en mètres.", ") and change its values: the title, image, room, wall, place and width in metres.", ") y cambiar sus valores: el título, la imagen, la sala, el muro, el lugar y el ancho en metros."))),
                ))
                .child(step(
                    "03",
                    t("Enregistrer", "Save", "Guardar"),
                    p().text(t(
                        "Valider la modification (« Commit changes ») sur la branche master. Quelques minutes plus \
                         tard, la toile est accrochée dans la galerie 3D, dans le catalogue et dans la visite guidée.",
                        "Commit the change (“Commit changes”) on the master branch. A few minutes later, the painting \
                         hangs in the 3D gallery, in the catalogue and on the guided tour.",
                        "Validar el cambio («Commit changes») en la rama master. Unos minutos después, el cuadro está \
                         colgado en la galería 3D, en el catálogo y en la visita guiada.",
                    )),
                )),
        )
        .child(h2().class("section-title").text(t("Déplacer une œuvre depuis le musée", "Move a work from inside the museum", "Mover una obra desde el museo")))
        .child(p().class("lead").text(t(
            "Dans la visite 3D, le bouton « Éditer » ouvre l'éditeur du musée : choisir une toile ou une sculpture \
             (ou « L'œuvre regardée »), puis sa salle, son mur et sa position avec les curseurs. La scène se met à jour \
             aussitôt, et l'éditeur dit si la place est libre. « Télécharger » donne le fichier de l'exposition modifié \
             (seules les lignes de placement changent), « Copier » le met dans le presse-papiers, et « Publier sur GitHub » \
             ouvre l'éditeur du fichier dans le dépôt : il reste à coller et à valider.",
            "In the 3D tour, the “Edit” button opens the museum editor: choose a painting or a sculpture (or “The work \
             in view”), then its room, its wall and its position with the sliders. The scene updates at once, and the \
             editor tells you whether the place is free. “Download” gives the modified exhibition file (only the \
             placement lines change), “Copy” puts it on the clipboard, and “Publish on GitHub” opens the file's editor \
             in the repository: all that remains is to paste and commit.",
            "En la visita 3D, el botón «Editar» abre el editor del museo: elegir un cuadro o una escultura (o «La obra \
             que se mira»), y luego su sala, su muro y su posición con los cursores. La escena se actualiza enseguida, \
             y el editor indica si el lugar está libre. «Descargar» da el archivo de la exposición modificado (solo \
             cambian las líneas de colocación), «Copiar» lo pone en el portapapeles y «Publicar en GitHub» abre el \
             editor del archivo en el repositorio: solo queda pegar y validar.",
        )))
        .child(h2().class("section-title").text(t("Un exemple", "An example", "Un ejemplo")))
        .child(pre().class("example").child(code().text(EXAMPLE)))
        .child(
            ul().class("rules")
                .child(li().child(strong().text("mur")).child(span().text(t(" : \"nord\", \"sud\", \"est\" ou \"ouest\" (le visiteur entre par le sud et avance vers le nord) ; \"gauche\", \"droite\" et \"fond\" marchent aussi.", ": \"nord\" (north), \"sud\" (south), \"est\" (east) or \"ouest\" (west) — visitors enter from the south and walk north; \"gauche\" (left), \"droite\" (right) and \"fond\" (back) also work.", ": \"nord\" (norte), \"sud\" (sur), \"est\" (este) u \"ouest\" (oeste) — el visitante entra por el sur y avanza hacia el norte; \"gauche\" (izquierda), \"droite\" (derecha) y \"fond\" (fondo) también funcionan."))))
                .child(li().child(strong().text("place")).child(span().text(t(" : 1, 2, 3… numérotées de gauche à droite quand on regarde le mur (une place tous les 3 m environ, voir le tableau ci-dessous). Ou bien position = 2.5 : le centre du tableau à 2,5 m du milieu du mur, vers la droite (négatif : vers la gauche).", ": 1, 2, 3… numbered from left to right when facing the wall (a place roughly every 3 m, see the table below). Or position = 2.5: the centre of the painting 2.5 m to the right of the middle of the wall (negative: to the left).", ": 1, 2, 3… numerados de izquierda a derecha mirando al muro (un lugar cada 3 m aproximadamente, ver la tabla de abajo). O bien position = 2.5: el centro del cuadro a 2,5 m del medio del muro, hacia la derecha (negativo: hacia la izquierda)."))))
                .child(li().child(strong().text("largeur")).child(span().text(t(" : en mètres ; la hauteur suit les proportions de l'image. Un grand tableau peut garder sa taille réelle si la salle est assez haute (la Ronde de nuit fait 4,5 m).", " (width): in metres; the height follows the image's proportions. A large painting can keep its real size if the room is tall enough (The Night Watch is 4.5 m wide).", " (ancho): en metros; la altura sigue las proporciones de la imagen. Un cuadro grande puede conservar su tamaño real si la sala es lo bastante alta (La ronda de noche mide 4,5 m)."))))
                .child(li().child(strong().text(t("Modifier ou retirer", "Edit or remove", "Modificar o retirar"))).child(span().text(t(" : changer les valeurs d'un bloc, ou le supprimer.", ": change a block's values, or delete it.", ": cambiar los valores de un bloque, o borrarlo."))))
                .child(li().child(strong().text(t("Une nouvelle exposition", "A new exhibition", "Una nueva exposición"))).child(span().text(t(" : copier un fichier de expositions/ sous un autre nom (par exemple renoir.toml : l'exposition sera à l'adresse /renoir), créer le dossier public/tableaux/renoir/ pour ses images, et changer les textes et les œuvres. Elle apparaît d'elle-même sur l'accueil et dans le menu.", ": copy a file from expositions/ under another name (for example renoir.toml: the exhibition will be at /renoir), create the folder public/tableaux/renoir/ for its images, and change the texts and the works. It appears by itself on the home page and in the menu.", ": copiar un archivo de expositions/ con otro nombre (por ejemplo renoir.toml: la exposición estará en /renoir), crear la carpeta public/tableaux/renoir/ para sus imágenes y cambiar los textos y las obras. Aparece sola en la portada y en el menú."))))
                .child(li().child(strong().text(t("Une erreur ?", "A mistake?", "¿Un error?"))).child(span().text(t(" La compilation s'arrête avec un message en français qui dit quoi corriger (image introuvable, place déjà prise, toiles qui se chevauchent, tableau sur une porte, salles qui se chevauchent, sculpture contre un mur…), et le site en ligne reste tel quel.", " The build stops with a message (in French) saying what to fix (image not found, place already taken, overlapping paintings, a painting over a door, overlapping rooms, a sculpture against a wall…), and the live site stays as it is.", " La compilación se detiene con un mensaje (en francés) que indica qué corregir (imagen no encontrada, lugar ya ocupado, cuadros que se solapan, cuadro sobre una puerta, salas que se solapan, escultura contra un muro…), y el sitio en línea se queda como está.")))),
        )
        .child(h2().class("section-title").text(t("Écrire la notice complète", "Write the full notice", "Escribir la ficha completa")))
        .child(p().class("lead").text(t(
            "Chaque œuvre a sa page (/<exposition>/oeuvres/<nom de l'image>) et son panneau « Tout savoir » dans le musée 3D. \
             Ils affichent les champs facultatifs ci-dessous : ce que l'on voit, ce qu'elle évoque, son histoire, la manière, \
             les détails à regarder, les thèmes et une anecdote, ainsi que la biographie de l'artiste.",
            "Each work has its page (/<exhibition>/oeuvres/<image name>) and its “All about it” panel in the 3D museum. \
             They show the optional fields below: what you see, what it evokes, its history, the technique, details to \
             look at, themes and an anecdote, as well as the artist's biography.",
            "Cada obra tiene su página (/<exposición>/oeuvres/<nombre de la imagen>) y su panel «Todo sobre la obra» en el \
             museo 3D. Muestran los campos opcionales de abajo: lo que se ve, lo que evoca, su historia, la técnica, los \
             detalles para mirar, los temas y una anécdota, así como la biografía del artista.",
        )))
        .child(pre().class("example").child(code().text(NOTICE)))
        .child(h2().class("section-title").text(t("Dessiner le plan du musée", "Draw the museum plan", "Dibujar el plano del museo")))
        .child(p().class("lead").text(t(
            "Sans plan, les salles s'enchaînent en enfilade, reliées par des couloirs. Avec un plan, chaque salle a sa place \
             (x vers l'est, z vers le sud, en mètres depuis le centre du musée), sa taille et ses portes : on dessine une \
             galerie d'honneur, une nef, un atrium… Une porte se déclare d'un seul côté : la salle voisine reçoit la sienne.",
            "Without a plan, the rooms follow one another in a row, linked by corridors. With a plan, each room has its \
             place (x to the east, z to the south, in metres from the museum's centre), its size and its doors: you can \
             draw a grand gallery, a nave, an atrium… A door is declared on one side only: the neighbouring room gets its own.",
            "Sin plano, las salas se suceden en fila, unidas por pasillos. Con un plano, cada sala tiene su lugar (x hacia \
             el este, z hacia el sur, en metros desde el centro del museo), su tamaño y sus puertas: se dibuja una galería \
             de honor, una nave, un atrio… Una puerta se declara en un solo lado: la sala vecina recibe la suya.",
        )))
        .child(pre().class("example").child(code().text(PLAN)))
        .child(h2().class("section-title").text(t("Présenter le bâtiment, choisir les cadres", "Present the building, choose the frames", "Presentar el edificio, elegir los marcos")))
        .child(p().class("lead").text(t(
            "Une section [musee] (nom, ville, architecte, construction, presentation, ici) présente le vrai musée dont \
             le bâtiment s'inspire, et ce qu'il lui emprunte : elle s'affiche dans « Le bâtiment », avec le plan des salles, \
             et dans le musée 3D. Le cadre des tableaux se choisit avec cadre = \"dore\" dans [exposition] (ou dans une \
             œuvre) : dore, baroque, ebene, bois, blanc, noir, estampe ou aucun.",
            "A [musee] section (nom, ville, architecte, construction, presentation, ici) presents the real museum that \
             inspires the building, and what it borrows from it: it appears in “The building”, with the room plan, and \
             in the 3D museum. The frame of the paintings is chosen with cadre = \"dore\" in [exposition] (or in a work): \
             dore, baroque, ebene, bois, blanc, noir, estampe or aucun.",
            "Una sección [musee] (nom, ville, architecte, construction, presentation, ici) presenta el museo real en que se \
             inspira el edificio, y lo que toma de él: aparece en «El edificio», con el plano de las salas, y en el museo \
             3D. El marco de los cuadros se elige con cadre = \"dore\" en [exposition] (o en una obra): dore, baroque, \
             ebene, bois, blanc, noir, estampe o aucun.",
        )))
        .child(h2().class("section-title").text(t("Ajouter une sculpture", "Add a sculpture", "Añadir una escultura")))
        .child(p().class("lead").text(t(
            "Déposer le modèle 3D (STL, OBJ ou glTF) dans le dossier modeles/ : il est simplifié à la compilation pour \
             rester léger, posé sur un socle, et une vignette est dessinée pour le catalogue. Des scans libres de droits \
             se trouvent par exemple au Smithsonian (3d.si.edu, licence CC0).",
            "Put the 3D model (STL, OBJ or glTF) in the modeles/ folder: it is simplified at build time to stay light, \
             set on a plinth, and a thumbnail is drawn for the catalogue. Copyright-free scans can be found, for example, \
             at the Smithsonian (3d.si.edu, CC0 licence).",
            "Colocar el modelo 3D (STL, OBJ o glTF) en la carpeta modeles/: se simplifica al compilar para que sea ligero, \
             se coloca sobre un pedestal y se dibuja una miniatura para el catálogo. Hay escaneos libres de derechos, por \
             ejemplo, en el Smithsonian (3d.si.edu, licencia CC0).",
        )))
        .child(pre().class("example").child(code().text(SCULPTURE)))
        .child(h2().class("section-title").text(t("Les emplacements", "The places", "Los lugares")))
        .children(exhibitions().iter().map(|ex| {
            div()
                .class("slots-block")
                .child(h3().text(tf!("{} · expositions/{}.toml · images dans public/tableaux/{}/", "{} · expositions/{}.toml · images in public/tableaux/{}/", "{} · expositions/{}.toml · imágenes en public/tableaux/{}/", ex.short_name, ex.slug, ex.slug)))
                .child(slots_table(ex))
        }));
    layout(
        t("Ajouter une œuvre", "Add an artwork", "Añadir una obra"),
        t(
            "Comment ajouter, déplacer ou modifier une œuvre, ou créer une exposition dans ARTEON : une image et quelques lignes dans un fichier.",
            "How to add, move or edit a work, or create an exhibition in ARTEON: an image and a few lines in a file.",
            "Cómo añadir, mover o modificar una obra, o crear una exposición en ARTEON: una imagen y unas líneas en un archivo.",
        ),
        "/ajouter",
        None,
        content,
    )
}

async fn not_found(req: Request) -> Page {
    let path = req.path();
    set_lang(Lang::ALL.into_iter().find(|l| !l.prefix().is_empty() && (path == l.prefix() || path.starts_with(&format!("{}/", l.prefix())))).unwrap_or(Lang::Fr));
    let content = section()
        .class("page-intro")
        .child(p().class("kicker").text(t("Erreur 404", "Error 404", "Error 404")))
        .child(h1().text(t("Cette salle n'existe pas", "This room does not exist", "Esta sala no existe")))
        .child(
            p().class("lead")
                .text(t("La page demandée est introuvable. Le reste du musée vous attend.", "The page you asked for cannot be found. The rest of the museum awaits you.", "La página solicitada no existe. El resto del museo le espera.")),
        )
        .child(
            div()
                .class("row")
                .child(a().class("button").href(u("/")).text(t("Retour à l'accueil", "Back to the home page", "Volver al inicio")))
                .child(
                    a().class("button ghost")
                        .href(u("/#expositions"))
                        .text(t("Voir les expositions", "See the exhibitions", "Ver las exposiciones")),
                ),
        );
    layout(
        t("Page introuvable", "Page not found", "Página no encontrada"),
        t("Cette page d'ARTEON n'existe pas.", "This ARTEON page does not exist.", "Esta página de ARTEON no existe."),
        "",
        None,
        content,
    )
    .status(404)
}

// ─── L'application ───────────────────────────────────────────────────────────

/// Toutes les pages du site : l'accueil, le guide, et trois pages par exposition.
pub(crate) fn routes() -> Vec<String> {
    let mut list = Vec::new();
    for l in Lang::ALL {
        let p = l.prefix();
        list.push(if p.is_empty() { "/".to_owned() } else { p.to_owned() });
        list.push(format!("{p}/ajouter"));
        list.push(format!("{p}/credits"));
        for ex in l.exhibitions() {
            list.push(format!("{p}/{}", ex.slug));
            list.push(format!("{p}/{}/galerie", ex.slug));
            list.push(format!("{p}/{}/oeuvres", ex.slug));
            for work in ex.works() {
                list.push(format!("{p}/{}/oeuvres/{}", ex.slug, work.slug()));
            }
        }
    }
    list
}

/// Une page dans une langue : la langue est fixée juste avant de la rendre (d'un trait).
macro_rules! in_lang {
    ($l:expr, $body:expr) => {{
        let l = $l;
        async move {
            set_lang(l);
            $body.await
        }
    }};
}

/// L'application : le même serveur pour `paquets dev` et pour l'export statique.
fn app() -> App {
    let mut app = App::new();
    // L'application installable (PWA) : le manifeste, les icônes et le service worker, qui
    // garde les pages visitées, les images et les îlots 3D pour la visite hors ligne.
    app.pwa(
        Pwa::new("ARTEON")
            .short_name("ARTEON")
            .description(
                "Un musée virtuel en 3D : une cinquantaine d'expositions, de l'Antiquité au XXᵉ siècle, \
                 en visite guidée ou libre, avec une notice complète pour chaque œuvre.",
            )
            .id("/")
            .start_url("/")
            .scope("/")
            .display("standalone")
            .orientation("any")
            .lang("fr")
            .theme_color("#1d3f8f")
            .background_color("#0f1a3a")
            .categories(["education", "entertainment"])
            .icon("/icons/icon-192.png", "192x192", "image/png")
            .icon("/icons/icon-512.png", "512x512", "image/png")
            .maskable_icon("/icons/icon-maskable-512.png", "512x512", "image/png"),
    );
    app.get("/style.css", |_req: Request| async {
        Response::new()
            .content_type("text/css; charset=utf-8")
            .header("cache-control", "public, max-age=31536000, immutable")
            .send(style::CSS.to_owned())
    });
    for l in Lang::ALL {
        let p = l.prefix();
        app.get(if p.is_empty() { "/" } else { p }, move |req: Request| in_lang!(l, home(req)));
        app.get(&format!("{p}/ajouter"), move |req: Request| in_lang!(l, add_artwork(req)));
        app.get(&format!("{p}/credits"), move |req: Request| in_lang!(l, credits(req)));
        // Le catalogue complet, pour l'application iPhone et iPad : un par langue.
        let catalogue_json = std::sync::Arc::new({
            set_lang(l);
            catalogue::catalogue_json()
        });
        app.get(&format!("{p}/catalogue.json"), move |_req: Request| {
            let body = catalogue_json.clone();
            async move {
                Response::new()
                    .content_type("application/json; charset=utf-8")
                    .header("access-control-allow-origin", "*")
                    .send(body.as_str().to_owned())
            }
        });
        for ex in l.exhibitions() {
            app.get(&format!("{p}/{}", ex.slug), move |req: Request| {
                in_lang!(l, exhibition_home(req, ex))
            });
            app.get(&format!("{p}/{}/galerie", ex.slug), move |req: Request| {
                in_lang!(l, gallery_page(req, ex))
            });
            app.get(&format!("{p}/{}/oeuvres", ex.slug), move |req: Request| {
                in_lang!(l, catalogue(req, ex))
            });
            for work in ex.works() {
                app.get(
                    &format!("{p}/{}/oeuvres/{}", ex.slug, work.slug()),
                    move |req: Request| in_lang!(l, work_page(req, ex, work)),
                );
            }
        }
    }
    set_lang(Lang::Fr);
    // Les fichiers de données des expositions (mosaïques, sculptures, vignettes).
    // Sur Heroku, la compilation a lieu ailleurs : heroku-build.sh copie les données dans
    // donnees/ et le Procfile l'indique par ARTEON_DONNEES.
    let data =
        std::env::var("ARTEON_DONNEES").unwrap_or_else(|_| arteon::artworks::DATA_DIR.to_owned());
    app.static_dir("/donnees", &data);
    // L'API : toutes les œuvres d'un artiste, en direct des musées (src/api.rs).
    api::routes(&mut app);
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
        // `cargo run --release -- api <dossier> <noms.txt> [minutes] [fils]` : l'API publiée en
        // fichiers (le catalogue complet de chaque artiste), pour GitHub Pages. Le cache des
        // réponses est dans ARTEON_API_CACHE (target/api-cache par défaut).
        Some("api") => {
            let dir = std::path::PathBuf::from(args.next().unwrap_or_else(|| "dist/api".into()));
            let noms_txt = args.next().unwrap_or_else(|| "noms.txt".into());
            let minutes = args.next().and_then(|m| m.parse().ok()).unwrap_or(25);
            let fils = args.next().and_then(|m| m.parse().ok()).unwrap_or(4);
            let cache = std::path::PathBuf::from(
                std::env::var("ARTEON_API_CACHE").unwrap_or_else(|_| "target/api-cache".into()),
            );
            let noms: Vec<String> = std::fs::read_to_string(&noms_txt)?.lines().map(str::to_owned).collect();
            let (publies, calcules) = api::publier(&dir, &noms, minutes, Some(&cache), fils)?;
            println!("📦 API statique : {publies} artistes publiés ({calcules} recalculés) dans {}", dir.display());
            Ok(())
        }
        Some(other) => {
            eprintln!("commande inconnue {other:?} : lancer sans argument, `export <dossier>` ou `api <dossier> <noms.txt>`");
            std::process::exit(2);
        }
        None => app().run(),
    }
}
