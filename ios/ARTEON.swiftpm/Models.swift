// Le catalogue d'ARTEON tel que le site le publie dans `catalogue.json` (voir
// src/catalogue.rs) : les expositions, leurs salles, leurs artistes et leurs œuvres avec
// leur notice. Rien que Foundation : ce fichier se teste aussi hors d'un appareil Apple.

import Foundation

/// La version du format que l'application sait lire.
let catalogueVersion = 1

struct Catalogue: Decodable, Sendable {
    var version: Int
    var exhibitions: [Exhibition]

    enum CodingKeys: String, CodingKey {
        case version, exhibitions
    }

    init(version: Int = catalogueVersion, exhibitions: [Exhibition]) {
        self.version = version
        self.exhibitions = exhibitions
    }

    init(from decoder: any Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        version = try c.decode(Int.self, forKey: .version)
        guard version <= catalogueVersion else {
            throw CatalogueError.tooNew(version)
        }
        exhibitions = try c.decode([Exhibition].self, forKey: .exhibitions)
    }

    /// Lit `catalogue.json`.
    static func decode(_ data: Data) throws -> Catalogue {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return try decoder.decode(Catalogue.self, from: data)
    }

    func exhibition(_ slug: String) -> Exhibition? {
        exhibitions.first { $0.slug == slug }
    }

    /// Une œuvre par sa clé « exposition/œuvre ».
    func work(_ key: WorkKey) -> (Exhibition, Work)? {
        guard let ex = exhibition(key.exhibition),
              let work = ex.works.first(where: { $0.slug == key.work })
        else { return nil }
        return (ex, work)
    }

    /// Toutes les œuvres, dans l'ordre du musée.
    var allWorks: [(Exhibition, Work)] {
        exhibitions.flatMap { ex in ex.works.map { (ex, $0) } }
    }

    var workCount: Int { exhibitions.reduce(0) { $0 + $1.works.count } }
    var roomCount: Int { exhibitions.reduce(0) { $0 + $1.rooms.count } }

    /// Les mouvements, dans l'ordre des expositions, sans doublon.
    var movements: [String] {
        var seen = Set<String>()
        return exhibitions.compactMap { ex in
            !ex.movement.isEmpty && seen.insert(ex.movement).inserted ? ex.movement : nil
        }
    }

    /// Les œuvres dont le titre, l'artiste, la technique, la collection ou l'année contient
    /// la recherche, sans tenir compte des accents ni des majuscules.
    func search(_ query: String) -> [(Exhibition, Work)] {
        let q = folded(query.trimmingCharacters(in: .whitespacesAndNewlines))
        guard !q.isEmpty else { return [] }
        return allWorks.filter { _, w in
            folded("\(w.title) \(w.artist) \(w.technique) \(w.collection) \(w.year)").contains(q)
        }
    }
}

enum CatalogueError: LocalizedError, Equatable {
    case tooNew(Int)

    var errorDescription: String? {
        switch self {
        case .tooNew(let v):
            "Ce catalogue (version \(v)) est plus récent que l'application : mettez-la à jour."
        }
    }
}

/// Le texte sans accents, en minuscules, pour chercher (« Œuvre » → « oeuvre »).
func folded(_ s: String) -> String {
    s.replacingOccurrences(of: "œ", with: "oe")
        .replacingOccurrences(of: "Œ", with: "oe")
        .replacingOccurrences(of: "æ", with: "ae")
        .replacingOccurrences(of: "Æ", with: "ae")
        .folding(options: [.diacriticInsensitive, .caseInsensitive], locale: Locale(identifier: "fr_FR"))
        .lowercased()
}

/// Désigne une œuvre dans tout le musée.
struct WorkKey: Hashable, Codable, Sendable {
    var exhibition: String
    var work: String

    init(exhibition: String, work: String) {
        self.exhibition = exhibition
        self.work = work
    }

