// Tout passe par l'API : l'application ne garde rien sur l'appareil.
//
// - Le musée (catalogue, images, galeries 3D) et les collections du monde viennent du site
//   ARTEON (des fichiers JSON et des images, servis par GitHub Pages).
// - Toutes les œuvres d'un artiste viennent de l'API Paquets (`/api/artiste?nom=…`, le
//   serveur ARTEON, src/api.rs), en direct des musées ; si le serveur ne répond pas,
//   l'application interroge les musées elle-même, avec le même moteur générique et la même
//   description des musées (outils/monde/sources.json, lue sur le site).
//
// Une seule session réseau, éphémère : ni cache sur le disque, ni cookies gardés. Rien que
// Foundation : ce fichier se teste aussi sous Linux.

import Foundation
#if canImport(FoundationNetworking)
import FoundationNetworking
#endif

/// Du JSON lu par JSONSerialization, passé d'une tâche à l'autre (il n'est jamais modifié).
struct Json: @unchecked Sendable {
    let value: Any
}

enum Api {
    /// Le serveur Paquets d'ARTEON (Heroku).
    static let server = URL(string: "https://arteon-1ed4594ee106.herokuapp.com/")!

    /// La session de toute l'application : en mémoire seulement.
    static let session: URLSession = {
        let configuration = URLSessionConfiguration.ephemeral
        // Pas de cache d'URLSession : la mémoire de la visite (SessionStore) garde déjà chaque
        // réponse, et 64 Mo de mémoire vive faisaient double emploi.
        configuration.urlCache = nil
        configuration.requestCachePolicy = .reloadIgnoringLocalCacheData
        configuration.timeoutIntervalForRequest = 120
        configuration.httpAdditionalHeaders = ["User-Agent": "ARTEON/3.0 (application iOS)"]
        return URLSession(configuration: configuration)
    }()

    /// Les octets d'une adresse (une erreur si le serveur ne répond pas 200) : de la mémoire
    /// de la visite (SQLite, effacée en quittant) si l'adresse a déjà été demandée.
    static func data(_ url: URL, headers: [String: String] = [:], timeout: TimeInterval = 20) async throws -> Data {
        #if canImport(SQLite3)
        if let kept = await SessionStore.shared.get(url.absoluteString) { return kept }
        #endif
        let data = try await network(url, headers: headers, timeout: timeout)
        #if canImport(SQLite3)
        // Gardée pour la suite de la visite, sans faire attendre celui qui la demande (la
        // compression d'un gros fichier prend du temps).
        let key = url.absoluteString
        Task.detached(priority: .utility) {
            let (stored, packed) = SessionStore.pack(data)
            await SessionStore.shared.put(key, stored: stored, packed: packed)
        }
        #endif
        return data
    }

    private static func network(_ url: URL, headers: [String: String], timeout: TimeInterval) async throws -> Data {
        var request = URLRequest(url: url)
        request.timeoutInterval = timeout
        for (name, value) in headers { request.setValue(value, forHTTPHeaderField: name) }
        if url.host?.hasSuffix("artic.edu") == true {
            request.setValue("ARTEON/3.0 (application iOS)", forHTTPHeaderField: "AIC-User-Agent")
        }
        let (data, response) = try await session.data(for: request)
        if let http = response as? HTTPURLResponse, http.statusCode != 200 {
            throw URLError(.badServerResponse)
        }
        return data
    }

    /// Un fichier du site (`catalogue.json`, `monde/donnees/liste.json`).
    static func site(_ path: String) async throws -> Data {
        try await data(Site.url(path))
    }
}

// MARK: - L'annuaire de tous les artistes

/// Un artiste de l'annuaire (`monde/annuaire.json`, outils/annuaire.py) : tous les artistes du
/// domaine public dont un musée montre une œuvre. Son catalogue vient de l'API, en direct.
struct DirectoryArtist: Sendable, Equatable {
    let key: String
    let name: String
    let life: String
    let works: Int
    /// Le nom sans accents ni majuscules, pour le tri et la recherche.
    let sort: String

