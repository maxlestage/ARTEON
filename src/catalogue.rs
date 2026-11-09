//! Le catalogue complet en JSON (`/catalogue.json`) : toutes les expositions, leurs salles,
//! leurs artistes et leurs œuvres avec leur notice, pour les applications (l'application
//! iPad en Flutter, dans `flutter/`). Les images sont des chemins relatifs au site
//! (`tableaux/van-gogh/iris.jpg`), à résoudre contre son adresse : le même catalogue sert
//! sur GitHub Pages (`/ARTEON/`) et sur Heroku (`/`).

use paquets::serde_json::{Value, json};

use arteon::artworks::{Artist, Artwork, Credit, EXHIBITIONS, Exhibition, Notice, Sculpture};

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
    json!({
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
        "credit": Value::Null,
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
        "rooms": ex.rooms.iter().map(|r| json!({
            "name": r.name,
            "theme": r.theme,
            "corridor": r.corridor,
        })).collect::<Vec<_>>(),
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
            }
            assert!(json["accent"].as_str().is_some_and(|a| a.len() == 7));
        }
        // Les textes longs sont bien là (ils ne manquent qu'au WebAssembly).
        let iris = &list[0]["works"][0]["notice"]["description"];
        assert!(iris.as_str().is_some_and(|d| d.len() > 40), "{iris}");
    }
}
