//! La visite dynamique : une aile de salles ajoutée au musée d'une exposition au moment
//! de la visite, où sont accrochées des œuvres qui ne sont pas dans le fichier de
//! l'exposition (les œuvres liées des collections du monde, choisies par le visiteur).
//!
//! Le musée de l'exposition est construit à la compilation (`build.rs`) ; l'aile, elle,
//! est calculée dans le navigateur : on cherche un mur extérieur libre, on y perce une
//! porte, et l'on aligne derrière autant de galeries de douze œuvres qu'il en faut. Le
//! résultat est une [`Exhibition`] comme les autres : le plan, la visite guidée, la
//! visite libre et l'interface la parcourent sans rien savoir de son origine.

use crate::artworks::{
    Artwork, Credit, Door, Exhibition, Frame, Hook, Mosaic, Notice, Room, Side,
};
use crate::data::ExhibitionData;

/// La longueur d'une galerie de l'aile (dans le sens de la marche) et sa largeur.
const LENGTH: f32 = 20.0;
const ACROSS: f32 = 9.0;
const HEIGHT: f32 = 5.0;
/// La largeur des portes de l'aile (celle qui la relie au musée peut être plus étroite).
const DOOR: f32 = 3.0;
/// Les œuvres d'une galerie : six par grand mur.
pub const PER_ROOM: usize = 12;
/// La plus grande dimension d'une œuvre de l'aile, en mètres.
const SIZE: f32 = 1.6;

/// Une œuvre à accrocher dans l'aile : son cartel, et sa mosaïque déjà calculée.
#[derive(Clone, Debug)]
pub struct Extra {
    /// Son identifiant dans les collections du monde (`chicago-27992`).
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: String,
    /// L'adresse de son image (absolue).
    pub image: String,
    /// Largeur / hauteur de l'image.
    pub ratio: f32,
    pub cols: u16,
    pub rows: u16,
    pub palette: Vec<[u8; 3]>,
    /// Une couleur de la palette par case, ligne par ligne depuis le haut.
    pub cells: Vec<u8>,
}

/// La taille d'une œuvre de l'aile, en mètres, d'après les proportions de son image.
pub fn size(ratio: f32) -> (f32, f32) {
    let ratio = ratio.clamp(0.3, 3.0);
    if ratio >= 1.0 {
        (SIZE, SIZE / ratio)
    } else {
        ((SIZE * ratio).max(0.5), SIZE)
    }
}

/// La grille de la mosaïque d'une œuvre de l'aile : une case d'un centimètre environ,
/// 160 colonnes au plus (les vignettes n'en montrent pas davantage).
pub fn grid(ratio: f32) -> (u16, u16) {
    let (w, h) = size(ratio);
    let cols = ((w / 0.01).round() as u16).clamp(16, 160);
    let rows = ((h / w * f32::from(cols)).round() as u16).clamp(8, 200);
    (cols, rows)
}

