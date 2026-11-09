//! La géométrie d'un musée, calculée à partir de son plan : les pavés des murs (percés de
//! portes), des sols et des plafonds, les lampes, les obstacles du visiteur et les
//! passages entre les salles. Des fonctions pures : elles se testent sans navigateur.

use std::collections::VecDeque;

use crate::artworks::{Artwork, Exhibition, Hook, Room, Side, WALL};

/// Un pavé de la scène : sa taille, son centre, sa couleur ; `solid` s'il arrête le visiteur.
#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub size: [f32; 3],
    pub center: [f32; 3],
    pub color: [u8; 3],
    pub solid: bool,
}

pub const FLOOR: [u8; 3] = [132, 98, 66];
pub const CEILING: [u8; 3] = [250, 248, 242];
pub const BASEBOARD: [u8; 3] = [58, 40, 28];
pub const GOLD: [u8; 3] = [242, 183, 5];

/// Les intervalles (le long de l'axe du mur, en coordonnées absolues) occupés par les
/// portes du mur `side`.
fn door_spans(room: &Room, side: Side) -> Vec<(f32, f32)> {
    room.doors
        .iter()
        .filter(|d| d.side == side)
        .map(|d| {
            let (x, z) = room.point_on(side, d.position);
            let along = match side {
                Side::North | Side::South => x,
                Side::East | Side::West => z,
            };
            (along - d.width / 2.0, along + d.width / 2.0)
        })
        .collect()
}

/// `[lo, hi]` privé des intervalles `holes`.
fn subtract(lo: f32, hi: f32, holes: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut parts = vec![(lo, hi)];
    for &(a, b) in holes {
        parts = parts
            .into_iter()
            .flat_map(|(p, q)| {
                if b <= p || a >= q {
                    vec![(p, q)]
                } else {
                    [(p, a.max(p)), (b.min(q), q)]
                        .into_iter()
                        .filter(|(s, e)| e - s > 0.01)
                        .collect()
                }
            })
            .collect();
    }
    parts
}

/// Un pavé le long du mur `side` de `room`, de `a` à `b` sur l'axe du mur, de `y0` à `y1`
/// en hauteur, épais de `thick` depuis la limite de la salle.
fn along_wall(
    room: &Room,
    side: Side,
    (a, b): (f32, f32),
    (y0, y1): (f32, f32),
    thick: f32,
    color: [u8; 3],
    solid: bool,
) -> Block {
    let (x0, x1) = (room.x - room.width / 2.0, room.x + room.width / 2.0);
    let (z0, z1) = (room.z - room.depth / 2.0, room.z + room.depth / 2.0);
    let (len, mid, y) = (b - a, (a + b) / 2.0, (y0 + y1) / 2.0);
    let h = y1 - y0;
    let (size, center) = match side {
        Side::North => ([len, h, thick], [mid, y, z0 + thick / 2.0]),
        Side::South => ([len, h, thick], [mid, y, z1 - thick / 2.0]),
        Side::West => ([thick, h, len], [x0 + thick / 2.0, y, mid]),
        Side::East => ([thick, h, len], [x1 - thick / 2.0, y, mid]),
    };
    Block {
        size,
        center,
        color,
        solid,
    }
}

