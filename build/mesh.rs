//! Les sculptures : lire un modèle 3D (STL binaire ou texte, OBJ, glTF), le simplifier pour
//! qu'il reste léger dans le navigateur, le poser debout sur l'origine à 1 m de haut, et
//! l'écrire en binaire compact (positions sur 16 bits, normales sur 8 bits).

// Les boucles sur les trois axes (x, y, z) sont plus lisibles avec un index.
#![allow(clippy::needless_range_loop)]

use std::collections::HashMap;

/// Un maillage : des sommets et des triangles.
pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

/// Lit un fichier `.stl` ou `.obj`.
pub fn read(path: &std::path::Path) -> Result<Mesh, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("illisible ({e})"))?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let soup = match ext.as_str() {
        "stl" => stl(&bytes)?,
        "obj" => obj(&bytes)?,
        "gltf" => gltf(path, &bytes)?,
        other => {
            return Err(format!(
                "format « .{other} » inconnu : il faut un fichier .stl, .obj ou .gltf"
            ));
        }
    };
    if soup.is_empty() {
        return Err("le modèle ne contient aucun triangle".into());
    }
    Ok(weld(&soup))
}

/// Des triangles isolés (trois sommets chacun).
type Soup = Vec<[[f32; 3]; 3]>;

fn stl(bytes: &[u8]) -> Result<Soup, String> {
    // Un STL binaire : 80 octets d'en-tête, le nombre de triangles, puis 50 octets par
    // triangle. Un STL texte commence par « solid » et ne respecte pas cette taille.
    if bytes.len() >= 84 {
        let count = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]) as usize;
        if 84 + count * 50 == bytes.len() {
            let mut soup = Vec::with_capacity(count);
            for t in 0..count {
                let base = 84 + t * 50 + 12;
                let f = |i: usize| {
                    let o = base + i * 4;
                    f32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]])
                };
                soup.push([[f(0), f(1), f(2)], [f(3), f(4), f(5)], [f(6), f(7), f(8)]]);
            }
            return Ok(soup);
        }
    }
    let text = String::from_utf8_lossy(bytes);
    let mut soup = Vec::new();
    let mut corner = Vec::with_capacity(3);
    for line in text.lines() {
        let mut words = line.split_whitespace();
        if words.next() == Some("vertex") {
            let v: Vec<f32> = words.filter_map(|w| w.parse().ok()).collect();
            if v.len() == 3 {
                corner.push([v[0], v[1], v[2]]);
            }
            if corner.len() == 3 {
                soup.push([corner[0], corner[1], corner[2]]);
                corner.clear();
            }
        }
    }
    if soup.is_empty() {
        return Err("fichier STL illisible".into());
    }
    Ok(soup)
}

fn obj(bytes: &[u8]) -> Result<Soup, String> {
    let text = String::from_utf8_lossy(bytes);
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut soup = Vec::new();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        match words.next() {
            Some("v") => {
                let v: Vec<f32> = words.take(3).filter_map(|w| w.parse().ok()).collect();
                if v.len() == 3 {
                    vertices.push([v[0], v[1], v[2]]);
                }
            }
            Some("f") => {
                // « f 1 2 3 4 », « f 1/1/1 2/2/2 … » ; les index négatifs comptent depuis la fin.
                let face: Vec<usize> = words
                    .filter_map(|w| w.split('/').next()?.parse::<i64>().ok())
                    .filter_map(|i| {
                        let n = vertices.len() as i64;
                        let k = if i < 0 { n + i } else { i - 1 };
                        (0..n).contains(&k).then_some(k as usize)
                    })
                    .collect();
                for k in 1..face.len().saturating_sub(1) {
                    soup.push([vertices[face[0]], vertices[face[k]], vertices[face[k + 1]]]);
                }
            }
            _ => {}
        }
    }
    Ok(soup)
}

