//! L'API des œuvres d'un artiste, en direct des musées : `GET /api/artiste?nom=Claude Monet`.
//!
//! Rien n'est stocké : à chaque demande, le serveur interroge les collections ouvertes de
//! l'Art Institute of Chicago, du Cleveland Museum of Art et du Metropolitan Museum, garde
//! les œuvres du domaine public (en Europe comme aux États-Unis : l'artiste est mort en 1955
//! ou avant, l'œuvre date d'avant 1931), et renvoie une liste commune, du plus ancien au
//! plus récent. Les réponses restent un jour en mémoire.
//!
//! ```text
//! GET /api/artiste?nom=Rembrandt
//! {"nom":"Rembrandt","total":182,"musees":{"chicago":64,"cleveland":71,"met":47},
//!  "oeuvres":[{"id":"chicago-14572","titre":"…","date":"1633","annee":1633,
//!              "technique":"…","musee":"chicago","vignette":"https://…","image":"https://…",
//!              "lien":"https://www.artic.edu/artworks/14572"}, …]}
//! ```

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use paquets::prelude::*;
use paquets::serde_json::{self, Value, json};

const AGENT: &str = "ARTEON/1.0 (musee virtuel ; https://github.com/maxlestage/ARTEON)";
const DERNIERE_ANNEE: i64 = 1930;
const DERNIERE_MORT: i64 = 1955;
/// Le Met n'a pas de recherche qui rende les fiches : chacune se demande à part.
const MET_MAX: usize = 80;
const GARDE: Duration = Duration::from_secs(24 * 3600);

/// Les variantes de noms connues (« titian » → « titien »), les mêmes que le site.
const ALIAS: &str = include_str!("../outils/monde/alias.json");

pub fn routes(app: &mut App) {
    app.get("/api/artiste", |req: Request| async move {
        let nom = req.query("nom").map(|n| n.trim().to_owned()).unwrap_or_default();
        if nom.chars().count() < 2 {
            return reponse(400, json!({ "erreur": "il faut ?nom=… (au moins deux lettres)" }).to_string());
        }
        let corps = match cache().lock().ok().and_then(|c| c.get(&cle(&nom)).cloned()) {
            Some((quand, corps)) if quand.elapsed() < GARDE => corps,
            _ => {
                let demande = nom.clone();
                let corps = tokio::task::spawn_blocking(move || oeuvres_de(&demande).to_string())
                    .await
                    .unwrap_or_else(|_| json!({ "erreur": "les musées n'ont pas répondu" }).to_string());
                if let Ok(mut c) = cache().lock() {
                    if c.len() > 2000 {
                        c.clear();
                    }
                    c.insert(cle(&nom), (Instant::now(), corps.clone()));
                }
                corps
            }
        };
        reponse(200, corps)
    });
    // Une page d'accueil de l'API : ce qu'elle sait faire.
    app.get("/api", |_req: Request| async move {
        reponse(200, json!({
            "api": "ARTEON",
            "routes": { "/api/artiste?nom=…": "toutes les œuvres du domaine public d'un artiste, en direct de l'Art Institute of Chicago, du Cleveland Museum of Art et du Metropolitan Museum" },
            "regle": "artiste mort en 1955 ou avant, œuvre d'avant 1931",
        }).to_string())
    });
}

fn reponse(statut: u16, corps: String) -> Response {
    Response::new()
        .status(StatusCode::from_u16(statut).unwrap_or(StatusCode::OK))
        .content_type("application/json; charset=utf-8")
        .header("access-control-allow-origin", "*")
        .header("cache-control", "public, max-age=3600")
        .send(corps)
}

fn cache() -> &'static Mutex<HashMap<String, (Instant, String)>> {
    static CACHE: OnceLock<Mutex<HashMap<String, (Instant, String)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

// ─── Les noms ───────────────────────────────────────────────────────────────

