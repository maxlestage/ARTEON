//! Le catalogue d'ARTEON : des œuvres du domaine public, réinterprétées en volumes.
//!
//! Tous les artistes sont morts depuis plus de 70 ans : leurs œuvres sont dans le
//! domaine public en Europe. Le moteur 3D n'affiche pas d'images ; chaque tableau est
//! donc décrit par ses aplats de couleur (des rectangles), puis construit en relief
//! dans la galerie. Ce sont des interprétations géométriques, pas des reproductions
//! exactes. Ce module ne contient que des données : il est partagé par la scène 3D
//! (src/gallery.rs) et par les pages rendues sur le serveur.

/// Le mur d'une salle qui porte un tableau.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wall {
    /// Le mur de gauche (x = -5), quand on avance dans la galerie.
    Left,
    /// Le mur de droite (x = +5).
    Right,
    /// Le mur du fond de la dernière salle.
    Back,
}

/// L'emplacement d'un tableau : une salle, un mur, et une position le long du mur
/// (la coordonnée z pour les murs latéraux, x pour le mur du fond).
#[derive(Clone, Copy, Debug)]
pub struct Hook {
    pub room: usize,
    pub wall: Wall,
    pub along: f32,
}

/// Un aplat de couleur, en coordonnées normalisées (0 à 1, origine en bas à gauche).
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub rgb: [u8; 3],
}

const fn s(x: f32, y: f32, w: f32, h: f32, rgb: [u8; 3]) -> Shape {
    Shape { x, y, w, h, rgb }
}

/// Une œuvre exposée.
#[derive(Clone, Copy, Debug)]
pub struct Artwork {
    pub title: &'static str,
    pub artist: &'static str,
    pub artist_dates: &'static str,
    pub year: u16,
    pub collection: &'static str,
    pub note: &'static str,
    /// Largeur et hauteur dans la galerie, en mètres.
    pub width: f32,
    pub height: f32,
    pub background: [u8; 3],
    pub shapes: &'static [Shape],
    pub hook: Hook,
}

/// Une salle d'exposition.
#[derive(Clone, Copy, Debug)]
pub struct Room {
    pub name: &'static str,
    pub theme: &'static str,
    /// Le centre de la salle sur l'axe z (la galerie s'enfonce vers les z négatifs).
    pub center_z: f32,
}

/// Les dimensions de la galerie, en mètres.
pub const ROOM_HALF_WIDTH: f32 = 5.0;
pub const ROOM_HALF_DEPTH: f32 = 5.0;
pub const CORRIDOR_HALF_WIDTH: f32 = 1.5;
pub const CORRIDOR_LENGTH: f32 = 6.0;
pub const WALL_HEIGHT: f32 = 4.0;
pub const WALL_THICKNESS: f32 = 0.2;
/// La hauteur du centre des tableaux.
pub const HANGING_HEIGHT: f32 = 1.8;

pub const ROOMS: [Room; 3] = [
    Room {
        name: "Salle I · De Stijl",
        theme: "Lignes noires, plans primaires : la peinture réduite à l'essentiel.",
        center_z: 0.0,
    },
    Room {
        name: "Salle II · Suprématisme",
        theme: "Des formes pures qui flottent sur le blanc, et un architectone au centre.",
        center_z: -16.0,
    },
    Room {
        name: "Salle III · Bauhaus, Dada et finale",
        theme: "Mosaïques de couleurs, puis un dernier Mondrian au fond de la galerie.",
        center_z: -32.0,
    },
];

const W: [u8; 3] = [242, 239, 230];
const K: [u8; 3] = [22, 22, 22];
const R: [u8; 3] = [200, 32, 38];
const Y: [u8; 3] = [242, 200, 40];
const B: [u8; 3] = [28, 58, 140];
const G: [u8; 3] = [196, 196, 192];

