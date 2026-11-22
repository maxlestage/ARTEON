// Le catalogue d'ARTEON tel que le site le publie dans `catalogue.json` (voir
// src/catalogue.rs) : les expositions, leurs salles, leurs artistes et leurs œuvres avec
// leur notice. Rien que Foundation : ce fichier se teste aussi hors d'un appareil Apple.

import Foundation

/// La version du format que l'application sait lire.
let catalogueVersion = 1

struct Catalogue: Decodable, Sendable {
    var version: Int
    var exhibitions: [Exhibition] {
        // Une exposition changée : les places et les textes de recherche sont à refaire.
        didSet { lookup = Lookup() }
    }
    /// Les places des expositions et des œuvres, et le texte de recherche de chaque œuvre :
    /// calculés une fois, à la première demande (`work(_:)` est appelée pour chaque œuvre
    /// d'une chronologie, d'un thème, des favoris… à chaque affichage), et partagés par les
    /// copies du catalogue.
    private var lookup = Lookup()

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
        lookup.places(exhibitions).exhibitions[slug].map { exhibitions[$0] }
    }

    /// Une œuvre par sa clé « exposition/œuvre ».
    func work(_ key: WorkKey) -> (Exhibition, Work)? {
        guard let place = lookup.places(exhibitions).works[key] else { return nil }
        let ex = exhibitions[place.0]
        return (ex, ex.works[place.1])
    }

    /// Prépare les places et les textes de recherche (en arrière-plan, avec les index), pour
    /// que la première recherche ou la première chronologie n'attende pas.
    func prepareLookup() {
        _ = lookup.places(exhibitions)
        _ = lookup.searchTexts(exhibitions)
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
    /// Chaque mot de la recherche doit se trouver quelque part (« monet nuit » trouve les
    /// nuits de Monet). Les œuvres dont le titre ou l'artiste correspond viennent d'abord.
    func search(_ query: String) -> [(Exhibition, Work)] {
        let words = folded(query.trimmingCharacters(in: .whitespacesAndNewlines))
            .split(whereSeparator: { $0 == " " || $0 == "," })
            .map(String.init)
        guard !words.isEmpty else { return [] }
        var strong: [(Exhibition, Work)] = []
        var weak: [(Exhibition, Work)] = []
        // Les textes pliés une fois pour toutes, pas à chaque lettre tapée.
        let texts = lookup.searchTexts(exhibitions)
        for (i, ex) in exhibitions.enumerated() {
            for (j, w) in ex.works.enumerated() {
                let (main, all) = texts[i][j]
                guard words.allSatisfy({ all.contains($0) }) else { continue }
                if words.allSatisfy({ main.contains($0) }) {
                    strong.append((ex, w))
                } else {
                    weak.append((ex, w))
                }
            }
        }
        return strong + weak
    }

    /// Le texte de recherche d'une œuvre : son titre et son artiste (`main`), puis tout le
    /// reste (`all`), sans accents ni majuscules.
    fileprivate static func searchText(_ w: Work, in ex: Exhibition) -> (main: String, all: String) {
        let main = folded("\(w.title) \(w.artist)")
        let all = main + " " + folded(
            "\(w.technique) \(w.collection) \(w.year) \(w.notice.themes.joined(separator: " ")) \(ex.name) \(ex.movement)"
        )
        return (main, all)
    }

    /// Les expositions dont le titre, le style ou l'artiste contient la recherche.
    func searchExhibitions(_ query: String) -> [Exhibition] {
        let q = folded(query.trimmingCharacters(in: .whitespacesAndNewlines))
        guard !q.isEmpty else { return [] }
        return exhibitions.filter {
            folded("\($0.title) \($0.shortName) \($0.movement) \($0.artist) \($0.kicker)").contains(q)
        }
    }
}

/// Les index paresseux d'un catalogue (voir `Catalogue.lookup`). Une classe, pour être
/// partagée par les copies du catalogue ; un verrou, car le catalogue passe d'une tâche à
/// l'autre (les index se calculent en arrière-plan, l'affichage les lit sur le fil principal).
private final class Lookup: @unchecked Sendable {
    struct Places {
        /// La place de chaque exposition (la première, si un nom revenait).
        var exhibitions: [String: Int] = [:]
        /// La place de chaque œuvre : son exposition, puis son rang dans l'exposition.
        var works: [WorkKey: (Int, Int)] = [:]
    }

