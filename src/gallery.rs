//! La galerie en 3D, avec le moteur de jeu d'Active réexporté par Paquets
//! (`paquets::ecs`). Deux scènes, pour chaque exposition :
//!
//! - `tour_app(ex)` : les salles de l'exposition reliées par des couloirs, chaque
//!   tableau reconstruit en mosaïque sur son mur, et une caméra qui fait la visite
//!   guidée, en boucle ;
//! - `showcase_app(ex)` : l'œuvre de la vitrine sur un chevalet tournant.
//!
//! Le moteur n'affiche pas d'images (pas de textures) : un tableau est une grille de
//! touches de couleur de 1 cm. Les touches voisines de même couleur sont fusionnées en
//! rectangles, et chaque couleur de la palette forme un seul maillage (64 au plus par
//! tableau) : la scène reste fluide malgré la finesse de la mosaïque.
//!
//! Deux limites du moteur dictent la construction :
//!
//! - `DefaultPlugins` contient la physique, qui donne un corps soumis à la gravité à
//!   chaque pavé : les murs, les sols et les tableaux tomberaient dans le vide. Un musée
//!   n'en a pas besoin, elle est donc retirée (`disable::<PhysicsPlugin>()`) ;
//! - le shader n'éclaire qu'avec 8 lampes ponctuelles au plus (les suivantes sont
//!   ignorées) : `lamp_plan` répartit ces 8 lampes entre les salles et les couloirs.

use std::collections::{BTreeMap, HashMap};
use std::f32::consts::FRAC_PI_2;
use std::sync::OnceLock;

use paquets::ecs::prelude::*;
use paquets::scene::BufferGeometry;

use crate::artworks::{
    Artwork, CORRIDOR_HALF_WIDTH, CORRIDOR_LENGTH, EXHIBITIONS, Exhibition, HANGING_HEIGHT, Hook,
    Mosaic, ROOM_HALF_DEPTH, ROOM_HALF_WIDTH, WALL_HEIGHT, WALL_THICKNESS, Wall, room_center_z,
};

// ─── Composants et ressources ─────────────────────────────────────────────────

/// L'exposition que montre la scène.
struct Current(&'static Exhibition);
paquets::active::resource!(Current);

/// La caméra de la visite guidée ; `t` est le temps écoulé depuis le début du tour.
struct Visitor {
    t: f32,
}
paquets::active::component!(Visitor);

/// Un morceau posé sur un plateau tournant : sa place est recalculée à chaque image
/// autour de `center`, ce qui fait tourner l'ensemble d'un bloc.
struct Turntable {
    center: [f32; 3],
    offset: [f32; 3],
    speed: f32,
    angle: f32,
}
paquets::active::component!(Turntable);

// ─── Petits outils de construction ───────────────────────────────────────────

type Meshes<'a> = &'a mut Assets<Mesh>;
type Materials<'a> = &'a mut Assets<StandardMaterial>;

fn color([r, g, b]: [u8; 3]) -> Color {
    Color::srgb_u8(r, g, b)
}

/// Un pavé de taille `size`, centré en `pos`, de couleur `rgb`.
fn block(
    commands: &mut Commands,
    meshes: Meshes,
    materials: Materials,
    size: [f32; 3],
    pos: [f32; 3],
    rgb: [u8; 3],
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size[0], size[1], size[2]))),
        MeshMaterial3d(materials.add(color(rgb))),
        Transform::from_xyz(pos[0], pos[1], pos[2]),
    ));
}

/// Le nombre maximal de lampes ponctuelles que le moteur prend en compte.
pub const MAX_LAMPS: usize = 8;

/// Combien de lampes par salle, et si les couloirs en ont une, pour `rooms` salles,
/// sans dépasser `MAX_LAMPS` (au-delà, la lumière d'ambiance éclaire seule).
pub fn lamp_plan(rooms: usize) -> (usize, bool) {
    let corridors = rooms.saturating_sub(1);
    if 2 * rooms + corridors <= MAX_LAMPS {
        (2, true)
    } else if rooms + corridors <= MAX_LAMPS {
        (1, true)
    } else {
        (1, false)
    }
}

