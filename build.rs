//! Lit les expositions (`expositions/<nom>.toml`) et leurs images
//! (`public/tableaux/<nom>/`), vérifie tout, puis écrit le catalogue en Rust
//! (`$OUT_DIR/expositions.rs`, inclus par `src/artworks.rs`).
//!
//! Chaque image devient une mosaïque : réduite à une grille (une case = 7 mm dans la
//! galerie), puis ramenée à une palette de 64 couleurs. Les cases sont écrites dans un
//! fichier binaire à côté (`include_bytes!`). La scène 3D en fait des touches en relief.
//!
//! Une erreur dans un fichier arrête la compilation avec un message qui dit quoi
//! corriger : le site publié n'est jamais cassé.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Doit correspondre aux dimensions de `src/artworks.rs`.
const PLACE_SPACING: f32 = 3.0;
const MIN_WIDTH: f32 = 0.6;
const MAX_WIDTH: f32 = 2.6;
const MAX_HEIGHT: f32 = 2.6;
/// La largeur d'une case de la mosaïque, en mètres.
const CELL: f32 = 0.007;
const MAX_CELLS: u32 = 340;
/// Le nombre de couleurs de la palette de chaque tableau.
const COLORS: usize = 64;

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
    #[serde(default = "last")]
    ordre: i64,
    nom_court: String,
    #[serde(default = "default_color")]
    couleur: String,
    titre: String,
    artiste: String,
    dates: String,
    accroche: String,
    introduction: String,
    vitrine: Option<String>,
}

fn last() -> i64 {
    i64::MAX
}

fn default_color() -> String {
    "#1d3f8f".into()
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

fn fail(file: &str, message: String) -> ! {
    eprintln!("\n\n  ✖ {file} : {message}\n\n");
    panic!("{file} : {message}");
}

fn hex(file: &str, color: &str, context: &str) -> [u8; 3] {
    let h = color.trim().trim_start_matches('#');
    let ok = h.len() == 6 && h.chars().all(|c| c.is_ascii_hexdigit());
    if !ok {
        fail(
            file,
            format!(
                "{context} : la couleur « {color} » doit s'écrire #rrggbb, par exemple #e7e1d3"
            ),
        );
    }
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(0);
    [byte(0), byte(2), byte(4)]
}

fn main() {
    println!("cargo::rerun-if-changed=expositions");
    println!("cargo::rerun-if-changed=public/tableaux");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));

    let mut files: Vec<PathBuf> = std::fs::read_dir("expositions")
        .unwrap_or_else(|e| fail("expositions/", format!("dossier illisible ({e})")))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    files.sort();
    let mut parsed = Vec::new();
    for path in &files {
        let name = path.display().to_string();
        println!("cargo::rerun-if-changed={name}");
        let slug = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let ok = !slug.is_empty()
            && slug
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !["ajouter", "pkg", "tableaux", "404"].contains(&slug.as_str());
        if !ok {
            fail(
                &name,
                "le nom du fichier donne l'adresse de l'exposition : minuscules sans accents, chiffres et tirets \
                 seulement (par exemple monet.toml), et pas ajouter, pkg, tableaux ou 404"
                    .into(),
            );
        }
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|e| fail(&name, format!("fichier illisible ({e})")));
        let file: File = toml::from_str(&text)
            .unwrap_or_else(|e| fail(&name, format!("le fichier est mal écrit :\n{e}")));
        parsed.push((name, slug, file));
    }
    if parsed.is_empty() {
        fail(
            "expositions/",
            "il faut au moins une exposition (un fichier .toml)".into(),
        );
    }
    parsed.sort_by(|a, b| {
        a.2.exposition
            .ordre
            .cmp(&b.2.exposition.ordre)
            .then(a.1.cmp(&b.1))
    });

    let mut out = String::from(
        "// Écrit par build.rs à partir de expositions/*.toml : ne pas modifier.\n\n\
         /// Les expositions, dans l'ordre de l'accueil.\npub static EXHIBITIONS: &[Exhibition] = &[\n",
    );
    for (name, slug, file) in &parsed {
        exhibition(&mut out, &out_dir, name, slug, file);
    }
    out.push_str("];\n");
    std::fs::write(out_dir.join("expositions.rs"), out).expect("écriture de expositions.rs");
}

