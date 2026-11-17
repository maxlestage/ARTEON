// Les collections du monde : des dizaines de milliers d'œuvres du domaine public, choisies
// dans les collections ouvertes de grands musées (outils/collections_monde.py). Leurs
// fiches et leurs vignettes sont dans l'application (Musee/collections/) ; l'image en
// grand se télécharge au premier affichage, puis reste sur l'appareil. Rien que
// Foundation : ce fichier se teste aussi sous Linux.

import Foundation

/// Un musée dont les collections ouvertes sont dans l'application.
struct CollectionMuseum: Decodable, Hashable, Sendable, Identifiable {
    var key: String
    var name: String
    var city: String
    var site: String
    var color: String
    var licence: String
    var count: Int
    /// Le fichier de ses vignettes (`vignettes-met.bin`).
    var pack: String
    /// Le fichier de ses images en grand, 800 px (`grandes-met.bin`), s'il y en a.
    var largePack: String?

    var id: String { key }

    enum CodingKeys: String, CodingKey {
        case key = "cle", name = "nom", city = "ville", site, color = "couleur", licence
        case count = "oeuvres", pack = "paquet", largePack = "grandes"
    }
}

/// Une œuvre des collections du monde.
struct CollectionWork: Decodable, Hashable, Sendable, Identifiable {
    var id: String
    var title: String
    var artist: String
    /// La vie de l'artiste ou sa nationalité (« Dutch, 1853–1890 »).
    var life: String?
    var date: String?
    var year: Int?
    var medium: String?
    var dimensions: String?
    var category: String?
    var origin: String?
    /// Le texte du musée (en anglais).
    var text: String?
    /// L'image en grand, sur le site du musée.
    var imageURL: String?
    /// La page de l'œuvre sur le site du musée.
    var page: String?
    /// Hauteur / largeur de l'image.
    var ratio: Double?
    /// La salle où la voir en vrai, si elle est exposée.
    var gallery: String?
    /// Sa vignette dans le paquet du musée : [décalage, longueur].
    var thumb: [Int]?
    /// Son image en grand (800 px) dans le paquet des grandes images : [décalage, longueur].
    var large: [Int]?
    /// Le musée (`met`).
    var museum: String

    enum CodingKeys: String, CodingKey {
        case id = "i", title = "t", artist = "a", life = "v", date = "d", year = "y", medium = "m"
        case dimensions = "s", category = "c", origin = "o", text = "x", imageURL = "u", page = "w"
        case ratio = "r", gallery = "g", thumb = "p", museum = "k", large = "gr"
    }

    /// « Vincent van Gogh · 1889 ».
    var byline: String { [artist, date ?? ""].filter { !$0.isEmpty }.joined(separator: " · ") }

    /// Largeur / hauteur (pour réserver la place de l'image).
    var aspect: Double { (ratio ?? 0) > 0 ? 1 / ratio! : 0.8 }

    /// La clé de l'œuvre dans les favoris et l'historique.
    var key: WorkKey { WorkKey(exhibition: WorldCollections.favoritePrefix, work: id) }
}

struct WorldCollections: Decodable, Sendable {
    var version: Int
    var date: String
    var museums: [CollectionMuseum]
    var works: [CollectionWork]

    /// Les favoris des collections : `monde/<identifiant>`.
    static let favoritePrefix = "monde"

    enum CodingKeys: String, CodingKey {
        case version, date, museums = "musees", works = "oeuvres"
    }

    static func decode(_ data: Data) throws -> WorldCollections {
        let c = try JSONDecoder().decode(WorldCollections.self, from: data)
        guard c.version <= 1 else { throw CatalogueError.tooNew(c.version) }
        return c
    }

    func museum(_ key: String) -> CollectionMuseum? {
        museums.first { $0.key == key }
    }
}

/// Ce que l'on cherche dans les collections.
enum CollectionFilter: Hashable, Sendable {
    case museum(String)
    case category(String)
    case century(Int)
    case artist(String)
    case search(String)
    case all
}

/// Les index des collections, calculés une fois (en arrière-plan) : par musée, par
/// catégorie, par siècle, par artiste, et le texte de recherche de chaque œuvre.
struct CollectionIndex: Sendable {
    let collections: WorldCollections
    private let positions: [String: Int]
    private let searchText: [String]
    private let byArtist: [String: [Int]]
    let categories: [(name: String, count: Int)]
    /// Les siècles (rang de `centuryRank`) et leur nombre d'œuvres, du plus ancien.
    let centuries: [(rank: Int, title: String, count: Int)]