/// Une lampe de plafond, assez douce pour ne pas brûler les couleurs des tableaux.
fn lamp(commands: &mut Commands, pos: [f32; 3], intensity: f32) {
    commands.spawn((
        PointLight {
            intensity,
            range: 12.0,
            ..default()
        },
        Transform::from_xyz(pos[0], pos[1], pos[2]),
    ));
}

/// La lumière d'ambiance, qui éclaire aussi les coins loin des lampes.
fn ambient() -> GlobalAmbientLight {
    GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 520.0,
        ..default()
    }
}

/// Les plugins des deux scènes : tout `DefaultPlugins`, sauf la physique.
fn plugins() -> PluginGroupBuilder {
    DefaultPlugins.disable::<PhysicsPlugin>()
}

const FLOOR: [u8; 3] = [132, 98, 66];
const CEILING: [u8; 3] = [250, 248, 242];
const GILDING: [u8; 3] = [168, 128, 56];
const CANVAS_BACK: [u8; 3] = [92, 84, 72];
const PLAQUE: [u8; 3] = [244, 240, 228];
const BENCH: [u8; 3] = [58, 40, 28];
const CORRIDOR_WALL: [u8; 3] = [30, 44, 84];
const DOOR_HEIGHT: f32 = 3.0;
/// La distance entre la face intérieure d'un mur latéral et l'axe de la galerie.
const INNER: f32 = ROOM_HALF_WIDTH - WALL_THICKNESS / 2.0;

// ─── Les tableaux en mosaïque ─────────────────────────────────────────────────

/// Les sommets, normales et triangles d'un maillage en construction.
#[derive(Default)]
struct Builder {
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
}

