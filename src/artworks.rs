//! Le catalogue d'ARTEON : l'exposition, ses salles et ses œuvres.
//!
//! Le contenu vient de `exposition.toml` et des images de `public/tableaux/` : `build.rs`
//! les vérifie et les écrit en Rust (avec la mosaïque de chaque tableau), inclus
//! ci-dessous. Ce module est partagé par la scène 3D (src/gallery.rs) et par les pages
//! rendues sur le serveur.

/// L'exposition : ses textes et l'œuvre de la vitrine (page d'accueil).
#[derive(Clone, Copy, Debug)]
pub struct Exhibition {
    pub title: &'static str,
    pub artist: &'static str,
    pub artist_dates: &'static str,
    pub kicker: &'static str,
    pub introduction: &'static str,
    /// L'index, dans `ARTWORKS`, de l'œuvre qui tourne sur le chevalet de l'accueil.
    pub showcase: Option<usize>,
}

/// Le mur d'une salle qui porte un tableau.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wall {
    /// Le mur de gauche (x < 0), quand on avance dans la galerie.
    Left,
    /// Le mur de droite (x > 0).
    Right,
    /// Le mur du fond de la salle.
    Back,
}

/// L'emplacement d'un tableau : une salle (à partir de 0), un mur et une place (1 à 3).
#[derive(Clone, Copy, Debug)]
pub struct Hook {
    pub room: usize,
    pub wall: Wall,
    pub place: u8,
}

impl Hook {
    /// La position du centre du tableau le long de son mur : z pour les murs latéraux
    /// (la place 1 est près de l'entrée), x pour le mur du fond (de gauche à droite).
    pub fn along(&self) -> f32 {
        let step = f32::from(self.place) - 2.0;
        match self.wall {
            Wall::Left | Wall::Right => ROOMS[self.room].center_z - step * PLACE_SPACING,
            Wall::Back => step * PLACE_SPACING,
        }
    }
}

/// La mosaïque d'un tableau : une palette et une couleur par case, ligne par ligne
/// depuis le haut.
#[derive(Clone, Copy, Debug)]
pub struct Mosaic {
    pub cols: u16,
    pub rows: u16,
    pub palette: &'static [[u8; 3]],
    pub cells: &'static [u8],
}

impl Mosaic {
    /// La couleur de la case (`col`, `row`), `row` 0 en haut.
    pub fn color(&self, col: usize, row: usize) -> [u8; 3] {
        let index = self.cells[row * usize::from(self.cols) + col];
        self.palette[usize::from(index)]
    }
}

/// Une œuvre exposée.
#[derive(Clone, Copy, Debug)]
pub struct Artwork {
    pub title: &'static str,
    pub year: &'static str,
    /// Le nom du fichier dans `public/tableaux/` (servi à `/tableaux/…`).
    pub image: &'static str,
    pub technique: &'static str,
    pub collection: &'static str,
    pub note: &'static str,
    /// Largeur et hauteur dans la galerie, en mètres.
    pub width: f32,
    pub height: f32,
    pub hook: Hook,
    pub mosaic: Mosaic,
}

impl Artwork {
    /// L'adresse de l'image, pour les pages.
    pub fn image_url(&self) -> String {
        format!("/tableaux/{}", self.image)
    }
}

/// Une salle d'exposition.
#[derive(Clone, Copy, Debug)]
pub struct Room {
    pub name: &'static str,
    pub theme: &'static str,
    /// La couleur des murs.
    pub walls: [u8; 3],
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
/// L'écart entre deux places voisines sur un mur (doit correspondre à build.rs).
pub const PLACE_SPACING: f32 = 3.0;

include!(concat!(env!("OUT_DIR"), "/exposition.rs"));

/// Les œuvres d'une salle, dans l'ordre de la visite : les murs latéraux de l'entrée
/// vers la sortie (gauche puis droite à chaque place), puis le mur du fond.
pub fn works_in_room(room: usize) -> Vec<&'static Artwork> {
    let mut works: Vec<&Artwork> = ARTWORKS.iter().filter(|a| a.hook.room == room).collect();
    works.sort_by_key(|a| match a.hook.wall {
        Wall::Left => (0, a.hook.place, 0),
        Wall::Right => (0, a.hook.place, 1),
        Wall::Back => (1, a.hook.place, 0),
    });
    works
}

/// `rgb(…)` pour le CSS.
pub fn css_rgb([r, g, b]: [u8; 3]) -> String {
    format!("rgb({r},{g},{b})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_exhibition_is_complete() {
        assert!(!ROOMS.is_empty());
        for art in ARTWORKS {
            assert!(art.hook.room < ROOMS.len(), "{}", art.title);
            let m = &art.mosaic;
            assert_eq!(
                m.cells.len(),
                usize::from(m.cols) * usize::from(m.rows),
                "{}",
                art.title
            );
            assert!(
                m.cells.iter().all(|&c| usize::from(c) < m.palette.len()),
                "{}",
                art.title
            );
            assert!(
                std::path::Path::new("public/tableaux")
                    .join(art.image)
                    .is_file(),
                "{}",
                art.image
            );
        }
        if let Some(i) = EXHIBITION.showcase {
            assert!(i < ARTWORKS.len());
        }
    }

    #[test]
    fn every_work_appears_once_in_the_visit() {
        let visited: usize = (0..ROOMS.len()).map(|r| works_in_room(r).len()).sum();
        assert_eq!(visited, ARTWORKS.len());
    }
}