    init(_ collections: WorldCollections) {
        self.collections = collections
        var positions: [String: Int] = [:]
        var searchText: [String] = []
        var byArtist: [String: [Int]] = [:]
        var categories: [String: Int] = [:]
        var centuries: [Int: (title: String, count: Int)] = [:]
        searchText.reserveCapacity(collections.works.count)
        for (i, w) in collections.works.enumerated() {
            positions[w.id] = i
            let artist = folded(w.artist)
            searchText.append(folded(
                "\(w.title) \(w.artist) \(w.date ?? "") \(w.medium ?? "") \(w.category ?? "") \(w.origin ?? "") \(w.museum)"
            ))
            byArtist[artist, default: []].append(i)
            categories[w.category ?? "Autres objets", default: 0] += 1
            if let y = w.year {
                let rank = centuryRank(y)
                centuries[rank, default: (centuryLabel(y), 0)].count += 1
            }
        }
        self.positions = positions
        self.searchText = searchText
        self.byArtist = byArtist
        self.categories = categories.map { ($0.key, $0.value) }.sorted { $0.count > $1.count }
        self.centuries = centuries.map { ($0.key, $0.value.title, $0.value.count) }.sorted { $0.rank < $1.rank }
    }

    var count: Int { collections.works.count }

    func work(_ id: String) -> CollectionWork? {
        positions[id].map { collections.works[$0] }
    }

    /// Les œuvres d'un filtre (au plus `limit`).
    func works(_ filter: CollectionFilter, limit: Int = .max) -> [CollectionWork] {
        let all = collections.works
        var result: [CollectionWork] = []
        func add(_ w: CollectionWork) -> Bool {
            result.append(w)
            return result.count < limit
        }
        switch filter {
        case .all:
            return Array(all.prefix(limit))
        case .museum(let key):
            for w in all where w.museum == key { if !add(w) { break } }
        case .category(let name):
            for w in all where (w.category ?? "Autres objets") == name { if !add(w) { break } }
        case .century(let rank):
            for w in all where w.year.map(centuryRank) == rank { if !add(w) { break } }
        case .artist(let name):
            return (byArtist[folded(name)] ?? []).prefix(limit).map { all[$0] }
        case .search(let query):
            let words = folded(query.trimmingCharacters(in: .whitespacesAndNewlines))
                .split(separator: " ").map(String.init)
            guard !words.isEmpty else { return [] }
            for (i, text) in searchText.enumerated() where words.allSatisfy({ text.contains($0) }) {
                if !add(all[i]) { break }
            }
        }
        return result
    }

    /// Le nombre d'œuvres d'un filtre.
    func count(_ filter: CollectionFilter) -> Int {
        switch filter {
        case .all: return count
        case .museum(let key): return collections.museum(key)?.count ?? 0
        case .category(let name): return categories.first { $0.name == name }?.count ?? 0
        case .century(let rank): return centuries.first { $0.rank == rank }?.count ?? 0
        case .artist(let name): return byArtist[folded(name)]?.count ?? 0
        case .search: return works(filter).count
        }
    }

    /// Le titre d'un filtre.
    func title(_ filter: CollectionFilter) -> String {
        switch filter {
        case .all: "Toutes les œuvres"
        case .museum(let key): collections.museum(key)?.name ?? key
        case .category(let name): name
        case .century(let rank): centuries.first { $0.rank == rank }?.title ?? "Un siècle"
        case .artist(let name): name
        case .search(let query): "« \(query) »"
        }
    }

    /// Des œuvres au hasard, les mêmes toute la journée.
    func picks(_ count: Int, day: Int) -> [CollectionWork] {
        guard !collections.works.isEmpty else { return [] }
        var rng = SplitMix64(seed: UInt64(truncatingIfNeeded: day) &* 2_654_435_761)
        return (0..<count).map { _ in collections.works[Int(rng.next() % UInt64(collections.works.count))] }
    }
}

extension LocalMuseum {
    /// Le fichier des collections du monde, s'il est dans l'application.
    static var collectionsFile: URL? { stored("collections/collections.json") }

    /// Les collections sont-elles à portée (sur l'appareil, ou sur Nuage) ?
    static var hasCollections: Bool { collectionsFile != nil || Nuage.isConfigured }

    /// Les octets de l'image en grand (800 px) d'une œuvre des collections, dans l'app.
    static func largeImage(of work: CollectionWork, pack: String?, in root: URL? = nil) -> Data? {
        guard let pack else { return nil }
        return packBytes(work.large, pack: pack, in: root)
    }

    /// Les octets de la vignette d'une œuvre des collections, lus dans le paquet de son musée.
    static func thumbnail(of work: CollectionWork, pack: String, in root: URL? = nil) -> Data? {
        packBytes(work.thumb, pack: pack, in: root)
    }

    /// Les octets [décalage, longueur] d'un paquet des collections.
    static func packBytes(_ range: [Int]?, pack: String, in root: URL? = nil) -> Data? {
        guard let range, range.count == 2,
              let url = root.map({ file("collections/\(pack)", in: $0) }) ?? stored("collections/\(pack)"),
              let handle = try? FileHandle(forReadingFrom: url)
        else { return nil }
        defer { try? handle.close() }
        do {
            try handle.seek(toOffset: UInt64(range[0]))
            return try handle.read(upToCount: range[1])
        } catch {
            return nil
        }
    }
}
