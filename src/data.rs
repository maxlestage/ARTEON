//! Le fichier de données d'une exposition (`donnees/<nom>.bin`, écrit par `build.rs`) :
//! les cases des mosaïques et les maillages des sculptures. Le navigateur le télécharge
//! avant de construire la scène ; les tests le lisent sur le disque.

use crate::artworks::{Exhibition, Mosaic, Sculpture};

/// Le fichier de données d'une exposition, en mémoire.
#[derive(Clone, Debug)]
pub struct ExhibitionData {
    bytes: Vec<u8>,
}

/// Le maillage d'une sculpture, à sa taille réelle (en mètres), posé sur y = 0 et
/// centré sur l'origine, avant rotation.
pub struct SculptureMesh {
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<u32>,
}

impl ExhibitionData {
    /// Vérifie un fichier téléchargé : son en-tête et sa taille.
    pub fn new(ex: &Exhibition, bytes: Vec<u8>) -> Result<ExhibitionData, String> {
        if bytes.len() != ex.data_size as usize || !bytes.starts_with(b"ARTN\x01") {
            return Err(format!(
                "données de « {} » invalides ({} octets au lieu de {})",
                ex.slug,
                bytes.len(),
                ex.data_size
            ));
        }
        Ok(ExhibitionData { bytes })
    }

    /// Le fichier écrit par `build.rs`, lu sur le disque (serveur et tests).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(ex: &Exhibition) -> Result<ExhibitionData, String> {
        let path = format!("{}/{}.bin", crate::artworks::DATA_DIR, ex.slug);
        let bytes = std::fs::read(&path).map_err(|e| format!("{path} : {e}"))?;
        ExhibitionData::new(ex, bytes)
    }

    /// Les cases d'une mosaïque.
    pub fn cells(&self, mosaic: &Mosaic) -> &[u8] {
        let start = mosaic.offset as usize;
        &self.bytes[start..start + mosaic.len()]
    }

    /// Le maillage d'une sculpture, mis à sa hauteur.
    pub fn sculpture(&self, sculpture: &Sculpture) -> SculptureMesh {
        let start = sculpture.mesh.offset as usize;
        let b = &self.bytes[start..start + sculpture.mesh.size as usize];
        let u32_at = |o: usize| u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
        let f32_at = |o: usize| f32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
        let (vertices, triangles) = (u32_at(0) as usize, u32_at(4) as usize);
        let lo = [f32_at(8), f32_at(12), f32_at(16)];
        let hi = [f32_at(20), f32_at(24), f32_at(28)];
        let scale = sculpture.height;
        let mut o = 32;
        let mut positions = Vec::with_capacity(vertices * 3);
        for _ in 0..vertices {
            for k in 0..3 {
                let q = f32::from(u16::from_le_bytes([b[o], b[o + 1]])) / 65535.0;
                positions.push((lo[k] + q * (hi[k] - lo[k])) * scale);
                o += 2;
            }
        }
        let mut normals = Vec::with_capacity(vertices * 3);
        for _ in 0..vertices * 3 {
            normals.push(f32::from(b[o] as i8) / 127.0);
            o += 1;
        }
        let small = vertices < 65_536;
        let mut indices = Vec::with_capacity(triangles * 3);
        for _ in 0..triangles * 3 {
            if small {
                indices.push(u32::from(u16::from_le_bytes([b[o], b[o + 1]])));
                o += 2;
            } else {
                indices.push(u32_at(o));
                o += 4;
            }
        }
        SculptureMesh {
            positions,
            normals,
            indices,
        }
    }
}
