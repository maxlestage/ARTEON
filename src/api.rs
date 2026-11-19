//! L'API des œuvres d'un artiste, en direct des musées : `GET /api/artiste?nom=Claude Monet`.
//!
//! Rien n'est stocké : à chaque demande, le serveur interroge les collections ouvertes des
//! musées décrits dans outils/monde/sources.json (Art Institute of Chicago, Cleveland Museum
//! of Art, Metropolitan Museum, SMK de Copenhague : un musée de plus = une entrée), garde
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
            "routes": { "/api/artiste?nom=…": "toutes les œuvres du domaine public d'un artiste, en direct des musées" },
            "musees": sources().iter().map(|s| s["nom"].clone()).collect::<Vec<_>>(),
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
    // « Hammershøi, Vilhelm » (SMK) : le prénom d'abord.
    if let Some((nom, prenom)) = trouve.split_once(", ")
        && !prenom.contains(',')
        && !prenom.trim().is_empty()
        && meme_artiste(demande, &format!("{} {}", prenom.trim(), nom.trim()))
    {
        return true;
    }
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
    lire_en(url, entetes, 20)
}

fn lire_en(url: &str, entetes: &[(&str, &str)], secondes: u64) -> Option<Value> {
    let mut requete = client().get(url).timeout(Duration::from_secs(secondes));
    for (k, v) in entetes {
        requete = requete.set(k, v);
    }
    // Un musée qui demande de ralentir (429, 503) est réessayé un peu plus tard, deux fois.
    let mut essai = 0;
    let texte = loop {
        match requete.clone().call() {
            Ok(r) => break r.into_string().ok()?,
            Err(ureq::Error::Status(code, _)) if (code == 429 || code == 503) && essai < 2 => {
                essai += 1;
                std::thread::sleep(Duration::from_secs(3 * essai));
            }
            Err(e) => {
                eprintln!("api : {url} : {e}");
                return None;
            }
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

// ─── Le moteur générique ────────────────────────────────────────────────────

/// Les musées, décrits sans code : outils/monde/sources.json (le même fichier que le site).
const SOURCES: &str = include_str!("../outils/monde/sources.json");

fn sources() -> &'static Vec<Value> {
    static TABLE: OnceLock<Vec<Value>> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str::<Value>(SOURCES)
            .ok()
            .and_then(|v| v["sources"].as_array().cloned())
            .unwrap_or_default()
    })
}

/// La valeur au bout d'un chemin (« production.0.creator »).
fn chemin<'a>(v: &'a Value, chemin: &str) -> &'a Value {
    chemin.split('.').fold(v, |v, cle| match cle.parse::<usize>() {
        Ok(i) => v.get(i).unwrap_or(&Value::Null),
        Err(_) => v.get(cle).unwrap_or(&Value::Null),
    })
}

