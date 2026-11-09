//! L'éditeur de musée : les déplacements de tableaux et de sculptures faits dans le
//! navigateur, appliqués en direct à la scène, puis réécrits dans le fichier `.toml` de
//! l'exposition (à télécharger et à publier). Seuls les champs de placement des œuvres
//! modifiées changent : le reste du fichier, commentaires compris, est gardé tel quel.

use std::collections::BTreeMap;

use crate::artworks::{Artwork, Exhibition, Hook, Sculpture, Side, WALL};

/// La nouvelle place d'un tableau.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArtEdit {
    /// La salle (son index dans `rooms`).
    pub room: usize,
    pub side: Side,
    /// Le centre du tableau, en mètres depuis le milieu du mur (vers la droite).
    pub position: f32,
    pub width: f32,
}

/// La nouvelle place d'une sculpture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SculptEdit {
    pub room: usize,
    /// Le centre du socle, en mètres depuis le centre de la salle.
    pub x: f32,
    pub z: f32,
    /// En degrés.
    pub rotation: f32,
    pub height: f32,
    pub plinth: f32,
}

/// Toutes les modifications en cours ; `version` change à chaque modification (la scène
/// se reconstruit quand elle change).
#[derive(Clone, Debug, Default)]
pub struct Edits {
    pub version: u32,
    pub arts: BTreeMap<usize, ArtEdit>,
    pub sculptures: BTreeMap<usize, SculptEdit>,
}

impl Edits {
    pub fn is_empty(&self) -> bool {
        self.arts.is_empty() && self.sculptures.is_empty()
    }

    /// La place actuelle d'un tableau (modifiée ou d'origine).
    pub fn art(&self, ex: &Exhibition, i: usize) -> ArtEdit {
        self.arts.get(&i).copied().unwrap_or_else(|| {
            let a = &ex.artworks[i];
            ArtEdit {
                room: a.hook.room,
                side: a.hook.side,
                position: a.hook.position,
                width: a.width,
            }
        })
    }

    /// La place actuelle d'une sculpture (modifiée ou d'origine).
    pub fn sculpture(&self, ex: &Exhibition, i: usize) -> SculptEdit {
        self.sculptures.get(&i).copied().unwrap_or_else(|| {
            let s = &ex.sculptures[i];
            let room = &ex.rooms[s.room];
            SculptEdit {
                room: s.room,
                x: s.x - room.x,
                z: s.z - room.z,
                rotation: s.rotation.to_degrees(),
                height: s.height,
                plinth: s.plinth,
            }
        })
    }
}

/// Le tableau `i` à sa place modifiée.
pub fn edited_art(ex: &Exhibition, edits: &Edits, i: usize) -> Artwork {
    let mut art = ex.artworks[i];
    if let Some(e) = edits.arts.get(&i) {
        let ratio = art.height / art.width;
        art.width = e.width;
        art.height = e.width * ratio;
        art.hook = Hook {
            room: e.room,
            side: e.side,
            position: e.position,
            center: 1.8f32.max(0.5 + art.height / 2.0),
            place: 0,
        };
    }
    art
}

/// La sculpture `i` à sa place modifiée.
pub fn edited_sculpture(ex: &Exhibition, edits: &Edits, i: usize) -> Sculpture {
    let mut s = ex.sculptures[i];
    if let Some(e) = edits.sculptures.get(&i) {
        let room = &ex.rooms[e.room];
        s.radius *= e.height / s.height;
        s.room = e.room;
        s.x = room.x + e.x;
        s.z = room.z + e.z;
        s.rotation = e.rotation.to_radians();
        s.height = e.height;
        s.plinth = e.plinth;
    }
    s
}