/// Les pavés d'une salle : sol, plafond, murs percés de portes, linteaux, plinthes (ou,
/// dans un couloir, un liseré doré).
pub fn room_blocks(room: &Room) -> Vec<Block> {
    let mut blocks = vec![
        Block {
            size: [room.width, 0.1, room.depth],
            center: [room.x, -0.05, room.z],
            color: FLOOR,
            solid: false,
        },
        Block {
            size: [room.width, 0.1, room.depth],
            center: [room.x, room.height + 0.05, room.z],
            color: CEILING,
            solid: false,
        },
    ];
    let door_top = room.door_height();
    for side in Side::ALL {
        // Les murs est et ouest vont d'un coin à l'autre ; les murs nord et sud s'arrêtent
        // contre eux (pas de pavés superposés).
        let (lo, hi) = match side {
            Side::North | Side::South => (
                room.x - room.width / 2.0 + WALL,
                room.x + room.width / 2.0 - WALL,
            ),
            Side::East | Side::West => (room.z - room.depth / 2.0, room.z + room.depth / 2.0),
        };
        let holes = door_spans(room, side);
        for part in subtract(lo, hi, &holes) {
            blocks.push(along_wall(
                room,
                side,
                part,
                (0.0, room.height),
                WALL,
                room.walls,
                true,
            ));
            if room.corridor {
                blocks.push(along_wall(
                    room,
                    side,
                    part,
                    (0.03, 0.07),
                    WALL + 0.04,
                    GOLD,
                    false,
                ));
            } else {
                blocks.push(along_wall(
                    room,
                    side,
                    part,
                    (0.0, 0.12),
                    WALL + 0.03,
                    BASEBOARD,
                    false,
                ));
            }
        }
        for &(a, b) in &holes {
            let (a, b) = (a.max(lo), b.min(hi));
            if b - a > 0.01 && room.height - door_top > 0.01 {
                blocks.push(along_wall(
                    room,
                    side,
                    (a, b),
                    (door_top, room.height),
                    WALL,
                    room.walls,
                    false,
                ));
            }
        }
    }
    blocks
}

/// Les lampes d'une salle : une tous les 7 m environ, sous le plafond.
pub fn lamps(room: &Room) -> Vec<[f32; 3]> {
    let nx = (room.width / 7.0).ceil().max(1.0) as usize;
    let nz = (room.depth / 7.0).ceil().max(1.0) as usize;
    let mut out = Vec::new();
    for i in 0..nx {
        for k in 0..nz {
            let x = room.x - room.width / 2.0 + room.width * (i as f32 + 0.5) / nx as f32;
            let z = room.z - room.depth / 2.0 + room.depth * (k as f32 + 0.5) / nz as f32;
            out.push([x, room.height - 0.4, z]);
        }
    }
    out
}

/// L'intensité des lampes d'une salle, en lumens : plus forte sous un plafond haut.
pub fn lamp_intensity(room: &Room) -> f32 {
    if room.corridor {
        110_000.0
    } else {
        300_000.0 * (room.height / 3.6).powi(2)
    }
}

/// Le banc au centre d'une galerie, s'il y a la place (pas de sculpture tout près).
pub fn bench(ex: &Exhibition, room_index: usize) -> Option<Vec<Block>> {
    let room = &ex.rooms[room_index];
    let clear = ex
        .sculptures_in_room(room_index)
        .iter()
        .all(|s| (s.x - room.x).hypot(s.z - room.z) > s.radius + 2.0);
    if room.corridor || room.width < 7.0 || room.depth < 7.0 || !clear {
        return None;
    }
    let long_x = room.width > room.depth;
    let (seat, leg) = if long_x {
        ([2.2, 0.08, 0.5], [0.1, 0.38, 0.42])
    } else {
        ([0.5, 0.08, 2.2], [0.42, 0.38, 0.1])
    };
    let mut blocks = vec![Block {
        size: seat,
        center: [room.x, 0.42, room.z],
        color: BASEBOARD,
        solid: true,
    }];
    for k in [-0.95, 0.95] {
        let (dx, dz) = if long_x { (k, 0.0) } else { (0.0, k) };
        blocks.push(Block {
            size: leg,
            center: [room.x + dx, 0.19, room.z + dz],
            color: BASEBOARD,
            solid: false,
        });
    }
    Some(blocks)
}

/// Où se trouve un point posé sur le mur d'un tableau : `u` vers la droite du visiteur
/// qui le regarde, `v` vers le haut depuis le centre du tableau, `layer` en avant du mur.
/// Renvoie la position et l'angle (autour de y) qui tourne +z vers la salle.
pub fn on_wall(ex: &Exhibition, hook: Hook, u: f32, v: f32, layer: f32) -> ([f32; 3], f32) {
    let room = &ex.rooms[hook.room];
    let (x, z) = room.point_on(hook.side, hook.position + u);
    let (ix, iz) = hook.side.inward();
    let depth = WALL + layer;
    (
        [x + ix * depth, hook.center + v, z + iz * depth],
        hook.side.facing(),
    )
}

