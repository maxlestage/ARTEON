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
        configuration.urlCache = URLCache(memoryCapacity: 64 * 1024 * 1024, diskCapacity: 0, diskPath: nil)
        configuration.requestCachePolicy = .useProtocolCachePolicy
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
        await SessionStore.shared.put(url.absoluteString, data)
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

    enum CodingKeys: String, CodingKey {
        case id, title = "titre", date, year = "annee", medium = "technique", museum = "musee"
        case thumbnail = "vignette", image, link = "lien", provider = "fournisseur"
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

    /// Toutes les œuvres libres de l'artiste dans les musées : par l'API ARTEON, sinon
    /// directement.
    static func of(_ name: String) async -> LiveCatalogue? {
        // 1. L'API statique, publiée avec le site : immédiate, sans serveur.
        if let index = try? await Api.site("api/index.json"),
           let json = try? JSONSerialization.jsonObject(with: index) as? [String: Any] {
            let byName = json["noms"] as? [String: String] ?? [:]
            let byKey = json["cles"] as? [String: String] ?? [:]
            if let file = byName[name] ?? byKey[ArtistKey.of(name)],
               let data = try? await Api.site("api/\(file).json"),
               var answer = try? JSONDecoder().decode(LiveCatalogue.self, from: data) {
                answer.source = "statique"
                // Un musée n'avait pas répondu au calcul : l'application le lui demande, elle.
                if let missing = answer.missing, !missing.isEmpty, let more = await direct(name, only: Set(missing)) {
                    let known = Set(answer.works.map(\.id))
                    answer.works = (answer.works + more.works.filter { !known.contains($0.id) })
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
        guard let data = try? await Api.site("monde/sources.json"),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else { return nil }
        return json["sources"] as? [[String: Any]]
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
        let works = lists.flatMap { $0 }.filter { seen.insert($0.id).inserted }
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
        for q in names {
            for page in 1...max(1, pages) {
                let encoded = q.addingPercentEncoding(withAllowedCharacters: .alphanumerics) ?? q
                let address = (source["recherche"] as? String ?? "")
                    .replacingOccurrences(of: "{q}", with: encoded)
                    .replacingOccurrences(of: "{n}", with: String(perPage))
                    .replacingOccurrences(of: "{page}", with: String(page))
                    .replacingOccurrences(of: "{offset}", with: String((page - 1) * perPage))
                    .replacingOccurrences(of: "{debut}", with: String((page - 1) * perPage + 1))
                    .replacingOccurrences(of: "[", with: "%5B").replacingOccurrences(of: "]", with: "%5D")
                guard let url = URL(string: address), let data = try? await Api.data(url, headers: headers, timeout: delay),
                      let json = try? JSONSerialization.jsonObject(with: data)
                else { break }
                if let key = source["ids"] as? String {
                    ids += ((path(json, key) as? [Any]) ?? []).map(text)
                    break
                }
                let list = (source["liste"] as? String).map { path(json, $0) as? [Any] ?? [] } ?? (json as? [Any] ?? [])
                records += list
                if list.count < perPage { break }
            }
        }
        if let detail = source["detail"] as? String {
            var seen = Set<String>()
            let wanted = ids.filter { seen.insert($0).inserted }.prefix(min(source["max"] as? Int ?? 3000, 300))
            records = await withTaskGroup(of: Json?.self) { group in
                for id in wanted {
                    group.addTask {
                        guard let url = URL(string: detail.replacingOccurrences(of: "{id}", with: id)),
                              let data = try? await Api.data(url, headers: headers),
                              let json = try? JSONSerialization.jsonObject(with: data)
                        else { return nil }
                        return Json(value: json)
                    }
                }
                var all: [Any] = []
                for await r in group { if let r { all.append(r.value) } }
                return all
            }
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
                id: "\(museum)-\(values["id"] ?? "")", title: values["titre"] ?? "", date: date,
                year: year(field("annee")), medium: values["technique"] ?? "", museum: museum,
                thumbnail: fill(source["vignette"] as? String ?? "", values),
                image: fill(source["grande"] as? String ?? "", values),
                link: fill(source["lien"] as? String ?? "", values),
                provider: values["fournisseur"].flatMap { $0.isEmpty || $0.hasPrefix("http") ? nil : $0 }
            )
        }
    }
}
