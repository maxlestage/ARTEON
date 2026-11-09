//! La galerie en 3D, avec le moteur de jeu d'Active réexporté par Paquets
//! (`paquets::ecs`). Deux scènes :
//!
//! - `tour_app()` : trois salles d'exposition reliées par deux couloirs, les œuvres de
//!   `artworks.rs` construites en relief sur les murs, et une caméra qui fait la visite
//!   guidée, en boucle ;
//! - `showcase_app()` : une petite sculpture tournante, pour la page de présentation.
//!
//! Seule l'API documentée du moteur est utilisée : des pavés (`Cuboid`), des couleurs,
//! des lampes, une caméra et des systèmes qui déplacent des `Transform`.
//!
//! Deux limites du moteur dictent la construction :
//!
//! - `DefaultPlugins` contient la physique, qui donne un corps soumis à la gravité à
//!   chaque pavé : les murs, les sols et les tableaux tomberaient dans le vide. Un musée
//!   n'en a pas besoin, elle est donc retirée (`disable::<PhysicsPlugin>()`) ;
//! - le shader n'éclaire qu'avec 8 lampes ponctuelles au plus (les suivantes sont
//!   ignorées) : deux par salle et une par couloir, soit 8 exactement.

use paquets::ecs::prelude::*;

use crate::artworks::{
    ARTWORKS, Artwork, CORRIDOR_HALF_WIDTH, CORRIDOR_LENGTH, HANGING_HEIGHT, ROOM_HALF_DEPTH,
    ROOM_HALF_WIDTH, ROOMS, WALL_HEIGHT, WALL_THICKNESS, Wall,
};

// ─── Composants ───────────────────────────────────────────────────────────────

/// La caméra de la visite guidée ; `t` est le temps écoulé depuis le début du tour.
struct Visitor {
    t: f32,
}
paquets::active::component!(Visitor);

/// Un objet qui tourne sur lui-même (radians par seconde).
struct Spinning(f32);
paquets::active::component!(Spinning);

/// Un morceau de sculpture posé sur un plateau tournant : sa place est recalculée à
/// chaque image autour de `center`, ce qui fait tourner la sculpture d'un bloc.
struct Turntable {
    center: [f32; 3],
    offset: [f32; 3],
    speed: f32,
    angle: f32,
}
paquets::active::component!(Turntable);

// ─── Petits outils de construction ───────────────────────────────────────────

/// Un pavé de taille `size`, centré en `pos`, de couleur `rgb`.
fn block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    size: [f32; 3],
    pos: [f32; 3],
    rgb: [u8; 3],
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size[0], size[1], size[2]))),
        MeshMaterial3d(materials.add(Color::srgb_u8(rgb[0], rgb[1], rgb[2]))),
        Transform::from_xyz(pos[0], pos[1], pos[2]),
    ));
}

/// Le nombre maximal de lampes ponctuelles que le moteur prend en compte.
pub const MAX_LAMPS: usize = 8;

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
        brightness: 450.0,
        ..default()
    }
}

/// Les plugins des deux scènes : tout `DefaultPlugins`, sauf la physique.
fn plugins() -> PluginGroupBuilder {
    DefaultPlugins.disable::<PhysicsPlugin>()
}

const FLOOR: [u8; 3] = [146, 112, 82];
const CEILING: [u8; 3] = [250, 249, 246];
const FRAME: [u8; 3] = [34, 30, 28];
const PLAQUE: [u8; 3] = [244, 242, 236];
const WALLS: [[u8; 3]; 3] = [[238, 235, 228], [226, 228, 232], [240, 230, 216]];
const CORRIDOR_WALL: [u8; 3] = [52, 58, 74];
const DOOR_HEIGHT: f32 = 3.0;

// ─── La galerie ──────────────────────────────────────────────────────────────

