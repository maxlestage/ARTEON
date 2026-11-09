//! Les scènes 3D, avec le moteur de jeu d'Active réexporté par Paquets (`paquets::ecs`,
//! l'équivalent de Bevy) :
//!
//! - `tour_app` : le musée d'une exposition, construit à partir de son plan
//!   (`museum.rs`), ses tableaux en mosaïque, ses sculptures sur socle, et une caméra
//!   qui fait la visite guidée (`tour.rs`) ou que le visiteur promène lui-même ;
//! - `showcase_app` : l'œuvre de la vitrine sur un chevalet tournant.
//!
//! Le moteur n'affiche pas d'images (pas de textures) : un tableau est une grille de
//! touches de couleur de 7 mm ; les touches voisines de même couleur sont fusionnées en
//! rectangles, une couleur par maillage. Les cases et les maillages des sculptures
//! viennent du fichier de données de l'exposition ([`ExhibitionData`]).
//!
//! Deux limites du moteur dictent la construction :
//!
//! - `DefaultPlugins` contient la physique, qui donnerait un corps soumis à la gravité à
//!   chaque pavé : elle est retirée (`disable::<PhysicsPlugin>()`) ;
//! - le shader n'éclaire qu'avec 8 lampes ponctuelles : chaque salle a les siennes, et à
//!   chaque image seules les 8 plus proches du visiteur restent allumées.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use paquets::ecs::prelude::*;
use paquets::scene::BufferGeometry;

use crate::artworks::{Artwork, Exhibition, Mosaic};
use crate::data::ExhibitionData;
use crate::editor::{self, Edits};
use crate::museum::{self, Block};
use crate::tour::{self, EYE, Focus, Pose};

// ─── Ressources et composants ─────────────────────────────────────────────────

/// L'exposition que montre la scène, et ses données.
struct Current(&'static Exhibition, Rc<ExhibitionData>);
paquets::active::resource!(Current);

/// Comment le visiteur se déplace.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// La caméra fait la visite toute seule.
    #[default]
    Guided,
    /// Le visiteur marche où il veut (clavier, souris, joysticks tactiles).
    Free,
}

/// Les commandes de la visite, partagées entre la scène et la page (interface egui,
/// boutons, clavier, écran tactile). La page écrit les demandes ; la scène écrit où en
/// est le visiteur et ce qu'il regarde.
#[derive(Default)]
pub struct TourControl {
    pub mode: Mode,
    /// La visite guidée est arrêtée.
    pub paused: bool,
    /// Le nombre d'œuvres à sauter dans la visite guidée : +1 la suivante, -1 la précédente.
    pub jump: i32,
    /// Aller voir cette œuvre (la visite guidée reprend devant elle).
    pub go_to: Option<Focus>,
    /// En visite libre : aller à ce point (x, z) du plan.
    pub teleport: Option<(f32, f32)>,
    /// En visite libre : avancer (+1) ou reculer (-1), aller à droite (+1) ou à gauche (-1).
    pub forward: f32,
    pub strafe: f32,
    /// En visite libre : de combien tourner la tête depuis la dernière image (en radians).
    pub turn: f32,
    pub tilt: f32,
    /// En visite libre : tourner en continu (flèches), en radians par seconde.
    pub spin: f32,
    /// Courir (Maj) en visite libre.
    pub run: bool,
    /// L'œuvre regardée.
    pub current: Option<Focus>,
    /// Où est le visiteur : x, z et le cap (pour la minicarte).
    pub position: [f32; 3],
    /// Appelé quand l'œuvre regardée change.
    pub on_change: Option<Rc<dyn Fn(Option<Focus>)>>,
    /// Les modifications de l'éditeur : la scène se reconstruit quand elles changent.
    pub edits: Edits,
}

/// Les commandes, partagées (`Rc<RefCell<…>>`) entre la page et la scène.
pub type SharedControl = Rc<RefCell<TourControl>>;

struct Control(SharedControl);
paquets::active::resource!(Control);

/// La caméra du visiteur : `t` est le temps de la visite guidée ; en visite libre, sa
/// position, son cap et son inclinaison.
struct Visitor {
    t: f32,
    x: f32,
    z: f32,
    yaw: f32,
    pitch: f32,
    mode: Mode,
}
paquets::active::component!(Visitor);