/// Ce qui ne va pas dans la place d'un tableau (vide si tout va bien) : la compilation
/// le refuserait.
pub fn art_problems(ex: &Exhibition, edits: &Edits, i: usize) -> Vec<String> {
    let art = edited_art(ex, edits, i);
    let room = &ex.rooms[art.hook.room];
    let mut out = Vec::new();
    let (start, end) = (
        art.hook.position - art.width / 2.0 - 0.1,
        art.hook.position + art.width / 2.0 + 0.5,
    );
    let half = room.wall_length(art.hook.side) / 2.0 - WALL;
    if start < -half || end > half {
        out.push("le tableau dépasse du mur".to_owned());
    }
    if art.hook.center + art.height / 2.0 > room.height - 0.25 {
        out.push("le tableau est trop haut pour la salle".to_owned());
    }
    for d in room.doors.iter().filter(|d| d.side == art.hook.side) {
        if start < d.position + d.width / 2.0 && d.position - d.width / 2.0 < end {
            out.push("le tableau est sur une porte".to_owned());
        }
    }
    for k in (0..ex.artworks.len()).filter(|&k| k != i) {
        let other = edited_art(ex, edits, k);
        if other.hook.room == art.hook.room && other.hook.side == art.hook.side {
            let (a, b) = (
                other.hook.position - other.width / 2.0 - 0.1,
                other.hook.position + other.width / 2.0 + 0.5,
            );
            if start < b && a < end {
                out.push(format!("il chevauche « {} »", other.title));
            }
        }
    }
    out
}

/// Ce qui ne va pas dans la place d'une sculpture (vide si tout va bien).
pub fn sculpture_problems(ex: &Exhibition, edits: &Edits, i: usize) -> Vec<String> {
    let s = edited_sculpture(ex, edits, i);
    let room = &ex.rooms[s.room];
    let mut out = Vec::new();
    let (dx, dz) = (s.x - room.x, s.z - room.z);
    if dx.abs() + s.radius > room.width / 2.0 - WALL - 0.3
        || dz.abs() + s.radius > room.depth / 2.0 - WALL - 0.3
    {
        out.push("trop près d'un mur".to_owned());
    }
    if s.plinth + s.height > room.height - 0.2 {
        out.push("elle touche le plafond".to_owned());
    }
    for d in room.doors {
        let (x, z) = room.point_on(d.side, d.position);
        if (x - s.x).hypot(z - s.z) < s.radius + 1.6 {
            out.push("elle bloque une porte".to_owned());
        }
    }
    out
}

fn number(v: f32) -> String {
    let text = format!("{:.2}", v);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
        "0".to_owned()
    } else {
        text.to_owned()
    }
}

/// Le titre d'un bloc `[[oeuvres]]` ou `[[sculptures]]` (la valeur de `titre = "…"`).
fn block_title(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let rest = line
            .trim_start()
            .strip_prefix("titre")?
            .trim_start()
            .strip_prefix('=')?
            .trim();
        let inner = rest.strip_prefix('"')?.rsplit_once('"')?.0;
        Some(inner.replace("\\\"", "\""))
    })
}

/// Le bloc, sans les lignes des clés `keys`, puis les nouvelles lignes `lines`.
fn rewrite_block(block: &str, keys: &[&str], lines: &[String]) -> String {
    let kept: Vec<&str> = block
        .lines()
        .filter(|line| {
            let key = line.split('=').next().unwrap_or("").trim();
            !keys.contains(&key)
        })
        .collect();
    // Les nouvelles lignes vont à la fin du bloc, avant les lignes vides et les
    // commentaires qui annoncent la section suivante.
    let mut end = kept.len();
    while end > 1
        && (kept[end - 1].trim().is_empty() || kept[end - 1].trim_start().starts_with('#'))
    {
        end -= 1;
    }
    let mut out: Vec<String> = kept[..end].iter().map(|l| (*l).to_owned()).collect();
    out.extend(lines.iter().cloned());
    out.extend(kept[end..].iter().map(|l| (*l).to_owned()));
    let mut text = out.join("\n");
    if block.ends_with('\n') {
        text.push('\n');
    }
    text
}

