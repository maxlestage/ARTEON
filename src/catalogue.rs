//! Le catalogue complet en JSON (`/catalogue.json`) : toutes les expositions, leurs salles,
//! leurs artistes et leurs œuvres avec leur notice, pour les applications (l'application
//! iPad en Flutter, dans `flutter/`). Les images sont des chemins relatifs au site
//! (`tableaux/van-gogh/iris.jpg`), à résoudre contre son adresse : le même catalogue sert
//! sur GitHub Pages (`/ARTEON/`) et sur Heroku (`/`).
//!
//! Le plan de chaque bâtiment y est aussi, en mètres (x vers l'est, z vers le sud) : les
//! salles (`x`, `z`, `width`, `depth`, couleur des murs, numéro dans le parcours), leurs
//! portes (segments), la place de chaque œuvre (`spot` : le segment d'un tableau le long
//! de son mur, ou le cercle du socle d'une sculpture) et l'entrée. L'application iOS le
//! dessine.

use paquets::serde_json::{Value, json};

use arteon::artworks::{Artist, Artwork, Credit, EXHIBITIONS, Exhibition, Notice, Sculpture};

/// Arrondi au centimètre (le JSON reste lisible et léger).
fn cm(v: f32) -> f64 {
    (f64::from(v) * 100.0).round() / 100.0
}

/// La version du format : l'application la vérifie.
pub const VERSION: u32 = 1;

