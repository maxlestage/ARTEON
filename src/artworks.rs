//! Le catalogue d'ARTEON : les expositions, le plan de leurs musées, leurs tableaux et
//! leurs sculptures.
//!
//! Le contenu vient de `expositions/<nom>.toml`, des images de `public/tableaux/<nom>/`
//! et des modèles 3D de `modeles/` : `build.rs` les vérifie et les écrit en Rust (inclus
//! ci-dessous). Les données lourdes (mosaïques, maillages) sont dans un fichier à part,
//! `donnees/<nom>.bin`, lu par [`crate::data`]. Les coordonnées sont en mètres : x vers
//! l'est, z vers le sud ; le visiteur entre par le sud et avance vers le nord.

/// Une exposition : ses textes, le plan de son musée et ses œuvres.
#[derive(Clone, Copy, Debug)]
pub struct Exhibition {
    /// Le nom du fichier `.toml`, et l'adresse de l'exposition (`/{slug}`).
    pub slug: &'static str,
    /// Le texte du fichier `.toml` (l'éditeur le réécrit avec les modifications).
    pub source: &'static str,
    /// Son nom dans le menu (« Van Gogh »).
    pub short_name: &'static str,
    pub title: &'static str,
    pub artist: &'static str,
    pub artist_dates: &'static str,
    pub kicker: &'static str,
    pub introduction: &'static str,
    /// La couleur d'accent de ses pages.
    pub accent: [u8; 3],
    /// L'index, dans `artworks`, de l'œuvre qui tourne sur le chevalet de sa présentation.
    pub showcase: Option<usize>,
    /// La taille de son fichier de données, en octets.
    pub data_size: u32,
    /// Les salles du musée, dans l'ordre de la visite (couloirs compris).
    pub rooms: &'static [Room],
    /// Les tableaux, dans l'ordre du fichier.
    pub artworks: &'static [Artwork],
    /// Les sculptures, dans l'ordre du fichier.
    pub sculptures: &'static [Sculpture],
    /// Les artistes exposés : leur vie et leur manière.
    pub artists: &'static [Artist],
}

/// Un artiste de l'exposition (bloc `[[artistes]]`).
#[derive(Debug)]
pub struct Artist {
    pub name: &'static str,
    /// Les dates et lieux de naissance et de mort.
    pub life: &'static str,
    pub biography: &'static str,
    pub style: &'static str,
}

/// La notice complète d'une œuvre. Chaque texte peut être vide.
#[derive(Clone, Copy, Debug)]
pub struct Notice {
    /// Ce que l'on voit.
    pub description: &'static str,
    /// Ce qu'elle évoque.
    pub evocation: &'static str,
    /// Son histoire.
    pub story: &'static str,
    /// La manière : technique, couleurs, composition.
    pub manner: &'static str,
    /// À regarder de près.
    pub details: &'static [&'static str],
    pub themes: &'static [&'static str],
    /// Le saviez-vous ?
    pub anecdote: &'static str,
}

impl Notice {
    /// Les rubriques de texte non vides, avec leur titre.
    pub fn sections(&self) -> impl Iterator<Item = (&'static str, &'static str)> {
        [
            ("Ce que l'on voit", self.description),
            ("Ce qu'elle évoque", self.evocation),
            ("Son histoire", self.story),
            ("La manière", self.manner),
        ]
        .into_iter()
        .filter(|(_, text)| !text.is_empty())
    }

    pub fn is_empty(&self) -> bool {
        self.sections().next().is_none() && self.details.is_empty() && self.anecdote.is_empty()
    }
}

/// Une œuvre de l'exposition, toile ou sculpture.
#[derive(Clone, Copy)]
pub enum Work {
    Art(&'static Artwork),
    Sculpture(&'static Sculpture),
}

impl Work {
    pub fn title(&self) -> &'static str {
        match self {
            Work::Art(a) => a.title,
            Work::Sculpture(s) => s.title,
        }
    }
    pub fn slug(&self) -> &'static str {
        match self {
            Work::Art(a) => a.slug,
            Work::Sculpture(s) => s.slug,
        }
    }
    pub fn notice(&self) -> &'static Notice {
        match self {
            Work::Art(a) => &a.notice,
            Work::Sculpture(s) => &s.notice,
        }
    }
    pub fn artist(&self, ex: &Exhibition) -> &'static str {
        match self {
            Work::Art(a) => a.artist_in(ex),
            Work::Sculpture(s) => s.artist,
        }
    }
}

impl Exhibition {
    /// Toutes les œuvres, toiles puis sculptures.
    pub fn works(&'static self) -> impl Iterator<Item = Work> {
        self.artworks
            .iter()
            .map(Work::Art)
            .chain(self.sculptures.iter().map(Work::Sculpture))
    }