/// Construit une salle : sol, plafond, murs (avec des portes) et éclairage.
fn build_room(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    index: usize,
) {
    let cz = ROOMS[index].center_z;
    let (w, d, h, t) = (
        ROOM_HALF_WIDTH,
        ROOM_HALF_DEPTH,
        WALL_HEIGHT,
        WALL_THICKNESS,
    );
    let wall = WALLS[index % WALLS.len()];

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
    // Les murs latéraux, qui portent les tableaux.
    for side in [-1.0, 1.0] {
        block(
            commands,
            meshes,
            materials,
            [t, h, 2.0 * d + t],
            [side * w, h / 2.0, cz],
            wall,
        );
    }
    // Les murs avant et arrière : une porte au milieu, sauf au fond de la dernière salle.
    let last = index == ROOMS.len() - 1;
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
    // Deux lampes au plafond, une au-dessus de chaque moitié de la salle.
    for z in [2.5, -2.5] {
        lamp(commands, [0.0, h - 0.4, cz + z], 320_000.0);
    }
}

/// Construit le couloir qui part du fond de la salle `index` vers la suivante.
fn build_corridor(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    index: usize,
) {
    let cz = ROOMS[index].center_z - ROOM_HALF_DEPTH - CORRIDOR_LENGTH / 2.0;
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
        // Un liseré lumineux au pied des murs, pour guider le visiteur.
        let x = side * (cw - 0.02);
        block(
            commands,
            meshes,
            materials,
            [0.04, 0.04, len],
            [x, 0.05, cz],
            [255, 214, 120],
        );
    }
    lamp(commands, [0.0, DOOR_HEIGHT - 0.3, cz], 120_000.0);
}

/// Accroche une œuvre : un cadre, la toile, puis chaque aplat en léger relief, et un
/// cartel à côté.
fn hang(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    art: &Artwork,
) {
    let room = ROOMS[art.hook.room];
    let along = art.hook.along;
    let inner = ROOM_HALF_WIDTH - WALL_THICKNESS / 2.0;
    let back = room.center_z - ROOM_HALF_DEPTH + WALL_THICKNESS / 2.0;

    // `u` : vers la droite du visiteur qui regarde le tableau ; `v` : vers le haut ;
    // `layer` : la distance au mur. Renvoie la taille et la place du pavé.
    let place = |u: f32, v: f32, layer: f32, w: f32, h: f32, thick: f32| -> ([f32; 3], [f32; 3]) {
        let y = HANGING_HEIGHT + v;
        match art.hook.wall {
            Wall::Left => ([thick, h, w], [-inner + layer, y, along - u]),
            Wall::Right => ([thick, h, w], [inner - layer, y, along + u]),
            Wall::Back => ([w, h, thick], [along + u, y, back + layer]),
        }
    };

    let (aw, ah) = (art.width, art.height);
    let (size, pos) = place(0.0, 0.0, 0.03, aw + 0.12, ah + 0.12, 0.06);
    block(commands, meshes, materials, size, pos, FRAME);
    let (size, pos) = place(0.0, 0.0, 0.065, aw, ah, 0.01);
    block(commands, meshes, materials, size, pos, art.background);
    for (i, shape) in art.shapes.iter().enumerate() {
        let u = (shape.x + shape.w / 2.0 - 0.5) * aw;
        let v = (shape.y + shape.h / 2.0 - 0.5) * ah;
        // Chaque aplat est un peu plus en avant que le précédent : les lignes noires,
        // listées en dernier, passent devant les plans de couleur.
        let layer = 0.072 + i as f32 * 0.002;
        let (size, pos) = place(u, v, layer, shape.w * aw, shape.h * ah, 0.004);
        block(commands, meshes, materials, size, pos, shape.rgb);
    }
    // Le cartel, à droite du tableau, à hauteur des yeux.
    let (size, pos) = place(aw / 2.0 + 0.3, -0.35, 0.01, 0.24, 0.14, 0.02);
    block(commands, meshes, materials, size, pos, PLAQUE);
}

