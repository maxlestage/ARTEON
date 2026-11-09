//! La visite guidée d'un musée, calculée à partir de son plan : l'entrée, puis chaque
//! salle d'exposition dans l'ordre du fichier, avec un arrêt devant chaque tableau et
//! autour de chaque sculpture ; d'une salle à l'autre, le chemin passe par les portes, et
//! la visite revient à l'entrée. Une fonction pure du temps : elle se teste sans
//! navigateur.

use std::sync::OnceLock;

use crate::artworks::{EXHIBITIONS, Exhibition, Sculpture};
use crate::museum::{self, Passage};

/// Ce que regarde le visiteur à un arrêt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    /// Un tableau (son index dans `artworks`).
    Art(usize),
    /// Une sculpture (son index dans `sculptures`).
    Sculpture(usize),
}

/// Un arrêt de la visite : où se tient le visiteur, ce qu'il regarde, combien de temps.
#[derive(Clone, Copy, Debug)]
pub struct Stop {
    pub eye: [f32; 3],
    pub look: [f32; 3],
    pub hold: f32,
    pub focus: Option<Focus>,
}

pub const EYE: f32 = 1.65;
/// La tangente de la moitié du champ de vision vertical de la caméra (45°).
pub const HALF_FOV_TAN: f32 = 0.414_213_56;

fn stop(eye: [f32; 3], look: [f32; 3], hold: f32) -> Stop {
    Stop {
        eye,
        look,
        hold,
        focus: None,
    }
}

/// La distance à laquelle regarder un tableau de `width` × `height` sur un écran de
/// proportions `aspect` (largeur / hauteur) : assez loin pour qu'il tienne dans l'image
/// avec de l'air autour ; sur un téléphone en portrait, le visiteur recule.
pub fn distance_for(width: f32, height: f32, aspect: f32) -> f32 {
    let fit_width = (width / 2.0) / (HALF_FOV_TAN * aspect.max(0.2) * 0.62);
    let fit_height = (height / 2.0) / (HALF_FOV_TAN * 0.72);
    fit_width.max(fit_height).max(1.8)
}

/// Le recul possible devant un tableau, jusqu'au mur d'en face.
fn room_for(ex: &Exhibition, art: usize) -> f32 {
    let a = &ex.artworks[art];
    let room = &ex.rooms[a.hook.room];
    let across = match a.hook.side {
        crate::artworks::Side::East | crate::artworks::Side::West => room.width,
        _ => room.depth,
    };
    (across - 2.0 * crate::artworks::WALL - 0.7).max(1.2)
}

/// L'œil et le point regardé devant le tableau `art`, sur un écran `aspect`.
pub fn art_view(ex: &Exhibition, art: usize, aspect: f32) -> ([f32; 3], [f32; 3]) {
    let a = &ex.artworks[art];
    let center = museum::artwork_center(ex, a);
    let mut d = distance_for(a.width, a.height, aspect).min(room_for(ex, art));
    let (ix, iz) = a.hook.side.inward();
    // Reculer, mais pas jusque dans un banc ou un socle : s'arrêter 40 cm avant.
    let blocked = |k: f32| {
        let (x, z) = (center[0] + ix * k, center[2] + iz * k);
        museum::cached_obstacles(ex)
            .iter()
            .any(|&(x0, x1, z0, z1)| x > x0 - 0.4 && x < x1 + 0.4 && z > z0 - 0.4 && z < z1 + 0.4)
    };
    let mut k = 0.6;
    while k < d {
        if blocked(k) {
            d = (k - 0.1).max(1.2);
            break;
        }
        k += 0.1;
    }
    ([center[0] + ix * d, EYE, center[2] + iz * d], center)
}