/// La mosaïque d'une image déjà réduite à sa grille (`rgba`, quatre octets par case) : une
/// palette de 48 couleurs au plus (k-moyennes sur un échantillon des cases) et la couleur
/// de chaque case.
pub fn mosaic(rgba: &[u8]) -> (Vec<[u8; 3]>, Vec<u8>) {
    let pixels: Vec<[f32; 3]> = rgba
        .chunks_exact(4)
        .map(|p| [f32::from(p[0]), f32::from(p[1]), f32::from(p[2])])
        .collect();
    if pixels.is_empty() {
        return (vec![[128, 128, 128]], Vec::new());
    }
    let distance = |a: &[f32; 3], b: &[f32; 3]| {
        let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        2.0 * d[0] * d[0] + 4.0 * d[1] * d[1] + 3.0 * d[2] * d[2]
    };
    let nearest = |centers: &[[f32; 3]], p: &[f32; 3]| {
        let mut best = (0, f32::MAX);
        for (i, c) in centers.iter().enumerate() {
            let d = distance(c, p);
            if d < best.1 {
                best = (i, d);
            }
        }
        best.0
    };
    // Un échantillon régulier : assez pour trouver les couleurs, vite.
    let step = (pixels.len() / 4000).max(1);
    let sample: Vec<[f32; 3]> = pixels.iter().step_by(step).copied().collect();
    let mut centers = vec![sample[0]];
    let mut far: Vec<f32> = sample.iter().map(|p| distance(p, &sample[0])).collect();
    while centers.len() < 48.min(sample.len()) {
        let (i, d) = far
            .iter()
            .enumerate()
            .fold((0, -1.0), |b, (i, &d)| if d > b.1 { (i, d) } else { b });
        if d <= 0.0 {
            break;
        }
        let c = sample[i];
        centers.push(c);
        for (d, p) in far.iter_mut().zip(&sample) {
            *d = d.min(distance(p, &c));
        }
    }
    for _ in 0..6 {
        let mut sums = vec![[0.0f32; 4]; centers.len()];
        for p in &sample {
            let s = &mut sums[nearest(&centers, p)];
            s[0] += p[0];
            s[1] += p[1];
            s[2] += p[2];
            s[3] += 1.0;
        }
        for (c, s) in centers.iter_mut().zip(&sums) {
            if s[3] > 0.0 {
                *c = [s[0] / s[3], s[1] / s[3], s[2] / s[3]];
            }
        }
    }
    let cells = pixels.iter().map(|p| nearest(&centers, p) as u8).collect();
    // Comme à la compilation : les toiles sombres sont éclaircies sous la lumière du musée.
    let mean = pixels
        .iter()
        .map(|p| (0.3 * p[0] + 0.55 * p[1] + 0.15 * p[2]) / 255.0)
        .sum::<f32>()
        / pixels.len() as f32;
    let gamma = if mean < 0.3 {
        0.7
    } else if mean < 0.45 {
        0.85
    } else {
        1.0
    };
    let palette = centers
        .iter()
        .map(|c| c.map(|v| (255.0 * (v / 255.0).clamp(0.0, 1.0).powf(gamma)).round() as u8))
        .collect();
    (palette, cells)
}

fn opposite(side: Side) -> Side {
    match side {
        Side::North => Side::South,
        Side::South => Side::North,
        Side::East => Side::West,
        Side::West => Side::East,
    }
}

/// Le rectangle (x0, x1, z0, z1) d'une salle.
fn rect(room: &Room) -> (f32, f32, f32, f32) {
    (
        room.x - room.width / 2.0,
        room.x + room.width / 2.0,
        room.z - room.depth / 2.0,
        room.z + room.depth / 2.0,
    )
}

fn overlap(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32), margin: f32) -> bool {
    a.0 < b.1 + margin && b.0 < a.1 + margin && a.2 < b.3 + margin && b.2 < a.3 + margin
}

/// Les galeries de l'aile derrière la porte (`x`, `z`) du mur `side` : (centre x, centre z,
/// largeur, profondeur) de chacune, de la plus proche à la plus lointaine.
fn wing(x: f32, z: f32, side: Side, rooms: usize) -> Vec<(f32, f32, f32, f32)> {
    let (ix, iz) = side.inward();
    (0..rooms)
        .map(|k| {
            let d = LENGTH * k as f32 + LENGTH / 2.0;
            let (cx, cz) = (x - ix * d, z - iz * d);
            match side {
                Side::North | Side::South => (cx, cz, ACROSS, LENGTH),
                Side::East | Side::West => (cx, cz, LENGTH, ACROSS),
            }
        })
        .collect()
}