impl Builder {
    /// Un rectangle de `x0..x1` × `y0..y1`, à la profondeur `z`, tourné vers +z.
    fn rect(&mut self, x0: f32, x1: f32, y0: f32, y1: f32, z: f32) {
        let base = (self.positions.len() / 3) as u32;
        for p in [[x0, y0, z], [x1, y0, z], [x1, y1, z], [x0, y1, z]] {
            self.positions.extend_from_slice(&p);
            self.normals.extend_from_slice(&[0.0, 0.0, 1.0]);
        }
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn mesh(self) -> Mesh {
        Mesh::from_geometry(BufferGeometry::new(
            self.positions,
            self.normals,
            Vec::new(),
            self.indices,
        ))
    }
}

/// Les rectangles d'une mosaïque : les cases voisines de même couleur fusionnées, d'abord
/// en lignes, puis les lignes identiques empilées. `(couleur, col0, col1, row0, row1)`,
/// bornes de fin exclues.
fn rectangles(mosaic: &Mosaic) -> Vec<(u8, usize, usize, usize, usize)> {
    let (cols, rows) = (usize::from(mosaic.cols), usize::from(mosaic.rows));
    let mut done = Vec::new();
    // Les rectangles encore ouverts, par (couleur, début, fin) : leur première ligne.
    let mut open: HashMap<(u8, usize, usize), usize> = HashMap::new();
    for row in 0..rows {
        let line = &mosaic.cells[row * cols..(row + 1) * cols];
        let mut next: HashMap<(u8, usize, usize), usize> = HashMap::new();
        let mut start = 0;
        while start < cols {
            let c = line[start];
            let mut end = start + 1;
            while end < cols && line[end] == c {
                end += 1;
            }
            let key = (c, start, end);
            next.insert(key, open.remove(&key).unwrap_or(row));
            start = end;
        }
        done.extend(open.drain().map(|((c, a, b), first)| (c, a, b, first, row)));
        open = next;
    }
    done.extend(
        open.drain()
            .map(|((c, a, b), first)| (c, a, b, first, rows)),
    );
    done.sort_unstable();
    done
}

/// Les maillages d'une mosaïque de `width` × `height` mètres, centrée sur l'origine,
/// tournée vers +z : un maillage par couleur de la palette. Les tons clairs sont un
/// peu plus en avant, comme les empâtements du peintre.
fn mosaic_meshes(mosaic: &Mosaic, width: f32, height: f32) -> Vec<([u8; 3], Mesh)> {
    let (cw, ch) = (
        width / f32::from(mosaic.cols),
        height / f32::from(mosaic.rows),
    );
    let mut by_color: BTreeMap<u8, Builder> = BTreeMap::new();
    for (c, a, b, r0, r1) in rectangles(mosaic) {
        let [red, green, blue] = mosaic.palette[usize::from(c)];
        let light =
            (f32::from(red) * 0.3 + f32::from(green) * 0.55 + f32::from(blue) * 0.15) / 255.0;
        let x0 = -width / 2.0 + a as f32 * cw;
        let x1 = -width / 2.0 + b as f32 * cw;
        let y1 = height / 2.0 - r0 as f32 * ch;
        let y0 = height / 2.0 - r1 as f32 * ch;
        by_color
            .entry(c)
            .or_default()
            .rect(x0, x1, y0, y1, 0.001 + 0.0015 * light);
    }
    by_color
        .into_iter()
        .map(|(c, builder)| (mosaic.palette[usize::from(c)], builder.mesh()))
        .collect()
}

/// Où se trouve un point posé sur le mur d'un tableau : `u` vers la droite du visiteur
/// qui le regarde, `v` vers le haut depuis le centre du tableau, `layer` en avant du mur.
/// Renvoie la position et l'angle (autour de y) qui tourne +z vers la salle.
fn on_wall(hook: Hook, u: f32, v: f32, layer: f32) -> ([f32; 3], f32) {
    let along = hook.along();
    let y = HANGING_HEIGHT + v;
    match hook.wall {
        Wall::Left => ([-INNER + layer, y, along - u], FRAC_PI_2),
        Wall::Right => ([INNER - layer, y, along + u], -FRAC_PI_2),
        Wall::Back => {
            let back = room_center_z(hook.room) - ROOM_HALF_DEPTH + WALL_THICKNESS / 2.0;
            ([along + u, y, back + layer], 0.0)
        }
    }
}

fn turned(pos: [f32; 3], angle: f32) -> Transform {
    let mut transform = Transform::from_xyz(pos[0], pos[1], pos[2]);
    transform.rotate_y(angle);
    transform
}

/// Accroche une œuvre : un cadre doré, le fond de la toile, la mosaïque et un cartel à
/// droite.
fn hang(commands: &mut Commands, meshes: Meshes, materials: Materials, art: &Artwork) {
    let (w, h) = (art.width, art.height);
    let parts: [([f32; 3], f32, [u8; 3]); 2] = [
        ([w + 0.16, h + 0.16, 0.06], 0.03, GILDING),
        ([w + 0.01, h + 0.01, 0.01], 0.065, CANVAS_BACK),
    ];
    for (size, layer, rgb) in parts {
        let (pos, angle) = on_wall(art.hook, 0.0, 0.0, layer);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size[0], size[1], size[2]))),
            MeshMaterial3d(materials.add(color(rgb))),
            turned(pos, angle),
        ));
    }
    let (pos, angle) = on_wall(art.hook, 0.0, 0.0, 0.07);
    for (rgb, mesh) in mosaic_meshes(&art.mosaic, w, h) {
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(color(rgb))),
            turned(pos, angle),
        ));
    }
    let (pos, angle) = on_wall(art.hook, w / 2.0 + 0.32, -0.35, 0.01);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.24, 0.15, 0.02))),
        MeshMaterial3d(materials.add(color(PLAQUE))),
        turned(pos, angle),
    ));
}

// ─── La galerie ──────────────────────────────────────────────────────────────