    /// L'annuaire tel que le site le publie : `[[nom, naissance, mort, œuvres], …]`.
    static func parse(_ data: Data, language: AppLanguage = .current) -> [DirectoryArtist] {
        guard let rows = try? JSONSerialization.jsonObject(with: data) as? [[Any]] else { return [] }
        return rows.compactMap { row in
            guard row.count >= 4, let name = row[0] as? String, !name.isEmpty else { return nil }
            let born = row[1] as? Int, died = row[2] as? Int
            let life: String
            switch (born, died) {
            case let (b?, d?): life = "\(b)–\(d)"
            case let (nil, d?): life = language.pick("mort en \(d)", "died \(d)", "murió en \(d)")
            case let (b?, nil): life = language.pick("né en \(b)", "born \(b)", "nació en \(b)")
            default: life = ""
            }
            return DirectoryArtist(key: ArtistKey.of(name), name: name, life: life, works: row[3] as? Int ?? 0, sort: folded(name))
        }
    }
}

// MARK: - Les collections du monde, lues sur le site

extension WorldCollections {
    /// Les adresses des musées des collections.
    static let museumSites = [
        "chicago": "https://www.artic.edu", "cleveland": "https://www.clevelandart.org",
        "met": "https://www.metmuseum.org", "washington": "https://www.nga.gov",
        "smithsonian": "https://www.si.edu",
    ]

    /// Les collections telles que le site les publie : `monde/donnees/index.json` (les
    /// musées, les identifiants) et `liste.json` (une ligne par œuvre : titre, artiste,
    /// date, année, catégorie, musée, proportions, et l'adresse de la vignette quand elle
    /// est chez le musée). Le détail de chaque œuvre se lit à la demande (`CollectionDetails`).
    static func fromSite(index: Data, list: Data) throws -> WorldCollections {
        guard let ix = try JSONSerialization.jsonObject(with: index) as? [String: Any],
              let rows = try JSONSerialization.jsonObject(with: list) as? [[Any]],
              let ids = ix["ids"] as? [String],
              let museumsInfo = ix["musees"] as? [[String: Any]]
        else { throw CatalogueError.unreadable }
        let museums = museumsInfo.map { m in
            let key = m["cle"] as? String ?? ""
            return CollectionMuseum(
                key: key, name: m["nom"] as? String ?? key, city: m["ville"] as? String ?? "",
                site: museumSites[key] ?? "", color: m["couleur"] as? String ?? "#1d3f8f",
                licence: m["licence"] as? String ?? "", count: m["oeuvres"] as? Int ?? 0
            )
        }
        var works: [CollectionWork] = []
        works.reserveCapacity(rows.count)
        for (i, row) in rows.enumerated() where i < ids.count && row.count >= 6 {
            let museumIndex = (row[5] as? Int) ?? 0
            let remote = row.count > 7 ? row[7] as? String : nil
            works.append(CollectionWork(
                id: ids[i], title: row[0] as? String ?? L("Sans titre", "Untitled", "Sin título"), artist: row[1] as? String ?? "",
                date: (row[2] as? String).flatMap { $0.isEmpty ? nil : $0 }, year: row[3] as? Int,
                category: row[4] as? String,
                ratio: (row[6] as? Double) ?? (row[6] as? Int).map(Double.init),
                museum: museums.indices.contains(museumIndex) ? museums[museumIndex].key : "",
                thumbURL: remote ?? Site.url("monde/v/\(ids[i]).webp").absoluteString,
                position: i
            ))
        }
        return WorldCollections(version: 1, date: ix["date"] as? String ?? "", museums: museums, works: works,
                                perFile: ix["parFichier"] as? Int ?? 1000)
    }
}

/// Le détail d'une œuvre des collections (vie de l'artiste, technique, mesures, origine,
/// texte du musée, image en grand, page) : lu sur le site à la demande, par fichiers de
/// mille œuvres, et gardé en mémoire le temps de la visite.
actor CollectionDetails {
    static let shared = CollectionDetails()

    private var parts: [Int: Task<Json, Never>] = [:]

    func complete(_ work: CollectionWork, perFile: Int) async -> CollectionWork {
        guard let position = work.position, perFile > 0 else { return work }
        let part = position / perFile
        let task = parts[part] ?? Task {
            let name = String(format: "monde/donnees/fiches-%03d.json", part)
            guard let data = try? await Api.site(name),
                  let list = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]]
            else { return Json(value: [[String: Any]]()) }
            return Json(value: list)
        }
        parts[part] = task
        let list = (await task.value).value as? [[String: Any]] ?? []
        if list.isEmpty { parts[part] = nil }
        guard list.indices.contains(position % perFile) else { return work }
        let f = list[position % perFile]
        var w = work
        w.life = f["v"] as? String ?? w.life
        w.medium = f["m"] as? String ?? w.medium
        w.dimensions = f["s"] as? String ?? w.dimensions
        w.origin = f["o"] as? String ?? w.origin
        w.text = f["x"] as? String ?? w.text
        w.imageURL = f["u"] as? String ?? w.imageURL
        w.page = f["w"] as? String ?? w.page
        w.gallery = f["g"] as? String ?? w.gallery
        return w
    }
}

