//! Les matières du musée, dessinées en Rust au lancement (aucune image à télécharger) :
//! un plâtre légèrement nuancé pour les murs et les plafonds, un parquet à lames pour les
//! sols, chacun avec sa carte de relief (les joints, le grain du plâtre).
//!
//! Les textures se répètent sans couture : le bruit est calculé sur une grille qui se
//! referme sur elle-même.

use paquets::scene::Texture;

/// Un nombre pseudo-aléatoire entre 0 et 1, toujours le même pour les mêmes entrées.
fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ seed.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0x00ff_ffff) as f32 / 16_777_215.0
}

/// Un bruit doux qui se répète toutes les `period` cases.
fn smooth_noise(x: f32, y: f32, period: i32, seed: u32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let wrap = |v: i32| v.rem_euclid(period);
    let (ix, iy) = (x0 as i32, y0 as i32);
    let at = |dx: i32, dy: i32| hash(wrap(ix + dx), wrap(iy + dy), seed);
    let top = at(0, 0) + (at(1, 0) - at(0, 0)) * sx;
    let bottom = at(0, 1) + (at(1, 1) - at(0, 1)) * sx;
    top + (bottom - top) * sy
}

/// Plusieurs octaves de bruit, de 0 à 1, pour `(u, v)` entre 0 et 1.
fn fractal(u: f32, v: f32, base: i32, octaves: u32, seed: u32) -> f32 {
    let (mut sum, mut amplitude, mut total, mut period) = (0.0, 1.0, 0.0, base);
    for o in 0..octaves {
        sum += smooth_noise(u * period as f32, v * period as f32, period, seed + o) * amplitude;
        total += amplitude;
        amplitude *= 0.5;
        period *= 2;
    }
    sum / total
}

/// Une carte de relief (normales en couleurs) tirée d'une carte de hauteurs `size` × `size`.
fn normal_map(size: u32, heights: &[f32], strength: f32) -> Texture {
    let at = |x: i32, y: i32| {
        let (x, y) = (x.rem_euclid(size as i32), y.rem_euclid(size as i32));
        heights[(y as u32 * size + x as u32) as usize]
    };
    Texture::from_fn(size, size, |x, y| {
        let (x, y) = (x as i32, y as i32);
        // L'image a sa première ligne en haut : v monte quand y descend.
        let dx = (at(x + 1, y) - at(x - 1, y)) * strength;
        let dy = (at(x, y - 1) - at(x, y + 1)) * strength;
        let (nx, ny, nz) = (-dx, -dy, 1.0f32);
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        let byte = |c: f32| ((c / len * 0.5 + 0.5) * 255.0).round() as u8;
        [byte(nx), byte(ny), byte(nz), 255]
    })
    .linear()
}

/// Le côté des textures, en pixels.
const PLASTER_SIZE: u32 = 256;
const PARQUET_SIZE: u32 = 512;

/// Le mètre couvert par une répétition du plâtre et du parquet.
pub const PLASTER_METERS: f32 = 2.0;
pub const PARQUET_METERS: f32 = 1.6;

/// Le plâtre : presque blanc, nuancé (la couleur du mur le teinte), et son grain.
pub fn plaster() -> (Texture, Texture) {
    let size = PLASTER_SIZE;
    let heights: Vec<f32> = (0..size * size)
        .map(|i| {
            let (u, v) = (
                (i % size) as f32 / size as f32,
                (i / size) as f32 / size as f32,
            );
            fractal(u, v, 8, 4, 11)
        })
        .collect();
    let color = Texture::from_fn(size, size, |x, y| {
        let (u, v) = (x as f32 / size as f32, y as f32 / size as f32);
        // De grandes taches très douces (le plâtre repris à la main) et un grain fin.
        let broad = fractal(u, v, 3, 3, 3);
        let fine = heights[(y * size + x) as usize];
        let k = 0.9 + 0.06 * broad + 0.04 * fine;
        let c = (k.min(1.0) * 255.0).round() as u8;
        [c, c, c.saturating_sub(1), 255]
    });
    (color, normal_map(size, &heights, 1.4))
}