/// Construit une salle : sol, plafond, murs (avec des portes), une banquette et ses lampes.
fn build_room(
    commands: &mut Commands,
    meshes: Meshes,
    materials: Materials,
    ex: &Exhibition,
    index: usize,
) {
    let room = &ex.rooms[index];
    let cz = room.center_z;
    let (w, d, h, t) = (
        ROOM_HALF_WIDTH,
        ROOM_HALF_DEPTH,
        WALL_HEIGHT,
        WALL_THICKNESS,
    );
    let wall = room.walls;

    block(
        commands,
        meshes,
        materials,
        [2.0 * w, 0.1, 2.0 * d],
        [0.0, -0.05, cz],
        FLOOR,
    );
    block(
        commands,
        meshes,
        materials,
        [2.0 * w, 0.1, 2.0 * d],
        [0.0, h + 0.05, cz],
        CEILING,
    );
    for side in [-1.0, 1.0] {
        block(
            commands,
            meshes,
            materials,
            [t, h, 2.0 * d + t],
            [side * w, h / 2.0, cz],
            wall,
        );
        // Une plinthe sombre au pied des murs.
        block(
            commands,
            meshes,
            materials,
            [0.03, 0.12, 2.0 * d],
            [side * (INNER - 0.015), 0.06, cz],
            BENCH,
        );
    }
    // Les murs avant et arrière : une porte au milieu, sauf au fond de la dernière salle.
    let last = index == ex.rooms.len() - 1;
    for (z, door) in [(cz + d, true), (cz - d, !last)] {
        if door {
            let seg = w - CORRIDOR_HALF_WIDTH;
            for side in [-1.0, 1.0] {
                let x = side * (CORRIDOR_HALF_WIDTH + seg / 2.0);
                block(
                    commands,
                    meshes,
                    materials,
                    [seg, h, t],
                    [x, h / 2.0, z],
                    wall,
                );
            }
            let lintel = h - DOOR_HEIGHT;
            block(
                commands,
                meshes,
                materials,
                [2.0 * CORRIDOR_HALF_WIDTH, lintel, t],
                [0.0, DOOR_HEIGHT + lintel / 2.0, z],
                wall,
            );
        } else {
            block(
                commands,
                meshes,
                materials,
                [2.0 * w, h, t],
                [0.0, h / 2.0, z],
                wall,
            );
        }
    }
    // Une banquette au centre, pour s'asseoir devant les toiles.
    block(
        commands,
        meshes,
        materials,
        [0.5, 0.08, 2.2],
        [0.0, 0.42, cz],
        BENCH,
    );
    for z in [-0.95, 0.95] {
        block(
            commands,
            meshes,
            materials,
            [0.42, 0.38, 0.1],
            [0.0, 0.19, cz + z],
            BENCH,
        );
    }
    let (per_room, _) = lamp_plan(ex.rooms.len());
    let spots: &[f32] = if per_room == 2 { &[2.5, -2.5] } else { &[0.0] };
    for z in spots {
        lamp(commands, [0.0, h - 0.4, cz + z], 300_000.0);
    }
}

/// Construit le couloir qui part du fond de la salle `index` vers la suivante.
fn build_corridor(
    commands: &mut Commands,
    meshes: Meshes,
    materials: Materials,
    ex: &Exhibition,
    index: usize,
) {
    let cz = ex.rooms[index].center_z - ROOM_HALF_DEPTH - CORRIDOR_LENGTH / 2.0;
    let (cw, len, t) = (CORRIDOR_HALF_WIDTH, CORRIDOR_LENGTH, WALL_THICKNESS);
    block(
        commands,
        meshes,
        materials,
        [2.0 * cw, 0.1, len],
        [0.0, -0.05, cz],
        FLOOR,
    );
    block(
        commands,
        meshes,
        materials,
        [2.0 * cw, 0.1, len],
        [0.0, DOOR_HEIGHT + 0.05, cz],
        CEILING,
    );
    for side in [-1.0, 1.0] {
        let x = side * (cw + t / 2.0);
        block(
            commands,
            meshes,
            materials,
            [t, DOOR_HEIGHT, len],
            [x, DOOR_HEIGHT / 2.0, cz],
            CORRIDOR_WALL,
        );
        // Un liseré doré au pied des murs, pour guider le visiteur.
        let x = side * (cw - 0.02);
        block(
            commands,
            meshes,
            materials,
            [0.04, 0.04, len],
            [x, 0.05, cz],
            [242, 183, 5],
        );
    }
    if lamp_plan(ex.rooms.len()).1 {
        lamp(commands, [0.0, DOOR_HEIGHT - 0.3, cz], 110_000.0);
    }
}