/// Le centre d'un tableau, sur la face du mur.
pub fn artwork_center(ex: &Exhibition, art: &Artwork) -> [f32; 3] {
    on_wall(ex, art.hook, 0.0, 0.0, 0.0).0
}

/// Un obstacle du visiteur, vu de dessus : un rectangle (x0, x1, z0, z1).
pub type Rect = (f32, f32, f32, f32);

/// Les obstacles du musée : les murs, les bancs et les socles des sculptures.
pub fn obstacles(ex: &Exhibition) -> Vec<Rect> {
    let mut out = Vec::new();
    let mut push = |b: &Block| {
        if b.solid {
            out.push((
                b.center[0] - b.size[0] / 2.0,
                b.center[0] + b.size[0] / 2.0,
                b.center[2] - b.size[2] / 2.0,
                b.center[2] + b.size[2] / 2.0,
            ));
        }
    };
    for (i, room) in ex.rooms.iter().enumerate() {
        room_blocks(room).iter().for_each(&mut push);
        bench(ex, i).iter().flatten().for_each(&mut push);
    }
    for s in ex.sculptures {
        let r = s.radius * 0.8;
        out.push((s.x - r, s.x + r, s.z - r, s.z + r));
    }
    out
}

/// Les obstacles de `ex`, calculés une fois pour toutes les expositions.
pub fn cached_obstacles(ex: &Exhibition) -> &'static [Rect] {
    static ALL: std::sync::OnceLock<Vec<Vec<Rect>>> = std::sync::OnceLock::new();
    let all = ALL.get_or_init(|| crate::artworks::EXHIBITIONS.iter().map(obstacles).collect());
    let index = crate::artworks::EXHIBITIONS
        .iter()
        .position(|e| e.slug == ex.slug)
        .unwrap_or(0);
    &all[index]
}

/// Un passage d'une salle à une autre, par une porte : `x`, `z` est le milieu de la porte,
/// sur la limite de la salle `from`, dont `side` est le mur.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Passage {
    pub from: usize,
    pub to: usize,
    pub x: f32,
    pub z: f32,
    pub side: Side,
}

/// Toutes les portes qui relient deux salles (dans les deux sens).
pub fn passages(ex: &Exhibition) -> Vec<Passage> {
    let mut out = Vec::new();
    for (from, room) in ex.rooms.iter().enumerate() {
        for door in room.doors {
            let (x, z) = room.point_on(door.side, door.position);
            let (ix, iz) = door.side.inward();
            let (ox, oz) = (x - ix * 0.3, z - iz * 0.3);
            if let Some(to) = ex
                .rooms
                .iter()
                .enumerate()
                .position(|(k, r)| k != from && r.contains(ox, oz, 0.0))
            {
                out.push(Passage {
                    from,
                    to,
                    x,
                    z,
                    side: door.side,
                });
            }
        }
    }
    out
}

/// Les portes extérieures d'une salle (qui ne mènent à aucune autre) : les entrées.
pub fn entrances(ex: &Exhibition, room_index: usize) -> Vec<(f32, f32, Side)> {
    let room = &ex.rooms[room_index];
    let links = passages(ex);
    room.doors
        .iter()
        .map(|d| {
            let (x, z) = room.point_on(d.side, d.position);
            (x, z, d.side)
        })
        .filter(|&(x, z, _)| {
            !links
                .iter()
                .any(|p| p.from == room_index && (p.x - x).abs() < 0.05 && (p.z - z).abs() < 0.05)
        })
        .collect()
}

/// L'entrée du musée : la première salle (dans l'ordre du fichier) qui a une porte
/// extérieure, et cette porte.
pub fn entrance(ex: &Exhibition) -> Option<(usize, f32, f32, Side)> {
    (0..ex.rooms.len()).find_map(|r| {
        entrances(ex, r)
            .into_iter()
            .next()
            .map(|(x, z, side)| (r, x, z, side))
    })
}