/// Un glTF « standard » (JSON et fichier .bin à côté, sans compression Draco) : toutes
/// les primitives triangulaires de la scène, avec les transformations de leurs nœuds.
fn gltf(path: &std::path::Path, bytes: &[u8]) -> Result<Soup, String> {
    let doc: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| format!("glTF illisible ({e})"))?;
    if doc["extensionsUsed"]
        .as_array()
        .is_some_and(|a| a.iter().any(|e| e == "KHR_draco_mesh_compression"))
    {
        return Err(
            "glTF compressé avec Draco : non pris en charge, prendre la version standard".into(),
        );
    }
    let dir = path.parent().unwrap_or(std::path::Path::new("."));
    let buffers: Vec<Vec<u8>> = doc["buffers"]
        .as_array()
        .ok_or("glTF sans buffers")?
        .iter()
        .map(|b| {
            let uri = b["uri"]
                .as_str()
                .ok_or("buffer intégré non pris en charge")?;
            std::fs::read(dir.join(uri)).map_err(|e| format!("{uri} illisible ({e})"))
        })
        .collect::<Result<_, String>>()?;
    let accessor = |index: u64| -> Result<(Vec<u8>, String, u64, usize), String> {
        let a = &doc["accessors"][index as usize];
        let view = &doc["bufferViews"]
            [a["bufferView"].as_u64().ok_or("accessor sans bufferView")? as usize];
        let buffer = &buffers[view["buffer"].as_u64().unwrap_or(0) as usize];
        let start = (view["byteOffset"].as_u64().unwrap_or(0)
            + a["byteOffset"].as_u64().unwrap_or(0)) as usize;
        let count = a["count"].as_u64().unwrap_or(0) as usize;
        let kind = a["type"].as_str().unwrap_or("").to_owned();
        let component = a["componentType"].as_u64().unwrap_or(0);
        let width = match (kind.as_str(), component) {
            ("VEC3", 5126) => 12,
            ("SCALAR", 5121) => 1,
            ("SCALAR", 5123) => 2,
            ("SCALAR", 5125) => 4,
            _ => return Err(format!("accessor {kind}/{component} non pris en charge")),
        };
        let stride = view["byteStride"].as_u64().map_or(width, |s| s as usize);
        let mut data = Vec::with_capacity(count * width);
        for i in 0..count {
            let o = start + i * stride;
            data.extend_from_slice(buffer.get(o..o + width).ok_or("glTF tronqué")?);
        }
        Ok((data, kind, component, count))
    };
    let mut soup = Vec::new();
    let mut stack: Vec<(u64, [f32; 16])> = doc["scenes"]
        [doc["scene"].as_u64().unwrap_or(0) as usize]["nodes"]
        .as_array()
        .map(|n| {
            n.iter()
                .filter_map(|v| v.as_u64())
                .map(|n| (n, IDENTITY))
                .collect()
        })
        .unwrap_or_default();
    while let Some((n, parent)) = stack.pop() {
        let node = &doc["nodes"][n as usize];
        let matrix = mul(&parent, &local_matrix(node));
        if let Some(children) = node["children"].as_array() {
            stack.extend(
                children
                    .iter()
                    .filter_map(|c| c.as_u64())
                    .map(|c| (c, matrix)),
            );
        }
        let Some(m) = node["mesh"].as_u64() else {
            continue;
        };
        for prim in doc["meshes"][m as usize]["primitives"]
            .as_array()
            .into_iter()
            .flatten()
        {
            if prim["mode"].as_u64().unwrap_or(4) != 4 {
                continue;
            }
            let (pos, _, _, count) = accessor(
                prim["attributes"]["POSITION"]
                    .as_u64()
                    .ok_or("primitive sans POSITION")?,
            )?;
            let f = |i: usize| f32::from_le_bytes([pos[i], pos[i + 1], pos[i + 2], pos[i + 3]]);
            let points: Vec<[f32; 3]> = (0..count)
                .map(|v| transform(&matrix, [f(v * 12), f(v * 12 + 4), f(v * 12 + 8)]))
                .collect();
            let indices: Vec<usize> = match prim["indices"].as_u64() {
                Some(i) => {
                    let (data, _, component, count) = accessor(i)?;
                    (0..count)
                        .map(|k| match component {
                            5121 => data[k] as usize,
                            5123 => u16::from_le_bytes([data[2 * k], data[2 * k + 1]]) as usize,
                            _ => u32::from_le_bytes([
                                data[4 * k],
                                data[4 * k + 1],
                                data[4 * k + 2],
                                data[4 * k + 3],
                            ]) as usize,
                        })
                        .collect()
                }
                None => (0..count).collect(),
            };
            for t in indices.chunks_exact(3) {
                if t.iter().all(|&i| i < points.len()) {
                    soup.push([points[t[0]], points[t[1]], points[t[2]]]);
                }
            }
        }
    }
    Ok(soup)
}