/// Le parquet : des lames de chêne décalées, chacune de son ton, avec leur fil et leurs
/// joints.
pub fn parquet() -> (Texture, Texture) {
    let size = PARQUET_SIZE;
    // 8 rangées de lames de 20 cm sur 1,6 m ; les lames font 80 cm, décalées d'une
    // rangée à l'autre.
    const ROWS: u32 = 8;
    const PER_ROW: f32 = 2.0;
    let plank = |u: f32, v: f32| {
        let row = ((v * ROWS as f32).floor() as u32).min(ROWS - 1);
        let shift = hash(row as i32, 0, 5) * 0.9 + (row % 2) as f32 * 0.5;
        let along = (u * PER_ROW + shift).rem_euclid(PER_ROW);
        let index = along.floor() as i32;
        let in_row = v * ROWS as f32 - row as f32;
        (row, index, along - index as f32, in_row)
    };
    let mut heights = vec![0.0f32; (size * size) as usize];
    let pixel = 1.0 / size as f32;
    let colors: Vec<[u8; 4]> = (0..size * size)
        .map(|i| {
            let (x, y) = (i % size, i / size);
            let (u, v) = (x as f32 * pixel, y as f32 * pixel);
            let (row, index, a, b) = plank(u, v);
            let id = (row as i32) * 7 + index;
            let tone = 0.88 + 0.18 * hash(id, 1, 9);
            let warm = hash(id, 2, 9) * 0.08;
            // Le fil du bois, le long des lames, ondulé.
            let wave = fractal(u, v, 4, 3, 21 + row);
            let grain =
                ((b * 9.0 + wave * 6.0 + hash(id, 3, 9) * 6.0) * std::f32::consts::TAU).sin();
            let fibers = fractal(u * 0.25, v * 4.0, 16, 2, 31);
            let wood = tone * (0.92 + 0.045 * grain + 0.06 * fibers);
            // Les joints : un trait sombre entre les lames.
            let (plank_px, row_px) = (size as f32 / PER_ROW, size as f32 / ROWS as f32);
            let seam = a.min(1.0 - a) * plank_px < 1.2 || b.min(1.0 - b) * row_px < 1.2;
            heights[i as usize] = if seam { 0.0 } else { 0.6 + 0.1 * grain };
            let k = if seam { 0.45 } else { wood };
            let r = (168.0 * k * (1.0 + warm)).min(255.0);
            let g = (120.0 * k).min(255.0);
            let b = (78.0 * k * (1.0 - warm)).min(255.0);
            [r as u8, g as u8, b as u8, 255]
        })
        .collect();
    let color = Texture::from_rgba8(size, size, colors.concat());
    (color, normal_map(size, &heights, 2.5))
}

#[cfg(test)]
mod tests {
    use super::*;
    use paquets::scene::TextureSource;

    fn pixels(t: &Texture) -> Vec<u8> {
        match t.source() {
            TextureSource::Rgba8 { data, .. } => data.to_vec(),
            TextureSource::Url(_) => panic!("des pixels"),
        }
    }

    #[test]
    fn the_noise_repeats_without_a_seam() {
        for (x, y) in [(0.0, 0.3), (0.5, 0.0), (0.25, 0.75)] {
            let a = fractal(x, y, 8, 4, 1);
            let b = fractal(x + 1.0, y, 8, 4, 1);
            let c = fractal(x, y + 1.0, 8, 4, 1);
            assert!((a - b).abs() < 1e-4 && (a - c).abs() < 1e-4, "{a} {b} {c}");
            assert!((0.0..=1.0).contains(&a));
        }
    }

    #[test]
    fn plaster_is_light_and_parquet_is_wood_with_seams() {
        let (plaster, relief) = plaster();
        assert_eq!(plaster.size(), Some((PLASTER_SIZE, PLASTER_SIZE)));
        let p = pixels(&plaster);
        let mean = p.chunks(4).map(|c| f32::from(c[0])).sum::<f32>() / (p.len() / 4) as f32;
        assert!(mean > 225.0 && mean < 255.0, "{mean}");
        // Le relief penche à peine : surtout du bleu (la normale vers le dehors).
        let n = pixels(&relief);
        assert!(n.chunks(4).all(|c| c[2] > 200), "relief trop fort");

        let (wood, _) = parquet();
        let w = pixels(&wood);
        let px = |i: usize| (w[i * 4], w[i * 4 + 1], w[i * 4 + 2]);
        // Du bois : plus rouge que vert, plus vert que bleu.
        let (r, g, b) = px(PARQUET_SIZE as usize * 37 + 101);
        assert!(r > g && g > b, "{r} {g} {b}");
        // Des joints sombres au bord des rangées (une rangée toutes les 64 lignes).
        let seam = px(PARQUET_SIZE as usize * 64 + 200).0;
        let inside = px(PARQUET_SIZE as usize * 96 + 200).0;
        assert!(seam + 30 < inside, "{seam} {inside}");
    }
}