/// Une lampe de plafond (seules les 8 plus proches du visiteur restent allumées).
struct Lamp;
paquets::active::component!(Lamp);

/// Une œuvre de la scène (tableau, cadre, cartel, sculpture, socle) : reconstruite quand
/// l'éditeur la déplace.
struct Exhibit;
paquets::active::component!(Exhibit);

/// La version des modifications déjà appliquées à la scène.
#[derive(Default)]
struct Applied(u32);
paquets::active::resource!(Applied);

/// Un morceau posé sur un plateau tournant (la vitrine).
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

fn spawn_block(commands: &mut Commands, meshes: Meshes, materials: Materials, block: &Block) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(block.size[0], block.size[1], block.size[2]))),
        MeshMaterial3d(materials.add(color(block.color))),
        Transform::from_xyz(block.center[0], block.center[1], block.center[2]),
    ));
}

/// Le nombre maximal de lampes ponctuelles que le moteur prend en compte.
pub const MAX_LAMPS: usize = 8;

fn lamp(commands: &mut Commands, pos: [f32; 3], intensity: f32) {
    commands.spawn((
        PointLight {
            intensity,
            range: 14.0,
            ..default()
        },
        Transform::from_xyz(pos[0], pos[1], pos[2]),
        Visibility::Visible,
        Lamp,
    ));
}

/// La lumière d'ambiance, qui éclaire aussi les coins loin des lampes.
fn ambient() -> GlobalAmbientLight {
    GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 560.0,
        ..default()
    }
}

/// Les plugins des deux scènes : tout `DefaultPlugins`, sauf la physique.
fn plugins() -> PluginGroupBuilder {
    DefaultPlugins.disable::<PhysicsPlugin>()
}

const GILDING: [u8; 3] = [168, 128, 56];
const CANVAS_BACK: [u8; 3] = [92, 84, 72];
const PLAQUE: [u8; 3] = [244, 240, 228];
const PLINTH: [u8; 3] = [214, 208, 196];

fn turned(pos: [f32; 3], angle: f32) -> Transform {
    let mut transform = Transform::from_xyz(pos[0], pos[1], pos[2]);
    transform.rotate_y(angle);
    transform
}

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
fn rectangles(mosaic: &Mosaic, cells: &[u8]) -> Vec<(u8, usize, usize, usize, usize)> {
    let (cols, rows) = (usize::from(mosaic.cols), usize::from(mosaic.rows));
    let mut done = Vec::new();
    let mut open: HashMap<(u8, usize, usize), usize> = HashMap::new();
    for row in 0..rows {
        let line = &cells[row * cols..(row + 1) * cols];
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
/// tournée vers +z : un maillage par couleur. Les tons clairs sont un peu plus en avant,
/// comme les empâtements du peintre.
fn mosaic_meshes(mosaic: &Mosaic, cells: &[u8], width: f32, height: f32) -> Vec<([u8; 3], Mesh)> {
    let (cw, ch) = (
        width / f32::from(mosaic.cols),
        height / f32::from(mosaic.rows),
    );
    let mut by_color: BTreeMap<u8, Builder> = BTreeMap::new();
    for (c, a, b, r0, r1) in rectangles(mosaic, cells) {
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

/// Accroche un tableau : un cadre doré, le fond de la toile, la mosaïque et un cartel.
fn hang(
    commands: &mut Commands,
    meshes: Meshes,
    materials: Materials,
    ex: &Exhibition,
    data: &ExhibitionData,
    art: &Artwork,
) {
    let (w, h) = (art.width, art.height);
    let parts: [([f32; 3], f32, [u8; 3]); 2] = [
        ([w + 0.16, h + 0.16, 0.06], 0.03, GILDING),
        ([w + 0.01, h + 0.01, 0.01], 0.065, CANVAS_BACK),
    ];
    for (size, layer, rgb) in parts {
        let (pos, angle) = museum::on_wall(ex, art.hook, 0.0, 0.0, layer);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size[0], size[1], size[2]))),
            MeshMaterial3d(materials.add(color(rgb))),
            turned(pos, angle),
            Exhibit,
        ));
    }
    let (pos, angle) = museum::on_wall(ex, art.hook, 0.0, 0.0, 0.07);
    for (rgb, mesh) in mosaic_meshes(&art.mosaic, data.cells(&art.mosaic), w, h) {
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(color(rgb))),
            turned(pos, angle),
            Exhibit,
        ));
    }
    let plaque_height = 1.45 - art.hook.center;
    let (pos, angle) = museum::on_wall(ex, art.hook, w / 2.0 + 0.32, plaque_height, 0.01);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.24, 0.15, 0.02))),
        MeshMaterial3d(materials.add(color(PLAQUE))),
        turned(pos, angle),
        Exhibit,
    ));
}