fn hex([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn notice(n: &Notice) -> Value {
    json!({
        "description": n.description,
        "evocation": n.evocation,
        "story": n.story,
        "manner": n.manner,
        "details": n.details,
        "themes": n.themes,
        "anecdote": n.anecdote,
    })
}

fn credit(c: &Credit) -> Value {
    json!({
        "author": c.author,
        "licence": c.licence_or_default(),
        "source": c.source,
    })
}

fn artist(a: &Artist) -> Value {
    json!({
        "name": a.name,
        "life": a.life,
        "biography": a.biography,
        "style": a.style,
    })
}

fn artwork(ex: &Exhibition, a: &Artwork) -> Value {
    // Le tableau sur le plan : le long de son mur, un peu en retrait (comme le plan du site).
    let r = &ex.rooms[a.hook.room];
    let (ix, iz) = a.hook.side.inward();
    let (ax, az) = r.point_on(a.hook.side, a.hook.position - a.width / 2.0);
    let (bx, bz) = r.point_on(a.hook.side, a.hook.position + a.width / 2.0);
    let spot = [
        cm(ax + ix * 0.35),
        cm(az + iz * 0.35),
        cm(bx + ix * 0.35),
        cm(bz + iz * 0.35),
    ];
    json!({
        "spot": spot,
        "kind": "painting",
        "slug": a.slug,
        "title": a.title,
        "artist": a.artist_in(ex),
        "year": a.year,
        "technique": a.technique,
        "collection": a.collection,
        "note": a.note,
        "image": a.image_url().trim_start_matches('/'),
        "room": a.hook.room,
        "width": a.width,
        "height": a.height,
        "credit": credit(&a.credit),
        "notice": notice(&a.notice),
    })
}

fn sculpture(s: &Sculpture) -> Value {
    json!({
        "spot": [cm(s.x), cm(s.z), cm(s.radius.max(0.5))],
        "kind": "sculpture",
        "slug": s.slug,
        "title": s.title,
        "artist": s.artist,
        "year": s.year,
        "technique": s.technique,
        "collection": s.collection,
        "note": s.note,
        "image": s.image_url().trim_start_matches('/'),
        "room": s.room,
        "width": s.radius * 2.0,
        "height": s.height,
        // Les sculptures sont des scans 3D du Smithsonian, publiés en CC0.
        "credit": json!({ "author": "Smithsonian Institution (scan 3D)", "licence": "CC0", "source": "https://3d.si.edu/" }),
        "notice": notice(&s.notice),
    })
}

fn exhibition(ex: &Exhibition) -> Value {
    let works: Vec<Value> = ex
        .artworks
        .iter()
        .map(|a| artwork(ex, a))
        .chain(ex.sculptures.iter().map(sculpture))
        .collect();
    // Les numéros des salles dans le parcours (les couloirs n'en ont pas).
    let numbers: Vec<usize> = ex.galleries().map(|(i, _)| i).collect();
    json!({
        "slug": ex.slug,
        "title": ex.title,
        "short_name": ex.short_name,
        "kicker": ex.kicker,
        "artist": ex.artist,
        "artist_dates": ex.artist_dates,
        "movement": ex.movement,
        "ephemeral": ex.ephemeral,
        "introduction": ex.introduction,
        "accent": hex(ex.accent),
        "cover": ex.cover().map(|a| a.image_url().trim_start_matches('/').to_owned()),
        "gallery": format!("{}/galerie", ex.slug),
        "museum": ex.museum.as_ref().map(|m| json!({
            "name": m.name,
            "city": m.city,
            "architect": m.architect,
            "built": m.built,
            "presentation": m.presentation,
            "here": m.here,
        })),
        "rooms": ex.rooms.iter().enumerate().map(|(i, r)| json!({
            "name": r.name,
            "theme": r.theme,
            "corridor": r.corridor,
            "x": cm(r.x),
            "z": cm(r.z),
            "width": cm(r.width),
            "depth": cm(r.depth),
            "walls": hex(r.walls),
            "number": numbers.iter().position(|&n| n == i).map(|n| n + 1),
            "doors": r.doors.iter().map(|d| {
                let (ax, az) = r.point_on(d.side, d.position - d.width / 2.0);
                let (bx, bz) = r.point_on(d.side, d.position + d.width / 2.0);
                [cm(ax), cm(az), cm(bx), cm(bz)]
            }).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "entrance": arteon::museum::entrance(ex).map(|(_, x, z, side)| {
            let (ix, iz) = side.inward();
            [cm(x), cm(z), cm(ix), cm(iz)]
        }),
        "artists": ex.artists.iter().map(artist).collect::<Vec<_>>(),
        "works": works,
    })
}

/// Tout le catalogue.
pub fn catalogue() -> Value {
    json!({
        "version": VERSION,
        "name": "ARTEON",
        "exhibitions": EXHIBITIONS.iter().map(exhibition).collect::<Vec<_>>(),
    })
}

/// Le catalogue en texte, compact.
pub fn catalogue_json() -> String {
    catalogue().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_exhibition_and_work_is_in_the_catalogue() {
        let c = catalogue();
        let list = c["exhibitions"].as_array().expect("des expositions");
        assert_eq!(list.len(), EXHIBITIONS.len());
        for (json, ex) in list.iter().zip(EXHIBITIONS) {
            assert_eq!(json["slug"], ex.slug);
            let works = json["works"].as_array().expect("des œuvres");
            assert_eq!(works.len(), ex.artworks.len() + ex.sculptures.len());
            for w in works {
                let image = w["image"].as_str().expect("une image");
                assert!(!image.starts_with('/'), "{image} : relatif au site");
                assert!(w["title"].as_str().is_some_and(|t| !t.is_empty()));
                let room = w["room"].as_u64().expect("une salle") as usize;
                assert!(room < ex.rooms.len());
                let spot = w["spot"].as_array().expect("une place sur le plan");
                assert!(spot.len() == 3 || spot.len() == 4, "{spot:?}");
            }
            assert!(json["accent"].as_str().is_some_and(|a| a.len() == 7));
            // Le plan : chaque salle a sa place, l'entrée existe, les salles sont numérotées.
            for r in json["rooms"].as_array().expect("des salles") {
                assert!(r["width"].as_f64().is_some_and(|w| w > 0.0), "{r}");
                assert!(r["depth"].as_f64().is_some_and(|d| d > 0.0), "{r}");
                assert!(r["doors"].as_array().is_some());
            }
            assert!(json["entrance"].as_array().is_some_and(|e| e.len() == 4), "{}", ex.slug);
            assert_eq!(
                json["rooms"].as_array().unwrap().iter().filter(|r| r["number"].is_u64()).count(),
                ex.galleries().count()
            );
        }
        // Les textes longs sont bien là (ils ne manquent qu'au WebAssembly).
        let iris = &list[0]["works"][0]["notice"]["description"];
        assert!(iris.as_str().is_some_and(|d| d.len() > 40), "{iris}");
    }
}