// MARK: - Les fichiers du site lus une fois par visite

/// Deux fichiers du site que chaque page d'artiste relisait : l'index de l'API statique
/// (`api/index.json`, plus de 4 Mo, des dizaines de milliers de noms) et la description des
/// musées (`monde/sources.json`). Ils sont demandés une fois et décodés une fois ; deux
/// demandes en même temps (le catalogue et les noms des musées d'une même page) n'en font
/// qu'une. Un échec n'est pas gardé : la demande suivante réessaie.
actor SiteMemo {
    static let shared = SiteMemo()

    private var index: Task<LiveWorks.StaticIndex?, Never>?
    private var sourcesTask: Task<Json?, Never>?

    func staticIndex() async -> LiveWorks.StaticIndex? {
        let task = index ?? Task {
            guard let data = try? await Api.site("api/index.json"),
                  let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
            else { return nil }
            return LiveWorks.StaticIndex(
                byName: json["noms"] as? [String: String] ?? [:],
                byKey: json["cles"] as? [String: String] ?? [:],
                gz: json["gz"] as? Bool ?? false
            )
        }
        index = task
        let value = await task.value
        if value == nil, index == task { index = nil }
        return value
    }

    /// La liste `sources` de `monde/sources.json`.
    func sources() async -> Json? {
        let task = sourcesTask ?? Task {
            guard let data = try? await Api.site("monde/sources.json"),
                  let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let list = json["sources"] as? [[String: Any]]
            else { return nil }
            return Json(value: list)
        }
        sourcesTask = task
        let value = await task.value
        if value == nil, sourcesTask == task { sourcesTask = nil }
        return value
    }
}

// MARK: - Toutes les œuvres d'un artiste, en direct des musées

/// Une œuvre trouvée en direct dans un musée.
struct LiveWork: Decodable, Hashable, Sendable, Identifiable {
    var id: String
    var title: String
    var date: String
    var year: Int?
    var medium: String
    var museum: String
    var thumbnail: String
    var image: String
    var link: String
    /// Le musée qui conserve l'œuvre, quand elle vient d'un agrégateur (Europeana).
    var provider: String? = nil
    /// Son auteur (utile pour un style ou un genre, qui réunit des artistes).
    var artist: String? = nil

    enum CodingKeys: String, CodingKey {
        case id, title = "titre", date, year = "annee", medium = "technique", museum = "musee"
        case thumbnail = "vignette", image, link = "lien", provider = "fournisseur", artist = "artiste"
    }
}

// MARK: - Les styles, les genres, les techniques et les sujets

/// Un sujet (un style, un genre, une technique, un sujet représenté) : une entrée de
/// `monde/sujets.json` (outils/monde/sujets.json, le même fichier que le site).
struct Subject: Hashable, Sendable, Identifiable {
    var key: String
    var fr: String
    var en: String
    var es: String
    /// Ses éléments de Wikidata (« wd:Q40415 wd:Q… »).
    var ids: String = ""
    /// Le nombre de ses œuvres libres dans Wikidata.
    var count: Int = 0
    /// Une de ses œuvres (pour les premiers de chaque famille seulement).
    var image: String? = nil

    var id: String { key }
    var name: String { L(fr, en, es) }
}

/// Une famille de sujets : « Styles et mouvements », « Genres », « Techniques »…
struct SubjectFamily: Hashable, Sendable, Identifiable {
    var key: String
    var fr: String
    var en: String
    var es: String
    var subjects: [Subject]
    /// Comment une œuvre se rattache à un sujet de la famille (SPARQL, {id} : ses éléments).
    var pattern: String = ""

    var id: String { key }
    var name: String { L(fr, en, es) }

    /// Toutes les catégories confondues : chacune une fois (un sujet peut être dans deux
    /// familles, il garde la première), des plus riches aux moins riches ; à égalité, l'ordre
    /// des familles et des sujets choisis à la main.
    static func merged(_ families: [SubjectFamily], _ published: [String: PublishedSubject] = [:]) -> [(subject: Subject, family: String)] {
        var seen = Set<String>()
        let all = families.flatMap { family in family.subjects.map { (subject: $0, family: family.name) } }
            .filter { seen.insert($0.subject.key).inserted }
        let total = { (s: Subject) in published[s.key]?.total ?? s.count }
        return all.enumerated()
            .sorted { (total($0.element.subject), -$0.offset) > (total($1.element.subject), -$1.offset) }
            .map(\.element)
    }