/// Pose une sculpture sur son socle.
fn sculpt(
    commands: &mut Commands,
    meshes: Meshes,
    materials: Materials,
    data: &ExhibitionData,
    s: &crate::artworks::Sculpture,
) {
    if s.plinth > 0.01 {
        let side = (s.radius * 2.0 * 0.62).max(0.4);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(side, s.plinth, side))),
            MeshMaterial3d(materials.add(color(PLINTH))),
            turned([s.x, s.plinth / 2.0, s.z], s.rotation),
            Exhibit,
        ));
    }
    let model = data.sculpture(s);
    let mesh = Mesh::from_geometry(BufferGeometry::new(
        model.positions,
        model.normals,
        Vec::new(),
        model.indices,
    ));
    let material = StandardMaterial {
        base_color: color(s.color),
        perceptual_roughness: s.roughness,
        metallic: s.metallic,
        ..default()
    };
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        turned([s.x, s.plinth, s.z], s.rotation),
        Exhibit,
    ));
}

// ─── Le musée ─────────────────────────────────────────────────────────────────

fn setup_tour(
    current: Res<Current>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (ex, data) = (current.0, current.1.clone());
    for (i, room) in ex.rooms.iter().enumerate() {
        for block in museum::room_blocks(room) {
            spawn_block(&mut commands, &mut meshes, &mut materials, &block);
        }
        for block in museum::bench(ex, i).into_iter().flatten() {
            spawn_block(&mut commands, &mut meshes, &mut materials, &block);
        }
        for pos in museum::lamps(room) {
            lamp(&mut commands, pos, museum::lamp_intensity(room));
        }
    }
    spawn_exhibits(
        &mut commands,
        &mut meshes,
        &mut materials,
        ex,
        &data,
        &Edits::default(),
    );
    let Pose { eye, look, .. } = tour::tour_pose(ex, 0.0, 16.0 / 9.0);
    let (yaw, pitch) = tour::heading(eye, look);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(eye[0], eye[1], eye[2])
            .looking_at(Vec3::new(look[0], look[1], look[2]), Vec3::Y),
        Visitor {
            t: 0.0,
            x: eye[0],
            z: eye[2],
            yaw,
            pitch,
            mode: Mode::Guided,
        },
    ));
}

/// Accroche les tableaux et pose les sculptures, à leur place (modifiée par l'éditeur).
fn spawn_exhibits(
    commands: &mut Commands,
    meshes: Meshes,
    materials: Materials,
    ex: &Exhibition,
    data: &ExhibitionData,
    edits: &Edits,
) {
    for i in 0..ex.artworks.len() {
        hang(
            commands,
            meshes,
            materials,
            ex,
            data,
            &editor::edited_art(ex, edits, i),
        );
    }
    for i in 0..ex.sculptures.len() {
        sculpt(
            commands,
            meshes,
            materials,
            data,
            &editor::edited_sculpture(ex, edits, i),
        );
    }
}

/// Quand l'éditeur a déplacé une œuvre : retire toutes les œuvres de la scène et les
/// remet à leur nouvelle place.
fn apply_edits(
    current: Res<Current>,
    control: Option<Res<Control>>,
    mut applied: ResMut<Applied>,
    exhibits: Query<Entity, With<Exhibit>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(control) = control else { return };
    let edits = {
        let c = control.0.borrow();
        if c.edits.version == applied.0 {
            return;
        }
        c.edits.clone()
    };
    applied.0 = edits.version;
    for entity in exhibits.iter() {
        commands.entity(entity).despawn();
    }
    spawn_exhibits(
        &mut commands,
        &mut meshes,
        &mut materials,
        current.0,
        &current.1,
        &edits,
    );
}

