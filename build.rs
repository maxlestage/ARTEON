//! Lit les expositions (`expositions/<nom>.toml`), les images de leurs tableaux
//! (`public/tableaux/<nom>/`) et les modèles 3D de leurs sculptures (`modeles/`),
//! vérifie tout, puis écrit :
//!
//! - le catalogue en Rust (`$OUT_DIR/expositions.rs`, inclus par `src/artworks.rs`) :
//!   les textes, le plan du musée, la place de chaque œuvre ;
//! - un fichier de données par exposition (`$OUT_DIR/donnees/<nom>.bin`) : les
//!   mosaïques des tableaux et les maillages des sculptures. Le navigateur ne le
//!   télécharge que pour l'exposition visitée.
//!
//! Une erreur dans un fichier arrête la compilation avec un message qui dit quoi
//! corriger : le site publié n'est jamais cassé.

#[path = "build/mesh.rs"]
mod mesh;
#[path = "build/mosaic.rs"]
mod mosaic;
#[path = "build/plan.rs"]
mod plan;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use plan::{DOOR_WIDTH, Door, Room, Side, WALL};
use serde::Deserialize;

const MIN_WIDTH: f32 = 0.3;
/// Le nombre de triangles d'une sculpture dans le navigateur, au plus.
const SCULPTURE_TRIANGLES: usize = 20_000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    exposition: Exposition,
    #[serde(default)]
    salles: Vec<Salle>,
    #[serde(default)]
    oeuvres: Vec<Oeuvre>,
    #[serde(default)]
    sculptures: Vec<SculptureToml>,
    #[serde(default)]
    artistes: Vec<Artiste>,
    musee: Option<Musee>,
}

/// Le bâtiment de l'exposition : le vrai musée dont il s'inspire, et ce qu'il en garde.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Musee {
    nom: String,
    #[serde(default)]
    ville: String,
    #[serde(default)]
    architecte: String,
    #[serde(default)]
    construction: String,
    #[serde(default)]
    presentation: String,
    #[serde(default)]
    ici: String,
}

/// Les styles de cadre, et leur nom dans le code.
const FRAMES: [(&str, &str); 9] = [
    ("vitrine", "Showcase"),
    ("dore", "Gilded"),
    ("baroque", "Baroque"),
    ("ebene", "Ebony"),
    ("bois", "Wood"),
    ("blanc", "White"),
    ("noir", "Black"),
    ("estampe", "Print"),
    ("aucun", "Bare"),
];

fn frame_code(file: &str, value: &str, who: &str) -> &'static str {
    FRAMES
        .iter()
        .find(|(name, _)| *name == value)
        .map(|(_, code)| *code)
        .unwrap_or_else(|| {
            fail(
                file,
                format!(
                    "{who} : cadre = \"{value}\" : dore, baroque, ebene, bois, blanc, noir, estampe, vitrine ou aucun"
                ),
            )
        })
}

/// Un artiste de l'exposition : sa vie et sa manière.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Artiste {
    nom: String,
    #[serde(default)]
    vie: String,
    #[serde(default)]
    biographie: String,
    #[serde(default)]
    style: String,
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
    /// Le cadre des œuvres, par défaut : dore, baroque, ebene, bois, blanc, noir,
    /// estampe ou aucun.
    #[serde(default = "default_frame")]
    cadre: String,
    /// Le style ou le mouvement (« Renaissance italienne »), pour l'accueil.
    #[serde(default)]
    mouvement: String,
    /// Une exposition éphémère, refaite chaque semaine par outils/ephemeres.py : ses
    /// œuvres n'ont pas de notice complète ni de biographie d'artiste.
    #[serde(default)]
    ephemere: bool,
}