/// Un « architectone » à la Malevitch : des blocs blancs empilés, sur un socle, qui
/// tournent lentement au centre de la salle II.
fn build_architecton(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    center: [f32; 3],
    speed: f32,
) {
    block(
        commands,
        meshes,
        materials,
        [1.2, center[1], 1.2],
        [center[0], center[1] / 2.0, center[2]],
        [60, 60, 64],
    );
    let parts: [([f32; 3], [f32; 3], [u8; 3]); 7] = [
        ([0.9, 0.25, 0.6], [0.0, 0.125, 0.0], [246, 246, 244]),
        ([0.5, 0.35, 0.4], [0.1, 0.425, 0.05], [240, 240, 236]),
        ([0.25, 0.6, 0.25], [-0.15, 0.55, -0.1], [250, 250, 248]),
        ([0.7, 0.12, 0.2], [0.25, 0.66, 0.1], [232, 232, 228]),
        ([0.15, 0.3, 0.15], [0.25, 0.87, 0.1], [246, 246, 244]),
        ([0.35, 0.08, 0.35], [-0.15, 0.89, -0.1], [236, 236, 232]),
        ([0.12, 0.12, 0.12], [0.3, 1.08, 0.1], [200, 32, 38]),
    ];
    for (size, offset, rgb) in parts {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size[0], size[1], size[2]))),
            MeshMaterial3d(materials.add(Color::srgb_u8(rgb[0], rgb[1], rgb[2]))),
            Transform::from_xyz(
                center[0] + offset[0],
                center[1] + offset[1],
                center[2] + offset[2],
            ),
            Turntable {
                center,
                offset,
                speed,
                angle: 0.0,
            },
        ));
    }
}

fn setup_tour(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for index in 0..ROOMS.len() {
        build_room(&mut commands, &mut meshes, &mut materials, index);
        if index + 1 < ROOMS.len() {
            build_corridor(&mut commands, &mut meshes, &mut materials, index);
        }
    }
    for art in &ARTWORKS {
        hang(&mut commands, &mut meshes, &mut materials, art);
    }
    build_architecton(
        &mut commands,
        &mut meshes,
        &mut materials,
        [0.0, 0.8, ROOMS[1].center_z],
        0.25,
    );

    let (eye, look) = tour_pose(0.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(eye[0], eye[1], eye[2])
            .looking_at(Vec3::new(look[0], look[1], look[2]), Vec3::Y),
        Visitor { t: 0.0 },
    ));
}

// ─── La visite guidée ────────────────────────────────────────────────────────

/// Un arrêt de la visite : où se tient le visiteur, ce qu'il regarde, combien de temps.
struct Stop {
    eye: [f32; 3],
    look: [f32; 3],
    hold: f32,
}

const fn stop(eye: [f32; 3], look: [f32; 3], hold: f32) -> Stop {
    Stop { eye, look, hold }
}

/// La durée du trajet entre deux arrêts, en secondes.
const TRAVEL: f32 = 3.5;
const EYE: f32 = 1.7;

/// Le parcours : chaque tableau, les couloirs, l'architectone, puis retour à l'entrée.
/// Avant chaque porte, un arrêt bref au milieu de la salle, le regard déjà dans l'axe
/// du couloir : la caméra passe la porte droit devant elle, sans frôler les murs.
const STOPS: [Stop; 17] = [
    stop([0.0, EYE, 4.0], [0.0, EYE, -6.0], 2.5),
    stop([-1.6, EYE, 2.5], [-5.0, HANGING_HEIGHT, 2.5], 3.0),
    stop([1.6, EYE, 2.5], [5.0, HANGING_HEIGHT, 2.5], 3.0),
    stop([-1.6, EYE, -2.5], [-5.0, HANGING_HEIGHT, -2.5], 3.0),
    stop([1.6, EYE, -2.5], [5.0, HANGING_HEIGHT, -2.5], 3.0),
    stop([0.0, EYE, -2.0], [0.0, EYE, -16.0], 0.0),
    stop([0.0, EYE, -6.0], [0.0, EYE, -16.0], 0.5),
    stop([0.0, EYE, -12.5], [0.0, 1.3, -16.0], 2.5),
    stop([-2.0, EYE, -16.0], [-5.0, HANGING_HEIGHT, -16.0], 3.0),
    stop([0.0, EYE, -19.0], [0.0, 1.3, -16.0], 1.5),
    stop([2.0, EYE, -16.0], [5.0, HANGING_HEIGHT, -16.0], 3.0),
    stop([0.0, EYE, -18.0], [0.0, EYE, -32.0], 0.0),
    stop([0.0, EYE, -22.0], [0.0, EYE, -32.0], 0.5),
    stop([-2.0, EYE, -32.0], [-5.0, HANGING_HEIGHT, -32.0], 3.0),
    stop([2.0, EYE, -32.0], [5.0, HANGING_HEIGHT, -32.0], 3.0),
    stop([0.0, EYE, -32.5], [0.0, HANGING_HEIGHT, -37.0], 4.0),
    stop([0.0, EYE, -24.0], [0.0, EYE, -10.0], 0.5),
];