/// L'œil et le point regardé devant une sculpture : de face (ou du côté le plus dégagé),
/// assez loin pour la voir en entier.
pub fn sculpture_view(ex: &Exhibition, s: &Sculpture) -> ([f32; 3], [f32; 3]) {
    let room = &ex.rooms[s.room];
    let top = s.plinth + s.height;
    let d = ((top / 2.0) / (HALF_FOV_TAN * 0.8)).max(s.radius + 1.5);
    let target = [s.x, (s.plinth + top) / 2.0, s.z];
    let eye_y = EYE.min(top + 0.2).max(1.2);
    // De face d'abord, puis en tournant d'un quart de tour, jusqu'à trouver la place.
    for turn in [0.0f32, 0.5, -0.5, 1.0, -1.0, 2.0] {
        let angle = s.rotation + turn * std::f32::consts::FRAC_PI_2;
        let (fx, fz) = (angle.sin(), angle.cos());
        for k in [1.0f32, 0.85, 0.7] {
            let (x, z) = (s.x + fx * d * k, s.z + fz * d * k);
            if room.contains(x, z, 0.8) {
                return ([x, eye_y, z], target);
            }
        }
    }
    ([room.x, eye_y, room.z], target)
}

/// Les trois points d'un passage de porte : un peu avant, au milieu, un peu après.
fn through(ex: &Exhibition, p: &Passage) -> [[f32; 3]; 3] {
    let (ix, iz) = p.side.inward();
    let reach = |room: usize, along: f32| -> f32 {
        let r = &ex.rooms[room];
        let half = match p.side {
            crate::artworks::Side::North | crate::artworks::Side::South => r.depth,
            _ => r.width,
        } / 2.0;
        along.min(half - 0.4).max(0.3)
    };
    let before = reach(p.from, 1.6);
    let after = reach(p.to, 1.6);
    [
        [p.x + ix * before, EYE, p.z + iz * before],
        [p.x, EYE, p.z],
        [p.x - ix * after, EYE, p.z - iz * after],
    ]
}

fn ahead(from: [f32; 3], to: [f32; 3]) -> [f32; 3] {
    let (dx, dz) = (to[0] - from[0], to[2] - from[2]);
    let len = (dx * dx + dz * dz).sqrt().max(1e-3);
    [to[0] + dx / len * 6.0, EYE, to[2] + dz / len * 6.0]
}

/// Le parcours : des arrêts reliés en ligne droite (la caméra glisse d'un arrêt au suivant).
fn plan(ex: &Exhibition) -> Vec<Stop> {
    let galleries: Vec<usize> = ex.galleries().map(|(i, _)| i).collect();
    let Some(&first) = galleries.first() else {
        return vec![stop([0.0, EYE, 0.0], [0.0, EYE, -5.0], 3.0)];
    };
    // L'entrée du musée (la porte extérieure ; au sud de la première salle par défaut).
    let room = &ex.rooms[first];
    let (hall, ex_x, ex_z, ex_side) = museum::entrance(ex).unwrap_or((
        first,
        room.x,
        room.z + room.depth / 2.0,
        crate::artworks::Side::South,
    ));
    let (ix, iz) = ex_side.inward();
    let start_eye = [ex_x + ix * 0.8, EYE, ex_z + iz * 0.8];
    let target = &ex.rooms[hall];
    let mut list = vec![stop(start_eye, [target.x, EYE, target.z], 2.5)];
    // De l'entrée à la première salle d'exposition.
    if hall != first {
        walk_to(ex, &mut list, hall, first);
        let eye = list.last().map_or(start_eye, |s| s.eye);
        list.push(stop(eye, [room.x, 1.4, room.z], 1.2));
    }
    let mut at = first;
    for (n, &g) in galleries.iter().enumerate() {
        if n > 0 {
            walk_to(ex, &mut list, at, g);
            let r = &ex.rooms[g];
            let eye = list.last().map_or([r.x, EYE, r.z], |s| s.eye);
            list.push(stop(eye, [r.x, 1.4, r.z], 1.2));
        }
        visit_room(ex, &mut list, g);
        at = g;
    }
    // Retour à l'entrée, par les portes.
    walk_to(ex, &mut list, at, hall);
    list
}

