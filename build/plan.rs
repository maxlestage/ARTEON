//! Le plan d'un musée : les salles placées sur le plan, leurs portes, et les murs qui
//! en découlent. Les coordonnées sont en mètres : x vers l'est, z vers le sud (le
//! visiteur entre par le sud et avance vers le nord, les z négatifs).

/// Un mur d'une salle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    North,
    South,
    East,
    West,
}

impl Side {
    pub fn parse(text: &str) -> Option<Side> {
        match text.trim() {
            "nord" | "fond" => Some(Side::North),
            "sud" | "entree" | "entrée" => Some(Side::South),
            "est" | "droite" => Some(Side::East),
            "ouest" | "gauche" => Some(Side::West),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::North => "North",
            Side::South => "South",
            Side::East => "East",
            Side::West => "West",
        }
    }

    pub fn french(self) -> &'static str {
        match self {
            Side::North => "nord",
            Side::South => "sud",
            Side::East => "est",
            Side::West => "ouest",
        }
    }

    pub fn opposite(self) -> Side {
        match self {
            Side::North => Side::South,
            Side::South => Side::North,
            Side::East => Side::West,
            Side::West => Side::East,
        }
    }

    /// La direction vers la droite du visiteur qui regarde ce mur depuis la salle :
    /// c'est le sens des positions le long du mur.
    pub fn right(self) -> (f32, f32) {
        match self {
            Side::West => (0.0, -1.0),
            Side::East => (0.0, 1.0),
            Side::North => (1.0, 0.0),
            Side::South => (-1.0, 0.0),
        }
    }
}

/// Une porte : sur un mur, à `position` mètres du milieu du mur (vers la droite de qui
/// regarde le mur depuis la salle), large de `width`.
#[derive(Clone, Copy, Debug)]
pub struct Door {
    pub side: Side,
    pub position: f32,
    pub width: f32,
}

/// Une salle placée sur le plan.
#[derive(Clone, Debug)]
pub struct Room {
    pub name: String,
    pub theme: String,
    pub walls: [u8; 3],
    pub x: f32,
    pub z: f32,
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub corridor: bool,
    pub doors: Vec<Door>,
    /// Le numéro de la salle dans le fichier (0 : un couloir ajouté par l'enfilade).
    pub number: usize,
}

/// L'épaisseur d'un mur (posé à l'intérieur de la salle, contre sa limite).
pub const WALL: f32 = 0.2;
pub const DOOR_WIDTH: f32 = 3.0;

impl Room {
    /// La longueur d'un mur.
    pub fn wall_length(&self, side: Side) -> f32 {
        match side {
            Side::North | Side::South => self.width,
            Side::East | Side::West => self.depth,
        }
    }

    /// Le point (x, z) du mur `side` à `position` mètres de son milieu, sur la limite de
    /// la salle.
    pub fn point_on(&self, side: Side, position: f32) -> (f32, f32) {
        let (rx, rz) = side.right();
        let (cx, cz) = match side {
            Side::North => (self.x, self.z - self.depth / 2.0),
            Side::South => (self.x, self.z + self.depth / 2.0),
            Side::East => (self.x + self.width / 2.0, self.z),
            Side::West => (self.x - self.width / 2.0, self.z),
        };
        (cx + rx * position, cz + rz * position)
    }

    /// La position le long du mur `side` du point (x, z) qui est sur sa limite.
    pub fn position_of(&self, side: Side, x: f32, z: f32) -> f32 {
        let (cx, cz) = self.point_on(side, 0.0);
        let (rx, rz) = side.right();
        (x - cx) * rx + (z - cz) * rz
    }

    /// Le nombre de places numérotées d'un mur (une tous les 3 m environ).
    pub fn places(&self, side: Side) -> u8 {
        ((self.wall_length(side) / 3.3).floor() as u8).max(1)
    }

    /// La position (le long du mur) de la place `place` (1 à `places`), de gauche à droite.
    pub fn place_position(&self, side: Side, place: u8) -> f32 {
        let n = f32::from(self.places(side));
        let l = self.wall_length(side);
        -l / 2.0 + l * (f32::from(place) - 0.5) / n
    }

    /// Les intervalles (le long du mur) occupés par les portes du mur `side`.
    pub fn door_spans(&self, side: Side) -> Vec<(f32, f32)> {
        self.doors
            .iter()
            .filter(|d| d.side == side)
            .map(|d| (d.position - d.width / 2.0, d.position + d.width / 2.0))
            .collect()
    }

    fn rect(&self) -> (f32, f32, f32, f32) {
        (
            self.x - self.width / 2.0,
            self.x + self.width / 2.0,
            self.z - self.depth / 2.0,
            self.z + self.depth / 2.0,
        )
    }
}

/// Deux salles se chevauchent-elles (plus que leur limite commune) ?
pub fn overlap(a: &Room, b: &Room) -> bool {
    let (ax0, ax1, az0, az1) = a.rect();
    let (bx0, bx1, bz0, bz1) = b.rect();
    ax0 < bx1 - 0.05 && bx0 < ax1 - 0.05 && az0 < bz1 - 0.05 && bz0 < az1 - 0.05
}

/// La salle (autre que `except`) dont la limite opposée passe par le point (x, z) d'une
/// porte du mur `side` : celle où mène la porte. Renvoie son index.
pub fn room_behind(rooms: &[Room], except: usize, side: Side, x: f32, z: f32) -> Option<usize> {
    rooms.iter().enumerate().position(|(i, r)| {
        if i == except {
            return false;
        }
        let (x0, x1, z0, z1) = r.rect();
        match side.opposite() {
            Side::North => (z - z0).abs() < 0.05 && x > x0 + 0.5 && x < x1 - 0.5,
            Side::South => (z - z1).abs() < 0.05 && x > x0 + 0.5 && x < x1 - 0.5,
            Side::East => (x - x1).abs() < 0.05 && z > z0 + 0.5 && z < z1 - 0.5,
            Side::West => (x - x0).abs() < 0.05 && z > z0 + 0.5 && z < z1 - 0.5,
        }
    })
}

/// Ajoute à chaque salle les portes qui mènent vers elle depuis ses voisines : une porte
/// se déclare d'un seul côté.
pub fn mirror_doors(rooms: &mut [Room]) {
    let mut added: Vec<(usize, Door)> = Vec::new();
    for (i, room) in rooms.iter().enumerate() {
        for door in &room.doors {
            let (x, z) = room.point_on(door.side, door.position);
            if let Some(j) = room_behind(rooms, i, door.side, x, z) {
                let side = door.side.opposite();
                let position = rooms[j].position_of(side, x, z);
                let exists = rooms[j]
                    .doors
                    .iter()
                    .any(|d| d.side == side && (d.position - position).abs() < 0.3)
                    || added.iter().any(|(k, d)| {
                        *k == j && d.side == side && (d.position - position).abs() < 0.3
                    });
                if !exists {
                    added.push((
                        j,
                        Door {
                            side,
                            position,
                            width: door.width,
                        },
                    ));
                }
            }
        }
    }
    for (j, door) in added {
        rooms[j].doors.push(door);
    }
}