const IDENTITY: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

/// La matrice locale d'un nœud glTF (en colonnes) : `matrix`, ou translation × rotation × échelle.
fn local_matrix(node: &serde_json::Value) -> [f32; 16] {
    let numbers = |key: &str| -> Option<Vec<f32>> {
        node[key]
            .as_array()
            .map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect())
    };
    if let Some(m) = numbers("matrix").filter(|m| m.len() == 16) {
        let mut out = [0.0; 16];
        out.copy_from_slice(&m);
        return out;
    }
    let t = numbers("translation").unwrap_or(vec![0.0; 3]);
    let r = numbers("rotation").unwrap_or(vec![0.0, 0.0, 0.0, 1.0]);
    let s = numbers("scale").unwrap_or(vec![1.0; 3]);
    let (x, y, z, w) = (r[0], r[1], r[2], r[3]);
    [
        (1.0 - 2.0 * (y * y + z * z)) * s[0],
        (2.0 * (x * y + z * w)) * s[0],
        (2.0 * (x * z - y * w)) * s[0],
        0.0,
        (2.0 * (x * y - z * w)) * s[1],
        (1.0 - 2.0 * (x * x + z * z)) * s[1],
        (2.0 * (y * z + x * w)) * s[1],
        0.0,
        (2.0 * (x * z + y * w)) * s[2],
        (2.0 * (y * z - x * w)) * s[2],
        (1.0 - 2.0 * (x * x + y * y)) * s[2],
        0.0,
        t[0],
        t[1],
        t[2],
        1.0,
    ]
}

fn mul(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut out = [0.0; 16];
    for c in 0..4 {
        for r in 0..4 {
            out[c * 4 + r] = (0..4).map(|k| a[k * 4 + r] * b[c * 4 + k]).sum();
        }
    }
    out
}

fn transform(m: &[f32; 16], p: [f32; 3]) -> [f32; 3] {
    [
        m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12],
        m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13],
        m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14],
    ]
}

/// Réunit les sommets identiques.
fn weld(soup: &Soup) -> Mesh {
    let mut index: HashMap<[u32; 3], u32> = HashMap::new();
    let mut positions = Vec::new();
    let mut triangles = Vec::with_capacity(soup.len());
    for tri in soup {
        let mut t = [0u32; 3];
        for (k, p) in tri.iter().enumerate() {
            let key = [p[0].to_bits(), p[1].to_bits(), p[2].to_bits()];
            t[k] = *index.entry(key).or_insert_with(|| {
                positions.push(*p);
                (positions.len() - 1) as u32
            });
        }
        if t[0] != t[1] && t[1] != t[2] && t[0] != t[2] {
            triangles.push(t);
        }
    }
    Mesh {
        positions,
        triangles,
    }
}

impl Mesh {
    /// Tourne le modèle : `up` dit quel axe du fichier pointe vers le haut ("y" ou "z").
    pub fn stand_up(&mut self, up: &str) {
        if up == "z" {
            for p in &mut self.positions {
                *p = [p[0], p[2], -p[1]];
            }
        } else if up == "-z" {
            for p in &mut self.positions {
                *p = [p[0], -p[2], p[1]];
            }
        }
    }