/// Sans accents, en minuscules, la ponctuation remplacée par des espaces.
fn plier(texte: &str) -> String {
    let mut s = String::with_capacity(texte.len());
    for c in texte.chars() {
        let r = match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' => "a",
            'ç' | 'Ç' => "c",
            'è' | 'é' | 'ê' | 'ë' | 'È' | 'É' | 'Ê' | 'Ë' => "e",
            'ì' | 'í' | 'î' | 'ï' | 'Ì' | 'Í' | 'Î' | 'Ï' => "i",
            'ñ' | 'Ñ' => "n",
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'ō' => "o",
            'ù' | 'ú' | 'û' | 'ü' | 'Ù' | 'Ú' | 'Û' | 'Ü' | 'ū' => "u",
            'ý' | 'ÿ' | 'Ý' => "y",
            'œ' | 'Œ' => "oe",
            'æ' | 'Æ' => "ae",
            'ß' => "ss",
            '-' | '‐' | '–' | '\'' | '’' | '.' | ',' => " ",
            _ => {
                s.extend(c.to_lowercase());
                continue;
            }
        };
        s.push_str(r);
    }
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn alias() -> &'static HashMap<String, String> {
    static TABLE: OnceLock<HashMap<String, String>> = OnceLock::new();
    TABLE.get_or_init(|| serde_json::from_str(ALIAS).unwrap_or_default())
}

/// La clé d'un artiste, comme sur le site (« Rembrandt (Rembrandt van Rijn) » → « rembrandt »).
pub fn cle(nom: &str) -> String {
    let mut n = nom.trim().to_owned();
    if n.ends_with(')')
        && let Some(i) = n.find('(')
        && i > 0
    {
        let tete = n[..i].trim().to_owned();
        let dedans = n[i + 1..n.len() - 1].to_owned();
        let premier = plier(&tete).split(' ').next().unwrap_or_default().to_owned();
        n = if plier(&dedans).contains(&premier) { dedans } else { tete };
    }
    let k = plier(&n);
    alias().get(&k).cloned().unwrap_or(k)
}

/// Les noms sous lesquels chercher : le nom demandé et ses variantes connues (« Titien »
/// se cherche aussi comme « Titian »).
fn recherches(nom: &str) -> Vec<String> {
    let k = cle(nom);
    let mut noms = vec![nom.to_owned()];
    for (variante, canon) in alias() {
        if *canon == k && !noms.iter().any(|n| plier(n) == *variante) {
            noms.push(variante.clone());
        }
    }
    noms
}

/// Le nom d'un musée correspond-il à l'artiste demandé ? Même clé, ou tous les mots
/// du nom demandé dans le nom du musée (« Monet » dans « Claude Monet »).
fn meme_artiste(demande: &str, trouve: &str) -> bool {
    let (k, t) = (cle(demande), cle(trouve));
    if k == t {
        return true;
    }
    if plier(trouve).contains("after ") || plier(trouve).starts_with("follower") || plier(trouve).contains("imitator") {
        return false;
    }
    // Tous les mots du nom demandé, et le même nom de famille (le dernier mot) :
    // « Monet » est « Claude Monet », mais « Rembrandt » n'est pas « Rembrandt Peale ».
    let mots: Vec<&str> = k.split(' ').filter(|m| m.len() > 2).collect();
    let dans: HashSet<&str> = t.split(' ').collect();
    !mots.is_empty() && mots.iter().all(|m| dans.contains(m)) && k.split(' ').last() == t.split(' ').last()
}

// ─── Les dates ──────────────────────────────────────────────────────────────

fn annees(texte: &str) -> Vec<i64> {
    let octets = texte.as_bytes();
    let mut vues = Vec::new();
    let mut i = 0;
    while i < octets.len() {
        if octets[i].is_ascii_digit() {
            let debut = i;
            while i < octets.len() && octets[i].is_ascii_digit() {
                i += 1;
            }
            if i - debut == 4
                && let Ok(a) = texte[debut..i].parse::<i64>()
                && (1000..=2100).contains(&a)
            {
                vues.push(a);
            }
        } else {
            i += 1;
        }
    }
    vues
}

/// Libre en Europe comme aux États-Unis : œuvre d'avant 1931, artiste mort en 1955 ou avant
/// (quand on sait quand il est mort ; né après 1890 sans date de mort : refusé).
fn libre(date: &str, fin: Option<i64>, mort: Option<i64>, ne: Option<i64>) -> bool {
    if annees(date).into_iter().chain(fin).any(|a| a > DERNIERE_ANNEE) {
        return false;
    }
    match (mort, ne) {
        (Some(m), _) => m <= DERNIERE_MORT,
        (None, Some(n)) => n <= 1890,
        _ => true,
    }
}

/// Les dates de vie dans « Dutch, 1606–1669 » : (naissance, mort).
fn vie(texte: &str) -> (Option<i64>, Option<i64>) {
    let a = annees(texte);
    match a.as_slice() {
        [] => (None, None),
        [n] if texte.contains("born") || texte.ends_with('–') || texte.ends_with('-') => (Some(*n), None),
        [n] => (None, Some(*n)),
        [n, m, ..] => (Some(*n), Some(*m)),
    }
}