    /// Les familles de `sujets.json`.
    static func parse(_ data: Data) -> [SubjectFamily] {
        guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let families = json["familles"] as? [[String: Any]] else { return [] }
        return families.compactMap { family in
            guard let key = family["cle"] as? String else { return nil }
            let subjects = (family["sujets"] as? [[String: Any]] ?? []).compactMap { subject -> Subject? in
                guard let key = subject["cle"] as? String, let fr = subject["fr"] as? String else { return nil }
                return Subject(key: key, fr: fr, en: subject["en"] as? String ?? fr, es: subject["es"] as? String ?? fr,
                               ids: subject["id"] as? String ?? "", count: subject["n"] as? Int ?? 0,
                               image: subject["image"] as? String)
            }
            let fr = family["fr"] as? String ?? key
            return SubjectFamily(key: key, fr: fr, en: family["en"] as? String ?? fr, es: family["es"] as? String ?? fr,
                                 subjects: subjects, pattern: family["wikidata"] as? String ?? "")
        }
    }

    /// La requête qui donne les œuvres d'un sujet (QLever, puis le service officiel).
    static func queries(_ data: Data) -> [String] {
        guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let wikidata = (json["requetes"] as? [String: Any])?["wikidata"] as? [String: Any] else { return [] }
        return [wikidata["sujet"], wikidata["sujet_repli"]].compactMap { $0 as? String }
    }
}

/// Ce que le site publie d'un sujet dans sa liste (`api/sujets/index.json`) : le nombre de ses
/// œuvres et l'image de la plus connue.
struct PublishedSubject: Sendable, Equatable {
    var total: Int
    var image: String

    static func parse(_ data: Data) -> [String: PublishedSubject] {
        guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let subjects = json["sujets"] as? [String: [String: Any]] else { return [:] }
        return subjects.compactMapValues { subject in
            (subject["total"] as? Int).map { PublishedSubject(total: $0, image: subject["image"] as? String ?? "") }
        }
    }
}

/// Le catalogue d'un sujet (`api/sujets/<clé>.json.gz`) : ses œuvres, les plus connues d'abord.
struct SubjectCatalogue: Decodable, Sendable {
    var total: Int
    var works: [LiveWork]

    enum CodingKeys: String, CodingKey { case total, works = "oeuvres" }
}

/// Les sujets, lus sur le site : la liste, ce qui est publié, et les œuvres de chacun
/// (calculées par l'API, `arteon sujets`, à chaque construction du site).
enum Subjects {
    static func families() async -> [SubjectFamily] {
        guard let data = try? await Api.site("monde/sujets.json") else { return [] }
        return SubjectFamily.parse(data)
    }

    static func published() async -> [String: PublishedSubject] {
        guard let data = try? await Api.site("api/sujets/index.json") else { return [:] }
        return PublishedSubject.parse(data)
    }

    /// Les œuvres d'un sujet : son catalogue publié avec le site, sinon Wikidata en direct (la
    /// plupart des milliers de catégories ne sont pas publiées : le site les demande aussi).
    static func works(_ key: String) async -> [LiveWork]? {
        if let raw = try? await Api.site("api/sujets/\(key).json.gz"),
           let data = LiveWorks.gunzip(raw),
           let catalogue = try? JSONDecoder().decode(SubjectCatalogue.self, from: data) {
            return catalogue.works
        }
        return await live(key)
    }

    /// Les œuvres d'un sujet demandées à Wikidata (la même requête que le site et l'API).
    static func live(_ key: String) async -> [LiveWork]? {
        guard let data = try? await Api.site("monde/sujets.json") else { return nil }
        let families = SubjectFamily.parse(data)
        guard let family = families.first(where: { $0.subjects.contains { $0.key == key } }),
              let subject = family.subjects.first(where: { $0.key == key }) else { return nil }
        let pattern = family.pattern.replacingOccurrences(of: "{id}", with: subject.ids)
        var allowed = CharacterSet.alphanumerics
        allowed.insert(charactersIn: "-_.~")
        let encoded = pattern.addingPercentEncoding(withAllowedCharacters: allowed) ?? pattern
        for template in SubjectFamily.queries(data) {
            guard let url = URL(string: template.replacingOccurrences(of: "{s}", with: encoded)),
                  let answer = try? await Api.data(url, headers: ["Accept": "application/sparql-results+json"], timeout: 90),
                  let works = parse(answer) else { continue }
            return works
        }
        return nil
    }