/// Ajoute le chemin de la salle `a` à la salle `b`, porte après porte.
fn walk_to(ex: &Exhibition, list: &mut Vec<Stop>, a: usize, b: usize) {
    for p in museum::route(ex, a, b).unwrap_or_default() {
        let [before, middle, after] = through(ex, &p);
        // Jusqu'à la porte, en contournant le banc et les sculptures de la salle.
        if let Some(last) = list.last().map(|s| s.eye) {
            detour(ex, list, p.from, last, before);
        }
        list.push(stop(before, ahead(before, middle), 0.0));
        list.push(stop(middle, ahead(middle, after), 0.0));
        list.push(stop(after, ahead(middle, after), 0.0));
    }
}

/// Les arrêts d'une salle : ses tableaux et ses sculptures, du plus proche au plus proche
/// en partant de la position actuelle, avec un détour autour des sculptures.
fn visit_room(ex: &Exhibition, list: &mut Vec<Stop>, room: usize) {
    let mut todo: Vec<Stop> = ex
        .works_in_room(room)
        .into_iter()
        .map(|a| {
            let i = ex
                .artworks
                .iter()
                .position(|b| std::ptr::eq(a, b))
                .unwrap_or(0);
            let (eye, look) = art_view(ex, i, 16.0 / 9.0);
            Stop {
                eye,
                look,
                hold: 4.0,
                focus: Some(Focus::Art(i)),
            }
        })
        .chain(
            ex.sculptures
                .iter()
                .enumerate()
                .filter(|(_, s)| s.room == room)
                .map(|(i, s)| {
                    let (eye, look) = sculpture_view(ex, s);
                    Stop {
                        eye,
                        look,
                        hold: 4.5,
                        focus: Some(Focus::Sculpture(i)),
                    }
                }),
        )
        .collect();
    while !todo.is_empty() {
        let here = list.last().map_or([0.0; 3], |s| s.eye);
        let (k, _) = todo
            .iter()
            .enumerate()
            .map(|(k, s)| (k, (s.eye[0] - here[0]).hypot(s.eye[2] - here[2])))
            .fold((0, f32::MAX), |m, c| if c.1 < m.1 { c } else { m });
        let next = todo.remove(k);
        detour(ex, list, room, here, next.eye);
        list.push(next);
    }
}

/// Ce qu'il faut contourner dans une salle, vu de dessus : (x, z, rayon). Les sculptures
/// et le banc.
fn round_obstacles(ex: &Exhibition, room: usize) -> Vec<(f32, f32, f32)> {
    let mut out: Vec<(f32, f32, f32)> = ex
        .sculptures
        .iter()
        .filter(|s| s.room == room)
        .map(|s| (s.x, s.z, s.radius))
        .collect();
    if museum::bench(ex, room).is_some() {
        let r = &ex.rooms[room];
        out.push((r.x, r.z, 1.25));
    }
    out
}

/// Si le trajet droit de `a` à `b` passe trop près d'une sculpture ou du banc, un point de
/// passage à côté.
fn detour(ex: &Exhibition, list: &mut Vec<Stop>, room: usize, a: [f32; 3], b: [f32; 3]) {
    for (sx, sz, radius) in round_obstacles(ex, room) {
        struct S {
            x: f32,
            z: f32,
            radius: f32,
        }
        let s = S {
            x: sx,
            z: sz,
            radius,
        };
        let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
        let len2 = (dx * dx + dz * dz).max(1e-6);
        let t = (((s.x - a[0]) * dx + (s.z - a[2]) * dz) / len2).clamp(0.0, 1.0);
        let (px, pz) = (a[0] + dx * t, a[2] + dz * t);
        let gap = (px - s.x).hypot(pz - s.z);
        let clear = s.radius + 0.7;
        if gap < clear && t > 0.05 && t < 0.95 {
            // De côté, perpendiculairement au trajet, du côté où il passait déjà.
            let len = len2.sqrt();
            let (nx, nz) = (-dz / len, dx / len);
            let sign = if (px - s.x) * nx + (pz - s.z) * nz >= 0.0 {
                1.0
            } else {
                -1.0
            };
            let (wx, wz) = (
                s.x + nx * sign * (clear + 0.3),
                s.z + nz * sign * (clear + 0.3),
            );
            let r = &ex.rooms[room];
            if r.contains(wx, wz, 0.6) {
                let point = [wx, EYE, wz];
                list.push(stop(point, ahead(a, point), 0.0));
            }
        }
    }
}