/// Garde allumées les 8 lampes les plus proches du visiteur (le shader n'en voit pas plus).
fn nearest_lamps(
    cameras: Query<&Transform, With<Visitor>>,
    mut lamps: Query<(&Transform, &mut Visibility), With<Lamp>>,
) {
    let Some(eye) = cameras.iter().next().map(|t| t.translation) else {
        return;
    };
    let mut order: Vec<(f32, usize)> = lamps
        .iter()
        .enumerate()
        .map(|(i, (t, _))| (t.translation.distance(eye), i))
        .collect();
    order.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut on = vec![false; order.len()];
    for &(_, i) in order.iter().take(MAX_LAMPS) {
        on[i] = true;
    }
    for (i, (_, mut visibility)) in lamps.iter_mut().enumerate() {
        let wanted = if on[i] {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}

/// L'œuvre que regarde le visiteur libre : la plus proche dans son champ de vision.
fn focus_in_view(ex: &Exhibition, eye: [f32; 3], yaw: f32) -> Option<Focus> {
    let (fx, fz) = (yaw.sin(), yaw.cos());
    let score = |x: f32, z: f32, reach: f32| -> Option<f32> {
        let (dx, dz) = (x - eye[0], z - eye[2]);
        let d = dx.hypot(dz);
        let cos = (dx * fx + dz * fz) / d.max(1e-3);
        (d < reach && cos > 0.9).then_some(d)
    };
    let arts = ex.artworks.iter().enumerate().filter_map(|(i, a)| {
        let c = museum::artwork_center(ex, a);
        score(c[0], c[2], 5.5 + a.width).map(|d| (d, Focus::Art(i)))
    });
    let statues = ex
        .sculptures
        .iter()
        .enumerate()
        .filter_map(|(i, s)| score(s.x, s.z, 5.0 + s.height).map(|d| (d, Focus::Sculpture(i))));
    arts.chain(statues)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, f)| f)
}