/// Une valeur en texte (« 1606 », « Rembrandt », « true »), vide si absente.
fn en_texte(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

/// La première année d'une valeur (un nombre, « 1888 », « 1606-07-15T… ») ; 0 et 9999 ne
/// sont pas des années.
fn annee_de(v: &Value) -> Option<i64> {
    let a = v.as_i64().or_else(|| annees(&en_texte(v)).first().copied())?;
    (a != 0 && a < 9999).then_some(a)
}

/// Remplit un modèle (« https://…/{image}/full/400,/0/default.jpg ») avec les champs.
fn remplir(modele: &str, valeurs: &HashMap<&str, String>) -> String {
    let mut s = modele.to_owned();
    for (k, v) in valeurs {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

/// Les œuvres libres de l'artiste dans un musée, et si le musée n'a pas répondu.
fn interroger(source: &Value, nom: &str) -> (Vec<Value>, bool) {
    let cle_musee = en_texte(&source["cle"]);
    let entetes: Vec<(String, String)> = source["entetes"]
        .as_object()
        .map(|o| o.iter().map(|(k, v)| (k.clone(), en_texte(v))).collect())
        .unwrap_or_default();
    let entetes: Vec<(&str, &str)> = entetes.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let par_page = source["par_page"].as_u64().unwrap_or(100);
    let delai = source["delai"].as_u64().unwrap_or(20);
    let pages = source["pages"].as_u64().unwrap_or(1);

    // Les fiches : directement dans les pages de résultats, ou une par une (le Met).
    let mut fiches = Vec::new();
    let mut ids = Vec::new();
    let mut echec = false;
    for q in recherches(nom) {
        for page in 1..=pages {
            let url = source["recherche"]
                .as_str()
                .unwrap_or_default()
                .replace("{q}", &encoder(&q))
                .replace("{n}", &par_page.to_string())
                .replace("{page}", &page.to_string())
                .replace("{offset}", &((page - 1) * par_page).to_string())
                .replace("{debut}", &((page - 1) * par_page + 1).to_string());
            // Une source qui n'accepte qu'une demande à la fois (Wikidata limite sévèrement les
            // serveurs qui en envoient plusieurs) : les fils d'exécution passent l'un après l'autre.
            let reponse = if source["un_a_la_fois"].as_bool().unwrap_or(false) {
                static FILE: Mutex<()> = Mutex::new(());
                let _tour = FILE.lock();
                let r = lire_en(&url, &entetes, delai);
                std::thread::sleep(Duration::from_millis(1200));
                r
            } else {
                lire_en(&url, &entetes, delai)
            };
            let Some(r) = reponse else {
                echec |= page == 1;
                break;
            };
            if let Some(champ) = source["ids"].as_str() {
                ids.extend(chemin(&r, champ).as_array().cloned().unwrap_or_default());
                break;
            }
            let lot = source["liste"].as_str().map(|l| chemin(&r, l).clone()).unwrap_or(r);
            let lot = lot.as_array().cloned().unwrap_or_default();
            let fini = (lot.len() as u64) < par_page;
            fiches.extend(lot);
            if fini {
                break;
            }
        }
    }
    if let Some(detail) = source["detail"].as_str() {
        let mut vus = HashSet::new();
        ids.retain(|id| vus.insert(en_texte(id)));
        ids.truncate(source["max"].as_u64().unwrap_or(3000) as usize);
        let taille = ids.len().div_ceil(16).max(1);
        fiches = std::thread::scope(|s| {
            ids.chunks(taille)
                .map(|lot| {
                    let entetes = &entetes;
                    s.spawn(move || {
                        lot.iter()
                            .filter_map(|id| lire(&detail.replace("{id}", &en_texte(id)), entetes))
                            .collect::<Vec<_>>()
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .flat_map(|t| t.join().unwrap_or_default())
                .collect()
        });
    }

    let champs = source["champs"].as_object().cloned().unwrap_or_default();
    let exige: Vec<String> = source["exige"].as_array().map(|a| a.iter().map(en_texte).collect()).unwrap_or_default();
    let oeuvres = fiches
        .iter()
        .filter_map(|f| {
            let lire_champ = |nom: &str| champs.get(nom).and_then(Value::as_str).map(|c| chemin(f, c)).unwrap_or(&Value::Null);
            for c in &exige {
                let v = lire_champ(c);
                if v.is_null() || v == &Value::Bool(false) || en_texte(v).is_empty() {
                    return None;
                }
            }
            if let Some(egal) = source["egal"].as_object()
                && egal.iter().any(|(c, attendu)| en_texte(lire_champ(c)) != en_texte(attendu))
            {
                return None;
            }
            let mut artiste = en_texte(lire_champ("artiste"));
            if let Some(coupe) = source["artiste_jusqua"].as_str() {
                artiste = artiste.split(coupe).next().unwrap_or_default().to_owned();
            }
            // Les autres noms possibles (Europeana : les libellés de ses entités).
            let autres: Vec<String> = lire_champ("artistes")
                .as_array()
                .map(|l| l.iter().map(|v| if v.is_object() { en_texte(&v["def"]) } else { en_texte(v) }).collect())
                .unwrap_or_default();
            // Seulement quand le créateur est un identifiant (une adresse) : un photographe
            // nommé qui a photographié l'œuvre n'en est pas l'auteur.
            let identifiant = artiste.is_empty() || artiste.starts_with("http");
            if !meme_artiste(nom, &artiste) {
                match autres.iter().find(|a| identifiant && meme_artiste(nom, a)) {
                    Some(a) => artiste = a.clone(),
                    None => return None,
                }
            }
            let (mut ne, mut mort) = (annee_de(lire_champ("ne")), annee_de(lire_champ("mort")));
            if ne.is_none() && mort.is_none() {
                (ne, mort) = vie(&en_texte(lire_champ("vie")));
            }
            let date = en_texte(lire_champ("date"));
            if !libre(&date, annee_de(lire_champ("fin")), mort, ne) {
                return None;
            }
            let valeurs: HashMap<&str, String> = champs.keys().map(|k| (k.as_str(), en_texte(lire_champ(k)))).collect();
            let modele = |m: &str| remplir(source[m].as_str().unwrap_or_default(), &valeurs);
            Some(json!({
                "id": format!("{cle_musee}-{}", valeurs.get("id").cloned().unwrap_or_default()),
                "titre": sans_balises(valeurs.get("titre").map(String::as_str).unwrap_or_default()),
                "artiste": artiste, "date": date, "annee": annee_de(lire_champ("annee")),
                "technique": valeurs.get("technique").cloned().unwrap_or_default(),
                "musee": cle_musee, "vignette": modele("vignette"), "image": modele("grande"), "lien": modele("lien"),
                "fournisseur": valeurs.get("fournisseur").filter(|f| !f.is_empty() && !f.starts_with("http")),
            }))
        })
        .collect();
    (oeuvres, echec)
}

/// Toutes les œuvres libres de l'artiste dans tous les musées décrits, interrogés ensemble.
pub fn oeuvres_de(nom: &str) -> Value {
    let lots: Vec<(Vec<Value>, bool)> = std::thread::scope(|s| {
        sources()
            .iter()
            .map(|source| s.spawn(move || interroger(source, nom)))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|t| t.join().unwrap_or_default())
            .collect()
    });
    // Les musées qui n'ont pas répondu (occupés, limités) : la réponse est incomplète.
    let manquants: Vec<Value> = sources()
        .iter()
        .zip(&lots)
        .filter(|(_, (_, echec))| *echec)
        .map(|(source, _)| source["cle"].clone())
        .collect();
    let mut vues = HashSet::new();
    let mut oeuvres: Vec<Value> = lots
        .into_iter()
        .flat_map(|(l, _)| l)
        .filter(|o| vues.insert(o["id"].as_str().unwrap_or_default().to_owned()))
        .collect();
    let mut oeuvres = sans_doublons(std::mem::take(&mut oeuvres));
    let musees: serde_json::Map<String, Value> = sources()
        .iter()
        .map(|source| {
            let cle = en_texte(&source["cle"]);
            let n = oeuvres.iter().filter(|o| o["musee"] == cle.as_str()).count();
            (cle, json!(n))
        })
        .collect();
    oeuvres.sort_by_key(|o| o["annee"].as_i64().unwrap_or(i64::MAX));
    json!({ "nom": nom, "total": oeuvres.len(), "musees": musees, "manquants": manquants, "oeuvres": oeuvres })
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
        assert!(meme_artiste("Vilhelm Hammershøi", "Hammershøi, Vilhelm"));
    }

    #[test]
    fn un_catalogue_refait_ne_s_appauvrit_pas() {
        let ancien = json!({ "nom": "X", "manquants": ["met"], "oeuvres": [
            { "id": "wikidata-1", "musee": "wikidata", "annee": 1650 },
            { "id": "met-1", "musee": "met", "annee": 1640 },
            { "id": "chicago-1", "musee": "chicago", "annee": 1630 } ] });
        // Cette fois Wikidata et le Met n'ont pas répondu ; Chicago a une œuvre de plus.
        let nouveau = json!({ "nom": "X", "manquants": ["wikidata", "met"], "oeuvres": [
            { "id": "chicago-1", "musee": "chicago", "annee": 1630 },
            { "id": "chicago-2", "musee": "chicago", "annee": 1660 } ] });
        let r = fusionner(Some(&ancien), nouveau.clone());
        let ids: Vec<String> = r["oeuvres"].as_array().unwrap().iter().map(|o| en_texte(&o["id"])).collect();
        assert_eq!(ids, ["chicago-1", "met-1", "wikidata-1", "chicago-2"]);
        assert_eq!(r["total"], 4);
        // Wikidata avait répondu en entier la fois d'avant : il ne manque plus. Le Met, si.
        assert_eq!(r["manquants"], json!(["met"]));
        assert_eq!(r["musees"]["wikidata"], 1);
        // Rien à compléter : le nouveau catalogue reste tel quel.
        assert_eq!(fusionner(None, nouveau.clone()), nouveau);
        let complet = json!({ "nom": "X", "manquants": [], "oeuvres": [] });
        assert_eq!(fusionner(Some(&ancien), complet.clone()), complet);
    }

    #[test]
    fn une_oeuvre_donnee_deux_fois_n_est_gardee_qu_une_fois() {
        let o = |id: &str, musee: &str, titre: &str, annee: i64, collection: Option<&str>| {
            json!({ "id": id, "musee": musee, "titre": titre, "annee": annee, "fournisseur": collection })
        };
        let r = sans_doublons(vec![
            o("chicago-1", "chicago", "Apples and Grapes", 1880, None),
            // La même, par Wikidata : collection « Art Institute of Chicago ».
            o("wikidata-1", "wikidata", "Apples and grapes", 1880, Some("Art Institute of Chicago")),
            // Une autre œuvre du même titre, dans un autre musée : elle reste.
            o("wikidata-2", "wikidata", "Apples and Grapes", 1880, Some("musée d'Orsay")),
            // Même musée mais autre année : elle reste.
            o("wikidata-3", "wikidata", "Apples and Grapes", 1881, Some("Art Institute of Chicago")),
            // Le SMK s'appelle « SMK – Statens Museum for Kunst, Copenhague » dans les sources.
            o("smk-1", "smk", "Interiør", 1901, None),
            o("europeana-1", "europeana", "Interiør", 1901, Some("Statens Museum for Kunst")),
        ]);
        let ids: Vec<String> = r.iter().map(|o| en_texte(&o["id"])).collect();
        assert_eq!(ids, ["chicago-1", "wikidata-2", "wikidata-3", "smk-1"]);
    }

    #[test]
    fn les_titres_perdent_leurs_balises() {
        assert_eq!(sans_balises("Emblem of the God Shiva (<em>Linga</em>)"), "Emblem of the God Shiva (Linga)");
        assert_eq!(sans_balises("Tom &amp; Jerry&nbsp;: <i>étude</i>"), "Tom & Jerry : étude");
        assert_eq!(sans_balises("Hauteur < 3 m, largeur > 2 m"), "Hauteur < 3 m, largeur > 2 m");
        assert_eq!(sans_balises("Sans rien"), "Sans rien");
        let c = nettoyer(json!({ "total": 1, "musees": {}, "oeuvres": [{ "id": "a-1", "musee": "a", "titre": "Une <em>étude</em>", "annee": 1 }] }));
        assert_eq!(c["oeuvres"][0]["titre"], "Une étude");
    }

    #[test]
    fn un_musee_qui_manque_partout_est_en_panne() {
        let m = |l: &[&str]| l.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let gardes = [m(&["met"]), m(&["met"]), m(&["met", "wikidata"]), m(&[])];
        let pannes = en_panne(&gardes);
        assert!(pannes.contains("met"));
        assert!(!pannes.contains("wikidata"));
        assert!(en_panne(&[]).is_empty());
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

// ─── L'API statique ─────────────────────────────────────────────────────────

/// Un nom de fichier stable pour la clé d'un artiste (FNV-1a, en hexadécimal).
fn fichier_de(cle: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in cle.bytes() {
        h = (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3);
    }
    format!("{h:016x}")
}

/// L'API publiée en fichiers, pour un site statique (GitHub Pages) : le catalogue complet de
/// chaque artiste de `noms`, calculé par le même moteur que `/api/artiste`, dans
/// `dossier/<fichier>.json`, et `dossier/index.json` qui relie chaque nom (et chaque clé) à
/// son fichier. Les réponses de moins d'une semaine de `cache` sont reprises ; les autres
/// sont recalculées, `fils` à la fois, tant qu'il reste du temps (`minutes`).
/// Un titre sans balises HTML (« Emblem of Shiva (<em>Linga</em>) », chez certains musées) ni
/// entités (« &amp; »).
fn sans_balises(texte: &str) -> String {
    if !texte.contains(['<', '&']) {
        return texte.to_owned();
    }
    let mut net = String::with_capacity(texte.len());
    let mut reste = texte;
    while let Some(i) = reste.find('<') {
        net.push_str(&reste[..i]);
        let suite = &reste[i + 1..];
        let balise = suite.starts_with(|c: char| c.is_ascii_alphabetic() || c == '/');
        match suite.find('>').filter(|_| balise) {
            Some(fin) => reste = &suite[fin + 1..],
            None => {
                net.push('<');
                reste = suite;
            }
        }
    }
    net.push_str(reste);
    for (entite, signe) in [("&amp;", "&"), ("&quot;", "\""), ("&#39;", "'"), ("&apos;", "'"), ("&nbsp;", " "), ("&lt;", "<"), ("&gt;", ">")] {
        net = net.replace(entite, signe);
    }
    net.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Sans les doublons : une œuvre qu'un musée interrogé donne lui-même et qu'un agrégateur
/// (Wikidata, Europeana) donne aussi : même titre, même année, et la collection indiquée
/// par l'agrégateur est ce musée. La fiche du musée reste. Deux œuvres de même titre dans
/// deux musées différents (deux épreuves d'une gravure, deux « Autoportrait ») restent.
fn sans_doublons(oeuvres: Vec<Value>) -> Vec<Value> {
    let noms: Vec<(String, String)> =
        sources().iter().map(|s| (en_texte(&s["cle"]), plier(&en_texte(&s["nom"])))).collect();
    let cle_de = |o: &Value| {
        let titre = plier(&en_texte(&o["titre"]));
        o["annee"].as_i64().filter(|_| !titre.is_empty()).map(|annee| (titre, annee))
    };
    let chez: HashSet<(String, i64, String)> = oeuvres
        .iter()
        .filter_map(|o| cle_de(o).map(|(titre, annee)| (titre, annee, en_texte(&o["musee"]))))
        .collect();
    oeuvres
        .into_iter()
        .filter(|o| {
            let Some(collection) = o["fournisseur"].as_str().map(plier).filter(|c| c.len() >= 8) else { return true };
            let Some((titre, annee)) = cle_de(o) else { return true };
            let musee = en_texte(&o["musee"]);
            !noms.iter().any(|(cle, nom)| {
                *cle != musee && nom.contains(&collection) && chez.contains(&(titre.clone(), annee, cle.clone()))
            })
        })
        .collect()
}

/// Un catalogue gardé, tel qu'il se publie : sans les doublons (les catalogues calculés avant
/// cette règle en portent encore), le total et les comptes par musée refaits.
fn nettoyer(mut catalogue: Value) -> Value {
    let Some(mut oeuvres) = catalogue["oeuvres"].as_array().cloned() else { return catalogue };
    let avant = oeuvres.len();
    let mut change = false;
    for o in &mut oeuvres {
        if let Some(titre) = o["titre"].as_str().filter(|t| t.contains(['<', '&'])) {
            let net = sans_balises(titre);
            change |= net != titre;
            o["titre"] = json!(net);
        }
    }
    let oeuvres = sans_doublons(oeuvres);
    if change && oeuvres.len() == avant {
        catalogue["oeuvres"] = json!(oeuvres.clone());
    }
    if oeuvres.len() != avant {
        for source in sources() {
            let cle = en_texte(&source["cle"]);
            catalogue["musees"][cle.as_str()] = json!(oeuvres.iter().filter(|o| o["musee"] == cle.as_str()).count());
        }
        catalogue["total"] = json!(oeuvres.len());
        catalogue["oeuvres"] = json!(oeuvres);
    }
    catalogue
}

/// Un catalogue recalculé, complété par celui qu'on gardait : quand un musée n'a pas répondu
/// cette fois, ses œuvres de la fois d'avant restent (et il ne « manque » plus s'il avait
/// répondu en entier). Un catalogue ne s'appauvrit donc jamais d'une construction à l'autre.
fn fusionner(ancien: Option<&Value>, nouveau: Value) -> Value {
    let Some(ancien) = ancien else { return nouveau };
    let absents: Vec<String> = nouveau["manquants"].as_array().into_iter().flatten().map(en_texte).collect();
    let Some(anciennes) = ancien["oeuvres"].as_array().filter(|_| !absents.is_empty()) else { return nouveau };
    // Sans suivi des musées manquants (anciennes réponses), on ne sait pas s'il était complet.
    let deja_absents: Option<Vec<String>> = ancien["manquants"].as_array().map(|m| m.iter().map(en_texte).collect());
    let mut oeuvres: Vec<Value> = nouveau["oeuvres"].as_array().cloned().unwrap_or_default();
    let mut vues: HashSet<String> = oeuvres.iter().map(|o| en_texte(&o["id"])).collect();
    for o in anciennes {
        if absents.contains(&en_texte(&o["musee"])) && vues.insert(en_texte(&o["id"])) {
            oeuvres.push(o.clone());
        }
    }
    let mut oeuvres = sans_doublons(oeuvres);
    let manquants: Vec<&String> = absents
        .iter()
        .filter(|m| deja_absents.as_ref().is_none_or(|d| d.contains(m)))
        .collect();
    let musees: serde_json::Map<String, Value> = sources()
        .iter()
        .map(|source| {
            let cle = en_texte(&source["cle"]);
            let n = oeuvres.iter().filter(|o| o["musee"] == cle.as_str()).count();
            (cle, json!(n))
        })
        .collect();
    oeuvres.sort_by_key(|o| o["annee"].as_i64().unwrap_or(i64::MAX));
    json!({ "nom": nouveau["nom"], "total": oeuvres.len(), "musees": musees, "manquants": manquants, "oeuvres": oeuvres })
}

/// Les musées « en panne » pour cette machine : ceux qui manquent à plus de la moitié des
/// catalogues gardés (un musée qui refuse les machines de construction, par exemple). Les
/// attendre ne sert à rien ; le site les interroge lui-même, depuis le navigateur.
fn en_panne(manquants: &[Vec<String>]) -> HashSet<String> {
    let mut absences: HashMap<&str, usize> = HashMap::new();
    for m in manquants {
        for source in m {
            *absences.entry(source.as_str()).or_default() += 1;
        }
    }
    absences.into_iter().filter(|(_, n)| n * 2 > manquants.len()).map(|(s, _)| s.to_owned()).collect()
}

/// Un texte compressé en gzip.
fn gzip(texte: &str) -> Option<Vec<u8>> {
    use std::io::Write;
    let mut encodeur = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    encodeur.write_all(texte.as_bytes()).ok()?;
    encodeur.finish().ok()
}

pub fn publier(dossier: &std::path::Path, noms: &[String], minutes: u64, cache: Option<&std::path::Path>, fils: usize)
    -> std::io::Result<(usize, usize)>
{
    use std::sync::atomic::{AtomicUsize, Ordering};
    std::fs::create_dir_all(dossier)?;
    if let Some(c) = cache {
        std::fs::create_dir_all(c)?;
    }
    // Les noms, groupés par clé : un fichier par artiste.
    let mut groupes: Vec<(String, Vec<String>)> = Vec::new();
    let mut place: HashMap<String, usize> = HashMap::new();
    for nom in noms {
        let nom = nom.trim();
        if nom.chars().count() < 2 || nom == "Artiste inconnu" {
            continue;
        }
        let k = cle(nom);
        match place.get(&k) {
            Some(&i) => groupes[i].1.push(nom.to_owned()),
            None => {
                place.insert(k.clone(), groupes.len());
                groupes.push((k, vec![nom.to_owned()]));
            }
        }
    }
    let fin = Instant::now() + Duration::from_secs(minutes * 60);
    let suivant = AtomicUsize::new(0);
    let calcules = AtomicUsize::new(0);
    // Le site a une taille limite : au-delà de ce poids (150 Mo, ou ARTEON_API_MO), les
    // artistes suivants (les moins riches) ne sont pas publiés ; le site les demande alors
    // aux musées, en direct.
    let poids_max = std::env::var("ARTEON_API_MO").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(150) * 1_000_000;
    let poids = AtomicUsize::new(0);
    let faits: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
    let a_refaire: Mutex<Vec<usize>> = Mutex::new(Vec::new());
    // Chaque catalogue gardé porte l'empreinte des sources (outils/monde/sources.json) qui
    // l'ont calculé : quand elles changent (une requête corrigée, un musée de plus), il est
    // périmé et se reprend.
    let empreinte = fichier_de(SOURCES);
    let signer = |mut r: Value| {
        r["sources"] = json!(empreinte);
        r
    };
    // D'abord les reprises : les catalogues gardés où manque un musée qui d'ordinaire répond
    // (depuis plus d'une heure), et ceux d'avant un changement des sources. Dans l'ordre des
    // artistes, pendant la moitié du temps au plus ; ce que les musées donnent cette fois
    // s'ajoute à ce qu'on avait (`fusionner`).
    if let Some(c) = cache {
        let gardes: Vec<(usize, Vec<String>, Duration, bool)> = groupes
            .iter()
            .enumerate()
            .filter_map(|(i, (k, _))| {
                let g = c.join(format!("{}.json", fichier_de(k)));
                let age = std::fs::metadata(&g).ok()?.modified().ok()?.elapsed().ok()?;
                let v: Value = serde_json::from_str(&std::fs::read_to_string(&g).ok()?).ok()?;
                let m = v["manquants"].as_array()?.iter().map(en_texte).collect();
                Some((i, m, age, v["sources"] != empreinte.as_str()))
            })
            .collect();
        let pannes = en_panne(&gardes.iter().map(|(_, m, _, _)| m.clone()).collect::<Vec<_>>());
        let reprises: Vec<usize> = gardes
            .iter()
            .filter(|(_, m, age, perime)| {
                *age < Duration::from_secs(7 * 24 * 3600)
                    && (*perime
                        || (*age > Duration::from_secs(3600) && m.iter().any(|source| !pannes.contains(source))))
            })
            .map(|(i, _, _, _)| *i)
            .collect();
        let fin_reprises = Instant::now() + Duration::from_secs(minutes * 30);
        let suivant = AtomicUsize::new(0);
        std::thread::scope(|s| {
            for _ in 0..fils.max(1) {
                s.spawn(|| loop {
                    let n = suivant.fetch_add(1, Ordering::SeqCst);
                    let Some(&i) = reprises.get(n) else { break };
                    if Instant::now() >= fin_reprises {
                        break;
                    }
                    let (k, noms) = &groupes[i];
                    let g = c.join(format!("{}.json", fichier_de(k)));
                    let ancienne = std::fs::read_to_string(&g).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok());
                    let r = signer(fusionner(ancienne.as_ref(), oeuvres_de(&noms[0])));
                    let _ = std::fs::write(&g, r.to_string());
                    calcules.fetch_add(1, Ordering::SeqCst);
                });
            }
        });
    }
    std::thread::scope(|s| {
        for _ in 0..fils.max(1) {
            s.spawn(|| loop {
                let i = suivant.fetch_add(1, Ordering::SeqCst);
                let Some((k, noms)) = groupes.get(i) else { break };
                let nom_fichier = fichier_de(k);
                let gardee = cache.map(|c| c.join(format!("{nom_fichier}.json")));
                let age = gardee.as_ref().and_then(|g| std::fs::metadata(g).ok()).and_then(|m| m.modified().ok())
                    .and_then(|t| t.elapsed().ok());
                // Les musées qui manquaient à la réponse gardée (une réponse d'avant ce suivi
                // n'en dit rien : elle se refait).
                let ancienne = gardee.as_ref().and_then(|g| std::fs::read_to_string(g).ok())
                    .and_then(|t| serde_json::from_str::<Value>(&t).ok());
                let manquants = ancienne.as_ref().and_then(|v| v["manquants"].as_array().map(|m| m.len()));
                let frais = age.is_some_and(|a| a < Duration::from_secs(7 * 24 * 3600)) && manquants.is_some();
                let corps = if frais {
                    // Une réponse où un musée manquait sert en attendant ; elle se refait à la
                    // fin, s'il reste du temps (pas plus d'une fois par heure).
                    if manquants.is_some_and(|n| n > 0)
                        && age.is_some_and(|a| a > Duration::from_secs(3600))
                        && let Ok(mut r) = a_refaire.lock()
                    {
                        r.push(i);
                    }
                    ancienne.map(|v| nettoyer(v).to_string())
                } else if Instant::now() < fin {
                    let r = signer(fusionner(ancienne.as_ref(), oeuvres_de(&noms[0])));
                    let texte = r.to_string();
                    if let Some(g) = &gardee {
                        let _ = std::fs::write(g, &texte);
                    }
                    calcules.fetch_add(1, Ordering::SeqCst);
                    Some(texte)
                } else {
                    // Plus de temps : la réponse de la semaine passée, s'il y en a une.
                    ancienne.map(|v| nettoyer(v).to_string())
                };
                if let Some(corps) = corps
                    && let Some(compresse) = gzip(&corps)
                    && poids.fetch_add(compresse.len(), Ordering::SeqCst) < poids_max
                    && std::fs::write(dossier.join(format!("{nom_fichier}.json.gz")), compresse).is_ok()
                    && let Ok(mut f) = faits.lock()
                {
                    f.push((i, nom_fichier));
                }
            });
        }
    });
    // S'il reste du temps : les réponses où un musée manquait, dans l'ordre des artistes.
    let mut a_refaire = a_refaire.into_inner().unwrap_or_default();
    a_refaire.sort_unstable();
    let suivant = AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..fils.max(1) {
            s.spawn(|| loop {
                let n = suivant.fetch_add(1, Ordering::SeqCst);
                let Some(&i) = a_refaire.get(n) else { break };
                if Instant::now() >= fin {
                    break;
                }
                let (k, noms) = &groupes[i];
                let nom_fichier = fichier_de(k);
                let ancienne = cache
                    .and_then(|c| std::fs::read_to_string(c.join(format!("{nom_fichier}.json"))).ok())
                    .and_then(|t| serde_json::from_str::<Value>(&t).ok());
                let r = signer(fusionner(ancienne.as_ref(), oeuvres_de(&noms[0])));
                let texte = r.to_string();
                if let Some(c) = cache {
                    let _ = std::fs::write(c.join(format!("{nom_fichier}.json")), &texte);
                }
                let publie = dossier.join(format!("{nom_fichier}.json.gz"));
                if publie.exists()
                    && let Some(compresse) = gzip(&texte)
                {
                    let _ = std::fs::write(publie, compresse);
                }
                calcules.fetch_add(1, Ordering::SeqCst);
            });
        }
    });
    let faits = faits.into_inner().unwrap_or_default();
    let mut par_nom = serde_json::Map::new();
    let mut par_cle = serde_json::Map::new();
    for (i, nom_fichier) in &faits {
        let (k, noms) = &groupes[*i];
        par_cle.insert(k.clone(), json!(nom_fichier));
        for n in noms {
            par_nom.insert(n.clone(), json!(nom_fichier));
        }
    }
    // `gz` : chaque catalogue est dans `<fichier>.json.gz`, que le site et l'application
    // décompressent eux-mêmes (un hébergement statique ne le fait pas).
    let index = json!({ "noms": par_nom, "cles": par_cle, "total": faits.len(), "gz": true });
    std::fs::write(dossier.join("index.json"), index.to_string())?;
    Ok((faits.len(), calcules.into_inner()))
}