fn setup_tour(
    current: Res<Current>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ex = current.0;
    // Les salles d'abord (avec leurs lampes, dans l'ordre de la visite), puis les couloirs.
    for index in 0..ex.rooms.len() {
        build_room(&mut commands, &mut meshes, &mut materials, ex, index);
    }
    for index in 0..ex.rooms.len().saturating_sub(1) {
        build_corridor(&mut commands, &mut meshes, &mut materials, ex, index);
    }
    for art in ex.artworks {
        hang(&mut commands, &mut meshes, &mut materials, art);
    }
    let (eye, look) = tour_pose(ex, 0.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(eye[0], eye[1], eye[2])
            .looking_at(Vec3::new(look[0], look[1], look[2]), Vec3::Y),
        Visitor { t: 0.0 },
    ));
}

// ─── La visite guidée ────────────────────────────────────────────────────────

/// Un arrêt de la visite : où se tient le visiteur, ce qu'il regarde, combien de temps.
#[derive(Clone, Copy, Debug)]
struct Stop {
    eye: [f32; 3],
    look: [f32; 3],
    hold: f32,
}

const EYE: f32 = 1.65;

fn stop(eye: [f32; 3], look: [f32; 3], hold: f32) -> Stop {
    Stop { eye, look, hold }
}

/// L'arrêt devant une œuvre : face à elle, d'autant plus loin qu'elle est grande.
fn stop_at(art: &Artwork) -> Stop {
    let distance = (art.width.max(art.height) * 1.3).clamp(1.8, 3.4);
    let (center, _) = on_wall(art.hook, 0.0, 0.0, 0.0);
    let eye = match art.hook.wall {
        Wall::Left => [center[0] + distance, EYE, center[2]],
        Wall::Right => [center[0] - distance, EYE, center[2]],
        Wall::Back => [center[0], EYE, center[2] + distance],
    };
    stop(eye, center, 3.5)
}

/// Le parcours d'une exposition, calculé à partir des œuvres accrochées : l'entrée,
/// chaque œuvre salle par salle, un passage dans l'axe de chaque porte, puis le retour.
fn plan(ex: &Exhibition) -> Vec<Stop> {
    let first = ex.rooms[0].center_z;
    let mut list = vec![stop([0.0, EYE, first + 4.2], [0.0, EYE, first - 6.0], 2.5)];
    let last = ex.rooms.len() - 1;
    for (i, room) in ex.rooms.iter().enumerate() {
        let cz = room.center_z;
        if i > 0 {
            list.push(stop([0.0, EYE, cz + 3.6], [0.0, 1.4, cz - 2.0], 1.2));
        }
        list.extend(ex.works_in_room(i).into_iter().map(stop_at));
        if i < last {
            // Dans l'axe de la porte avant de la passer : la caméra ne frôle pas les murs.
            let ahead = [0.0, EYE, cz - 16.0];
            list.push(stop([0.0, EYE, cz - 2.0], ahead, 0.0));
            list.push(stop(
                [0.0, EYE, cz - ROOM_HALF_DEPTH - CORRIDOR_LENGTH / 2.0],
                ahead,
                0.3,
            ));
        } else {
            // Demi-tour, puis retour vers l'entrée par l'axe de la galerie en regardant
            // droit devant ; arrivé à l'entrée, la caméra se retourne vers la galerie.
            list.push(stop([0.0, EYE, cz + 1.0], [0.0, EYE, cz + 12.0], 0.5));
            if i > 0 {
                list.push(stop([0.0, EYE, first + 3.0], [0.0, EYE, first + 14.0], 0.0));
            }
        }
    }
    list
}

