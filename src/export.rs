//! `cargo run --release -- export dist` : tout le site en fichiers statiques, pour
//! GitHub Pages ou n'importe quel hébergeur statique.
//!
//! Chaque page est rendue par l'application elle-même, à travers le client de test en
//! mémoire de Paquets (`app.test()`) : le site exporté est identique au serveur.
//! `BASE_PATH` (par exemple `/ARTEON`) préfixe tous les liens absolus, car un site de
//! projet GitHub Pages est servi sous `https://<compte>.github.io/<dépôt>/`.

use std::io;
use std::path::{Path, PathBuf};

use paquets::App;

use crate::routes;

/// Écrit le site dans `out`, avec les liens derrière `base` (`""` ou `/ARTEON`).
/// Renvoie le nombre de fichiers écrits.
pub async fn export(mut app: App, out: &Path, base: &str) -> io::Result<usize> {
    let base = normalize_base(base);
    let out = std::path::absolute(out)?;
    if !Path::new("pkg").is_dir() {
        return Err(io::Error::other(
            "pkg/ est absent : compiler d'abord les îlots WebAssembly (voir le README), \
             sinon la galerie 3D ne s'afficherait pas",
        ));
    }
    // Des fichiers statiques n'ont pas de serveur à protéger, et l'export demande toutes
    // les pages d'un coup (la limite de débit l'arrêterait).
    app.security(None);
    let client = app.test().await;

    let mut written = 0;
    let mut write = |path: &str, bytes: &[u8]| -> io::Result<()> {
        let file = out.join(path.trim_start_matches('/'));
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file, bytes)?;
        written += 1;
        Ok(())
    };

    let pages = routes();
    for route in &pages {
        let route = route.as_str();
        let res = client.get(route).await;
        if res.status() != 200 {
            return Err(io::Error::other(format!(
                "GET {route} : {} au lieu de 200",
                res.status()
            )));
        }
        let html = res.text();
        // La présentation et la galerie de chaque exposition contiennent une scène 3D :
        // sans le bundle, elles resteraient vides.
        let is_scene = route.ends_with("/galerie")
            || arteon::artworks::EXHIBITIONS
                .iter()
                .any(|e| route == format!("/{}", e.slug));
        if is_scene && !html.contains("/pkg/") {
            return Err(io::Error::other(format!(
                "GET {route} : la page ne charge pas les îlots"
            )));
        }
        let file = if route == "/" {
            "index.html".to_owned()
        } else {
            format!("{route}/index.html")
        };
        write(&file, with_base(&html, &base, &pages).as_bytes())?;
    }

    // La page « introuvable », servie par GitHub Pages pour toute adresse inconnue.
    let res = client.get("/cette-page-n-existe-pas").await;
    if res.status() != 404 {
        return Err(io::Error::other(format!(
            "la page 404 répond {}",
            res.status()
        )));
    }
    write("404.html", with_base(&res.text(), &base, &pages).as_bytes())?;

    // Les fichiers statiques (`public/` : les images des tableaux, servies à `/…`), le
    // bundle du navigateur (`pkg/` : les îlots compilés en WebAssembly).
    // Et les données des expositions (`donnees/` : mosaïques, sculptures, vignettes).
    for (folder, prefix) in [
        ("public", ""),
        ("pkg", "pkg/"),
        (arteon::artworks::DATA_DIR, "donnees/"),
    ] {
        let root = std::path::absolute(folder)?;
        for file in files(&root) {
            let relative = file.strip_prefix(&root).map_err(io::Error::other)?;
            let url = relative.to_string_lossy().replace('\\', "/");
            // Les fichiers annexes de wasm-bindgen, inutiles pour lancer le bundle.
            if url.ends_with(".gitignore")
                || url.ends_with("package.json")
                || url.ends_with(".d.ts")
            {
                continue;
            }
            write(&format!("{prefix}{url}"), &std::fs::read(&file)?)?;
        }
    }
    // Les fichiers sont servis tels quels : pas de traitement Jekyll sur GitHub Pages.
    write(".nojekyll", b"")?;
    Ok(written)
}