    /// L'œuvre dont la page est `/<exposition>/oeuvres/<slug>`.
    pub fn work(&'static self, slug: &str) -> Option<Work> {
        self.works().find(|w| w.slug() == slug)
    }

    /// Le bloc `[[artistes]]` d'un artiste.
    pub fn artist_named(&self, name: &str) -> Option<&'static Artist> {
        self.artists.iter().find(|a| a.name == name)
    }

    /// L'exposition dont l'adresse est `/{slug}`.
    pub fn find(slug: &str) -> Option<&'static Exhibition> {
        EXHIBITIONS.iter().find(|e| e.slug == slug)
    }

    /// L'œuvre de la vitrine, ou la première.
    pub fn cover(&self) -> Option<&'static Artwork> {
        self.showcase
            .or(if self.artworks.is_empty() {
                None
            } else {
                Some(0)
            })
            .map(|i| &self.artworks[i])
    }

    /// Les salles d'exposition (sans les couloirs), avec leur index dans `rooms`.
    pub fn galleries(&self) -> impl Iterator<Item = (usize, &'static Room)> {
        self.rooms.iter().enumerate().filter(|(_, r)| !r.corridor)
    }

    /// Les tableaux d'une salle, de gauche à droite, mur après mur (ouest, nord, est, sud).
    pub fn works_in_room(&self, room: usize) -> Vec<&'static Artwork> {
        let mut works: Vec<&Artwork> = self
            .artworks
            .iter()
            .filter(|a| a.hook.room == room)
            .collect();
        works.sort_by(|a, b| {
            (a.hook.side.order(), a.hook.position)
                .partial_cmp(&(b.hook.side.order(), b.hook.position))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        works
    }

    /// Les sculptures d'une salle.
    pub fn sculptures_in_room(&self, room: usize) -> Vec<&'static Sculpture> {
        self.sculptures.iter().filter(|s| s.room == room).collect()
    }

    /// Les années couvertes par ses œuvres (« 1885–1890 »).
    pub fn years(&self) -> String {
        let numbers: Vec<u32> = self
            .artworks
            .iter()
            .map(|a| a.year)
            .chain(self.sculptures.iter().map(|s| s.year))
            .filter_map(|y| {
                y.chars()
                    .filter(char::is_ascii_digit)
                    .take(4)
                    .collect::<String>()
                    .parse()
                    .ok()
            })
            .filter(|y| *y > 999)
            .collect();
        match (numbers.iter().min(), numbers.iter().max()) {
            (Some(first), Some(last)) if first != last => format!("{first}–{last}"),
            (Some(year), _) => year.to_string(),
            _ => "—".into(),
        }
    }
}

/// Un mur d'une salle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    North,
    South,
    East,
    West,
}

impl Side {
    /// Son nom en français.
    pub fn french(self) -> &'static str {
        match self {
            Side::North => "nord",
            Side::South => "sud",
            Side::East => "est",
            Side::West => "ouest",
        }
    }

    /// L'ordre des murs dans la visite d'une salle (en tournant depuis l'entrée sud).
    pub fn order(self) -> u8 {
        match self {
            Side::West => 0,
            Side::North => 1,
            Side::East => 2,
            Side::South => 3,
        }
    }

    /// La direction vers la droite du visiteur qui regarde ce mur depuis la salle (x, z).
    pub fn right(self) -> (f32, f32) {
        match self {
            Side::West => (0.0, -1.0),
            Side::East => (0.0, 1.0),
            Side::North => (1.0, 0.0),
            Side::South => (-1.0, 0.0),
        }
    }

    /// La direction qui va du mur vers l'intérieur de la salle (x, z).
    pub fn inward(self) -> (f32, f32) {
        match self {
            Side::West => (1.0, 0.0),
            Side::East => (-1.0, 0.0),
            Side::North => (0.0, 1.0),
            Side::South => (0.0, -1.0),
        }
    }

    /// L'angle (autour de y) qui tourne +z vers l'intérieur de la salle.
    pub fn facing(self) -> f32 {
        use std::f32::consts::{FRAC_PI_2, PI};
        match self {
            Side::West => FRAC_PI_2,
            Side::East => -FRAC_PI_2,
            Side::North => 0.0,
            Side::South => PI,
        }
    }

    pub const ALL: [Side; 4] = [Side::West, Side::North, Side::East, Side::South];
}

/// Une porte, sur un mur, à `position` mètres du milieu du mur (vers la droite de qui
/// le regarde depuis la salle).
#[derive(Clone, Copy, Debug)]
pub struct Door {
    pub side: Side,
    pub position: f32,
    pub width: f32,
}