/// Le plus court chemin (en nombre de portes) de la salle `a` à la salle `b`.
pub fn route(ex: &Exhibition, a: usize, b: usize) -> Option<Vec<Passage>> {
    let links = passages(ex);
    let mut previous: Vec<Option<Passage>> = vec![None; ex.rooms.len()];
    let mut seen = vec![false; ex.rooms.len()];
    let mut queue = VecDeque::from([a]);
    seen[a] = true;
    while let Some(r) = queue.pop_front() {
        if r == b {
            let mut path = Vec::new();
            let mut at = b;
            while let Some(p) = previous[at] {
                path.push(p);
                at = p.from;
            }
            path.reverse();
            return Some(path);
        }
        for p in links.iter().filter(|p| p.from == r) {
            if !seen[p.to] {
                seen[p.to] = true;
                previous[p.to] = Some(*p);
                queue.push_back(p.to);
            }
        }
    }
    None
}

/// La salle qui contient le point (x, z).
pub fn room_at(ex: &Exhibition, x: f32, z: f32) -> Option<usize> {
    ex.rooms.iter().position(|r| r.contains(x, z, 0.0))
}

/// Pousse un cercle de rayon `radius` centré en (x, z) hors des obstacles.
pub fn push_out(obstacles: &[Rect], mut x: f32, mut z: f32, radius: f32) -> (f32, f32) {
    for _ in 0..3 {
        for &(x0, x1, z0, z1) in obstacles {
            let (cx, cz) = (x.clamp(x0, x1), z.clamp(z0, z1));
            let (dx, dz) = (x - cx, z - cz);
            let d = (dx * dx + dz * dz).sqrt();
            if d < radius {
                if d > 1e-5 {
                    x = cx + dx / d * radius;
                    z = cz + dz / d * radius;
                } else {
                    // Le centre est dans l'obstacle : sortir par le côté le plus proche.
                    let exits = [
                        (x - x0, -1.0, 0.0),
                        (x1 - x, 1.0, 0.0),
                        (z - z0, 0.0, -1.0),
                        (z1 - z, 0.0, 1.0),
                    ];
                    let (gap, ex_, ez) = exits
                        .into_iter()
                        .fold((f32::MAX, 0.0, 0.0), |m, e| if e.0 < m.0 { e } else { m });
                    x += ex_ * (gap + radius);
                    z += ez * (gap + radius);
                }
            }
        }
    }
    (x, z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artworks::EXHIBITIONS;

    #[test]
    fn every_room_can_be_reached_from_the_entrance() {
        for ex in EXHIBITIONS {
            let (start, ..) =
                entrance(ex).unwrap_or_else(|| panic!("{} : le musée n'a pas d'entrée", ex.slug));
            for (i, room) in ex.rooms.iter().enumerate() {
                assert!(
                    route(ex, start, i).is_some(),
                    "{} : « {} » est inaccessible",
                    ex.slug,
                    room.name
                );
            }
        }
    }

    #[test]
    fn doors_are_open_on_both_sides() {
        for ex in EXHIBITIONS {
            let obstacles = obstacles(ex);
            for p in passages(ex) {
                // Le milieu d'une porte n'est dans aucun mur.
                let inside = obstacles.iter().any(|&(x0, x1, z0, z1)| {
                    p.x > x0 + 0.01 && p.x < x1 - 0.01 && p.z > z0 + 0.01 && p.z < z1 - 0.01
                });
                assert!(
                    !inside,
                    "{} : la porte en ({}, {}) est murée",
                    ex.slug, p.x, p.z
                );
            }
        }
    }

    #[test]
    fn push_out_leaves_walls_behind() {
        let wall = [(0.0, 0.2, -5.0, 5.0)];
        let (x, _) = push_out(&wall, 0.25, 0.0, 0.3);
        assert!(x >= 0.2 + 0.3 - 1e-4);
        let (x, _) = push_out(&wall, 0.05, 0.0, 0.3);
        assert!(x <= -0.3 + 1e-4);
    }
}