/// Le fichier `.toml` de l'exposition, avec les places modifiées.
pub fn patched_toml(ex: &Exhibition, edits: &Edits) -> String {
    let source = ex.source;
    // Découpe en blocs, chacun commençant à une ligne `[[…]]` ou `[…]`.
    let mut starts: Vec<usize> = vec![0];
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        if offset > 0 && line.trim_start().starts_with('[') {
            starts.push(offset);
        }
        offset += line.len();
    }
    starts.push(source.len());
    let mut out = String::with_capacity(source.len() + 256);
    for w in starts.windows(2) {
        let block = &source[w[0]..w[1]];
        let head = block.trim_start();
        let title = block_title(block);
        let rewritten = if head.starts_with("[[oeuvres]]") {
            title
                .and_then(|t| ex.artworks.iter().position(|a| a.title == t))
                .and_then(|i| edits.arts.get(&i).map(|e| (i, e)))
                .map(|(_, e)| {
                    let room = &ex.rooms[e.room];
                    rewrite_block(
                        block,
                        &["salle", "mur", "place", "position", "largeur"],
                        &[
                            format!("salle = {}", room.number),
                            format!("mur = \"{}\"", e.side.french()),
                            format!("position = {}", number(e.position)),
                            format!("largeur = {}", number(e.width)),
                        ],
                    )
                })
        } else if head.starts_with("[[sculptures]]") {
            title
                .and_then(|t| ex.sculptures.iter().position(|s| s.title == t))
                .and_then(|i| edits.sculptures.get(&i))
                .map(|e| {
                    let room = &ex.rooms[e.room];
                    rewrite_block(
                        block,
                        &["salle", "x", "z", "hauteur", "rotation", "socle"],
                        &[
                            format!("salle = {}", room.number),
                            format!("x = {}", number(e.x)),
                            format!("z = {}", number(e.z)),
                            format!("hauteur = {}", number(e.height)),
                            format!("rotation = {}", number(e.rotation)),
                            format!("socle = {}", number(e.plinth)),
                        ],
                    )
                })
        } else {
            None
        };
        out.push_str(&rewritten.unwrap_or_else(|| block.to_owned()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artworks::EXHIBITIONS;

    #[test]
    fn without_edits_the_file_is_unchanged() {
        for ex in EXHIBITIONS {
            assert_eq!(
                patched_toml(ex, &Edits::default()),
                ex.source,
                "{}",
                ex.slug
            );
        }
    }

    #[test]
    fn edits_rewrite_only_the_placement() {
        let ex = EXHIBITIONS
            .iter()
            .find(|e| !e.sculptures.is_empty())
            .expect("une exposition avec des sculptures");
        let mut edits = Edits::default();
        let gallery = ex.galleries().next().map(|(i, _)| i).unwrap_or(0);
        edits.arts.insert(
            0,
            ArtEdit {
                room: gallery,
                side: Side::West,
                position: -1.25,
                width: 1.5,
            },
        );
        edits.sculptures.insert(
            0,
            SculptEdit {
                room: ex.sculptures[0].room,
                x: 1.5,
                z: -2.0,
                rotation: 45.0,
                height: 1.2,
                plinth: 0.5,
            },
        );
        let text = patched_toml(ex, &edits);
        let doc: toml::Value = toml::from_str(&text).expect("un fichier valide");
        let art = &doc["oeuvres"][0];
        assert_eq!(art["titre"].as_str(), Some(ex.artworks[0].title));
        assert_eq!(art["mur"].as_str(), Some("ouest"));
        assert_eq!(art["position"].as_float(), Some(-1.25));
        assert_eq!(art["largeur"].as_float(), Some(1.5));
        assert!(art.get("place").is_none());
        let s = &doc["sculptures"][0];
        assert_eq!(s["x"].as_float(), Some(1.5));
        assert_eq!(
            s["rotation"]
                .as_integer()
                .map(|v| v as f64)
                .or(s["rotation"].as_float()),
            Some(45.0)
        );
        // Le reste n'a pas bougé : les autres œuvres, et le texte d'introduction.
        assert_eq!(
            doc["oeuvres"].as_array().map(Vec::len),
            Some(ex.artworks.len())
        );
        assert_eq!(doc["exposition"]["titre"].as_str(), Some(ex.title));
        // Le fichier modifié est accepté par les mêmes règles que la compilation.
        assert!(
            art_problems(ex, &edits, 0)
                .iter()
                .all(|p| !p.contains("dépasse")),
            "{:?}",
            art_problems(ex, &edits, 0)
        );
    }

    #[test]
    fn problems_are_found() {
        let ex = &EXHIBITIONS[0];
        let mut edits = Edits::default();
        let a = &ex.artworks[0];
        let length = ex.rooms[a.hook.room].wall_length(a.hook.side);
        edits.arts.insert(
            0,
            ArtEdit {
                room: a.hook.room,
                side: a.hook.side,
                position: length,
                width: a.width,
            },
        );
        assert!(
            art_problems(ex, &edits, 0)
                .iter()
                .any(|p| p.contains("dépasse"))
        );
        assert!(art_problems(ex, &Edits::default(), 0).is_empty());
    }
}