    /// Les œuvres d'une réponse SPARQL de la requête des sujets : les plus connues d'abord.
    static func parse(_ data: Data) -> [LiveWork]? {
        guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let rows = (json["results"] as? [String: Any])?["bindings"] as? [[String: Any]] else { return nil }
        func value(_ row: [String: Any], _ key: String) -> String {
            (row[key] as? [String: Any])?["value"] as? String ?? ""
        }
        let ranked: [(LiveWork, Int)] = rows.compactMap { row in
            let entity = value(row, "oeuvre"), image = value(row, "image")
            guard !entity.isEmpty, !image.isEmpty else { return nil }
            // Une œuvre d'après 1930 n'est pas encore libre (la requête a déjà écarté les artistes morts après 1955).
            if let end = Int(value(row, "fin")), end > LiveWorks.lastYear { return nil }
            let year = value(row, "annee")
            let collection = value(row, "collection")
            let work = LiveWork(id: "wikidata-\(entity)", title: value(row, "titre"), date: year, year: Int(year), medium: "",
                                museum: "wikidata", thumbnail: "\(image)?width=250", image: "\(image)?width=1280", link: entity,
                                provider: collection.isEmpty ? nil : collection, artist: value(row, "nom"))
            return (work, Int(value(row, "liens")) ?? 0)
        }
        return ranked.sorted { $0.1 != $1.1 ? $0.1 > $1.1 : ($0.0.year ?? .max) < ($1.0.year ?? .max) }.map(\.0)
    }
}

/// La réponse de l'API (`/api/artiste`), ou celle du moteur de l'application.
struct LiveCatalogue: Decodable, Sendable {
    var name: String
    var total: Int
    var works: [LiveWork]
    /// `api` (le serveur Paquets) ou `musees` (l'application elle-même).
    var source: String = "api"
    /// Les musées qui n'avaient pas répondu quand la réponse a été calculée.
    var missing: [String]? = nil

    enum CodingKeys: String, CodingKey { case name = "nom", total, works = "oeuvres", missing = "manquants" }
}

enum LiveWorks {
    static let lastYear = 1930, lastDeath = 1955

    /// Le contenu d'un fichier gzip écrit par l'API statique : un en-tête de 10 octets sans
    /// champ optionnel, du DEFLATE brut, puis 8 octets de contrôle.
    static func gunzip(_ data: Data) -> Data? {
        #if canImport(Darwin)
        let bytes = [UInt8](data.prefix(4))
        guard data.count > 18, bytes.count == 4, bytes[0] == 0x1f, bytes[1] == 0x8b, bytes[3] == 0 else { return nil }
        let body = data.subdata(in: (data.startIndex + 10)..<(data.endIndex - 8))
        return try? (body as NSData).decompressed(using: .zlib) as Data
        #else
        return nil
        #endif
    }

    /// Toutes les œuvres libres de l'artiste dans les musées : par l'API ARTEON, sinon
    /// directement.
    static func of(_ name: String) async -> LiveCatalogue? {
        // 1. L'API statique, publiée avec le site : immédiate, sans serveur.
        if let index = await SiteMemo.shared.staticIndex() {
            let (byName, byKey, gz) = (index.byName, index.byKey, index.gz)
            if let file = byName[name] ?? byKey[ArtistKey.of(name)],
               let raw = try? await Api.site(gz ? "api/\(file).json.gz" : "api/\(file).json"),
               let data = gz ? gunzip(raw) : raw,
               var answer = try? JSONDecoder().decode(LiveCatalogue.self, from: data) {
                answer.source = "statique"
                // Un musée n'avait pas répondu au calcul : l'application le lui demande, elle.
                if let missing = answer.missing, !missing.isEmpty, let more = await direct(name, only: Set(missing)) {
                    let known = Set(answer.works.map(\.id))
                    answer.works = withoutDuplicates(answer.works + more.works.filter { !known.contains($0.id) },
                                                     sources: await sources() ?? [])
                        .sorted { ($0.year ?? .max) < ($1.year ?? .max) }
                    answer.total = answer.works.count
                }
                return answer
            }
        }
        // 2. Le serveur Paquets, 3. les musées eux-mêmes.
        var parts = URLComponents(url: Api.server.appendingPathComponent("api/artiste"), resolvingAgainstBaseURL: false)
        parts?.queryItems = [URLQueryItem(name: "nom", value: name)]
        if let url = parts?.url, let data = try? await Api.data(url, timeout: 12),
           let answer = try? JSONDecoder().decode(LiveCatalogue.self, from: data) {
            return answer
        }
        return await direct(name)
    }