/// La durée d'un tour complet, en secondes.
pub fn tour_duration() -> f32 {
    STOPS.iter().map(|s| s.hold + TRAVEL).sum()
}

fn lerp3(a: [f32; 3], b: [f32; 3], k: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * k,
        a[1] + (b[1] - a[1]) * k,
        a[2] + (b[2] - a[2]) * k,
    ]
}

/// La position de l'œil et le point regardé au temps `t` de la visite (en boucle).
/// Une fonction pure : elle se teste sans navigateur.
pub fn tour_pose(t: f32) -> ([f32; 3], [f32; 3]) {
    let mut rest = t.rem_euclid(tour_duration());
    for (i, here) in STOPS.iter().enumerate() {
        if rest < here.hold {
            return (here.eye, here.look);
        }
        rest -= here.hold;
        let next = &STOPS[(i + 1) % STOPS.len()];
        if rest < TRAVEL {
            let k = rest / TRAVEL;
            let k = k * k * (3.0 - 2.0 * k); // départ et arrivée en douceur
            return (lerp3(here.eye, next.eye, k), lerp3(here.look, next.look, k));
        }
        rest -= TRAVEL;
    }
    (STOPS[0].eye, STOPS[0].look)
}

fn walk(time: Res<Time>, mut visitors: Query<(&mut Transform, &mut Visitor)>) {
    for (mut transform, mut visitor) in &mut visitors {
        visitor.t += time.delta_secs();
        let (eye, look) = tour_pose(visitor.t);
        *transform = Transform::from_xyz(eye[0], eye[1], eye[2])
            .looking_at(Vec3::new(look[0], look[1], look[2]), Vec3::Y);
    }
}