/// Les bancs et les sculptures de tout le musée, vus de dessus : (x, z, rayon).
fn all_round_obstacles(ex: &Exhibition) -> &'static [(f32, f32, f32)] {
    static ALL: OnceLock<Vec<Vec<(f32, f32, f32)>>> = OnceLock::new();
    let all = ALL.get_or_init(|| {
        EXHIBITIONS
            .iter()
            .map(|e| {
                (0..e.rooms.len())
                    .flat_map(|r| round_obstacles(e, r))
                    .collect()
            })
            .collect()
    });
    let index = EXHIBITIONS
        .iter()
        .position(|e| e.slug == ex.slug)
        .unwrap_or(0);
    &all[index]
}

/// Écarte un point de trajet des bancs et des sculptures : s'il entre dans leur cercle de
/// sécurité, il est repoussé sur son bord. Le trajet contourne l'obstacle en arc, quelle
/// que soit la forme de l'écran (les arrêts, eux, changent avec elle).
fn steer_clear(ex: &Exhibition, mut p: [f32; 3]) -> [f32; 3] {
    for &(x, z, r) in all_round_obstacles(ex) {
        let clear = r + 0.45;
        let (dx, dz) = (p[0] - x, p[2] - z);
        let d = dx.hypot(dz);
        if d < clear {
            let (ux, uz) = if d > 1e-4 {
                (dx / d, dz / d)
            } else {
                (1.0, 0.0)
            };
            p[0] = x + ux * clear;
            p[2] = z + uz * clear;
        }
    }
    p
}

/// Le parcours de `ex`, calculé une fois pour toutes les expositions.
pub fn stops(ex: &Exhibition) -> &'static [Stop] {
    static PLANS: OnceLock<Vec<Vec<Stop>>> = OnceLock::new();
    let plans = PLANS.get_or_init(|| EXHIBITIONS.iter().map(plan).collect());
    let index = EXHIBITIONS
        .iter()
        .position(|e| e.slug == ex.slug)
        .unwrap_or(0);
    &plans[index]
}

/// La durée du trajet entre deux arrêts : selon la distance et l'angle à tourner.
fn travel(a: &Stop, b: &Stop) -> f32 {
    let distance = (b.eye[0] - a.eye[0]).hypot(b.eye[2] - a.eye[2]);
    let (ya, _) = heading(a.eye, a.look);
    let (yb, _) = heading(b.eye, b.look);
    let turn = ((yb - ya + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI)
        .abs();
    let walking = distance / 1.3;
    let turning = turn.to_degrees() / 50.0;
    // Les points de passage (sans arrêt) s'enchaînent vite ; les arrêts prennent le temps.
    let least = if a.hold == 0.0 || b.hold == 0.0 {
        0.8
    } else {
        2.5
    };
    walking.max(turning).clamp(least, 12.0)
}

/// La durée d'un tour complet, en secondes.
pub fn tour_duration(ex: &Exhibition) -> f32 {
    let list = stops(ex);
    (0..list.len())
        .map(|i| list[i].hold + travel(&list[i], &list[(i + 1) % list.len()]))
        .sum()
}

/// Le moment de la visite où le visiteur arrive à l'arrêt `index`.
fn arrival(ex: &Exhibition, index: usize) -> f32 {
    let list = stops(ex);
    (0..index)
        .map(|i| list[i].hold + travel(&list[i], &list[(i + 1) % list.len()]))
        .sum()
}

fn lerp3(a: [f32; 3], b: [f32; 3], k: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * k,
        a[1] + (b[1] - a[1]) * k,
        a[2] + (b[2] - a[2]) * k,
    ]
}

/// La direction du regard : (cap autour de y, inclinaison), en radians.
pub fn heading(eye: [f32; 3], look: [f32; 3]) -> (f32, f32) {
    let d = [look[0] - eye[0], look[1] - eye[1], look[2] - eye[2]];
    let flat = (d[0] * d[0] + d[2] * d[2]).sqrt();
    (d[0].atan2(d[2]), d[1].atan2(flat))
}