    /// « van-gogh/tournesols ».
    init?(_ text: String) {
        let parts = text.split(separator: "/", maxSplits: 1).map(String.init)
        guard parts.count == 2, !parts[0].isEmpty, !parts[1].isEmpty else { return nil }
        self.init(exhibition: parts[0], work: parts[1])
    }

    var text: String { "\(exhibition)/\(work)" }
}

struct Museum: Decodable, Hashable, Sendable {
    var name: String
    var city: String
    var architect: String
    var built: String
    var presentation: String
    var here: String
}

struct Room: Decodable, Hashable, Sendable {
    var name: String
    var theme: String
    var corridor: Bool
}

struct Artist: Decodable, Hashable, Sendable, Identifiable {
    var name: String
    var life: String
    var biography: String
    var style: String

    var id: String { name }
}

struct Notice: Decodable, Hashable, Sendable {
    var description: String
    var evocation: String
    var story: String
    var manner: String
    var details: [String]
    var themes: [String]
    var anecdote: String

    /// Les parties rédigées, dans l'ordre du site.
    var sections: [(title: String, text: String)] {
        [
            ("Ce que l'on voit", description),
            ("Ce qu'elle évoque", evocation),
            ("Son histoire", story),
            ("La manière", manner),
        ].filter { !$0.1.isEmpty }
    }
}

struct Credit: Decodable, Hashable, Sendable {
    var author: String
    var licence: String
    var source: String
}

struct Work: Decodable, Hashable, Sendable, Identifiable {
    enum Kind: String, Decodable, Sendable {
        case painting, sculpture
    }

    var kind: Kind
    var slug: String
    var title: String
    var artist: String
    var year: String
    var technique: String
    var collection: String
    var note: String
    /// Le chemin de l'image sur le site (`tableaux/van-gogh/iris.jpg`).
    var image: String
    var room: Int
    var width: Double
    var height: Double
    var credit: Credit?
    var notice: Notice

    var id: String { slug }

    /// Largeur / hauteur de l'œuvre (pour réserver la place de son image).
    var aspect: Double { width > 0 && height > 0 ? width / height : 0.8 }

    /// « Vincent van Gogh · 1888 ».
    var byline: String { [artist, year].filter { !$0.isEmpty }.joined(separator: " · ") }
}

struct Exhibition: Decodable, Hashable, Sendable, Identifiable {
    var slug: String
    var title: String
    var shortName: String
    var kicker: String
    var artist: String
    var artistDates: String
    var movement: String
    var ephemeral: Bool
    var introduction: String
    /// « #1d3f8f ».
    var accent: String
    var cover: String?
    /// Le chemin de la galerie 3D sur le site (`van-gogh/galerie`).
    var gallery: String
    var museum: Museum?
    var rooms: [Room]
    var artists: [Artist]
    var works: [Work]

    var id: String { slug }

    var name: String { shortName.isEmpty ? title : shortName }

    var coverImage: String? { cover ?? works.first?.image }

    /// La couleur de l'exposition, en rouge, vert, bleu de 0 à 1.
    var accentRGB: (red: Double, green: Double, blue: Double) {
        let hex = accent.trimmingCharacters(in: CharacterSet(charactersIn: "#"))
        let value = UInt32(hex, radix: 16) ?? 0x1d3f8f
        return (
            Double((value >> 16) & 0xff) / 255,
            Double((value >> 8) & 0xff) / 255,
            Double(value & 0xff) / 255
        )
    }

    /// Les œuvres de chaque salle qui en a, dans l'ordre.
    var worksByRoom: [(index: Int, room: Room, works: [Work])] {
        rooms.enumerated().compactMap { i, room in
            let works = works.filter { $0.room == i }
            return works.isEmpty ? nil : (i, room, works)
        }
    }

    func artist(named name: String) -> Artist? {
        artists.first { $0.name == name }
    }

    func key(_ work: Work) -> WorkKey {
        WorkKey(exhibition: slug, work: work.slug)
    }
}