/// Le parcours de `ex`, calculé une fois pour toutes les expositions.
fn stops(ex: &Exhibition) -> &'static [Stop] {
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
    let length = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let sub = |p: [f32; 3], q: [f32; 3]| [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
    let distance = length(sub(b.eye, a.eye));
    let (da, db) = (sub(a.look, a.eye), sub(b.look, b.eye));
    let cos = (da[0] * db[0] + da[2] * db[2]) / (length(da) * length(db)).max(1e-6);
    let degrees = cos.clamp(-1.0, 1.0).acos().to_degrees();
    (distance / 1.3).max(degrees / 40.0).clamp(2.5, 10.0)
}

/// La durée d'un tour complet de l'exposition, en secondes.
pub fn tour_duration(ex: &Exhibition) -> f32 {
    let list = stops(ex);
    (0..list.len())
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

/// La direction du regard d'un arrêt : (cap autour de y, inclinaison), en radians.
fn heading(stop: &Stop) -> (f32, f32) {
    let d = [
        stop.look[0] - stop.eye[0],
        stop.look[1] - stop.eye[1],
        stop.look[2] - stop.eye[2],
    ];
    let flat = (d[0] * d[0] + d[2] * d[2]).sqrt();
    (d[0].atan2(d[2]), d[1].atan2(flat))
}

/// Le regard entre deux arrêts : le cap tourne par le plus court chemin. On interpole la
/// direction et non le point regardé : entre deux murs opposés, ce point passerait par
/// l'œil, et la caméra basculerait vers le plafond.
fn blend_look(eye: [f32; 3], a: &Stop, b: &Stop, k: f32) -> [f32; 3] {
    use std::f32::consts::{PI, TAU};
    let ((yaw_a, pitch_a), (yaw_b, pitch_b)) = (heading(a), heading(b));
    let turn = (yaw_b - yaw_a + PI).rem_euclid(TAU) - PI;
    let (yaw, pitch) = (yaw_a + turn * k, pitch_a + (pitch_b - pitch_a) * k);
    [
        eye[0] + yaw.sin() * pitch.cos(),
        eye[1] + pitch.sin(),
        eye[2] + yaw.cos() * pitch.cos(),
    ]
}

/// La position de l'œil et le point regardé au temps `t` de la visite (en boucle).
/// Une fonction pure : elle se teste sans navigateur.
pub fn tour_pose(ex: &Exhibition, t: f32) -> ([f32; 3], [f32; 3]) {
    let list = stops(ex);
    let mut rest = t.rem_euclid(tour_duration(ex));
    for (i, here) in list.iter().enumerate() {
        if rest < here.hold {
            return (here.eye, here.look);
        }
        rest -= here.hold;
        let next = &list[(i + 1) % list.len()];
        let duration = travel(here, next);
        if rest < duration {
            let k = rest / duration;
            let k = k * k * (3.0 - 2.0 * k); // départ et arrivée en douceur
            let eye = lerp3(here.eye, next.eye, k);
            return (eye, blend_look(eye, here, next, k));
        }
        rest -= duration;
    }
    (list[0].eye, list[0].look)
}

fn walk(
    current: Res<Current>,
    time: Res<Time>,
    mut visitors: Query<(&mut Transform, &mut Visitor)>,
) {
    for (mut transform, mut visitor) in &mut visitors {
        visitor.t += time.delta_secs();
        let (eye, look) = tour_pose(current.0, visitor.t);
        *transform = Transform::from_xyz(eye[0], eye[1], eye[2])
            .looking_at(Vec3::new(look[0], look[1], look[2]), Vec3::Y);
    }
}

fn turn(time: Res<Time>, mut parts: Query<(&mut Transform, &mut Turntable)>) {
    for (mut transform, mut part) in &mut parts {
        part.angle += part.speed * time.delta_secs();
        let (sin, cos) = part.angle.sin_cos();
        let [ox, oy, oz] = part.offset;
        let [cx, cy, cz] = part.center;
        let mut next =
            Transform::from_xyz(cx + ox * cos + oz * sin, cy + oy, cz - ox * sin + oz * cos);
        next.rotate_y(part.angle);
        *transform = next;
    }
}

/// La galerie complète d'une exposition, avec sa visite guidée.
pub fn tour_app(ex: &'static Exhibition) -> App {
    let mut app = App::new();
    app.add_plugins(plugins())
        .insert_resource(ambient())
        .insert_resource(Current(ex))
        .add_systems(Startup, setup_tour)
        .add_systems(Update, walk);
    app
}

// ─── La vitrine ──────────────────────────────────────────────────────────────

fn setup_showcase(
    current: Res<Current>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let center = [0.0, 0.0, 0.0];
    let speed = 0.4;
    let mut part = |commands: &mut Commands, mesh: Mesh, rgb: [u8; 3], offset: [f32; 3]| {
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(color(rgb))),
            Transform::from_xyz(offset[0], offset[1], offset[2]),
            Turntable {
                center,
                offset,
                speed,
                angle: 0.0,
            },
        ));
    };
    // Le socle tournant et le chevalet : deux montants, un pied arrière et la tablette.
    const WOOD: [u8; 3] = [120, 78, 40];
    part(
        &mut commands,
        Cuboid::new(2.4, 0.16, 1.4).into(),
        [26, 28, 40],
        [0.0, -0.08, 0.0],
    );
    for x in [-0.55, 0.55] {
        part(
            &mut commands,
            Cuboid::new(0.07, 2.3, 0.07).into(),
            WOOD,
            [x, 1.15, -0.12],
        );
    }
    part(
        &mut commands,
        Cuboid::new(0.07, 2.1, 0.07).into(),
        WOOD,
        [0.0, 1.05, -0.5],
    );
    part(
        &mut commands,
        Cuboid::new(1.3, 0.06, 0.12).into(),
        WOOD,
        [0.0, 0.45, -0.06],
    );

    // L'œuvre de la vitrine, au plus 1,4 m de large et 1,7 m de haut.
    let art = current.0.cover();
    let ratio = art.map_or(0.8, |a| a.height / a.width);
    let w = 1.4f32.min(1.7 / ratio);
    let h = w * ratio;
    let y = 0.48 + h / 2.0;
    part(
        &mut commands,
        Cuboid::new(w + 0.12, h + 0.12, 0.05).into(),
        GILDING,
        [0.0, y, -0.02],
    );
    match art {
        Some(art) => {
            for (rgb, mesh) in mosaic_meshes(&art.mosaic, w, h) {
                part(&mut commands, mesh, rgb, [0.0, y, 0.005]);
            }
        }
        None => part(
            &mut commands,
            Cuboid::new(w, h, 0.02).into(),
            [242, 236, 220],
            [0.0, y, 0.01],
        ),
    }

    lamp(&mut commands, [2.5, 3.5, 3.5], 700_000.0);
    lamp(&mut commands, [-3.0, 2.5, 2.5], 350_000.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, 4.2).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));
}