/// Où percer la porte de l'aile : une salle, un mur et une position le long de ce mur. Le
/// mur doit donner sur l'extérieur, la porte ne toucher ni un tableau, ni une autre porte,
/// ni une sculpture, et l'aile ne recouvrir aucune salle ni barrer l'entrée du musée.
fn doorway(ex: &Exhibition, rooms: usize, door: f32, margin: f32) -> Option<(usize, Side, f32)> {
    let entrance = crate::museum::entrance(ex).map(|(_, x, z, side)| {
        let (ix, iz) = side.inward();
        (x - ix * 3.0, z - iz * 3.0)
    });
    // Les dernières salles d'abord : l'aile prolonge la visite.
    for r in (0..ex.rooms.len()).rev() {
        let room = &ex.rooms[r];
        if room.corridor {
            continue;
        }
        for side in [Side::North, Side::East, Side::West, Side::South] {
            let length = room.wall_length(side);
            let reach = length / 2.0 - door / 2.0 - 0.8;
            if reach < 0.0 {
                continue;
            }
            let mut positions = vec![0.0];
            let mut p = 0.5;
            while p <= reach {
                positions.push(p);
                positions.push(-p);
                p += 0.5;
            }
            'position: for position in positions {
                let (lo, hi) = (position - door / 2.0 - margin, position + door / 2.0 + margin);
                // Rien sur le mur à cet endroit.
                for d in room.doors.iter().filter(|d| d.side == side) {
                    if d.position - d.width / 2.0 < hi && lo < d.position + d.width / 2.0 {
                        continue 'position;
                    }
                }
                for a in ex.artworks.iter().filter(|a| a.hook.room == r && a.hook.side == side) {
                    if a.hook.position - a.width / 2.0 < hi && lo < a.hook.position + a.width / 2.0 {
                        continue 'position;
                    }
                }
                let (x, z) = room.point_on(side, position);
                // Pas de sculpture devant la porte.
                if ex.sculptures.iter().any(|s| (s.x - x).hypot(s.z - z) < s.radius + door) {
                    continue;
                }
                // L'aile tient dehors, loin de l'entrée.
                let free = wing(x, z, side, rooms).iter().all(|&(cx, cz, w, d)| {
                    let wing_rect = (cx - w / 2.0, cx + w / 2.0, cz - d / 2.0, cz + d / 2.0);
                    !ex.rooms.iter().enumerate().any(|(k, other)| {
                        let margin = if k == r { -0.05 } else { 0.3 };
                        overlap(wing_rect, rect(other), margin)
                    }) && entrance.is_none_or(|(ex_x, ex_z)| {
                        !(wing_rect.0 - 2.0 < ex_x
                            && ex_x < wing_rect.1 + 2.0
                            && wing_rect.2 - 2.0 < ex_z
                            && ex_z < wing_rect.3 + 2.0)
                    })
                });
                if free {
                    return Some((r, side, position));
                }
            }
        }
    }
    None
}

fn leak<T>(value: Vec<T>) -> &'static [T] {
    Box::leak(value.into_boxed_slice())
}

