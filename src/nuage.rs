//! Les images des tableaux, gardées sur Nuage plutôt que dans l'application.
//!
//! Le dépôt et l'application déployée ne gardent pas les images : chaque dossier de
//! `public/tableaux/` n'a que son `nuage.lock` (voir `outils/nuage.py`), et les octets
//! sont dans les fichiers de l'application ARTEON sur Nuage. Une image présente sur le
//! disque (en développement, ou une exposition éphémère) est servie telle quelle par
//! `public/` ; sinon, `/tableaux/<exposition>/<image>` la demande à Nuage la première fois
//! (un lien signé, puis les octets depuis le bucket), la garde dans un cache sur le disque
//! et la sert ensuite depuis ce cache. Elle reste servie par le site lui-même, pas par une
//! redirection : le musée 3D en fait des textures WebGL, qui doivent venir du même site.
//!
//! Réglages : `NUAGE_URL` (le serveur Nuage, celui d'ARTEON par défaut), `NUAGE_TOKEN`
//! (un jeton de l'application ARTEON, en lecture seule) et `ARTEON_CACHE` (le dossier du
//! cache, un dossier temporaire par défaut).

use std::collections::HashSet;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use paquets::prelude::*;

use arteon::artworks::EXHIBITIONS;

/// Le serveur Nuage d'ARTEON, quand `NUAGE_URL` ne dit pas autre chose.
pub const DEFAULT_URL: &str = "https://nuage-f48a8945f31c.herokuapp.com";

/// Une image ne pèse jamais plus que cela (les plus grandes font quelques Mo).
const MAX_BYTES: u64 = 64 << 20;

/// Combien de temps un navigateur garde une image sans la redemander.
const MAX_AGE: &str = "public, max-age=604800";

/// Où prendre les images qui ne sont pas sur le disque.
pub struct Images {
    /// Les images du catalogue (`van-gogh/iris.jpg`) : les seules que l'on demande à Nuage.
    known: HashSet<&'static str>,
    url: String,
    token: Option<String>,
    cache: PathBuf,
    agent: ureq::Agent,
}

impl Images {
    /// Les réglages de l'environnement (`NUAGE_URL`, `NUAGE_TOKEN`, `ARTEON_CACHE`).
    pub fn from_env() -> Images {
        let var = |name: &str| {
            std::env::var(name)
                .ok()
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
        };
        Images::new(
            var("NUAGE_URL").unwrap_or_else(|| DEFAULT_URL.to_owned()),
            var("NUAGE_TOKEN"),
            var("ARTEON_CACHE")
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::temp_dir().join("arteon-tableaux")),
        )
    }

    pub fn new(url: String, token: Option<String>, cache: PathBuf) -> Images {
        let known = EXHIBITIONS
            .iter()
            .flat_map(|ex| ex.artworks.iter().map(|art| art.image))
            .collect();
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(60)))
            .http_status_as_error(false)
            .build()
            .into();
        Images {
            known,
            url: url.trim_end_matches('/').to_owned(),
            token,
            cache,
            agent,
        }
    }

    /// L'image `name` (`van-gogh/iris.jpg`) : du cache, ou de Nuage. `None` pour une image
    /// qui n'est pas au catalogue ; une erreur quand Nuage ne la donne pas.
    pub fn get(&self, name: &str) -> Option<Result<Vec<u8>, String>> {
        if !self.known.contains(name) {
            return None;
        }
        let cached = self.cache.join(name);
        if let Ok(bytes) = std::fs::read(&cached) {
            return Some(Ok(bytes));
        }
        Some(self.download(name).inspect(|bytes| save(&cached, bytes)))
    }

    fn download(&self, name: &str) -> Result<Vec<u8>, String> {
        let token = self.token.as_deref().ok_or(
            "NUAGE_TOKEN n'est pas réglé : les images gardées sur Nuage ne peuvent pas être servies",
        )?;
        // Le lien signé vers le bucket, puis les octets, sans le jeton.
        let mut res = self
            .agent
            .get(format!("{}/v1/files/tableaux/{name}?link", self.url))
            .header("authorization", format!("Bearer {token}"))
            .header("accept", "application/json")
            .call()
            .map_err(|e| format!("Nuage ne répond pas : {e}"))?;
        let status = res.status().as_u16();
        let body = res
            .body_mut()
            .read_to_string()
            .map_err(|e| format!("Nuage : {e}"))?;
        if status != 200 {
            return Err(format!("Nuage répond {status} : {body}"));
        }
        let link: paquets::serde_json::Value =
            paquets::serde_json::from_str(&body).map_err(|e| format!("Nuage : {e}"))?;
        let url = link["url"]
            .as_str()
            .ok_or_else(|| format!("Nuage ne donne pas de lien : {body}"))?;
        let mut res = self
            .agent
            .get(url)
            .call()
            .map_err(|e| format!("le bucket ne répond pas : {e}"))?;
        if res.status() != 200 {
            return Err(format!("le bucket répond {}", res.status().as_u16()));
        }
        let mut bytes = Vec::new();
        res.body_mut()
            .as_reader()
            .take(MAX_BYTES)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("le bucket : {e}"))?;
        Ok(bytes)
    }
}