pub const ARTWORKS: [Artwork; 9] = [
    Artwork {
        title: "Composition avec grand plan rouge, jaune, noir, gris et bleu",
        artist: "Piet Mondrian",
        artist_dates: "1872–1944",
        year: 1921,
        collection: "Kunstmuseum, La Haye",
        note: "Un grand plan rouge domine une grille de lignes noires : l'équilibre par l'asymétrie.",
        width: 1.3,
        height: 1.3,
        background: W,
        shapes: &[
            s(0.30, 0.36, 0.70, 0.64, R),
            s(0.0, 0.62, 0.27, 0.38, G),
            s(0.0, 0.0, 0.27, 0.18, B),
            s(0.84, 0.0, 0.16, 0.33, Y),
            s(0.31, 0.0, 0.17, 0.12, K),
            s(0.27, 0.0, 0.03, 1.0, K),
            s(0.0, 0.33, 1.0, 0.03, K),
            s(0.0, 0.59, 0.27, 0.03, K),
            s(0.0, 0.18, 0.27, 0.03, K),
            s(0.81, 0.0, 0.03, 0.33, K),
            s(0.48, 0.0, 0.03, 0.33, K),
        ],
        hook: Hook {
            room: 0,
            wall: Wall::Left,
            along: 2.5,
        },
    },
    Artwork {
        title: "Composition en rouge, jaune, bleu et noir",
        artist: "Piet Mondrian",
        artist_dates: "1872–1944",
        year: 1921,
        collection: "Kunstmuseum, La Haye",
        note: "Le blanc devient le sujet : les couleurs sont repoussées sur les bords.",
        width: 1.2,
        height: 1.2,
        background: W,
        shapes: &[
            s(0.0, 0.75, 0.22, 0.25, Y),
            s(0.0, 0.30, 0.22, 0.42, R),
            s(0.78, 0.0, 0.22, 0.27, B),
            s(0.25, 0.0, 0.18, 0.08, K),
            s(0.78, 0.88, 0.22, 0.12, K),
            s(0.22, 0.0, 0.03, 1.0, K),
            s(0.75, 0.0, 0.03, 1.0, K),
            s(0.0, 0.72, 0.22, 0.03, K),
            s(0.0, 0.27, 1.0, 0.03, K),
            s(0.78, 0.85, 0.22, 0.03, K),
            s(0.43, 0.0, 0.03, 0.27, K),
        ],
        hook: Hook {
            room: 0,
            wall: Wall::Left,
            along: -2.5,
        },
    },
    Artwork {
        title: "Composition XI",
        artist: "Theo van Doesburg",
        artist_dates: "1883–1931",
        year: 1918,
        collection: "Solomon R. Guggenheim Museum, New York",
        note: "Des rectangles libres, sans ligne noire, qui rythment la surface comme une partition.",
        width: 1.1,
        height: 1.4,
        background: W,
        shapes: &[
            s(0.08, 0.70, 0.22, 0.12, B),
            s(0.40, 0.78, 0.10, 0.14, Y),
            s(0.62, 0.62, 0.28, 0.08, R),
            s(0.10, 0.42, 0.08, 0.20, K),
            s(0.30, 0.48, 0.30, 0.08, G),
            s(0.70, 0.30, 0.12, 0.22, B),
            s(0.15, 0.12, 0.26, 0.10, R),
            s(0.50, 0.10, 0.10, 0.24, Y),
            s(0.80, 0.08, 0.12, 0.10, K),
        ],
        hook: Hook {
            room: 0,
            wall: Wall::Right,
            along: 2.5,
        },
    },
    Artwork {
        title: "Composition VIII (La Vache)",
        artist: "Theo van Doesburg",
        artist_dates: "1883–1931",
        year: 1918,
        collection: "Museum of Modern Art, New York",
        note: "Une vache, abstraite étape par étape jusqu'à ne plus être que des plans colorés.",
        width: 1.4,
        height: 1.0,
        background: W,
        shapes: &[
            s(0.10, 0.55, 0.20, 0.25, B),
            s(0.32, 0.62, 0.12, 0.30, K),
            s(0.48, 0.70, 0.25, 0.12, R),
            s(0.20, 0.30, 0.30, 0.15, Y),
            s(0.55, 0.35, 0.15, 0.25, B),
            s(0.75, 0.45, 0.15, 0.10, K),
            s(0.10, 0.10, 0.15, 0.12, R),
            s(0.40, 0.08, 0.25, 0.12, G),
            s(0.75, 0.12, 0.15, 0.20, Y),
        ],
        hook: Hook {
            room: 0,
            wall: Wall::Right,
            along: -2.5,
        },
    },
    Artwork {
        title: "Carré noir",
        artist: "Kasimir Malevitch",
        artist_dates: "1879–1935",
        year: 1915,
        collection: "Galerie Tretiakov, Moscou",
        note: "Le « degré zéro » de la peinture : un carré noir sur fond blanc.",
        width: 1.4,
        height: 1.4,
        background: [236, 230, 214],
        shapes: &[s(0.12, 0.12, 0.76, 0.76, K)],
        hook: Hook {
            room: 1,
            wall: Wall::Left,
            along: -16.0,
        },
    },
    Artwork {
        title: "Supremus n° 58",
        artist: "Kasimir Malevitch",
        artist_dates: "1879–1935",
        year: 1916,
        collection: "Musée russe, Saint-Pétersbourg",
        note: "Des barres colorées en apesanteur : la peinture comme espace sans gravité.",
        width: 1.2,
        height: 1.5,
        background: [238, 234, 222],
        shapes: &[
            s(0.15, 0.65, 0.45, 0.10, K),
            s(0.30, 0.40, 0.12, 0.40, R),
            s(0.50, 0.45, 0.35, 0.06, B),
            s(0.20, 0.20, 0.20, 0.12, Y),
            s(0.55, 0.18, 0.08, 0.20, K),
            s(0.65, 0.70, 0.15, 0.05, R),
            s(0.10, 0.45, 0.10, 0.04, G),
            s(0.70, 0.30, 0.18, 0.08, [60, 110, 70]),
        ],
        hook: Hook {
            room: 1,
            wall: Wall::Right,
            along: -16.0,
        },
    },
    Artwork {
        title: "Château et soleil",
        artist: "Paul Klee",
        artist_dates: "1879–1940",
        year: 1928,
        collection: "Collection particulière",
        note: "Une ville rêvée en mosaïque de tons chauds, sous un petit soleil rouge.",
        width: 1.3,
        height: 1.2,
        background: [120, 70, 50],
        shapes: &[
            s(0.05, 0.05, 0.25, 0.20, [214, 120, 48]),
            s(0.30, 0.05, 0.20, 0.20, [170, 70, 40]),
            s(0.50, 0.05, 0.25, 0.20, [230, 170, 70]),
            s(0.75, 0.05, 0.20, 0.20, [150, 60, 50]),
            s(0.15, 0.25, 0.25, 0.20, [236, 190, 90]),
            s(0.40, 0.25, 0.20, 0.20, [200, 90, 50]),
            s(0.60, 0.25, 0.25, 0.20, [180, 110, 60]),
            s(0.25, 0.45, 0.20, 0.15, [220, 140, 60]),
            s(0.45, 0.45, 0.20, 0.15, [160, 80, 60]),
            s(0.35, 0.60, 0.20, 0.12, [236, 200, 110]),
            s(0.42, 0.72, 0.06, 0.10, [150, 60, 50]),
            s(0.08, 0.75, 0.10, 0.10, [220, 60, 40]),
        ],
        hook: Hook {
            room: 2,
            wall: Wall::Left,
            along: -32.0,
        },
    },
    Artwork {
        title: "Composition verticale-horizontale",
        artist: "Sophie Taeuber-Arp",
        artist_dates: "1889–1943",
        year: 1916,
        collection: "Fondation Arp, Clamart",
        note: "Une grille textile de rectangles, entre art appliqué et abstraction pure.",
        width: 1.1,
        height: 1.4,
        background: [230, 220, 200],
        shapes: &[
            s(0.05, 0.78, 0.40, 0.17, R),
            s(0.55, 0.78, 0.40, 0.17, B),
            s(0.05, 0.56, 0.18, 0.18, Y),
            s(0.27, 0.56, 0.46, 0.18, K),
            s(0.77, 0.56, 0.18, 0.18, Y),
            s(0.05, 0.34, 0.40, 0.18, B),
            s(0.55, 0.34, 0.40, 0.18, R),
            s(0.05, 0.05, 0.18, 0.25, K),
            s(0.27, 0.05, 0.20, 0.25, Y),
            s(0.51, 0.05, 0.20, 0.25, G),
            s(0.75, 0.05, 0.20, 0.25, B),
        ],
        hook: Hook {
            room: 2,
            wall: Wall::Right,
            along: -32.0,
        },
    },
    Artwork {
        title: "Composition avec rouge, bleu et jaune",
        artist: "Piet Mondrian",
        artist_dates: "1872–1944",
        year: 1930,
        collection: "Kunsthaus, Zurich",
        note: "Le grand carré rouge qui clôt la visite, au fond de la dernière salle.",
        width: 1.6,
        height: 1.6,
        background: W,
        shapes: &[
            s(0.28, 0.30, 0.72, 0.70, R),
            s(0.0, 0.0, 0.25, 0.27, B),
            s(0.92, 0.0, 0.08, 0.12, Y),
            s(0.25, 0.0, 0.03, 1.0, K),
            s(0.0, 0.27, 1.0, 0.03, K),
            s(0.0, 0.66, 0.25, 0.03, K),
            s(0.89, 0.0, 0.03, 0.27, K),
            s(0.92, 0.12, 0.08, 0.03, K),
        ],
        hook: Hook {
            room: 2,
            wall: Wall::Back,
            along: 0.0,
        },
    },
];

/// Une couleur au format CSS (`rgb(r g b)`), pour les vignettes des pages.
pub fn css_rgb(rgb: [u8; 3]) -> String {
    format!("rgb({} {} {})", rgb[0], rgb[1], rgb[2])
}