    /// Les noms des musées décrits (`chicago` → « Art Institute of Chicago »).
    static func museumNames() async -> [String: String] {
        guard let sources = await sources() else { return [:] }
        return Dictionary(uniqueKeysWithValues: sources.compactMap { s in
            (s["cle"] as? String).map { ($0, s["nom"] as? String ?? $0) }
        })
    }

    // Le moteur générique, comme src/api.rs et outils/monde/direct.js.

    private static func sources() async -> [[String: Any]]? {
        await SiteMemo.shared.sources()?.value as? [[String: Any]]
    }

    static func years(_ text: String) -> [Int] {
        var found: [Int] = []
        var digits = ""
        for c in text + " " {
            if c.isASCII, c.isNumber { digits.append(c); continue }
            if digits.count == 4, let y = Int(digits), (1000...2100).contains(y) { found.append(y) }
            digits = ""
        }
        return found
    }

    static func text(_ value: Any?) -> String {
        switch value {
        case let s as String: s
        case let n as Int: String(n)
        case let d as Double: d == d.rounded() ? String(Int(d)) : String(d)
        case let b as Bool: b ? "true" : "false"
        default: ""
        }
    }

    static func year(_ value: Any?) -> Int? {
        let y = (value as? Int) ?? years(text(value)).first
        guard let y, y != 0, y < 9999 else { return nil }
        return y
    }

    static func path(_ value: Any?, _ path: String) -> Any? {
        path.split(separator: ".").reduce(value) { current, key in
            if let i = Int(key), let list = current as? [Any] { return list.indices.contains(i) ? list[i] : nil }
            return (current as? [String: Any])?[String(key)]
        }
    }

    static func isFree(date: String, end: Int?, death: Int?, birth: Int?) -> Bool {
        if (years(date) + (end.map { [$0] } ?? [])).contains(where: { $0 > lastYear }) { return false }
        if let death { return death <= lastDeath }
        if let birth { return birth <= 1890 }
        return true
    }

    static func life(_ text: String) -> (birth: Int?, death: Int?) {
        let y = years(text)
        switch y.count {
        case 0: return (nil, nil)
        case 1: return text.contains("born") || text.hasSuffix("–") || text.hasSuffix("-") ? (y[0], nil) : (nil, y[0])
        default: return (y[0], y[1])
        }
    }

    /// Le nom d'un musée correspond-il à l'artiste demandé ?
    static func sameArtist(_ asked: String, _ found: String) -> Bool {
        if let comma = found.range(of: ", "), !found[comma.upperBound...].contains(Character(",")) {
            let turned = "\(found[comma.upperBound...]) \(found[..<comma.lowerBound])"
            if sameArtist(asked, turned) { return true }
        }
        let k = ArtistKey.of(asked), t = ArtistKey.of(found)
        if k == t { return true }
        let f = folded(found)
        if f.contains("after ") || f.hasPrefix("follower") || f.contains("imitator") { return false }
        let words: [String] = k.split(separator: " ").map(String.init).filter { $0.count > 2 }
        let inside: Set<String> = Set(t.split(separator: " ").map(String.init))
        let lastAsked = k.split(separator: " ").last.map(String.init)
        let lastFound = t.split(separator: " ").last.map(String.init)
        return !words.isEmpty && words.allSatisfy { inside.contains($0) } && lastAsked == lastFound
    }

    private static func fill(_ template: String, _ values: [String: String]) -> String {
        values.reduce(template) { $0.replacingOccurrences(of: "{\($1.key)}", with: $1.value) }
    }