fn leak_str(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// Le nom d'une galerie de l'aile, dans la langue de la page.
fn room_name(n: usize) -> String {
    crate::tf!(
        "Aile des collections · galerie {}",
        "Collections wing · gallery {}",
        "Ala de las colecciones · galería {}",
        n
    )
}

/// Le musée de `ex` agrandi d'une aile où sont accrochées les œuvres `extras`, et ses
/// données (les mosaïques des œuvres ajoutées à la suite). `None` si le musée n'a pas de
/// mur libre pour l'aile, ou s'il n'y a rien à accrocher : la visite reste celle du fichier.
pub fn extend(
    ex: &'static Exhibition,
    data: &ExhibitionData,
    extras: Vec<Extra>,
) -> Option<(&'static Exhibition, ExhibitionData)> {
    if extras.is_empty() {
        return None;
    }
    let count = extras.len().div_ceil(PER_ROOM);
    // Une porte large si un mur le permet ; sinon plus étroite, entre deux tableaux.
    let (door, (attach, side, position)) = [(DOOR, 0.5), (2.2, 0.3), (1.6, 0.2), (1.2, 0.12)]
        .into_iter()
        .find_map(|(door, margin)| doorway(ex, count, door, margin).map(|found| (door, found)))?;
    let (x, z) = ex.rooms[attach].point_on(side, position);
    let mut rooms: Vec<Room> = ex.rooms.to_vec();
    // La porte de l'aile, dans la salle du musée.
    let mut doors = rooms[attach].doors.to_vec();
    doors.push(Door { side, position, width: door });
    rooms[attach].doors = leak(doors);
    let first = rooms.len();
    let walls = ex.rooms[attach].walls;
    let theme = leak_str(
        crate::langue::t(
            "Les œuvres liées des collections du monde, de la plus ancienne à la plus récente.",
            "Related works from the world collections, from oldest to most recent.",
            "Las obras relacionadas de las colecciones del mundo, de la más antigua a la más reciente.",
        )
        .to_owned(),
    );
    let number = ex.rooms.iter().map(|r| r.number).max().unwrap_or(0);
    for (k, (cx, cz, width, depth)) in wing(x, z, side, count).into_iter().enumerate() {
        let back = if k == 0 { door } else { DOOR };
        let mut doors = vec![Door { side: opposite(side), position: 0.0, width: back }];
        if k + 1 < count {
            doors.push(Door { side, position: 0.0, width: DOOR });
        }
        rooms.push(Room {
            name: leak_str(room_name(k + 1)),
            theme,
            walls,
            x: cx,
            z: cz,
            width,
            depth,
            height: HEIGHT,
            corridor: false,
            number: number + k + 1,
            doors: leak(doors),
        });
    }
    // Les deux grands murs de chaque galerie, six places chacun.
    let long_walls = match side {
        Side::North | Side::South => [Side::West, Side::East],
        Side::East | Side::West => [Side::North, Side::South],
    };
    let frame = ex.artworks.first().map_or(Frame::Wood, |a| a.frame);
    let mut bytes = data.bytes().to_vec();
    let mut artworks: Vec<Artwork> = ex.artworks.to_vec();
    let per_wall = PER_ROOM / 2;
    for (n, extra) in extras.into_iter().enumerate() {
        let (room, place) = (first + n / PER_ROOM, n % PER_ROOM);
        let wall = long_walls[place / per_wall];
        let slot = place % per_wall;
        let position = -LENGTH / 2.0 + LENGTH * (slot as f32 + 0.5) / per_wall as f32;
        let (width, height) = size(extra.ratio);
        let offset = bytes.len() as u32;
        let expected = usize::from(extra.cols) * usize::from(extra.rows);
        let mut cells = extra.cells;
        cells.resize(expected, 0);
        bytes.extend_from_slice(&cells);
        artworks.push(Artwork {
            title: leak_str(extra.title),
            slug: leak_str(extra.id),
            artist: Some(leak_str(extra.artist)),
            year: leak_str(extra.year),
            image: leak_str(extra.image),
            technique: "",
            collection: crate::langue::t("Collections du monde", "World collections", "Colecciones del mundo"),
            note: "",
            notice: Notice {
                description: "",
                evocation: "",
                story: "",
                manner: "",
                details: &[],
                themes: &[],
                anecdote: "",
            },
            frame,
            credit: Credit { author: "", licence: "", source: "" },
            width,
            height,
            hook: Hook {
                room,
                side: wall,
                position,
                center: 1.8f32.max(0.5 + height / 2.0),
                place: slot as u8 + 1,
            },
            mosaic: Mosaic {
                cols: extra.cols,
                rows: extra.rows,
                palette: leak(extra.palette),
                offset,
            },
        });
    }
    let extended: &'static Exhibition = Box::leak(Box::new(Exhibition {
        rooms: leak(rooms),
        artworks: leak(artworks),
        data_size: bytes.len() as u32,
        ..*ex
    }));
    Some((extended, ExhibitionData::extended(bytes)))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::artworks::EXHIBITIONS;
    use crate::museum;

    fn extras(n: usize) -> Vec<Extra> {
        (0..n)
            .map(|i| {
                let ratio = [1.4, 0.7, 1.0][i % 3];
                let (cols, rows) = grid(ratio);
                let rgba: Vec<u8> = (0..usize::from(cols) * usize::from(rows))
                    .flat_map(|p| [(p % 251) as u8, (p % 83) as u8 * 3, (i * 40) as u8, 255])
                    .collect();
                let (palette, cells) = mosaic(&rgba);
                Extra {
                    id: format!("essai-{i}"),
                    title: format!("Œuvre {i}"),
                    artist: "Anonyme".into(),
                    year: "1900".into(),
                    image: format!("https://exemple.org/{i}.jpg"),
                    ratio,
                    cols,
                    rows,
                    palette,
                    cells,
                }
            })
            .collect()
    }

    /// La mosaïque d'une image : une palette, et une couleur valide par case.
    #[test]
    fn a_mosaic_has_one_valid_colour_per_cell() {
        let (cols, rows) = grid(1.4);
        let rgba: Vec<u8> = (0..usize::from(cols) * usize::from(rows))
            .flat_map(|p| [(p % 256) as u8, (p / 7 % 256) as u8, 90, 255])
            .collect();
        let (palette, cells) = mosaic(&rgba);
        assert!(!palette.is_empty() && palette.len() <= 48);
        assert_eq!(cells.len(), usize::from(cols) * usize::from(rows));
        assert!(cells.iter().all(|&c| usize::from(c) < palette.len()));
    }

    /// Chaque musée accepte une aile : ses galeries ne recouvrent rien, on y entre par une
    /// porte, la visite guidée s'arrête devant chaque œuvre ajoutée, et rien ne change
    /// pour les œuvres du fichier.
    #[test]
    fn every_museum_takes_a_wing() {
        let mut without = Vec::new();
        for ex in EXHIBITIONS {
            let data = ExhibitionData::load(ex).expect("données");
            let Some((big, more)) = extend(ex, &data, extras(30)) else {
                // Une exposition éphémère change chaque semaine : sans mur libre, sa visite
                // reste celle du fichier, ce n'est pas une erreur.
                if !ex.ephemeral {
                    without.push(ex.slug);
                }
                continue;
            };
            assert_eq!(big.artworks.len(), ex.artworks.len() + 30, "{}", ex.slug);
            assert_eq!(big.rooms.len(), ex.rooms.len() + 3, "{}", ex.slug);
            // Les salles de l'aile ne recouvrent aucune autre salle.
            for (i, a) in big.rooms.iter().enumerate().skip(ex.rooms.len()) {
                for (k, b) in big.rooms.iter().enumerate() {
                    assert!(i == k || !overlap(rect(a), rect(b), -0.05), "{} : salles {i} et {k}", ex.slug);
                }
            }
            // Toutes les galeries de l'aile sont reliées au musée.
            let hall = museum::entrance(big).map_or(0, |e| e.0);
            for r in ex.rooms.len()..big.rooms.len() {
                assert!(museum::route(big, hall, r).is_some(), "{} : salle {r} inaccessible", ex.slug);
            }
            assert_eq!(museum::entrance(big).map(|e| e.0), museum::entrance(ex).map(|e| e.0), "{}", ex.slug);
            // Les mosaïques ajoutées se lisent, celles du fichier sont intactes.
            for (a, b) in ex.artworks.iter().zip(big.artworks) {
                assert_eq!(data.cells(&a.mosaic), more.cells(&b.mosaic));
            }
            for a in &big.artworks[ex.artworks.len()..] {
                assert_eq!(more.cells(&a.mosaic).len(), a.mosaic.len());
                assert!(a.hook.center + a.height / 2.0 < HEIGHT - 0.25);
            }
            // La visite guidée passe devant chaque œuvre de l'aile.
            let stops = crate::tour::stops(big);
            for i in ex.artworks.len()..big.artworks.len() {
                assert!(
                    stops.iter().any(|s| s.focus == Some(crate::tour::Focus::Art(i))),
                    "{} : la visite oublie l'œuvre {i}",
                    ex.slug
                );
            }
            assert!(crate::tour::stops(ex).len() < stops.len());
        }
        assert!(without.is_empty(), "sans aile possible : {without:?}");
    }
}