/// La vitrine animée de la présentation d'une exposition.
pub fn showcase_app(ex: &'static Exhibition) -> App {
    let mut app = App::new();
    app.add_plugins(plugins())
        .insert_resource(ambient())
        .insert_resource(Current(ex))
        .add_systems(Startup, setup_showcase)
        .add_systems(Update, turn);
    app
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tour_starts_at_the_entrance_and_loops() {
        for ex in EXHIBITIONS {
            assert_eq!(tour_pose(ex, 0.0).0, stops(ex)[0].eye);
            assert_eq!(tour_pose(ex, tour_duration(ex)).0, stops(ex)[0].eye);
        }
    }

    #[test]
    fn the_tour_stops_in_front_of_every_work() {
        for ex in EXHIBITIONS {
            for art in ex.artworks {
                let target = stop_at(art);
                assert!(
                    stops(ex).iter().any(|s| s.eye == target.eye),
                    "{}",
                    art.title
                );
            }
        }
    }

    #[test]
    fn the_visitor_stays_inside_the_gallery() {
        for ex in EXHIBITIONS {
            let last = ex.rooms[ex.rooms.len() - 1].center_z - ROOM_HALF_DEPTH;
            let mut t = 0.0;
            while t < tour_duration(ex) {
                let ([x, y, z], _) = tour_pose(ex, t);
                assert!(
                    x.abs() < INNER - 0.5 && y > 0.0 && y < WALL_HEIGHT,
                    "{} t = {t}",
                    ex.slug
                );
                assert!(z < ROOM_HALF_DEPTH && z > last, "{} t = {t}", ex.slug);
                // Dans un couloir, le visiteur reste au milieu.
                let in_room = ex
                    .rooms
                    .iter()
                    .any(|r| (z - r.center_z).abs() <= ROOM_HALF_DEPTH);
                assert!(
                    in_room || x.abs() < CORRIDOR_HALF_WIDTH - 0.3,
                    "{} t = {t} : x = {x}, z = {z}",
                    ex.slug
                );
                t += 0.05;
            }
        }
    }

    /// La caméra regarde toujours à peu près à l'horizontale : jamais le plafond ni le sol.
    #[test]
    fn the_camera_never_looks_at_the_ceiling() {
        for ex in EXHIBITIONS {
            let mut t = 0.0;
            while t < tour_duration(ex) {
                let (eye, look) = tour_pose(ex, t);
                let d = [look[0] - eye[0], look[1] - eye[1], look[2] - eye[2]];
                let pitch = d[1].atan2((d[0] * d[0] + d[2] * d[2]).sqrt()).to_degrees();
                assert!(
                    pitch.abs() < 25.0,
                    "{} t = {t} : inclinaison {pitch:.0}°",
                    ex.slug
                );
                t += 0.05;
            }
        }
    }

    #[test]
    fn the_lamps_fit_the_renderer() {
        for rooms in 1..12 {
            let (per_room, corridors) = lamp_plan(rooms);
            let total = per_room * rooms + if corridors { rooms - 1 } else { 0 };
            assert!(
                total <= MAX_LAMPS || (per_room == 1 && !corridors),
                "{rooms} salles"
            );
        }
    }

    /// Les rectangles fusionnés couvrent chaque case une fois, de la bonne couleur.
    #[test]
    fn the_rectangles_cover_the_mosaic_exactly() {
        for ex in EXHIBITIONS {
            for art in ex.artworks {
                let m = &art.mosaic;
                let cols = usize::from(m.cols);
                let mut seen = vec![false; m.cells.len()];
                let rects = rectangles(m);
                for &(c, a, b, r0, r1) in &rects {
                    for row in r0..r1 {
                        for col in a..b {
                            let i = row * cols + col;
                            assert!(!seen[i] && m.cells[i] == c, "{}", art.title);
                            seen[i] = true;
                        }
                    }
                }
                assert!(seen.iter().all(|&s| s), "{}", art.title);
                assert!(
                    rects.len() < m.cells.len(),
                    "{} : rien n'a été fusionné",
                    art.title
                );
            }
        }
    }

    /// Après plusieurs secondes de visite, chaque pavé est toujours à sa place : rien ne
    /// tombe (la physique est bien retirée) et le nombre de lampes tient dans le shader.
    #[test]
    fn the_gallery_stands_still_and_fits_the_renderer() {
        for ex in EXHIBITIONS {
            for mut app in [tour_app(ex), showcase_app(ex)] {
                app.update();
                let first =
                    RenderFrame::extract(app.world(), Vec2::new(800.0, 450.0)).expect("une caméra");
                let before: Vec<[f32; 3]> = first
                    .frame()
                    .meshes
                    .iter()
                    .map(|d| d.world.translation().to_array())
                    .collect();
                for _ in 0..120 {
                    app.update();
                }
                let later =
                    RenderFrame::extract(app.world(), Vec2::new(800.0, 450.0)).expect("une caméra");
                let frame = later.frame();
                assert_eq!(frame.meshes.len(), before.len());
                assert!(
                    frame.lights.point.len() <= MAX_LAMPS,
                    "{} lampes",
                    frame.lights.point.len()
                );
                for (draw, start) in frame.meshes.iter().zip(&before) {
                    let now = draw.world.translation().to_array();
                    assert!(
                        (now[1] - start[1]).abs() < 1e-3,
                        "un pavé est tombé : {start:?} → {now:?}"
                    );
                }
            }
        }
    }
}