    /// Un titre sans balises HTML ni entités (comme `sans_balises` de src/api.rs).
    static func withoutTags(_ text: String) -> String {
        guard text.contains("<") || text.contains("&") else { return text }
        var clean = text.replacingOccurrences(of: "</?[a-zA-Z][^>]*>", with: "", options: .regularExpression)
        for (entity, sign) in [("&amp;", "&"), ("&quot;", "\""), ("&#39;", "'"), ("&apos;", "'"), ("&nbsp;", " "), ("&lt;", "<"), ("&gt;", ">")] {
            clean = clean.replacingOccurrences(of: entity, with: sign)
        }
        return clean.split(whereSeparator: { $0 == " " || $0 == "\n" }).joined(separator: " ")
    }

    /// L'index de l'API statique (`api/index.json`) : le fichier de chaque artiste, par son
    /// nom ou par sa clé.
    struct StaticIndex: Sendable {
        var byName: [String: String]
        var byKey: [String: String]
        /// Les catalogues sont publiés compressés (`gz`) : l'application les décompresse.
        var gz: Bool
    }

    /// Sans les doublons (comme `sans_doublons` de src/api.rs) : une œuvre qu'un musée interrogé
    /// donne lui-même et qu'un agrégateur (Wikidata, Europeana) donne aussi : même titre, même
    /// année, et la collection indiquée par l'agrégateur est ce musée. La fiche du musée reste.
    static func withoutDuplicates(_ works: [LiveWork], sources: [[String: Any]]) -> [LiveWork] {
        let fold = { (s: String) in
            folded(s).map { $0.isLetter || $0.isNumber ? String($0) : " " }.joined()
                .split(separator: " ").joined(separator: " ")
        }
        let names = sources.map { (key: $0["cle"] as? String ?? "", name: fold($0["nom"] as? String ?? "")) }
        // La clé de chaque œuvre, calculée une fois (le titre plié servait jusqu'à quatre fois).
        let keys: [String?] = works.map { w in
            guard let year = w.year else { return nil }
            let title = fold(w.title)
            return title.isEmpty ? nil : "\(title)|\(year)"
        }
        let held = Set(zip(works, keys).compactMap { w, k in k.map { "\($0)|\(w.museum)" } })
        return zip(works, keys).filter { w, k in
            guard let collection = w.provider.map(fold), collection.count >= 8, let k else { return true }
            return !names.contains { $0.key != w.museum && $0.name.contains(collection) && held.contains("\(k)|\($0.key)") }
        }.map(\.0)
    }

    static func direct(_ name: String, only: Set<String>? = nil) async -> LiveCatalogue? {
        guard let all = await sources() else { return nil }
        let sources = all.filter { source in only.map { $0.contains(source["cle"] as? String ?? "") } ?? true }
        let key = ArtistKey.of(name)
        let names = [name] + ArtistKey.aliases.filter { $0.value == key }.map(\.key)
        let lists = await withTaskGroup(of: [LiveWork].self) { group in
            for source in sources {
                let description = Json(value: source)
                group.addTask { await ask(description, name: name, names: names) }
            }
            var all: [[LiveWork]] = []
            for await list in group { all.append(list) }
            return all
        }
        var seen = Set<String>()
        let works = withoutDuplicates(lists.flatMap { $0 }.filter { seen.insert($0.id).inserted }, sources: all)
            .sorted { ($0.year ?? .max) < ($1.year ?? .max) }
        return LiveCatalogue(name: name, total: works.count, works: works, source: "musees")
    }