// ─── Les musées ─────────────────────────────────────────────────────────────

/// Le client HTTP : un seul pour tout le serveur, qui suit HTTPS_PROXY s'il y en a un, et
/// fait confiance aux certificats du système.
fn client() -> &'static ureq::Agent {
    static CLIENT: OnceLock<ureq::Agent> = OnceLock::new();
    CLIENT.get_or_init(|| {
        ureq::AgentBuilder::new()
            .try_proxy_from_env(true)
            .timeout(Duration::from_secs(20))
            .user_agent(AGENT)
            .build()
    })
}

fn lire(url: &str, entetes: &[(&str, &str)]) -> Option<Value> {
    let mut requete = client().get(url);
    for (k, v) in entetes {
        requete = requete.set(k, v);
    }
    let texte = match requete.call() {
        Ok(r) => r.into_string().ok()?,
        Err(e) => {
            eprintln!("api : {url} : {e}");
            return None;
        }
    };
    serde_json::from_str(&texte).ok()
}

fn encoder(texte: &str) -> String {
    texte
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn texte(v: &Value, cle: &str) -> String {
    v.get(cle).and_then(Value::as_str).unwrap_or_default().to_owned()
}

fn chicago(nom: &str) -> Vec<Value> {
    let champs = "id,title,artist_title,artist_display,date_display,date_end,medium_display,image_id";
    let mut oeuvres = Vec::new();
    for q in recherches(nom) {
        for page in 1..=5 {
            let url = format!(
                "https://api.artic.edu/api/v1/artworks/search?q={}&query[term][is_public_domain]=true&fields={champs}&limit=100&page={page}",
                encoder(&q)
            );
            let Some(r) = lire(&url, &[("AIC-User-Agent", AGENT)]) else { break };
            let lot = r["data"].as_array().cloned().unwrap_or_default();
            for o in &lot {
                let artiste = texte(o, "artist_title");
                let image = texte(o, "image_id");
                if image.is_empty() || !meme_artiste(nom, &artiste) {
                    continue;
                }
                let (ne, mort) = vie(&texte(o, "artist_display"));
                let date = texte(o, "date_display");
                if !libre(&date, o["date_end"].as_i64(), mort, ne) {
                    continue;
                }
                let id = o["id"].as_i64().unwrap_or_default();
                oeuvres.push(json!({
                    "id": format!("chicago-{id}"), "titre": texte(o, "title"), "artiste": artiste,
                    "date": date, "annee": annees(&texte(o, "date_display")).first(),
                    "technique": texte(o, "medium_display"), "musee": "chicago",
                    "vignette": format!("https://www.artic.edu/iiif/2/{image}/full/400,/0/default.jpg"),
                    "image": format!("https://www.artic.edu/iiif/2/{image}/full/843,/0/default.jpg"),
                    "lien": format!("https://www.artic.edu/artworks/{id}"),
                }));
            }
            if lot.len() < 100 {
                break;
            }
        }
    }
    oeuvres
}

fn cleveland(nom: &str) -> Vec<Value> {
    let mut oeuvres = Vec::new();
    for q in recherches(nom) {
        let url = format!(
            "https://openaccess-api.clevelandart.org/api/artworks/?artists={}&cc0=1&has_image=1&limit=1000",
            encoder(&q)
        );
        let Some(r) = lire(&url, &[]) else { continue };
        for o in r["data"].as_array().cloned().unwrap_or_default() {
            let createur = o["creators"].get(0).cloned().unwrap_or(Value::Null);
            let description = texte(&createur, "description");
            let artiste = description.split(" (").next().unwrap_or_default().to_owned();
            if !meme_artiste(nom, &artiste) {
                continue;
            }
            let annee = |c: &str| texte(&createur, c).parse::<i64>().ok().or(createur[c].as_i64());
            let date = texte(&o, "creation_date");
            if !libre(&date, o["creation_date_latest"].as_i64(), annee("death_year"), annee("birth_year")) {
                continue;
            }
            let web = texte(&o["images"]["web"], "url");
            if web.is_empty() {
                continue;
            }
            oeuvres.push(json!({
                "id": format!("cleveland-{}", o["id"]), "titre": texte(&o, "title"), "artiste": artiste,
                "date": date, "annee": o["creation_date_earliest"].as_i64(),
                "technique": texte(&o, "technique"), "musee": "cleveland",
                "vignette": web, "image": web, "lien": texte(&o, "url"),
            }));
        }
    }
    oeuvres
}

fn met(nom: &str) -> Vec<Value> {
    let mut ids = Vec::new();
    for q in recherches(nom) {
        let url = format!(
            "https://collectionapi.metmuseum.org/public/collection/v1.1/search?artistOrCulture=true&hasImages=true&q={}",
            encoder(&q)
        );
        if let Some(r) = lire(&url, &[]) {
            ids.extend(r["objectIDs"].as_array().cloned().unwrap_or_default().into_iter().filter_map(|v| v.as_i64()));
        }
    }
    // Dans l'ordre de pertinence du Met, sans doublon.
    let mut vus = HashSet::new();
    ids.retain(|id| vus.insert(*id));
    ids.truncate(MET_MAX);
    // Les fiches, huit à la fois.
    let fiches: Vec<Value> = std::thread::scope(|s| {
        ids.chunks(MET_MAX.div_ceil(8).max(1))
            .map(|lot| {
                s.spawn(move || {
                    lot.iter()
                        .filter_map(|id| lire(&format!("https://collectionapi.metmuseum.org/public/collection/v1/objects/{id}"), &[]))
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|t| t.join().unwrap_or_default())
            .collect()
    });
    fiches
        .into_iter()
        .filter_map(|o| {
            let artiste = texte(&o, "artistDisplayName");
            let petite = texte(&o, "primaryImageSmall");
            let public = o["isPublicDomain"].as_bool().unwrap_or(false);
            let an = |c: &str| texte(&o, c).parse::<i64>().ok().filter(|a| *a != 0 && *a < 9999);
            let date = texte(&o, "objectDate");
            if !public || petite.is_empty() || !meme_artiste(nom, &artiste)
                || !libre(&date, o["objectEndDate"].as_i64(), an("artistEndDate"), an("artistBeginDate"))
            {
                return None;
            }
            Some(json!({
                "id": format!("met-{}", o["objectID"]), "titre": texte(&o, "title"), "artiste": artiste,
                "date": date, "annee": o["objectBeginDate"].as_i64(),
                "technique": texte(&o, "medium"), "musee": "met",
                "vignette": petite, "image": texte(&o, "primaryImage"), "lien": texte(&o, "objectURL"),
            }))
        })
        .collect()
}

/// Toutes les œuvres libres de l'artiste dans les trois musées, interrogés ensemble.
pub fn oeuvres_de(nom: &str) -> Value {
    let (a, b, c) = std::thread::scope(|s| {
        let a = s.spawn(|| chicago(nom));
        let b = s.spawn(|| cleveland(nom));
        let c = s.spawn(|| met(nom));
        (a.join().unwrap_or_default(), b.join().unwrap_or_default(), c.join().unwrap_or_default())
    });
    let mut vues = HashSet::new();
    let mut oeuvres: Vec<Value> = a
        .into_iter()
        .chain(b)
        .chain(c)
        .filter(|o| vues.insert(o["id"].as_str().unwrap_or_default().to_owned()))
        .collect();
    let compte = |m: &str| oeuvres.iter().filter(|o| o["musee"] == m).count();
    let musees = json!({ "chicago": compte("chicago"), "cleveland": compte("cleveland"), "met": compte("met") });
    oeuvres.sort_by_key(|o| o["annee"].as_i64().unwrap_or(i64::MAX));
    json!({ "nom": nom, "total": oeuvres.len(), "musees": musees, "oeuvres": oeuvres })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_noms_se_reconnaissent() {
        assert_eq!(cle("Rembrandt (Rembrandt van Rijn)"), "rembrandt");
        assert_eq!(cle("Rembrandt van Rijn"), "rembrandt");
        assert!(meme_artiste("Claude Monet", "Claude Monet"));
        assert!(meme_artiste("Monet", "Claude Monet"));
        assert!(meme_artiste("Titien", "Titian"));
        assert!(!meme_artiste("Claude Monet", "Follower of Claude Monet"));
        assert!(!meme_artiste("Rembrandt", "Rembrandt Peale"));
    }

    #[test]
    fn le_domaine_public_se_verifie() {
        assert!(libre("1888", Some(1888), Some(1890), Some(1853)));
        assert!(!libre("1937", Some(1937), Some(1940), None));
        assert!(!libre("1907", Some(1907), Some(1973), Some(1881)));
        assert!(!libre("1920", None, None, Some(1902)));
        assert_eq!(vie("Dutch, 1606–1669"), (Some(1606), Some(1669)));
    }
}