/// Une salle du musée, placée sur le plan.
#[derive(Clone, Copy, Debug)]
pub struct Room {
    pub name: &'static str,
    pub theme: &'static str,
    /// La couleur des murs.
    pub walls: [u8; 3],
    /// Le centre de la salle.
    pub x: f32,
    pub z: f32,
    /// Sa largeur (est-ouest), sa profondeur (nord-sud) et sa hauteur.
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    /// Un couloir (des murs sombres, un liseré doré, pas d'arrêt de la visite).
    pub corridor: bool,
    /// Son numéro dans le fichier `.toml` (`salle = …`) ; 0 pour un couloir ajouté par
    /// l'enfilade.
    pub number: usize,
    pub doors: &'static [Door],
}

/// L'épaisseur des murs, posés à l'intérieur de chaque salle contre sa limite.
pub const WALL: f32 = 0.2;
/// La hauteur des portes.
pub const DOOR_HEIGHT: f32 = 3.0;

impl Room {
    /// La longueur d'un mur.
    pub fn wall_length(&self, side: Side) -> f32 {
        match side {
            Side::North | Side::South => self.width,
            Side::East | Side::West => self.depth,
        }
    }

    /// Le point (x, z) sur la limite de la salle, au mur `side`, à `position` mètres de
    /// son milieu.
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

    /// Le nombre de places numérotées d'un mur (une tous les 3 m environ).
    pub fn places(&self, side: Side) -> u8 {
        ((self.wall_length(side) / 3.3).floor() as u8).max(1)
    }

    /// Le point (x, z) est-il dans la salle, à plus de `margin` de ses limites ?
    pub fn contains(&self, x: f32, z: f32, margin: f32) -> bool {
        (x - self.x).abs() <= self.width / 2.0 - margin
            && (z - self.z).abs() <= self.depth / 2.0 - margin
    }

    /// La hauteur des portes de la salle.
    pub fn door_height(&self) -> f32 {
        DOOR_HEIGHT.min(self.height - 0.3)
    }
}

/// L'emplacement d'un tableau : une salle (son index dans `rooms`), un mur, la position
/// du centre du tableau le long du mur et sa hauteur ; `place` est le numéro de la place
/// (0 quand une position en mètres a été donnée).
#[derive(Clone, Copy, Debug)]
pub struct Hook {
    pub room: usize,
    pub side: Side,
    pub position: f32,
    pub center: f32,
    pub place: u8,
}

/// La mosaïque d'un tableau : sa grille, sa palette, et où sont ses cases dans le
/// fichier de données (une couleur par case, ligne par ligne depuis le haut).
#[derive(Clone, Copy, Debug)]
pub struct Mosaic {
    pub cols: u16,
    pub rows: u16,
    pub palette: &'static [[u8; 3]],
    pub offset: u32,
}