fn spin(time: Res<Time>, mut things: Query<(&mut Transform, &Spinning)>) {
    for (mut transform, spinning) in &mut things {
        transform.rotate_y(spinning.0 * time.delta_secs());
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

/// La galerie complète, avec sa visite guidée.
pub fn tour_app() -> App {
    let mut app = App::new();
    app.add_plugins(plugins())
        .insert_resource(ambient())
        .add_systems(Startup, setup_tour)
        .add_systems(Update, (walk, turn));
    app
}

// ─── La vitrine de la page de présentation ───────────────────────────────────

fn setup_showcase(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Un socle, et dessus une sculpture faite des couleurs de Mondrian, qui tourne.
    block(
        &mut commands,
        &mut meshes,
        &mut materials,
        [2.4, 0.2, 2.4],
        [0.0, -0.1, 0.0],
        [30, 30, 36],
    );
    let center = [0.0, 0.0, 0.0];
    let parts: [([f32; 3], [f32; 3], [u8; 3]); 8] = [
        ([1.2, 1.2, 0.12], [0.0, 0.6, 0.0], [242, 239, 230]),
        ([0.7, 0.7, 0.14], [0.22, 0.85, 0.0], [200, 32, 38]),
        ([0.35, 0.3, 0.14], [-0.4, 0.15, 0.0], [28, 58, 140]),
        ([0.2, 0.35, 0.14], [0.5, 0.18, 0.0], [242, 200, 40]),
        ([0.05, 1.2, 0.16], [-0.2, 0.6, 0.0], [22, 22, 22]),
        ([1.2, 0.05, 0.16], [0.0, 0.48, 0.0], [22, 22, 22]),
        ([0.12, 1.2, 0.8], [0.0, 0.6, -0.45], [242, 239, 230]),
        ([0.13, 0.5, 0.5], [0.0, 0.95, -0.55], [28, 58, 140]),
    ];
    for (size, offset, rgb) in parts {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size[0], size[1], size[2]))),
            MeshMaterial3d(materials.add(Color::srgb_u8(rgb[0], rgb[1], rgb[2]))),
            Transform::from_xyz(offset[0], offset[1], offset[2]),
            Turntable {
                center,
                offset,
                speed: 0.45,
                angle: 0.0,
            },
        ));
    }
    // Trois « satellites » : le carré noir, un rouge et un jaune, qui tournent sur eux-mêmes.
    for (pos, rgb, speed) in [
        ([-1.7, 1.6, -0.6], [22, 22, 22], 0.8),
        ([1.8, 1.9, -0.4], [200, 32, 38], -1.1),
        ([1.5, 0.5, 0.9], [242, 200, 40], 1.4),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.35, 0.35, 0.35))),
            MeshMaterial3d(materials.add(Color::srgb_u8(rgb[0], rgb[1], rgb[2]))),
            Transform::from_xyz(pos[0], pos[1], pos[2]),
            Spinning(speed),
        ));
    }
    lamp(&mut commands, [3.0, 4.0, 3.0], 900_000.0);
    lamp(&mut commands, [-3.0, 3.0, 2.0], 500_000.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.6, 4.6).looking_at(Vec3::new(0.0, 0.8, 0.0), Vec3::Y),
    ));
}

/// La vitrine animée de la page de présentation.
pub fn showcase_app() -> App {
    let mut app = App::new();
    app.add_plugins(plugins())
        .insert_resource(ambient())
        .add_systems(Startup, setup_showcase)
        .add_systems(Update, (spin, turn));
    app
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tour_starts_at_the_entrance_and_loops() {
        assert_eq!(tour_pose(0.0).0, STOPS[0].eye);
        assert_eq!(tour_pose(tour_duration()).0, STOPS[0].eye);
    }

    #[test]
    fn the_visitor_stays_inside_the_gallery() {
        let last = ROOMS[ROOMS.len() - 1].center_z - ROOM_HALF_DEPTH;
        let mut t = 0.0;
        while t < tour_duration() {
            let ([x, y, z], _) = tour_pose(t);
            assert!(
                x.abs() < ROOM_HALF_WIDTH && y > 0.0 && y < WALL_HEIGHT,
                "t = {t}"
            );
            assert!(z < ROOM_HALF_DEPTH && z > last, "t = {t}");
            t += 0.1;
        }
    }

    /// Après plusieurs secondes de visite, chaque pavé est toujours à sa place : rien ne
    /// tombe (la physique est bien retirée) et le nombre de lampes tient dans le shader.
    #[test]
    fn the_gallery_stands_still_and_fits_the_renderer() {
        for mut app in [tour_app(), showcase_app()] {
            app.update();
            let first =
                RenderFrame::extract(app.world(), Vec2::new(800.0, 450.0)).expect("une caméra");
            let before: Vec<[f32; 3]> = first
                .frame()
                .meshes
                .iter()
                .map(|d| d.world.translation().to_array())
                .collect();
            for _ in 0..300 {
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
            // Les morceaux qui tournent restent à la même hauteur ; les autres ne bougent pas.
            for (draw, start) in frame.meshes.iter().zip(&before) {
                let now = draw.world.translation().to_array();
                assert!(
                    (now[1] - start[1]).abs() < 1e-3,
                    "un pavé est tombé : {start:?} → {now:?}"
                );
            }
        }
    }

    #[test]
    fn every_artwork_hangs_in_an_existing_room() {
        for art in &ARTWORKS {
            assert!(art.hook.room < ROOMS.len(), "{}", art.title);
        }
    }
}