/// `"ARTEON/"` → `"/ARTEON"`, `"/"` → `""`.
fn normalize_base(base: &str) -> String {
    let trimmed = base.trim().trim_matches('/');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("/{trimmed}")
    }
}

/// Place les URL absolues de la page (`"/…"`) derrière `base`. Les liens vers une page
/// (`/galerie`) prennent la forme d'un dossier (`/ARTEON/galerie/`), comme les fichiers
/// exportés : GitHub Pages n'a pas à les rediriger.
pub fn with_base(html: &str, base: &str, pages: &[String]) -> String {
    let mut out = String::with_capacity(html.len() + 256);
    let mut rest = html;
    while let Some(i) = rest.find("=\"/") {
        out.push_str(&rest[..i + 2]);
        rest = &rest[i + 2..]; // commence par '/'
        let end = rest.find('"').unwrap_or(rest.len());
        let url = &rest[..end];
        // `//hôte/…` est une URL externe : on n'y touche pas.
        if url.starts_with("//") {
            out.push_str(url);
        } else {
            out.push_str(&rewrite(url, base, pages));
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    // Le chargeur des îlots écrit aussi `import init from "/pkg/…"` et
    // `module_or_path:"/pkg/…"`.
    out.replace(":\"/pkg/", &format!(":\"{base}/pkg/"))
        .replace("from \"/pkg/", &format!("from \"{base}/pkg/"))
}

fn rewrite(url: &str, base: &str, pages: &[String]) -> String {
    let (path, suffix) = match url.find(['#', '?']) {
        Some(i) => (&url[..i], &url[i..]),
        None => (url, ""),
    };
    let is_page = path != "/" && pages.iter().any(|p| p == path);
    let path = if is_page {
        format!("{path}/")
    } else {
        path.to_owned()
    };
    format!("{base}{path}{suffix}")
}

/// Tous les fichiers sous `dir`, récursivement.
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            found.push(path);
        }
    }
    found.sort();
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_links_and_the_islands_loader() {
        let html = r##"<a href="/">a</a><a href="/van-gogh/galerie">b</a><a href="/van-gogh/oeuvres#x">c</a><img src="/tableaux/van-gogh/iris.jpg"/><a href="#top">d</a><a href="https://github.com/x">e</a><link rel="modulepreload" href="/pkg/a.js"/><script type="module">import init from "/pkg/a.js";init({module_or_path:"/pkg/a_bg.wasm"});</script>"##;
        let pages = vec![
            "/".to_owned(),
            "/van-gogh/galerie".to_owned(),
            "/van-gogh/oeuvres".to_owned(),
        ];
        let out = with_base(html, "/ARTEON", &pages);
        assert!(out.contains(r#"href="/ARTEON/""#), "{out}");
        assert!(out.contains(r#"href="/ARTEON/van-gogh/galerie/""#), "{out}");
        assert!(
            out.contains(r#"href="/ARTEON/van-gogh/oeuvres/#x""#),
            "{out}"
        );
        assert!(
            out.contains(r#"src="/ARTEON/tableaux/van-gogh/iris.jpg""#),
            "{out}"
        );
        assert!(out.contains(r##"href="#top""##), "{out}");
        assert!(out.contains(r#"href="https://github.com/x""#), "{out}");
        assert_eq!(out.matches("\"/ARTEON/pkg/").count(), 3, "{out}");
    }

    #[test]
    fn without_base_only_pages_become_folders() {
        let pages = vec!["/".to_owned(), "/van-gogh/galerie".to_owned()];
        let out = with_base(
            r#"<a href="/van-gogh/galerie">g</a><a href="/">h</a>"#,
            "",
            &pages,
        );
        assert_eq!(
            out,
            r#"<a href="/van-gogh/galerie/">g</a><a href="/">h</a>"#
        );
    }

    #[test]
    fn normalizes_the_base() {
        assert_eq!(normalize_base(""), "");
        assert_eq!(normalize_base("/"), "");
        assert_eq!(normalize_base("ARTEON/"), "/ARTEON");
        assert_eq!(normalize_base("/ARTEON"), "/ARTEON");
    }
}