fn walk(
    current: Res<Current>,
    control: Option<Res<Control>>,
    time: Res<Time>,
    windows: Query<&Window>,
    mut visitors: Query<(&mut Transform, &mut Visitor)>,
) {
    let ex = current.0;
    let aspect = windows.iter().next().map_or(16.0 / 9.0, |w| {
        w.resolution.width / w.resolution.height.max(1.0)
    });
    let dt = time.delta_secs().min(0.1);
    let mut asked = TourControl::default();
    if let Some(c) = control.as_ref() {
        let mut c = c.0.borrow_mut();
        asked.mode = c.mode;
        asked.paused = c.paused;
        asked.jump = std::mem::take(&mut c.jump);
        asked.go_to = c.go_to.take();
        asked.teleport = c.teleport.take();
        (asked.forward, asked.strafe, asked.run) = (c.forward, c.strafe, c.run);
        asked.turn = std::mem::take(&mut c.turn) + c.spin * dt;
        asked.tilt = std::mem::take(&mut c.tilt);
    }
    for (mut transform, mut v) in &mut visitors {
        // Passer d'un mode à l'autre : la visite libre part de là où était la caméra ; la
        // visite guidée reprend à l'œuvre suivante.
        if asked.mode != v.mode {
            if asked.mode == Mode::Guided {
                v.t = tour::jump_time(ex, v.t, 1);
            }
            v.mode = asked.mode;
        }
        if let Some(focus) = asked.go_to {
            if let Some(t) = tour::time_of(ex, focus) {
                v.t = t;
            }
            if v.mode == Mode::Free {
                let pose = tour::tour_pose(ex, v.t, aspect);
                (v.x, v.z) = (pose.eye[0], pose.eye[2]);
                (v.yaw, v.pitch) = tour::heading(pose.eye, pose.look);
            }
        }
        if let Some((x, z)) = asked.teleport
            && v.mode == Mode::Free
        {
            (v.x, v.z) = museum::push_out(museum::cached_obstacles(ex), x, z, 0.3);
        }
        let pose = match v.mode {
            Mode::Guided => {
                if asked.jump != 0 {
                    v.t = tour::jump_time(ex, v.t, asked.jump);
                } else if !asked.paused {
                    v.t += dt;
                }
                let pose = tour::tour_pose(ex, v.t, aspect);
                (v.x, v.z) = (pose.eye[0], pose.eye[2]);
                (v.yaw, v.pitch) = tour::heading(pose.eye, pose.look);
                pose
            }
            Mode::Free => {
                v.yaw -= asked.turn;
                v.pitch = (v.pitch - asked.tilt).clamp(-0.6, 0.6);
                let speed = if asked.run { 3.2 } else { 1.6 };
                let (fx, fz) = (v.yaw.sin(), v.yaw.cos());
                let (rx, rz) = (-fz, fx);
                let step = (
                    asked.forward * fx + asked.strafe * rx,
                    asked.forward * fz + asked.strafe * rz,
                );
                let (nx, nz) = museum::push_out(
                    museum::cached_obstacles(ex),
                    v.x + step.0 * speed * dt,
                    v.z + step.1 * speed * dt,
                    0.3,
                );
                // Ne pas sortir du musée par une porte extérieure.
                if museum::room_at(ex, nx, nz).is_some() {
                    (v.x, v.z) = (nx, nz);
                }
                let eye = [v.x, EYE, v.z];
                let look = [
                    v.x + fx * v.pitch.cos(),
                    EYE + v.pitch.sin(),
                    v.z + fz * v.pitch.cos(),
                ];
                Pose {
                    eye,
                    look,
                    focus: focus_in_view(ex, eye, v.yaw),
                }
            }
        };
        *transform = Transform::from_xyz(pose.eye[0], pose.eye[1], pose.eye[2])
            .looking_at(Vec3::new(pose.look[0], pose.look[1], pose.look[2]), Vec3::Y);
        if let Some(control) = control.as_ref() {
            let callback = {
                let mut c = control.0.borrow_mut();
                c.position = [v.x, v.z, v.yaw];
                // Le cartel garde la dernière œuvre vue jusqu'à la suivante : le bouton
                // « Voir en HD » reste toujours utilisable.
                match pose.focus {
                    Some(f) if c.current != Some(f) => {
                        c.current = Some(f);
                        c.on_change.clone().map(|cb| (cb, f))
                    }
                    _ => None,
                }
            };
            if let Some((callback, focus)) = callback {
                callback(Some(focus));
            }
        }
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

/// Le musée d'une exposition, avec sa visite guidée.
pub fn tour_app(ex: &'static Exhibition, data: Rc<ExhibitionData>) -> App {
    let mut app = App::new();
    app.add_plugins(plugins())
        .insert_resource(ambient())
        .insert_resource(Current(ex, data))
        .init_resource::<Applied>()
        .add_systems(Startup, setup_tour)
        .add_systems(Update, (walk, nearest_lamps, apply_edits));
    app
}

/// Le musée, piloté par la page : visite guidée ou libre, pause, œuvre suivante, cartel.
pub fn tour_app_with(
    ex: &'static Exhibition,
    data: Rc<ExhibitionData>,
    control: SharedControl,
) -> App {
    let mut app = tour_app(ex, data);
    app.insert_resource(Control(control));
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
            for (rgb, mesh) in mosaic_meshes(&art.mosaic, current.1.cells(&art.mosaic), w, h) {
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
pub fn showcase_app(ex: &'static Exhibition, data: Rc<ExhibitionData>) -> App {
    let mut app = App::new();
    app.add_plugins(plugins())
        .insert_resource(ambient())
        .insert_resource(Current(ex, data))
        .add_systems(Startup, setup_showcase)
        .add_systems(Update, turn);
    app
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artworks::EXHIBITIONS;

    fn data(ex: &Exhibition) -> Rc<ExhibitionData> {
        Rc::new(ExhibitionData::load(ex).expect("données"))
    }

    /// Les rectangles fusionnés couvrent chaque case une fois, de la bonne couleur.
    #[test]
    fn the_rectangles_cover_the_mosaic_exactly() {
        for ex in EXHIBITIONS {
            let data = data(ex);
            for art in ex.artworks {
                let m = &art.mosaic;
                let cells = data.cells(m);
                let cols = usize::from(m.cols);
                let mut seen = vec![false; cells.len()];
                let rects = rectangles(m, cells);
                for &(c, a, b, r0, r1) in &rects {
                    for row in r0..r1 {
                        for col in a..b {
                            let i = row * cols + col;
                            assert!(!seen[i] && cells[i] == c, "{}", art.title);
                            seen[i] = true;
                        }
                    }
                }
                assert!(seen.iter().all(|&s| s), "{}", art.title);
                assert!(
                    rects.len() < cells.len(),
                    "{} : rien n'a été fusionné",
                    art.title
                );
            }
        }
    }

    /// Le maillage de chaque sculpture est cohérent, à sa taille.
    #[test]
    fn sculptures_have_sound_meshes() {
        for ex in EXHIBITIONS {
            let data = data(ex);
            for s in ex.sculptures {
                let m = data.sculpture(s);
                let vertices = m.positions.len() / 3;
                assert!(
                    vertices > 100 && m.normals.len() == m.positions.len(),
                    "{}",
                    s.title
                );
                assert!(
                    m.indices.iter().all(|&i| (i as usize) < vertices),
                    "{}",
                    s.title
                );
                let top = m.positions.chunks(3).map(|p| p[1]).fold(0.0f32, f32::max);
                assert!(
                    (top - s.height).abs() < 0.02,
                    "{} : {top} m au lieu de {}",
                    s.title,
                    s.height
                );
            }
        }
    }

    /// Après plusieurs secondes, chaque pavé est toujours à sa place (la physique est bien
    /// retirée) et 8 lampes au plus sont allumées.
    #[test]
    fn the_museum_stands_still_and_fits_the_renderer() {
        for ex in EXHIBITIONS {
            for mut app in [tour_app(ex, data(ex)), showcase_app(ex, data(ex))] {
                app.update();
                let first =
                    RenderFrame::extract(app.world(), Vec2::new(800.0, 450.0)).expect("une caméra");
                let before: Vec<[f32; 3]> = first
                    .frame()
                    .meshes
                    .iter()
                    .map(|d| d.world.translation().to_array())
                    .collect();
                for _ in 0..60 {
                    app.update();
                }
                let later =
                    RenderFrame::extract(app.world(), Vec2::new(800.0, 450.0)).expect("une caméra");
                let frame = later.frame();
                assert_eq!(frame.meshes.len(), before.len());
                assert!(
                    frame.lights.point.len() <= MAX_LAMPS,
                    "{} : {} lampes",
                    ex.slug,
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

    /// Déplacer une œuvre dans l'éditeur reconstruit la scène, sans rien perdre ni doubler.
    #[test]
    fn edits_move_works_in_the_scene() {
        let ex = EXHIBITIONS
            .iter()
            .find(|e| !e.sculptures.is_empty())
            .expect("des sculptures");
        let control = SharedControl::default();
        let mut app = tour_app_with(ex, data(ex), control.clone());
        app.update();
        let before =
            RenderFrame::extract(app.world(), Vec2::new(800.0, 450.0)).map(|f| f.mesh_count());
        {
            let mut c = control.borrow_mut();
            let mut e = c.edits.sculpture(ex, 0);
            e.x += 0.5;
            c.edits.sculptures.insert(0, e);
            c.edits.version += 1;
        }
        app.update();
        app.update();
        let after =
            RenderFrame::extract(app.world(), Vec2::new(800.0, 450.0)).map(|f| f.mesh_count());
        assert_eq!(after, before);
    }

    /// En visite libre, le visiteur avance, tourne, et s'arrête contre les murs.
    #[test]
    fn free_walking_stops_at_walls() {
        let ex = &EXHIBITIONS[0];
        let control = SharedControl::default();
        let mut app = tour_app_with(ex, data(ex), control.clone());
        app.update();
        {
            let mut c = control.borrow_mut();
            c.mode = Mode::Free;
            c.forward = 1.0;
            c.run = true;
        }
        let start = control.borrow().position;
        for _ in 0..600 {
            app.update();
        }
        let end = control.borrow().position;
        assert!(
            (end[0] - start[0]).hypot(end[1] - start[1]) > 0.5,
            "le visiteur n'a pas bougé"
        );
        assert!(
            museum::room_at(ex, end[0], end[1]).is_some(),
            "le visiteur est sorti du musée"
        );
        let walls = museum::cached_obstacles(ex);
        assert!(
            !walls
                .iter()
                .any(|&(x0, x1, z0, z1)| end[0] > x0 && end[0] < x1 && end[1] > z0 && end[1] < z1)
        );
    }
}
