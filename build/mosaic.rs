//! Les mosaïques des tableaux : l'image réduite à une grille (une case = 7 mm dans la
//! galerie), puis ramenée à une palette de 64 couleurs par k-moyennes.

/// La largeur d'une case de la mosaïque, en mètres.
const CELL: f32 = 0.007;
const MAX_CELLS: u32 = 340;
/// Le nombre de couleurs de la palette de chaque tableau.
const COLORS: usize = 64;

/// La grille de l'image et sa palette : (colonnes, lignes, palette, une couleur par case,
/// ligne par ligne depuis le haut).
pub fn mosaic(
    image: &image::RgbImage,
    width: f32,
    height: f32,
) -> (u32, u32, Vec<[u8; 3]>, Vec<u8>) {
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