/// Vérifie une exposition et écrit son code (et les mosaïques de ses œuvres).
fn exhibition(out: &mut String, out_dir: &Path, name: &str, slug: &str, file: &File) {
    if file.salles.is_empty() {
        fail(name, "il faut au moins une salle ([[salles]])".into());
    }
    let rooms = file.salles.len();
    let mut used = HashSet::new();
    let mut artworks = String::new();
    let mut showcase = None;
    for (index, art) in file.oeuvres.iter().enumerate() {
        let who = format!("l'œuvre « {} »", art.titre);
        if art.salle == 0 || art.salle > rooms {
            fail(
                name,
                format!(
                    "{who} : salle = {}, mais les salles vont de 1 à {rooms}",
                    art.salle
                ),
            );
        }
        let wall = match art.mur.trim() {
            "gauche" => "Left",
            "droite" => "Right",
            "fond" => "Back",
            other => fail(
                name,
                format!("{who} : mur = \"{other}\", il faut \"gauche\", \"droite\" ou \"fond\""),
            ),
        };
        if !(1..=3).contains(&art.place) {
            fail(
                name,
                format!("{who} : place = {}, il faut 1, 2 ou 3", art.place),
            );
        }
        if wall == "Back" && art.place == 2 && art.salle < rooms {
            fail(
                name,
                format!(
                    "{who} : le milieu du mur du fond de la salle {} est une porte ; choisir la place 1 ou 3",
                    art.salle
                ),
            );
        }
        if !used.insert((art.salle, wall, art.place)) {
            fail(
                name,
                format!(
                    "{who} : la salle {}, mur {}, place {} est déjà occupée par une autre œuvre",
                    art.salle, art.mur, art.place
                ),
            );
        }
        if !(MIN_WIDTH..=MAX_WIDTH).contains(&art.largeur) {
            fail(
                name,
                format!(
                    "{who} : largeur = {}, il faut entre {MIN_WIDTH} et {MAX_WIDTH} mètres",
                    art.largeur
                ),
            );
        }

        let folder = format!("public/tableaux/{slug}");
        let path = Path::new(&folder).join(&art.image);
        if art.image.contains('/') || art.image.contains('\\') || !path.is_file() {
            fail(
                name,
                format!(
                    "{who} : l'image « {} » est introuvable dans {folder}/",
                    art.image
                ),
            );
        }
        let image = image::open(&path)
            .unwrap_or_else(|e| {
                fail(
                    name,
                    format!(
                        "{who} : l'image « {} » ne peut pas être lue ({e})",
                        art.image
                    ),
                )
            })
            .to_rgb8();
        let ratio = image.height() as f32 / image.width().max(1) as f32;
        let height = art.largeur * ratio;
        if height > MAX_HEIGHT {
            fail(
                name,
                format!(
                    "{who} : avec largeur = {}, le tableau ferait {height:.2} m de haut (au plus {MAX_HEIGHT} m) ; réduire la largeur",
                    art.largeur
                ),
            );
        }
        if file.exposition.vitrine.as_deref() == Some(art.image.as_str()) && showcase.is_none() {
            showcase = Some(index);
        }

        let (cols, rows, palette, cells) = mosaic(&image, art.largeur, height);
        let bin = format!("mosaic-{slug}-{index}.bin");
        std::fs::write(out_dir.join(&bin), &cells).expect("écriture d'une mosaïque");
        let mut palette_code = String::new();
        for [r, g, b] in &palette {
            let _ = write!(palette_code, "[{r}, {g}, {b}], ");
        }
        let _ = writeln!(
            artworks,
            "        Artwork {{\n            title: {:?},\n            year: {:?},\n            image: {:?},\n            technique: {:?},\n            collection: {:?},\n            note: {:?},\n            width: {:?},\n            height: {:?},\n            hook: Hook {{ room: {}, wall: Wall::{wall}, place: {} }},\n            mosaic: Mosaic {{ cols: {cols}, rows: {rows}, palette: &[{palette_code}], cells: include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{bin}\")) }},\n        }},",
            art.titre.trim(),
            art.annee.trim(),
            format!("{slug}/{}", art.image),
            art.technique.trim(),
            art.collection.trim(),
            art.note.trim(),
            art.largeur,
            (height * 1000.0).round() / 1000.0,
            art.salle - 1,
            art.place,
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
                fail(
                    name,
                    format!(
                        "« {} » et « {} » se chevauchent sur le mur {} de la salle {} : réduire leurs largeurs ou les écarter (places 1 et 3)",
                        a.titre, b.titre, a.mur, a.salle
                    ),
                );
            }
        }
    }
    if let Some(vitrine) = &file.exposition.vitrine
        && showcase.is_none()
    {
        fail(
            name,
            format!("vitrine = \"{vitrine}\" : aucune œuvre n'utilise cette image"),
        );
    }

    let e = &file.exposition;
    let [ar, ag, ab] = hex(name, &e.couleur, "couleur");
    let _ = writeln!(
        out,
        "    Exhibition {{\n        slug: {slug:?},\n        short_name: {:?},\n        title: {:?},\n        artist: {:?},\n        artist_dates: {:?},\n        kicker: {:?},\n        introduction: {:?},\n        accent: [{ar}, {ag}, {ab}],\n        showcase: {:?},\n        rooms: &[",
        e.nom_court.trim(),
        e.titre.trim(),
        e.artiste.trim(),
        e.dates.trim(),
        e.accroche.trim(),
        e.introduction.trim(),
        showcase,
    );
    for (i, salle) in file.salles.iter().enumerate() {
        let [r, g, b] = hex(name, &salle.murs, &format!("la salle « {} »", salle.nom));
        let _ = writeln!(
            out,
            "            Room {{ name: {:?}, theme: {:?}, walls: [{r}, {g}, {b}], center_z: {:?} }},",
            salle.nom.trim(),
            salle.theme.trim(),
            -(i as f32) * 16.0
        );
    }
    out.push_str("        ],\n        artworks: &[\n");
    out.push_str(&artworks);
    out.push_str("        ],\n    },\n");
}

/// La grille de l'image et sa palette : (colonnes, lignes, palette, une couleur par case,
/// ligne par ligne depuis le haut).
fn mosaic(image: &image::RgbImage, width: f32, height: f32) -> (u32, u32, Vec<[u8; 3]>, Vec<u8>) {
    let cols = ((width / CELL).round() as u32).clamp(16, MAX_CELLS);
    let rows = ((height / width * cols as f32).round() as u32).clamp(8, MAX_CELLS);
    let small = image::imageops::resize(image, cols, rows, image::imageops::FilterType::Lanczos3);
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
    for _ in 0..12 {
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