    private let lock = NSLock()
    private var places: Places?
    private var texts: [[(main: String, all: String)]]?

    func places(_ exhibitions: [Exhibition]) -> Places {
        lock.lock()
        defer { lock.unlock() }
        if let places { return places }
        var built = Places()
        for (i, ex) in exhibitions.enumerated() where built.exhibitions[ex.slug] == nil {
            built.exhibitions[ex.slug] = i
            for (j, w) in ex.works.enumerated() {
                let key = WorkKey(exhibition: ex.slug, work: w.slug)
                if built.works[key] == nil { built.works[key] = (i, j) }
            }
        }
        places = built
        return built
    }

    func searchTexts(_ exhibitions: [Exhibition]) -> [[(main: String, all: String)]] {
        lock.lock()
        defer { lock.unlock() }
        if let texts { return texts }
        let built = exhibitions.map { ex in ex.works.map { Catalogue.searchText($0, in: ex) } }
        texts = built
        return built
    }
}

enum CatalogueError: LocalizedError, Equatable {
    case tooNew(Int)
    case unreadable

    var errorDescription: String? {
        switch self {
        case .tooNew(let v):
            L("Ce catalogue (version \(v)) est plus récent que l'application : mettez-la à jour.",
              "This catalogue (version \(v)) is newer than the app: please update it.",
              "Este catálogo (versión \(v)) es más reciente que la aplicación: actualícela.")
        case .unreadable:
            L("La réponse du site ARTEON est illisible.",
              "The ARTEON website's response could not be read.",
              "La respuesta del sitio ARTEON no se puede leer.")
        }
    }
}

/// La locale du pliage, créée une fois (et non à chaque appel : `folded` sert des dizaines
/// de milliers de fois pour les index des collections et de l'annuaire).
private let foldingLocale = Locale(identifier: "fr_FR")

/// Le texte sans accents, en minuscules, pour chercher (« Œuvre » → « oeuvre »).
func folded(_ s: String) -> String {
    // Le cas le plus courant, un texte tout en ASCII : ni accents ni ligatures, seules les
    // majuscules changent. Le même résultat, sans le pliage de Foundation.
    if s.utf8.allSatisfy({ $0 < 0x80 }) { return s.lowercased() }
    // Les ligatures ne se cherchent (quatre passages sur le texte) que s'il y en a une.
    let ligature = s.unicodeScalars.contains { $0 == "œ" || $0 == "Œ" || $0 == "æ" || $0 == "Æ" }
    let plain = !ligature ? s : s.replacingOccurrences(of: "œ", with: "oe")
        .replacingOccurrences(of: "Œ", with: "oe")
        .replacingOccurrences(of: "æ", with: "ae")
        .replacingOccurrences(of: "Æ", with: "ae")
    return plain
        .folding(options: [.diacriticInsensitive, .caseInsensitive], locale: foldingLocale)
        .lowercased()
}