    fn bounds(&self) -> ([f32; 3], [f32; 3]) {
        let mut lo = [f32::MAX; 3];
        let mut hi = [f32::MIN; 3];
        for p in &self.positions {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        (lo, hi)
    }

    /// Pose le modèle : centré en x et z, le bas à y = 0, 1 m de haut.
    pub fn normalize(&mut self) {
        let (lo, hi) = self.bounds();
        let height = (hi[1] - lo[1]).max(1e-6);
        let (cx, cz) = ((lo[0] + hi[0]) / 2.0, (lo[2] + hi[2]) / 2.0);
        for p in &mut self.positions {
            *p = [
                (p[0] - cx) / height,
                (p[1] - lo[1]) / height,
                (p[2] - cz) / height,
            ];
        }
    }

    /// Simplifie le maillage jusqu'à `target` triangles environ : les sommets proches
    /// sont regroupés sur une grille, de plus en plus fine tant qu'il reste de la marge.
    pub fn simplify(&mut self, target: usize) {
        if self.triangles.len() <= target {
            return;
        }
        let (lo, hi) = self.bounds();
        let size = (0..3)
            .map(|k| hi[k] - lo[k])
            .fold(0.0f32, f32::max)
            .max(1e-6);
        let (mut low, mut high) = (8.0f32, 2048.0f32);
        let mut best: Option<Mesh> = None;
        for _ in 0..14 {
            let cells = (low + high) / 2.0;
            let candidate = self.clustered(lo, size / cells);
            if candidate.triangles.len() > target {
                high = cells;
            } else {
                low = cells;
                best = Some(candidate);
            }
        }
        if let Some(mesh) = best {
            *self = mesh;
        }
    }

    fn clustered(&self, lo: [f32; 3], cell: f32) -> Mesh {
        let mut cluster: HashMap<[i32; 3], u32> = HashMap::new();
        let mut sums: Vec<([f64; 3], u32)> = Vec::new();
        let remap: Vec<u32> = self
            .positions
            .iter()
            .map(|p| {
                let key = [
                    ((p[0] - lo[0]) / cell) as i32,
                    ((p[1] - lo[1]) / cell) as i32,
                    ((p[2] - lo[2]) / cell) as i32,
                ];
                let id = *cluster.entry(key).or_insert_with(|| {
                    sums.push(([0.0; 3], 0));
                    (sums.len() - 1) as u32
                });
                let s = &mut sums[id as usize];
                for k in 0..3 {
                    s.0[k] += f64::from(p[k]);
                }
                s.1 += 1;
                id
            })
            .collect();
        let positions = sums
            .iter()
            .map(|(s, n)| {
                let n = f64::from((*n).max(1));
                [(s[0] / n) as f32, (s[1] / n) as f32, (s[2] / n) as f32]
            })
            .collect();
        let mut seen = std::collections::HashSet::new();
        let mut triangles = Vec::new();
        for t in &self.triangles {
            let r = [
                remap[t[0] as usize],
                remap[t[1] as usize],
                remap[t[2] as usize],
            ];
            if r[0] == r[1] || r[1] == r[2] || r[0] == r[2] {
                continue;
            }
            // Un même triangle, dans n'importe quel ordre de départ, n'est gardé qu'une fois.
            let start = (0..3).min_by_key(|&k| r[k]).unwrap_or(0);
            let key = [r[start], r[(start + 1) % 3], r[(start + 2) % 3]];
            if seen.insert(key) {
                triangles.push(r);
            }
        }
        Mesh {
            positions,
            triangles,
        }
    }

    /// Les normales des sommets : la moyenne des faces qui les touchent, pondérée par
    /// leur surface (un modèle lisse).
    fn normals(&self) -> Vec<[f32; 3]> {
        let mut normals = vec![[0.0f32; 3]; self.positions.len()];
        for t in &self.triangles {
            let [a, b, c] = t.map(|i| self.positions[i as usize]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            for &i in t {
                for k in 0..3 {
                    normals[i as usize][k] += n[k];
                }
            }
        }
        for n in &mut normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-12);
            *n = [n[0] / len, n[1] / len, n[2] / len];
        }
        normals
    }

    /// Le maillage en STL binaire (pour pré-simplifier un gros scan avant de l'ajouter à
    /// `modeles/`).
    #[allow(dead_code)]
    pub fn write_stl(&self) -> Vec<u8> {
        let mut out = vec![0u8; 80];
        out.extend_from_slice(&(self.triangles.len() as u32).to_le_bytes());
        for t in &self.triangles {
            out.extend_from_slice(&[0u8; 12]);
            for &i in t {
                for v in self.positions[i as usize] {
                    out.extend_from_slice(&v.to_le_bytes());
                }
            }
            out.extend_from_slice(&[0, 0]);
        }
        out
    }