    private static func ask(_ description: Json, name: String, names: [String]) async -> [LiveWork] {
        guard let source = description.value as? [String: Any] else { return [] }
        let museum = source["cle"] as? String ?? ""
        let perPage = source["par_page"] as? Int ?? 100
        let pages = source["pages"] as? Int ?? 1
        let delay = TimeInterval(source["delai"] as? Int ?? 20)
        let headers = (source["entetes"] as? [String: String]) ?? [:]
        let fields = (source["champs"] as? [String: String]) ?? [:]
        var records: [Any] = []
        var ids: [String] = []
        // Une recherche limitée en pages (Chicago : 10 pages de 100) qui les a toutes remplies :
        // « au_dela », la même dans l'ordre inverse, donne les suivantes (comme src/api.rs).
        for q in names {
            var templates = [source["recherche"] as? String ?? ""]
            var m = 0
            while m < templates.count {
            var full = false
            for page in 1...max(1, pages) {
                let encoded = q.addingPercentEncoding(withAllowedCharacters: .alphanumerics) ?? q
                let address = templates[m]
                    .replacingOccurrences(of: "{q}", with: encoded)
                    .replacingOccurrences(of: "{n}", with: String(perPage))
                    .replacingOccurrences(of: "{page}", with: String(page))
                    .replacingOccurrences(of: "{offset}", with: String((page - 1) * perPage))
                    .replacingOccurrences(of: "{debut}", with: String((page - 1) * perPage + 1))
                    .replacingOccurrences(of: "[", with: "%5B").replacingOccurrences(of: "]", with: "%5D")
                guard let url = URL(string: address), let data = try? await Api.data(url, headers: headers, timeout: delay),
                      let json = try? JSONSerialization.jsonObject(with: data)
                else { break }
                // Des identifiants (le Met : 500 par page au plus), puis les fiches une par une.
                if let key = source["ids"] as? String {
                    let found = (path(json, key) as? [Any]) ?? []
                    ids += found.map(text)
                    if found.count < perPage { break }
                    continue
                }
                let list = (source["liste"] as? String).map { path(json, $0) as? [Any] ?? [] } ?? (json as? [Any] ?? [])
                records += list
                if list.count < perPage { break }
                if page == pages { full = true }
            }
            if full, m == 0, let more = source["au_dela"] as? String { templates.append(more) }
            m += 1
            }
        }
        if let detail = source["detail"] as? String {
            var seen = Set<String>()
            let wanted = ids.filter { seen.insert($0).inserted }.prefix(min(source["max"] as? Int ?? 3000, 1000))
            // Seize à la fois : mille demandes lancées ensemble attendraient leur tour dans
            // URLSession, et les dernières dépasseraient leur délai avant même de partir.
            let wantedIds = Array(wanted)
            var all: [Any] = []
            for start in stride(from: 0, to: wantedIds.count, by: 16) {
                if Task.isCancelled { break }
                let batch = wantedIds[start..<min(start + 16, wantedIds.count)]
                let found = await withTaskGroup(of: Json?.self) { group in
                    for id in batch {
                        group.addTask {
                            guard let url = URL(string: detail.replacingOccurrences(of: "{id}", with: id)),
                                  let data = try? await Api.data(url, headers: headers),
                                  let json = try? JSONSerialization.jsonObject(with: data)
                            else { return nil }
                            return Json(value: json)
                        }
                    }
                    var some: [Any] = []
                    for await r in group { if let r { some.append(r.value) } }
                    return some
                }
                all += found
            }
            records = all
        }
        let required = (source["exige"] as? [String]) ?? []
        return records.compactMap { record in
            func field(_ k: String) -> Any? { fields[k].flatMap { path(record, $0) } }
            for k in required {
                let v = field(k)
                if v == nil || (v as? Bool) == false || text(v).isEmpty { return nil }
            }
            for (k, expected) in (source["egal"] as? [String: Any]) ?? [:] where text(field(k)) != text(expected) {
                return nil
            }
            var artist = text(field("artiste"))
            if let cut = source["artiste_jusqua"] as? String, let r = artist.range(of: cut) {
                artist = String(artist[..<r.lowerBound])
            }
            if !sameArtist(name, artist) {
                // Les autres noms possibles (Europeana : les libellés de ses entités).
                let others = ((field("artistes") as? [Any]) ?? []).map { v in
                    (v as? [String: Any]).map { text($0["def"]) } ?? text(v)
                }
                // Seulement quand le créateur est un identifiant (une adresse) : un photographe
                // nommé qui a photographié l'œuvre n'en est pas l'auteur.
                let identifier = artist.isEmpty || artist.hasPrefix("http")
                guard identifier, let found = others.first(where: { sameArtist(name, $0) }) else { return nil }
                artist = found
            }
            var (birth, death) = (year(field("ne")), year(field("mort")))
            if birth == nil, death == nil { (birth, death) = life(text(field("vie"))) }
            let date = text(field("date"))
            guard isFree(date: date, end: year(field("fin")), death: death, birth: birth) else { return nil }
            var values: [String: String] = [:]
            for k in fields.keys { values[k] = text(field(k)) }
            return LiveWork(
                id: "\(museum)-\(values["id"] ?? "")", title: withoutTags(values["titre"] ?? ""), date: date,
                year: year(field("annee")), medium: values["technique"] ?? "", museum: museum,
                thumbnail: fill(source["vignette"] as? String ?? "", values),
                image: fill(source["grande"] as? String ?? "", values),
                link: fill(source["lien"] as? String ?? "", values),
                provider: values["fournisseur"].flatMap { $0.isEmpty || $0.hasPrefix("http") ? nil : $0 }
            )
        }
    }
}