impl Mosaic {
    /// Le nombre de cases.
    pub fn len(&self) -> usize {
        usize::from(self.cols) * usize::from(self.rows)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Un tableau exposé.
#[derive(Clone, Copy, Debug)]
pub struct Artwork {
    pub title: &'static str,
    /// Le nom de sa page : celui de son image, sans extension.
    pub slug: &'static str,
    /// Le peintre, quand ce n'est pas celui de l'exposition : voir [`Artwork::artist_in`].
    pub artist: Option<&'static str>,
    pub year: &'static str,
    /// Le chemin de l'image dans `public/tableaux/` (`van-gogh/iris.jpg`, servi à
    /// `/tableaux/van-gogh/iris.jpg`).
    pub image: &'static str,
    pub technique: &'static str,
    pub collection: &'static str,
    pub note: &'static str,
    pub notice: Notice,
    /// Largeur et hauteur dans la galerie, en mètres.
    pub width: f32,
    pub height: f32,
    pub hook: Hook,
    pub mosaic: Mosaic,
}

impl Artwork {
    /// Le peintre de l'œuvre : le sien s'il est donné, sinon celui de l'exposition.
    pub fn artist_in(&self, ex: &Exhibition) -> &'static str {
        self.artist.unwrap_or(ex.artist)
    }

    /// L'adresse de l'image, pour les pages.
    pub fn image_url(&self) -> String {
        format!("/tableaux/{}", self.image)
    }
}

/// Où est le maillage d'une sculpture dans le fichier de données, et sa boîte englobante
/// (pour 1 m de haut) : x, y, z minimum puis maximum.
#[derive(Clone, Copy, Debug)]
pub struct MeshRef {
    pub offset: u32,
    pub size: u32,
    pub bounds: [f32; 6],
}

/// Une sculpture, posée sur un socle dans une salle.
#[derive(Clone, Copy, Debug)]
pub struct Sculpture {
    pub title: &'static str,
    /// Le nom de sa page : celui de son modèle, sans extension.
    pub slug: &'static str,
    pub artist: &'static str,
    pub year: &'static str,
    pub technique: &'static str,
    pub collection: &'static str,
    pub note: &'static str,
    pub notice: Notice,
    /// Le nom de sa vignette (vue de face), servie à `/donnees/…`.
    pub image: &'static str,
    /// La salle (son index dans `rooms`) et la position du centre du socle.
    pub room: usize,
    pub x: f32,
    pub z: f32,
    /// La hauteur de la sculpture (sans le socle), en mètres.
    pub height: f32,
    /// L'angle dont elle est tournée autour de la verticale (0 : elle regarde le sud).
    pub rotation: f32,
    /// La hauteur du socle.
    pub plinth: f32,
    /// Le rayon de son emprise au sol (socle compris).
    pub radius: f32,
    pub color: [u8; 3],
    pub roughness: f32,
    pub metallic: f32,
    pub mesh: MeshRef,
}

/// Les dimensions par défaut d'une galerie en enfilade, en mètres.
pub const ROOM_HALF_WIDTH: f32 = 5.0;
pub const ROOM_HALF_DEPTH: f32 = 5.0;
pub const CORRIDOR_HALF_WIDTH: f32 = 1.5;
pub const CORRIDOR_LENGTH: f32 = 6.0;

include!(concat!(env!("OUT_DIR"), "/expositions.rs"));

/// Le dossier des fichiers de données écrits par `build.rs` (servi à `/donnees/`).
pub const DATA_DIR: &str = concat!(env!("OUT_DIR"), "/donnees");

impl Sculpture {
    /// L'adresse de sa vignette, pour les pages.
    pub fn image_url(&self) -> String {
        format!("/donnees/{}", self.image)
    }
}

/// `rgb(…)` pour le CSS.
pub fn css_rgb([r, g, b]: [u8; 3]) -> String {
    format!("rgb({r},{g},{b})")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chaque œuvre a sa notice complète, chaque artiste sa biographie, et chaque œuvre
    /// sa page (des noms uniques).
    #[test]
    fn every_work_has_its_notice_and_its_artist() {
        for ex in EXHIBITIONS {
            let mut slugs = Vec::new();
            for work in ex.works() {
                let n = work.notice();
                let who = format!("{} · {}", ex.slug, work.title());
                for (part, text) in [
                    ("description", n.description),
                    ("evocation", n.evocation),
                    ("histoire", n.story),
                    ("facture", n.manner),
                    ("anecdote", n.anecdote),
                ] {
                    assert!(text.split_whitespace().count() >= 15, "{who} : {part}");
                    assert!(!text.contains('\\') && !text.contains("  "), "{who} : {part}");
                }
                assert!((3..=6).contains(&n.details.len()), "{who} : details");
                assert!((2..=8).contains(&n.themes.len()), "{who} : themes");
                let artist = ex
                    .artist_named(work.artist(ex))
                    .unwrap_or_else(|| panic!("{who} : pas de bloc [[artistes]]"));
                assert!(
                    !artist.life.is_empty() && artist.biography.split_whitespace().count() > 80,
                    "{who} : {}",
                    artist.name
                );
                assert!(!slugs.contains(&work.slug()), "{who}");
                slugs.push(work.slug());
                assert_eq!(ex.work(work.slug()).map(|w| w.title()), Some(work.title()));
            }
        }
    }

    #[test]
    fn every_exhibition_is_complete() {
        assert!(!EXHIBITIONS.is_empty());
        for ex in EXHIBITIONS {
            assert!(!ex.rooms.is_empty(), "{}", ex.slug);
            assert_eq!(Exhibition::find(ex.slug).map(|e| e.slug), Some(ex.slug));
            let data = std::fs::read(format!("{DATA_DIR}/{}.bin", ex.slug))
                .expect("le fichier de données");
            assert_eq!(data.len(), ex.data_size as usize, "{}", ex.slug);
            for art in ex.artworks {
                assert!(art.hook.room < ex.rooms.len(), "{}", art.title);
                let m = &art.mosaic;
                let cells = &data[m.offset as usize..m.offset as usize + m.len()];
                assert!(
                    cells.iter().all(|&c| usize::from(c) < m.palette.len()),
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
            for s in ex.sculptures {
                assert!(s.room < ex.rooms.len(), "{}", s.title);
                assert!(
                    (s.mesh.offset + s.mesh.size) as usize <= data.len(),
                    "{}",
                    s.title
                );
            }
            if let Some(i) = ex.showcase {
                assert!(i < ex.artworks.len());
            }
            let hung: usize = (0..ex.rooms.len()).map(|r| ex.works_in_room(r).len()).sum();
            assert_eq!(hung, ex.artworks.len());
        }
    }
}