fn default_frame() -> String {
    "dore".into()
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
    #[serde(default)]
    theme: String,
    murs: Option<String>,
    x: Option<f32>,
    z: Option<f32>,
    largeur: Option<f32>,
    profondeur: Option<f32>,
    hauteur: Option<f32>,
    #[serde(default)]
    couloir: bool,
    portes: Option<Vec<PorteToml>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PorteToml {
    Mur(String),
    Detail {
        mur: String,
        position: Option<f32>,
        largeur: Option<f32>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Oeuvre {
    titre: String,
    /// Le peintre, quand ce n'est pas celui de l'exposition (une exposition à plusieurs).
    artiste: Option<String>,
    annee: String,
    image: String,
    #[serde(default)]
    technique: String,
    #[serde(default)]
    collection: String,
    #[serde(default)]
    note: String,
    /// La notice complète : ce que l'on voit, ce qu'elle évoque, son histoire, la
    /// manière, les détails à regarder, les thèmes et une anecdote.
    #[serde(default)]
    description: String,
    #[serde(default)]
    evocation: String,
    #[serde(default)]
    histoire: String,
    #[serde(default)]
    facture: String,
    #[serde(default)]
    details: Vec<String>,
    #[serde(default)]
    themes: Vec<String>,
    #[serde(default)]
    anecdote: String,
    salle: usize,
    mur: String,
    place: Option<u8>,
    position: Option<f32>,
    largeur: f32,
    /// Le cadre de cette œuvre, s'il diffère de celui de l'exposition.
    cadre: Option<String>,
    /// L'auteur de la reproduction (photographe), sa licence et sa source : obligatoires
    /// pour une image sous licence libre (CC BY, CC BY-SA…).
    #[serde(default)]
    credit: String,
    #[serde(default)]
    licence: String,
    #[serde(default)]
    source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SculptureToml {
    titre: String,
    artiste: String,
    annee: String,
    modele: String,
    #[serde(default)]
    technique: String,
    #[serde(default)]
    collection: String,
    #[serde(default)]
    note: String,
    /// La notice complète : ce que l'on voit, ce qu'elle évoque, son histoire, la
    /// manière, les détails à regarder, les thèmes et une anecdote.
    #[serde(default)]
    description: String,
    #[serde(default)]
    evocation: String,
    #[serde(default)]
    histoire: String,
    #[serde(default)]
    facture: String,
    #[serde(default)]
    details: Vec<String>,
    #[serde(default)]
    themes: Vec<String>,
    #[serde(default)]
    anecdote: String,
    salle: usize,
    #[serde(default)]
    x: f32,
    #[serde(default)]
    z: f32,
    hauteur: f32,
    #[serde(default)]
    rotation: f32,
    #[serde(default = "default_matiere")]
    matiere: String,
    socle: Option<f32>,
    /// L'axe du fichier qui pointe vers le haut : "y" (par défaut), "z" ou "-z".
    #[serde(default = "default_up")]
    haut: String,
}

/// Le code Rust d'une notice.
fn notice_code(
    description: &str,
    evocation: &str,
    histoire: &str,
    facture: &str,
    details: &[String],
    themes: &[String],
    anecdote: &str,
) -> String {
    let list = |items: &[String]| {
        items
            .iter()
            .map(|t| format!("{:?}", t.trim()))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "Notice {{ description: text!({:?}), evocation: text!({:?}), story: text!({:?}), manner: text!({:?}), details: texts!(&[{}]), themes: &[{}], anecdote: text!({:?}) }}",
        description.trim(),
        evocation.trim(),
        histoire.trim(),
        facture.trim(),
        list(details),
        list(themes),
        anecdote.trim(),
    )
}

/// Les textes longs d'une exposition, en JSON : `works` (par nom de page), `artists`
/// (par nom), `museum` et `source` (le fichier .toml, pour l'éditeur du musée).
fn texts_json(name: &str, file: &File) -> String {
    let notice = |d: &str, e: &str, h: &str, f: &str, details: &[String], a: &str| {
        serde_json::json!({
            "description": d.trim(), "evocation": e.trim(), "story": h.trim(), "manner": f.trim(),
            "details": details.iter().map(|t| t.trim()).collect::<Vec<_>>(), "anecdote": a.trim(),
        })
    };
    let mut works = serde_json::Map::new();
    for o in &file.oeuvres {
        works.insert(
            work_slug(&o.image),
            notice(
                &o.description,
                &o.evocation,
                &o.histoire,
                &o.facture,
                &o.details,
                &o.anecdote,
            ),
        );
    }
    for s in &file.sculptures {
        works.insert(
            work_slug(&s.modele),
            notice(
                &s.description,
                &s.evocation,
                &s.histoire,
                &s.facture,
                &s.details,
                &s.anecdote,
            ),
        );
    }
    let artists: serde_json::Map<String, serde_json::Value> = file
        .artistes
        .iter()
        .map(|a| {
            (
                a.nom.trim().to_owned(),
                serde_json::json!({ "life": a.vie.trim(), "biography": a.biographie.trim(), "style": a.style.trim() }),
            )
        })
        .collect();
    let museum = file.musee.as_ref().map(
        |m| serde_json::json!({ "presentation": m.presentation.trim(), "here": m.ici.trim() }),
    );
    let source = std::fs::read_to_string(name).unwrap_or_default();
    serde_json::json!({ "works": works, "artists": artists, "museum": museum, "source": source })
        .to_string()
}

/// L'adresse d'une œuvre dans son exposition : le nom de son fichier, sans extension.
fn work_slug(file: &str) -> String {
    Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file)
        .to_owned()
}

fn default_matiere() -> String {
    "marbre".into()
}

fn default_up() -> String {
    "y".into()
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
    println!("cargo::rerun-if-changed=modeles");
    println!("cargo::rerun-if-changed=build");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let data_dir = out_dir.join("donnees");
    let _ = std::fs::remove_dir_all(&data_dir);
    std::fs::create_dir_all(&data_dir).expect("dossier des données");

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
            && !["ajouter", "pkg", "tableaux", "donnees", "404"].contains(&slug.as_str());
        if !ok {
            fail(
                &name,
                "le nom du fichier donne l'adresse de l'exposition : minuscules sans accents, chiffres et tirets \
                 seulement (par exemple monet.toml), et pas ajouter, pkg, tableaux, donnees ou 404"
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
        let mut data = Vec::new();
        data.extend_from_slice(b"ARTN\x01\0\0\0");
        exhibition(&mut out, &mut data, &data_dir, name, slug, file);
        std::fs::write(data_dir.join(format!("{slug}.bin")), &data).expect("écriture des données");
        // Les textes longs, que le navigateur télécharge à la demande (ils ne sont pas
        // compilés dans le WebAssembly) : notices, artistes, bâtiment, fichier source.
        std::fs::write(
            data_dir.join(format!("{slug}.json")),
            texts_json(name, file),
        )
        .expect("écriture des textes");
    }
    out.push_str("];\n");
    std::fs::write(out_dir.join("expositions.rs"), out).expect("écriture de expositions.rs");
}

/// Le plan du musée : les salles du fichier, placées par leurs `x` et `z` ; ou, si aucune
/// n'est placée, une enfilade de salles de 10 × 10 m reliées par des couloirs de 6 m.
fn layout(name: &str, file: &File) -> (Vec<Room>, Vec<usize>) {
    let placed = file
        .salles
        .iter()
        .filter(|s| s.x.is_some() || s.z.is_some())
        .count();
    if placed != 0 && placed != file.salles.len() {
        fail(
            name,
            "soit toutes les salles ont un x et un z (un plan), soit aucune (une enfilade)".into(),
        );
    }
    let color = |s: &Salle, default: [u8; 3]| {
        s.murs.as_deref().map_or(default, |c| {
            hex(name, c, &format!("la salle « {} »", s.nom))
        })
    };
    let doors = |s: &Salle| -> Vec<Door> {
        s.portes
            .iter()
            .flatten()
            .map(|p| {
                let (mur, position, largeur) = match p {
                    PorteToml::Mur(m) => (m.as_str(), None, None),
                    PorteToml::Detail { mur, position, largeur } => (mur.as_str(), *position, *largeur),
                };
                let side = Side::parse(mur).unwrap_or_else(|| {
                    fail(name, format!("la salle « {} » : porte sur le mur « {mur} » : il faut nord, sud, est ou ouest", s.nom))
                });
                Door { side, position: position.unwrap_or(0.0), width: largeur.unwrap_or(DOOR_WIDTH) }
            })
            .collect()
    };
    let mut rooms = Vec::new();
    // L'index, dans le plan, de chaque salle du fichier (le plan ajoute des couloirs).
    let mut index = Vec::new();
    if placed == 0 {
        let count = file.salles.len();
        for (i, s) in file.salles.iter().enumerate() {
            let (w, d) = (s.largeur.unwrap_or(10.0), s.profondeur.unwrap_or(10.0));
            let z = -(i as f32) * 16.0;
            index.push(rooms.len());
            let mut list = doors(s);
            if s.portes.is_none() {
                list.push(Door {
                    side: Side::South,
                    position: 0.0,
                    width: DOOR_WIDTH,
                });
                if i + 1 < count {
                    list.push(Door {
                        side: Side::North,
                        position: 0.0,
                        width: DOOR_WIDTH,
                    });
                }
            }
            rooms.push(Room {
                name: s.nom.clone(),
                theme: s.theme.clone(),
                walls: color(s, [238, 235, 228]),
                x: 0.0,
                z,
                width: w,
                depth: d,
                height: s.hauteur.unwrap_or(4.0),
                corridor: s.couloir,
                doors: list,
                number: i + 1,
            });
            if i + 1 < count {
                rooms.push(Room {
                    name: format!("Couloir {}", i + 1),
                    theme: String::new(),
                    walls: [30, 44, 84],
                    x: 0.0,
                    z: z - 8.0,
                    width: 3.0,
                    depth: 6.0,
                    height: 3.0,
                    corridor: true,
                    doors: Vec::new(),
                    number: 0,
                });
            }
        }
    } else {
        for (n, s) in file.salles.iter().enumerate() {
            index.push(rooms.len());
            let corridor = s.couloir;
            rooms.push(Room {
                name: s.nom.clone(),
                theme: s.theme.clone(),
                walls: color(
                    s,
                    if corridor {
                        [30, 44, 84]
                    } else {
                        [238, 235, 228]
                    },
                ),
                x: s.x.unwrap_or(0.0),
                z: s.z.unwrap_or(0.0),
                width: s.largeur.unwrap_or(10.0),
                depth: s.profondeur.unwrap_or(10.0),
                height: s.hauteur.unwrap_or(if corridor { 3.2 } else { 4.5 }),
                corridor,
                doors: doors(s),
                number: n + 1,
            });
        }
    }
    for (i, r) in rooms.iter().enumerate() {
        if r.width < 2.5
            || r.depth < 2.5
            || r.height < 2.6
            || r.width > 120.0
            || r.depth > 120.0
            || r.height > 20.0
        {
            fail(
                name,
                format!(
                    "la salle « {} » : largeur et profondeur de 2,5 à 120 m, hauteur de 2,6 à 20 m",
                    r.name
                ),
            );
        }
        for other in &rooms[..i] {
            if plan::overlap(r, other) {
                fail(
                    name,
                    format!(
                        "les salles « {} » et « {} » se chevauchent sur le plan",
                        other.name, r.name
                    ),
                );
            }
        }
        for d in &r.doors {
            let half = r.wall_length(d.side) / 2.0;
            if d.width < 1.0
                || d.position - d.width / 2.0 < -half - 0.01
                || d.position + d.width / 2.0 > half + 0.01
            {
                fail(
                    name,
                    format!(
                        "la salle « {} » : une porte du mur {} dépasse du mur",
                        r.name,
                        d.side.french()
                    ),
                );
            }
        }
    }
    plan::mirror_doors(&mut rooms);
    (rooms, index)
}

/// Vérifie une exposition et écrit son code et ses données.
fn exhibition(
    out: &mut String,
    data: &mut Vec<u8>,
    data_dir: &Path,
    name: &str,
    slug: &str,
    file: &File,
) {
    if file.salles.is_empty() {
        fail(name, "il faut au moins une salle ([[salles]])".into());
    }
    let (rooms, index_of) = layout(name, file);
    let room_index = |salle: usize| salle.checked_sub(1).and_then(|i| index_of.get(i).copied());
    let mut artworks = String::new();
    let mut showcase = None;
    // Ce qui occupe chaque mur : (salle, mur, début, fin, titre).
    let mut spans: Vec<(usize, Side, f32, f32, String)> = Vec::new();
    for (index, art) in file.oeuvres.iter().enumerate() {
        let who = format!("l'œuvre « {} »", art.titre);
        let r = room_index(art.salle).unwrap_or_else(|| {
            fail(
                name,
                format!(
                    "{who} : salle = {}, mais les salles vont de 1 à {}",
                    art.salle,
                    file.salles.len()
                ),
            )
        });
        let room = &rooms[r];
        let side = Side::parse(&art.mur)
            .unwrap_or_else(|| fail(name, format!("{who} : mur = \"{}\", il faut nord, sud, est ou ouest (ou gauche, droite, fond)", art.mur)));
        let places = room.places(side);
        let (position, place) = match (art.position, art.place) {
            (Some(p), _) => (p, 0),
            (None, Some(p)) if (1..=places).contains(&p) => (room.place_position(side, p), p),
            (None, Some(p)) => fail(
                name,
                format!(
                    "{who} : place = {p}, mais le mur {} de cette salle a {places} place(s)",
                    side.french()
                ),
            ),
            (None, None) => fail(
                name,
                format!("{who} : il faut une place (1 à {places}) ou une position en mètres"),
            ),
        };

        let free = ["", "CC0", "Domaine public", "PD"];
        if !free.contains(&art.licence.trim())
            && (art.credit.trim().is_empty() || art.source.trim().is_empty())
        {
            fail(
                name,
                format!(
                    "{who} : licence = \"{}\" : il faut aussi credit (l'auteur de la photo) et source (l'adresse du fichier)",
                    art.licence
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
        let height = art.largeur * image.height() as f32 / image.width().max(1) as f32;
        let center = 1.8f32.max(0.5 + height / 2.0);
        if art.largeur < MIN_WIDTH || center + height / 2.0 > room.height - 0.25 {
            fail(
                name,
                format!(
                    "{who} : avec largeur = {}, le tableau ({:.2} × {height:.2} m) ne tient pas dans la hauteur de la salle ({} m)",
                    art.largeur, art.largeur, room.height
                ),
            );
        }
        // Le tableau, son cadre et son cartel à droite doivent tenir sur le mur, hors des portes.
        let (start, end) = (
            position - art.largeur / 2.0 - 0.1,
            position + art.largeur / 2.0 + 0.5,
        );
        let half = room.wall_length(side) / 2.0 - WALL;
        if start < -half || end > half {
            fail(
                name,
                format!(
                    "{who} : le tableau dépasse du mur {} de « {} » (place ou position trop près du bord)",
                    side.french(),
                    room.name
                ),
            );
        }
        if room
            .door_spans(side)
            .iter()
            .any(|&(a, b)| start < b && a < end)
        {
            fail(
                name,
                format!(
                    "{who} : le tableau est accroché sur une porte du mur {} de « {} »",
                    side.french(),
                    room.name
                ),
            );
        }
        if let Some((.., other)) = spans
            .iter()
            .find(|(rr, s, a, b, _)| *rr == r && *s == side && start < *b && *a < end)
        {
            fail(
                name,
                format!(
                    "« {other} » et « {} » se chevauchent sur le mur {} de « {} »",
                    art.titre,
                    side.french(),
                    room.name
                ),
            );
        }
        spans.push((r, side, start, end, art.titre.clone()));
        if file.exposition.vitrine.as_deref() == Some(art.image.as_str()) && showcase.is_none() {
            showcase = Some(index);
        }

        let (cols, rows, palette, cells) = mosaic::mosaic(&image, art.largeur, height);
        let offset = data.len();
        data.extend_from_slice(&cells);
        let mut palette_code = String::new();
        for [r, g, b] in &palette {
            let _ = write!(palette_code, "[{r}, {g}, {b}], ");
        }
        let _ = writeln!(
            artworks,
            "            Artwork {{ title: {:?}, slug: {:?}, artist: {:?}, year: {:?}, image: {:?}, technique: {:?}, collection: {:?}, note: {:?}, notice: {}, frame: Frame::{}, credit: Credit {{ author: {:?}, licence: {:?}, source: {:?} }}, \
             width: {:?}, height: {:?}, hook: Hook {{ room: {r}, side: Side::{}, position: {position:?}, center: {center:?}, place: {place} }}, \
             mosaic: Mosaic {{ cols: {cols}, rows: {rows}, palette: &[{palette_code}], offset: {offset} }} }},",
            art.titre.trim(),
            work_slug(&art.image),
            art.artiste
                .as_deref()
                .map(str::trim)
                .filter(|a| !a.is_empty()),
            art.annee.trim(),
            format!("{slug}/{}", art.image),
            art.technique.trim(),
            art.collection.trim(),
            art.note.trim(),
            notice_code(
                &art.description,
                &art.evocation,
                &art.histoire,
                &art.facture,
                &art.details,
                &art.themes,
                &art.anecdote
            ),
            frame_code(
                name,
                art.cadre.as_deref().unwrap_or(&file.exposition.cadre),
                &who
            ),
            art.credit.trim(),
            art.licence.trim(),
            art.source.trim(),
            art.largeur,
            (height * 1000.0).round() / 1000.0,
            side.name(),
        );
    }
    if let Some(vitrine) = &file.exposition.vitrine
        && showcase.is_none()
    {
        fail(
            name,
            format!("vitrine = \"{vitrine}\" : aucune œuvre n'utilise cette image"),
        );
    }

    // Les sculptures : posées sur le sol de leur salle, loin des murs et des portes.
    let mut sculptures = String::new();
    let mut footprints: Vec<(usize, f32, f32, f32, String)> = Vec::new();
    for (number, sc) in file.sculptures.iter().enumerate() {
        let who = format!("la sculpture « {} »", sc.titre);
        let r = room_index(sc.salle).unwrap_or_else(|| {
            fail(
                name,
                format!(
                    "{who} : salle = {}, mais les salles vont de 1 à {}",
                    sc.salle,
                    file.salles.len()
                ),
            )
        });
        let room = &rooms[r];
        let path = Path::new("modeles").join(&sc.modele);
        if sc.modele.contains("..") || !path.is_file() {
            fail(
                name,
                format!(
                    "{who} : le modèle « {} » est introuvable dans modeles/",
                    sc.modele
                ),
            );
        }
        let mut model = mesh::read(&path)
            .unwrap_or_else(|e| fail(name, format!("{who} : modèle « {} » : {e}", sc.modele)));
        model.stand_up(sc.haut.trim());
        model.normalize();
        model.simplify(SCULPTURE_TRIANGLES);
        let (bytes, bounds) = model.encode();
        let plinth = sc.socle.unwrap_or(if sc.hauteur < 1.2 { 0.9 } else { 0.4 });
        if !(0.1..=8.0).contains(&sc.hauteur) || !(0.0..=1.5).contains(&plinth) {
            fail(
                name,
                format!("{who} : hauteur de 0,1 à 8 m, socle de 0 à 1,5 m"),
            );
        }
        if plinth + sc.hauteur > room.height - 0.2 {
            fail(
                name,
                format!(
                    "{who} : avec son socle, elle touche le plafond de « {} »",
                    room.name
                ),
            );
        }
        // Le rayon de l'emprise au sol (socle compris).
        let radius = (bounds[3] - bounds[0]).max(bounds[5] - bounds[2]) * sc.hauteur / 2.0
            * std::f32::consts::SQRT_2
            + 0.15;
        let (x, z) = (room.x + sc.x, room.z + sc.z);
        let inside = (sc.x.abs() + radius) < room.width / 2.0 - WALL - 0.3
            && (sc.z.abs() + radius) < room.depth / 2.0 - WALL - 0.3;
        if !inside {
            fail(
                name,
                format!(
                    "{who} : trop près d'un mur de « {} » (x et z sont comptés depuis le centre de la salle)",
                    room.name
                ),
            );
        }
        for d in &room.doors {
            let (dx, dz) = room.point_on(d.side, d.position);
            if ((dx - x).powi(2) + (dz - z).powi(2)).sqrt() < radius + 1.6 {
                fail(
                    name,
                    format!("{who} : elle bloque une porte de « {} »", room.name),
                );
            }
        }
        if let Some((.., other)) = footprints.iter().find(|(rr, ox, oz, or, _)| {
            *rr == r && ((ox - x).powi(2) + (oz - z).powi(2)).sqrt() < or + radius + 0.6
        }) {
            fail(
                name,
                format!("« {other} » et « {} » sont trop proches", sc.titre),
            );
        }
        footprints.push((r, x, z, radius, sc.titre.clone()));
        let (color, roughness, metallic) = match sc.matiere.trim() {
            "marbre" => ([222, 217, 206], 0.45, 0.0),
            "platre" | "plâtre" => ([214, 207, 192], 0.85, 0.0),
            "pierre" => ([176, 166, 148], 0.9, 0.0),
            "bronze" => ([122, 86, 52], 0.4, 0.7),
            "or" => ([212, 175, 55], 0.3, 0.9),
            other if other.starts_with('#') => (hex(name, other, &who), 0.6, 0.0),
            other => fail(
                name,
                format!(
                    "{who} : matiere = \"{other}\" : marbre, platre, pierre, bronze, or ou une couleur #rrggbb"
                ),
            ),
        };
        // Une vignette pour le catalogue, servie avec les données.
        let picture = format!("{slug}-sculpture-{}.png", number + 1);
        let tint = color.map(|v| v.saturating_add(18));
        model
            .thumbnail(tint, 520)
            .save(data_dir.join(&picture))
            .expect("écriture d'une vignette");
        let offset = data.len();
        data.extend_from_slice(&bytes);
        let _ = writeln!(
            sculptures,
            "            Sculpture {{ title: {:?}, slug: {:?}, artist: {:?}, year: {:?}, technique: {:?}, collection: {:?}, note: {:?}, notice: {}, image: {picture:?}, room: {r}, \
             x: {x:?}, z: {z:?}, height: {:?}, rotation: {:?}, plinth: {plinth:?}, radius: {radius:?}, color: {color:?}, \
             roughness: {roughness:?}, metallic: {metallic:?}, mesh: MeshRef {{ offset: {offset}, size: {}, bounds: {bounds:?} }} }},",
            sc.titre.trim(),
            work_slug(&sc.modele),
            sc.artiste.trim(),
            sc.annee.trim(),
            sc.technique.trim(),
            sc.collection.trim(),
            sc.note.trim(),
            notice_code(
                &sc.description,
                &sc.evocation,
                &sc.histoire,
                &sc.facture,
                &sc.details,
                &sc.themes,
                &sc.anecdote
            ),
            sc.hauteur,
            sc.rotation.to_radians(),
            bytes.len(),
        );
    }

    // Chaque œuvre a sa page (/<exposition>/oeuvres/<nom>) : les noms sont uniques.
    let mut slugs: Vec<(String, &str)> = Vec::new();
    let works = file
        .oeuvres
        .iter()
        .map(|o| (work_slug(&o.image), o.titre.as_str()))
        .chain(
            file.sculptures
                .iter()
                .map(|s| (work_slug(&s.modele), s.titre.as_str())),
        );
    for (s, title) in works {
        if let Some((_, other)) = slugs.iter().find(|(o, _)| *o == s) {
            fail(
                name,
                format!(
                    "« {other} » et « {title} » auraient la même page « {s} » : renommez l'image ou le modèle de l'une des deux"
                ),
            );
        }
        slugs.push((s, title));
    }
    // Un bloc [[artistes]] par nom, et chacun pour un artiste de l'exposition.
    let painters: Vec<&str> = file
        .oeuvres
        .iter()
        .map(|o| {
            o.artiste
                .as_deref()
                .unwrap_or(&file.exposition.artiste)
                .trim()
        })
        .chain(file.sculptures.iter().map(|s| s.artiste.trim()))
        .collect();
    for (i, a) in file.artistes.iter().enumerate() {
        let n = a.nom.trim();
        if file.artistes[..i].iter().any(|b| b.nom.trim() == n) {
            fail(name, format!("l'artiste « {n} » a deux blocs [[artistes]]"));
        }
        if !painters.contains(&n) {
            fail(
                name,
                format!(
                    "le bloc [[artistes]] « {n} » ne correspond à aucune œuvre : son nom doit être celui du champ artiste des œuvres"
                ),
            );
        }
    }

    let e = &file.exposition;
    let [ar, ag, ab] = hex(name, &e.couleur, "couleur");
    let _ = writeln!(
        out,
        "    Exhibition {{\n        slug: {slug:?},\n        source: text!(include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/expositions/{slug}.toml\"))),\n        short_name: {:?},\n        title: {:?},\n        artist: {:?},\n        artist_dates: {:?},\n        movement: {:?},\n        ephemeral: {},\n        kicker: {:?},\n        introduction: {:?},\n        accent: [{ar}, {ag}, {ab}],\n        showcase: {:?},\n        museum: {},\n        data_size: {},\n        rooms: &[",
        e.nom_court.trim(),
        e.titre.trim(),
        e.artiste.trim(),
        e.dates.trim(),
        e.mouvement.trim(),
        e.ephemere,
        e.accroche.trim(),
        e.introduction.trim(),
        showcase,
        file.musee.as_ref().map_or("None".to_owned(), |m| format!(
            "Some(Museum {{ name: {:?}, city: {:?}, architect: {:?}, built: {:?}, presentation: text!({:?}), here: text!({:?}) }})",
            m.nom.trim(),
            m.ville.trim(),
            m.architecte.trim(),
            m.construction.trim(),
            m.presentation.trim(),
            m.ici.trim()
        )),
        data.len(),
    );
    for room in &rooms {
        let mut doors = String::new();
        for d in &room.doors {
            let _ = write!(
                doors,
                "Door {{ side: Side::{}, position: {:?}, width: {:?} }}, ",
                d.side.name(),
                d.position,
                d.width
            );
        }
        let _ = writeln!(
            out,
            "            Room {{ name: {:?}, theme: {:?}, walls: {:?}, x: {:?}, z: {:?}, width: {:?}, depth: {:?}, height: {:?}, corridor: {}, number: {}, doors: &[{doors}] }},",
            room.name.trim(),
            room.theme.trim(),
            room.walls,
            room.x,
            room.z,
            room.width,
            room.depth,
            room.height,
            room.corridor,
            room.number,
        );
    }
    out.push_str("        ],\n        artworks: &[\n");
    out.push_str(&artworks);
    out.push_str("        ],\n        sculptures: &[\n");
    out.push_str(&sculptures);
    out.push_str("        ],\n        artists: &[\n");
    for a in &file.artistes {
        let _ = writeln!(
            out,
            "            Artist {{ name: {:?}, life: text!({:?}), biography: text!({:?}), style: text!({:?}) }},",
            a.nom.trim(),
            a.vie.trim(),
            a.biographie.trim(),
            a.style.trim(),
        );
    }
    out.push_str("        ],\n    },\n");
}