/// Garde une image dans le cache : écrite à côté, puis renommée, pour qu'une lecture ne
/// tombe jamais sur un fichier à moitié écrit.
fn save(path: &Path, bytes: &[u8]) {
    let Some(parent) = path.parent() else { return };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let partial = parent.join(format!(".{unique}.partiel"));
    if std::fs::write(&partial, bytes).is_ok() && std::fs::rename(&partial, path).is_err() {
        let _ = std::fs::remove_file(&partial);
    }
}

fn content_type(name: &str) -> &'static str {
    match name
        .rsplit('.')
        .next()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        _ => "image/jpeg",
    }
}

static IMAGES: OnceLock<Arc<Images>> = OnceLock::new();

/// `/tableaux/*chemin` : les images qui ne sont pas dans `public/tableaux/`.
pub async fn tableau(req: Request) -> Response {
    let name = req
        .path()
        .strip_prefix("/tableaux/")
        .unwrap_or_default()
        .to_owned();
    let images = IMAGES.get_or_init(|| Arc::new(Images::from_env())).clone();
    let lookup = {
        let name = name.clone();
        tokio::task::spawn_blocking(move || images.get(&name)).await
    };
    match lookup {
        Ok(Some(Ok(bytes))) => Response::new()
            .content_type(content_type(&name))
            .header("cache-control", MAX_AGE)
            .send(bytes),
        Ok(None) => crate::not_found(req).await.into_response(),
        Ok(Some(Err(e))) => {
            eprintln!("tableaux/{name} : {e}");
            Response::new()
                .status(502)
                .content_type("text/plain; charset=utf-8")
                .send(format!("L'image {name} n'a pas pu être prise sur Nuage."))
        }
        Err(e) => Response::new()
            .status(500)
            .content_type("text/plain; charset=utf-8")
            .send(format!("{e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    /// Un faux Nuage, sur un port local : `?link` donne un lien vers lui-même, qui donne
    /// les octets. Il compte les requêtes.
    fn fake_nuage(bytes: &'static [u8]) -> (String, Arc<std::sync::atomic::AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (base, seen) = (url.clone(), count.clone());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                seen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut first = String::new();
                reader.read_line(&mut first).unwrap();
                let mut auth = false;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line
                        .to_ascii_lowercase()
                        .starts_with("authorization: bearer secret")
                    {
                        auth = true;
                    }
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                }
                let mut stream = stream;
                let (status, body): (&str, Vec<u8>) = if first.contains("?link") {
                    if auth {
                        let link = format!("{{\"url\":\"{base}/bucket/x\"}}");
                        ("200 OK", link.into_bytes())
                    } else {
                        ("401 Unauthorized", b"{}".to_vec())
                    }
                } else if first.contains("/bucket/") && !auth {
                    ("200 OK", bytes.to_vec())
                } else {
                    ("400 Bad Request", Vec::new())
                };
                let _ = write!(
                    stream,
                    "HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(&body);
            }
        });
        (url, count)
    }

    fn some_image() -> &'static str {
        EXHIBITIONS[0].artworks[0].image
    }

    #[test]
    fn downloads_once_then_serves_from_the_cache() {
        let (url, count) = fake_nuage(b"JPEG");
        let cache = std::env::temp_dir().join(format!("arteon-test-{}", std::process::id()));
        let images = Images::new(url, Some("secret".into()), cache.clone());
        assert_eq!(images.get(some_image()), Some(Ok(b"JPEG".to_vec())));
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 2);
        assert_eq!(images.get(some_image()), Some(Ok(b"JPEG".to_vec())));
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 2);
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn only_catalogue_images_are_asked() {
        let (url, count) = fake_nuage(b"JPEG");
        let images = Images::new(url, Some("secret".into()), std::env::temp_dir());
        assert_eq!(images.get("../nuage.lock"), None);
        assert_eq!(images.get("van-gogh/nuage.lock"), None);
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[test]
    fn says_what_is_missing() {
        let cache = std::env::temp_dir().join(format!("arteon-test-vide-{}", std::process::id()));
        let images = Images::new("http://127.0.0.1:9".into(), None, cache);
        let err = images.get(some_image()).unwrap().unwrap_err();
        assert!(err.contains("NUAGE_TOKEN"), "{err}");
        let (url, _) = fake_nuage(b"JPEG");
        let cache = std::env::temp_dir().join(format!("arteon-test-jeton-{}", std::process::id()));
        let images = Images::new(url, Some("faux".into()), cache);
        let err = images.get(some_image()).unwrap().unwrap_err();
        assert!(err.contains("401"), "{err}");
    }
}