/// Le regard entre deux arrêts : le cap tourne par le plus court chemin. On interpole la
/// direction et non le point regardé : entre deux murs opposés, ce point passerait par
/// l'œil, et la caméra basculerait vers le plafond.
fn blend_look(eye: [f32; 3], a: ([f32; 3], [f32; 3]), b: ([f32; 3], [f32; 3]), k: f32) -> [f32; 3] {
    use std::f32::consts::{PI, TAU};
    let ((yaw_a, pitch_a), (yaw_b, pitch_b)) = (heading(a.0, a.1), heading(b.0, b.1));
    let turn = (yaw_b - yaw_a + PI).rem_euclid(TAU) - PI;
    let (yaw, pitch) = (yaw_a + turn * k, pitch_a + (pitch_b - pitch_a) * k);
    [
        eye[0] + yaw.sin() * pitch.cos(),
        eye[1] + pitch.sin(),
        eye[2] + yaw.cos() * pitch.cos(),
    ]
}

/// Ce que voit le visiteur à un moment de la visite.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub eye: [f32; 3],
    pub look: [f32; 3],
    /// L'œuvre regardée, s'il y en a une.
    pub focus: Option<Focus>,
}

/// L'œil et le point regardé d'un arrêt, sur un écran de proportions `aspect`.
fn placed(ex: &Exhibition, stop: &Stop, aspect: f32) -> ([f32; 3], [f32; 3]) {
    match stop.focus {
        Some(Focus::Art(i)) => art_view(ex, i, aspect),
        _ => (stop.eye, stop.look),
    }
}

/// Ce que voit le visiteur au temps `t` de la visite (en boucle), sur un écran de
/// proportions `aspect` (largeur / hauteur).
pub fn tour_pose(ex: &Exhibition, t: f32, aspect: f32) -> Pose {
    let list = stops(ex);
    let mut rest = t.rem_euclid(tour_duration(ex));
    for (i, here) in list.iter().enumerate() {
        let a = placed(ex, here, aspect);
        if rest < here.hold {
            return Pose {
                eye: a.0,
                look: a.1,
                focus: here.focus,
            };
        }
        rest -= here.hold;
        let next = &list[(i + 1) % list.len()];
        let duration = travel(here, next);
        if rest < duration {
            let b = placed(ex, next, aspect);
            let k = rest / duration;
            // Le cartel garde l'œuvre quittée jusqu'à mi-chemin, puis montre la suivante.
            let focus = if k < 0.5 {
                here.focus
            } else {
                next.focus.or(here.focus)
            };
            // Départ et arrivée en douceur aux arrêts ; vitesse constante aux points de passage.
            let k = match (here.hold > 0.0, next.hold > 0.0) {
                (true, true) => k * k * (3.0 - 2.0 * k),
                (true, false) => k * k,
                (false, true) => 1.0 - (1.0 - k) * (1.0 - k),
                (false, false) => k,
            };
            let eye = steer_clear(ex, lerp3(a.0, b.0, k));
            return Pose {
                eye,
                look: blend_look(eye, a, b, k),
                focus,
            };
        }
        rest -= duration;
    }
    let (eye, look) = placed(ex, &list[0], aspect);
    Pose {
        eye,
        look,
        focus: list[0].focus,
    }
}

/// Le moment où l'on arrive devant l'œuvre suivante (`step` > 0) ou précédente
/// (`step` < 0), à partir du temps `t`.
pub fn jump_time(ex: &Exhibition, t: f32, step: i32) -> f32 {
    let list = stops(ex);
    let total = tour_duration(ex);
    let now = t.rem_euclid(total);
    let mut works: Vec<f32> = (0..list.len())
        .filter(|&i| list[i].focus.is_some())
        .map(|i| arrival(ex, i))
        .collect();
    if works.is_empty() || step == 0 {
        return t;
    }
    works.sort_by(f32::total_cmp);
    let here = works.iter().rposition(|&w| w <= now + 0.01);
    let target = match here {
        Some(i) => (i as i32 + step).rem_euclid(works.len() as i32) as usize,
        None if step > 0 => (step as usize - 1) % works.len(),
        None => (works.len() as i32 + step).rem_euclid(works.len() as i32) as usize,
    };
    t - now + works[target]
}