/// La clé d'un artiste : un même artiste porte des noms différents selon les musées
/// (« Rembrandt (Rembrandt van Rijn) », « Rembrandt van Rijn », « Rembrandt » ; « Titian »
/// et « Titien »). Même règle que le site (outils/monde/explorer.js, cleNom), son pied de
/// page (outils/monde_web.py, cle_nom) et la même table de variantes
/// (outils/monde/alias.json, vérifiée par les tests).
enum ArtistKey {
    static let aliases: [String: String] = [
        "alphonse marie mucha": "alfons mucha",
        "alphonse mucha": "alfons mucha",
        "ando hiroshige": "utagawa hiroshige",
        "anthony van dyck": "antoine van dyck",
        "canaletto": "giovanni antonio canal dit canaletto",
        "caravaggio": "le caravage",
        "domenikos theotokopoulos": "le greco",
        "duccio": "duccio di buoninsegna",
        "el greco": "le greco",
        "francisco de goya y lucientes": "francisco de goya",
        "francisco jose de goya y lucientes": "francisco de goya",
        "gian lorenzo bernini": "le bernin",
        "giotto di bondone": "giotto",
        "giovanni antonio canal": "giovanni antonio canal dit canaletto",
        "goya": "francisco de goya",
        "hans holbein the younger": "hans holbein le jeune",
        "hieronymus bosch": "jerome bosch",
        "hilaire germain edgar degas": "edgar degas",
        "jacopo tintoretto": "le tintoret",
        "jan vermeer": "johannes vermeer",
        "jean baptiste simeon chardin": "jean simeon chardin",
        "leonardo da vinci": "leonard de vinci",
        "lucas cranach the elder": "lucas cranach l ancien",
        "michelangelo": "michel ange",
        "michelangelo buonarroti": "michel ange",
        "michelangelo merisi da caravaggio": "le caravage",
        "peter paul rubens": "pierre paul rubens",
        "pieter bruegel": "pieter bruegel l ancien",
        "pieter bruegel the elder": "pieter bruegel l ancien",
        "raffaello sanzio": "raphael",
        "rembrandt van rijn": "rembrandt",
        "sandro botticelli alessandro filipepi": "sandro botticelli",
        "sir anthony van dyck": "antoine van dyck",
        "sir peter paul rubens": "pierre paul rubens",
        "tintoretto": "le tintoret",
        "titian": "titien",
        "tiziano vecellio": "titien",
        "vasily kandinsky": "vassily kandinsky",
        "veronese": "paolo veronese",
        "wassily kandinsky": "vassily kandinsky",
    ]

    static func of(_ name: String) -> String {
        var n = name.trimmingCharacters(in: .whitespaces)
        if n.hasSuffix(")"), let open = n.firstIndex(of: "("), open > n.startIndex {
            let head = n[..<open].trimmingCharacters(in: .whitespaces)
            let inner = String(n[n.index(after: open)..<n.index(before: n.endIndex)])
            if !head.isEmpty, !inner.isEmpty {
                let first = folded(head).split(separator: " ").first.map(String.init) ?? ""
                n = folded(inner).contains(first) ? inner : head
            }
        }
        let cleaned = String(folded(n).map { "-‐–'’.,".contains($0) ? " " : $0 })
        let key = cleaned.split(whereSeparator: \.isWhitespace).joined(separator: " ")
        return aliases[key] ?? key
    }
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
    /// Le plan, en mètres (x vers l'est, z vers le sud) : le centre de la salle, sa
    /// largeur et sa profondeur. Absents d'un catalogue plus ancien.
    var x: Double?
    var z: Double?
    var width: Double?
    var depth: Double?
    /// La couleur des murs (« #e8e2d6 »).
    var walls: String?
    /// Son numéro dans le parcours (pas pour un couloir).
    var number: Int?
    /// Les portes : des segments [x1, z1, x2, z2].
    var doors: [[Double]]?

    /// Le rectangle de la salle sur le plan, s'il est connu.
    var frame: (minX: Double, minZ: Double, maxX: Double, maxZ: Double)? {
        guard let x, let z, let width, let depth else { return nil }
        return (x - width / 2, z - depth / 2, x + width / 2, z + depth / 2)
    }
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
    var sections: [(title: String, text: String)] { sections(in: .current) }

    /// Les parties rédigées, titrées dans une langue.
    func sections(in language: AppLanguage) -> [(title: String, text: String)] {
        [
            (language.pick("Ce que l'on voit", "What we see", "Lo que vemos"), description),
            (language.pick("Ce qu'elle évoque", "What it evokes", "Lo que evoca"), evocation),
            (language.pick("Son histoire", "Its history", "Su historia"), story),
            (language.pick("La manière", "The technique", "La técnica"), manner),
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
    /// Sa place sur le plan : [x1, z1, x2, z2] (un tableau le long de son mur) ou
    /// [x, z, rayon] (le socle d'une sculpture).
    var spot: [Double]?

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
    /// L'entrée du bâtiment : [x, z, dx, dz] (le point et la direction vers l'intérieur).
    var entrance: [Double]?

    /// Le plan est-il connu (catalogue récent) ?
    var hasPlan: Bool { rooms.contains { $0.frame != nil } }

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
