//! Lit `exposition.toml` et les images de `public/tableaux/`, vérifie tout, puis écrit
//! le catalogue en Rust (`$OUT_DIR/exposition.rs`, inclus par `src/artworks.rs`).
//!
//! Chaque image devient une mosaïque : réduite à une grille (une case ≈ 2,8 cm dans la
//! galerie), puis ramenée à une palette de quelques couleurs. La scène 3D construit
//! une touche en relief par case, et un seul maillage par couleur.
//!
//! Une erreur dans le fichier arrête la compilation avec un message qui dit quoi
//! corriger : le site publié n'est jamais cassé.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

/// Doit correspondre aux dimensions de `src/artworks.rs`.
const PLACE_SPACING: f32 = 3.0;
const MIN_WIDTH: f32 = 0.6;
const MAX_WIDTH: f32 = 2.6;
const MAX_HEIGHT: f32 = 2.6;
/// La largeur d'une case de la mosaïque, en mètres.
const CELL: f32 = 0.028;
const MAX_COLS: u32 = 96;
const MAX_ROWS: u32 = 96;
/// Le nombre de couleurs de la palette de chaque tableau.
const COLORS: usize = 20;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    exposition: Exposition,
    #[serde(default)]
    salles: Vec<Salle>,
    #[serde(default)]
    oeuvres: Vec<Oeuvre>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Exposition {
    titre: String,
    artiste: String,
    dates: String,
    accroche: String,
    introduction: String,
    vitrine: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Salle {
    nom: String,
    theme: String,
    murs: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Oeuvre {
    titre: String,
    annee: String,
    image: String,
    #[serde(default)]
    technique: String,
    #[serde(default)]
    collection: String,
    #[serde(default)]
    note: String,
    salle: usize,
    mur: String,
    place: u8,
    largeur: f32,
}

fn fail(message: String) -> ! {
    eprintln!("\n\n  ✖ exposition.toml : {message}\n\n");
    panic!("exposition.toml : {message}");
}

fn hex(color: &str, context: &str) -> [u8; 3] {
    let h = color.trim().trim_start_matches('#');
    let ok = h.len() == 6 && h.chars().all(|c| c.is_ascii_hexdigit());
    if !ok {
        fail(format!(
            "{context} : la couleur « {color} » doit s'écrire #rrggbb, par exemple #e7e1d3"
        ));
    }
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(0);
    [byte(0), byte(2), byte(4)]
}

fn main() {
    println!("cargo::rerun-if-changed=exposition.toml");
    println!("cargo::rerun-if-changed=public/tableaux");

    let text = std::fs::read_to_string("exposition.toml")
        .unwrap_or_else(|e| fail(format!("fichier illisible ({e})")));
    let file: File =
        toml::from_str(&text).unwrap_or_else(|e| fail(format!("le fichier est mal écrit :\n{e}")));
    if file.salles.is_empty() {
        fail("il faut au moins une salle ([[salles]])".into());
    }

    let rooms = file.salles.len();
    let mut used = HashSet::new();
    let mut out =
        String::from("// Écrit par build.rs à partir de exposition.toml : ne pas modifier.\n\n");

    let mut artworks = String::new();
    let mut showcase = None;
    for (index, art) in file.oeuvres.iter().enumerate() {
        let who = format!("l'œuvre « {} »", art.titre);
        if art.salle == 0 || art.salle > rooms {
            fail(format!(
                "{who} : salle = {}, mais les salles vont de 1 à {rooms}",
                art.salle
            ));
        }
        let wall = match art.mur.trim() {
            "gauche" => "Left",
            "droite" => "Right",
            "fond" => "Back",
            other => fail(format!(
                "{who} : mur = \"{other}\", il faut \"gauche\", \"droite\" ou \"fond\""
            )),
        };
        if !(1..=3).contains(&art.place) {
            fail(format!("{who} : place = {}, il faut 1, 2 ou 3", art.place));
        }
        if wall == "Back" && art.place == 2 && art.salle < rooms {
            fail(format!(
                "{who} : le milieu du mur du fond de la salle {} est une porte ; choisir la place 1 ou 3",
                art.salle
            ));
        }
        if !used.insert((art.salle, wall, art.place)) {
            fail(format!(
                "{who} : la salle {}, mur {}, place {} est déjà occupée par une autre œuvre",
                art.salle, art.mur, art.place
            ));
        }
        if !(MIN_WIDTH..=MAX_WIDTH).contains(&art.largeur) {
            fail(format!(
                "{who} : largeur = {}, il faut entre {MIN_WIDTH} et {MAX_WIDTH} mètres",
                art.largeur
            ));
        }

        let path = Path::new("public/tableaux").join(&art.image);
        if art.image.contains('/') || art.image.contains('\\') || !path.is_file() {
            fail(format!(
                "{who} : l'image « {} » est introuvable dans public/tableaux/",
                art.image
            ));
        }
        let image = image::open(&path)
            .unwrap_or_else(|e| {
                fail(format!(
                    "{who} : l'image « {} » ne peut pas être lue ({e})",
                    art.image
                ))
            })
            .to_rgb8();
        let ratio = image.height() as f32 / image.width().max(1) as f32;
        let height = art.largeur * ratio;
        if height > MAX_HEIGHT {
            fail(format!(
                "{who} : avec largeur = {}, le tableau ferait {height:.2} m de haut (au plus {MAX_HEIGHT} m) ; réduire la largeur",
                art.largeur
            ));
        }
        if file.exposition.vitrine.as_deref() == Some(art.image.as_str()) && showcase.is_none() {
            showcase = Some(index);
        }

        let (cols, rows, palette, cells) = mosaic(&image, art.largeur, height);
        let mut palette_code = String::new();
        for [r, g, b] in &palette {
            let _ = write!(palette_code, "[{r}, {g}, {b}], ");
        }
        let cells_code: Vec<String> = cells.iter().map(u8::to_string).collect();
        let _ = writeln!(
            artworks,
            "    Artwork {{\n        title: {:?},\n        year: {:?},\n        image: {:?},\n        technique: {:?},\n        collection: {:?},\n        note: {:?},\n        width: {:?},\n        height: {:?},\n        hook: Hook {{ room: {}, wall: Wall::{wall}, place: {} }},\n        mosaic: Mosaic {{ cols: {cols}, rows: {rows}, palette: &[{palette_code}], cells: &[{}] }},\n    }},",
            art.titre.trim(),
            art.annee.trim(),
            art.image,
            art.technique.trim(),
            art.collection.trim(),
            art.note.trim(),
            art.largeur,
            (height * 1000.0).round() / 1000.0,
            art.salle - 1,
            art.place,
            cells_code.join(",")
        );
    }

    // Deux œuvres voisines sur le même mur ne doivent pas se chevaucher (avec leur cartel).
    for a in &file.oeuvres {
        for b in &file.oeuvres {
            if std::ptr::eq(a, b)
                || a.salle != b.salle
                || a.mur.trim() != b.mur.trim()
                || a.place >= b.place
            {
                continue;
            }
            let gap = PLACE_SPACING * f32::from(b.place - a.place);
            if gap < (a.largeur + b.largeur) / 2.0 + 0.7 {
                fail(format!(
                    "« {} » et « {} » se chevauchent sur le mur {} de la salle {} : réduire leurs largeurs ou les écarter (places 1 et 3)",
                    a.titre, b.titre, a.mur, a.salle
                ));
            }
        }
    }
    if let Some(vitrine) = &file.exposition.vitrine
        && showcase.is_none()
    {
        fail(format!(
            "vitrine = \"{vitrine}\" : aucune œuvre n'utilise cette image"
        ));
    }

    let e = &file.exposition;
    let _ = writeln!(
        out,
        "/// L'exposition.\npub static EXHIBITION: Exhibition = Exhibition {{\n    title: {:?},\n    artist: {:?},\n    artist_dates: {:?},\n    kicker: {:?},\n    introduction: {:?},\n    showcase: {:?},\n}};\n",
        e.titre.trim(),
        e.artiste.trim(),
        e.dates.trim(),
        e.accroche.trim(),
        e.introduction.trim(),
        showcase,
    );
    out.push_str("/// Les salles, dans l'ordre de la visite.\npub static ROOMS: &[Room] = &[\n");
    for (i, salle) in file.salles.iter().enumerate() {
        let [r, g, b] = hex(&salle.murs, &format!("la salle « {} »", salle.nom));
        let _ = writeln!(
            out,
            "    Room {{ name: {:?}, theme: {:?}, walls: [{r}, {g}, {b}], center_z: {:?} }},",
            salle.nom.trim(),
            salle.theme.trim(),
            -(i as f32) * 16.0
        );
    }
    out.push_str(
        "];\n\n/// Les œuvres, dans l'ordre du fichier.\npub static ARTWORKS: &[Artwork] = &[\n",
    );
    out.push_str(&artworks);
    out.push_str("];\n");

    let dest = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("exposition.rs");
    std::fs::write(dest, out).expect("écriture de exposition.rs");
}

/// La grille de l'image et sa palette : (colonnes, lignes, palette, une couleur par case,
/// ligne par ligne depuis le haut).
fn mosaic(image: &image::RgbImage, width: f32, height: f32) -> (u32, u32, Vec<[u8; 3]>, Vec<u8>) {
    let cols = ((width / CELL).round() as u32).clamp(12, MAX_COLS);
    let rows = ((height / width * cols as f32).round() as u32).clamp(8, MAX_ROWS);
    let small = image::imageops::resize(image, cols, rows, image::imageops::FilterType::Triangle);
    let pixels: Vec<[f32; 3]> = small
        .pixels()
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();
    let palette = kmeans(&pixels, COLORS);
    let cells = pixels.iter().map(|p| nearest(&palette, p) as u8).collect();
    // Sous l'éclairage du musée, les toiles sombres deviendraient illisibles : leurs
    // couleurs sont éclaircies (une courbe qui relève les ombres sans brûler les clairs).
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
    let palette = palette
        .iter()
        .map(|c| c.map(|v| (255.0 * (v / 255.0).clamp(0.0, 1.0).powf(gamma)).round() as u8))
        .collect();
    (cols, rows, palette, cells)
}

fn distance(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    // Le vert compte plus pour l'œil que le rouge, et le rouge plus que le bleu.
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    2.0 * d[0] * d[0] + 4.0 * d[1] * d[1] + 3.0 * d[2] * d[2]
}

fn nearest(palette: &[[f32; 3]], p: &[f32; 3]) -> usize {
    let mut best = (0, f32::MAX);
    for (i, c) in palette.iter().enumerate() {
        let d = distance(c, p);
        if d < best.1 {
            best = (i, d);
        }
    }
    best.0
}

/// Une palette de `k` couleurs par k-moyennes, déterministe (départ : les pixels les plus
/// éloignés les uns des autres).
fn kmeans(pixels: &[[f32; 3]], k: usize) -> Vec<[f32; 3]> {
    let mut centers: Vec<[f32; 3]> = vec![pixels[0]];
    let mut far: Vec<f32> = pixels.iter().map(|p| distance(p, &pixels[0])).collect();
    while centers.len() < k.min(pixels.len()) {
        let (i, _) = far
            .iter()
            .enumerate()
            .fold((0, -1.0), |b, (i, &d)| if d > b.1 { (i, d) } else { b });
        let c = pixels[i];
        centers.push(c);
        for (d, p) in far.iter_mut().zip(pixels) {
            *d = d.min(distance(p, &c));
        }
    }
    for _ in 0..16 {
        let mut sums = vec![[0.0f32; 4]; centers.len()];
        for p in pixels {
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
    centers
}