/// Le moment de la visite où l'on arrive devant `focus`.
pub fn time_of(ex: &Exhibition, focus: Focus) -> Option<f32> {
    let list = stops(ex);
    (0..list.len())
        .find(|&i| list[i].focus == Some(focus))
        .map(|i| arrival(ex, i))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::museum::obstacles;

    /// Un téléphone en portrait, un écran carré, un écran large.
    const ASPECTS: [f32; 3] = [0.46, 1.0, 16.0 / 9.0];

    #[test]
    fn the_tour_starts_at_the_entrance_and_loops() {
        for ex in EXHIBITIONS {
            assert_eq!(tour_pose(ex, 0.0, 1.5).eye, stops(ex)[0].eye);
            assert_eq!(tour_pose(ex, tour_duration(ex), 1.5).eye, stops(ex)[0].eye);
        }
    }

    #[test]
    fn the_tour_visits_every_work() {
        for ex in EXHIBITIONS {
            for i in 0..ex.artworks.len() {
                assert!(
                    time_of(ex, Focus::Art(i)).is_some(),
                    "{} : {}",
                    ex.slug,
                    ex.artworks[i].title
                );
            }
            for i in 0..ex.sculptures.len() {
                assert!(
                    time_of(ex, Focus::Sculpture(i)).is_some(),
                    "{} : {}",
                    ex.slug,
                    ex.sculptures[i].title
                );
            }
        }
    }

    /// « Suivante » et « précédente » passent d'une œuvre à l'autre, dans l'ordre de la
    /// visite, en boucle.
    #[test]
    fn next_and_previous_go_from_work_to_work() {
        for ex in EXHIBITIONS {
            let order: Vec<Option<Focus>> = stops(ex)
                .iter()
                .filter(|s| s.focus.is_some())
                .map(|s| s.focus)
                .collect();
            let mut t = 0.0;
            for expected in order.iter().chain(order.iter().take(1)) {
                t = jump_time(ex, t, 1);
                assert_eq!(tour_pose(ex, t + 0.01, 1.5).focus, *expected, "{}", ex.slug);
            }
            let back = jump_time(ex, t, -1);
            assert_eq!(
                tour_pose(ex, back + 0.01, 1.5).focus,
                *order.last().unwrap(),
                "{}",
                ex.slug
            );
        }
    }

    /// Le visiteur reste toujours dans une salle, ne traverse jamais un mur ni un socle, et
    /// regarde à peu près à l'horizontale, sur tous les écrans.
    #[test]
    fn the_visitor_never_goes_through_walls() {
        for ex in EXHIBITIONS {
            let walls = obstacles(ex);
            for aspect in ASPECTS {
                let mut t = 0.0;
                while t < tour_duration(ex) {
                    let Pose { eye, look, .. } = tour_pose(ex, t, aspect);
                    let [x, y, z] = eye;
                    assert!(
                        museum::room_at(ex, x, z).is_some(),
                        "{} t = {t} : ({x}, {z}) hors du musée",
                        ex.slug
                    );
                    assert!(y > 0.5 && y < 3.0, "{} t = {t}", ex.slug);
                    let hit = walls.iter().find(|&&(x0, x1, z0, z1)| {
                        x > x0 + 0.05 && x < x1 - 0.05 && z > z0 + 0.05 && z < z1 - 0.05
                    });
                    assert!(
                        hit.is_none(),
                        "{} t = {t} : ({x:.2}, {z:.2}) dans un obstacle {hit:?}",
                        ex.slug
                    );
                    let (_, pitch) = heading(eye, look);
                    assert!(
                        pitch.to_degrees().abs() < 30.0,
                        "{} t = {t} : inclinaison {:.0}°",
                        ex.slug,
                        pitch.to_degrees()
                    );
                    t += 0.05;
                }
            }
        }
    }
}