    #[allow(dead_code)]
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Une vignette de la sculpture vue de face (depuis +z), éclairée de trois quarts,
    /// sur fond transparent, haute de `size` pixels.
    pub fn thumbnail(&self, rgb: [u8; 3], size: u32) -> image::RgbaImage {
        let (lo, hi) = self.bounds();
        let span = (hi[1] - lo[1]).max(hi[0] - lo[0]).max(1e-6);
        let scale = (size as f32 - 16.0) / span;
        let width = (((hi[0] - lo[0]) * scale) as u32 + 16).max(16);
        let mut img = image::RgbaImage::new(width, size);
        let mut depth = vec![f32::MIN; (width * size) as usize];
        let normals = self.normals();
        let light = {
            let l = [-0.45f32, 0.55, 0.7];
            let n = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
            [l[0] / n, l[1] / n, l[2] / n]
        };
        let project = |p: [f32; 3]| -> [f32; 3] {
            [
                (p[0] - lo[0]) * scale + 8.0,
                size as f32 - ((p[1] - lo[1]) * scale + 8.0),
                p[2],
            ]
        };
        for t in &self.triangles {
            let v = t.map(|i| project(self.positions[i as usize]));
            let n = t.map(|i| normals[i as usize]);
            let (x0, x1) = (
                v.iter().map(|p| p[0]).fold(f32::MAX, f32::min),
                v.iter().map(|p| p[0]).fold(f32::MIN, f32::max),
            );
            let (y0, y1) = (
                v.iter().map(|p| p[1]).fold(f32::MAX, f32::min),
                v.iter().map(|p| p[1]).fold(f32::MIN, f32::max),
            );
            let area = (v[1][0] - v[0][0]) * (v[2][1] - v[0][1])
                - (v[2][0] - v[0][0]) * (v[1][1] - v[0][1]);
            if area.abs() < 1e-9 {
                continue;
            }
            for py in (y0.floor().max(0.0) as u32)..=(y1.ceil().min(size as f32 - 1.0) as u32) {
                for px in (x0.floor().max(0.0) as u32)..=(x1.ceil().min(width as f32 - 1.0) as u32)
                {
                    let (x, y) = (px as f32 + 0.5, py as f32 + 0.5);
                    let w0 = ((v[1][0] - x) * (v[2][1] - y) - (v[2][0] - x) * (v[1][1] - y)) / area;
                    let w1 = ((v[2][0] - x) * (v[0][1] - y) - (v[0][0] - x) * (v[2][1] - y)) / area;
                    let w2 = 1.0 - w0 - w1;
                    if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                        continue;
                    }
                    let z = w0 * v[0][2] + w1 * v[1][2] + w2 * v[2][2];
                    let k = (py * width + px) as usize;
                    if z <= depth[k] {
                        continue;
                    }
                    depth[k] = z;
                    let nn = [0, 1, 2].map(|c| w0 * n[0][c] + w1 * n[1][c] + w2 * n[2][c]);
                    let len = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2])
                        .sqrt()
                        .max(1e-6);
                    let lambert =
                        ((nn[0] * light[0] + nn[1] * light[1] + nn[2] * light[2]) / len).abs();
                    let shade = 0.32 + 0.68 * lambert;
                    let c = rgb.map(|v| (f32::from(v) * shade).min(255.0) as u8);
                    img.put_pixel(px, py, image::Rgba([c[0], c[1], c[2], 255]));
                }
            }
        }
        img
    }

    /// Le maillage en binaire : le nombre de sommets et de triangles (u32), la boîte
    /// englobante (6 × f32), les positions sur 16 bits dans cette boîte, les normales
    /// sur 8 bits signés, puis les index (16 bits s'il y a moins de 65 536 sommets,
    /// 32 bits sinon). Renvoie aussi la boîte englobante.
    pub fn encode(&self) -> (Vec<u8>, [f32; 6]) {
        let (lo, hi) = self.bounds();
        let mut out = Vec::new();
        out.extend_from_slice(&(self.positions.len() as u32).to_le_bytes());
        out.extend_from_slice(&(self.triangles.len() as u32).to_le_bytes());
        for v in lo.iter().chain(hi.iter()) {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for p in &self.positions {
            for k in 0..3 {
                let span = (hi[k] - lo[k]).max(1e-9);
                let q = (((p[k] - lo[k]) / span) * 65535.0)
                    .round()
                    .clamp(0.0, 65535.0) as u16;
                out.extend_from_slice(&q.to_le_bytes());
            }
        }
        for n in self.normals() {
            for k in 0..3 {
                out.push((n[k] * 127.0).round().clamp(-127.0, 127.0) as i8 as u8);
            }
        }
        let small = self.positions.len() < 65_536;
        for t in &self.triangles {
            for &i in t {
                if small {
                    out.extend_from_slice(&(i as u16).to_le_bytes());
                } else {
                    out.extend_from_slice(&i.to_le_bytes());
                }
            }
        }
        (out, [lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]])
    }
}
