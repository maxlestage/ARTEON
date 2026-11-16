// Les index du musée, calculés une fois à partir du catalogue : tous les artistes de
// toutes les expositions, la chronologie par siècle, les thèmes, les collections, l'œuvre
// du jour et les questions du jeu. Rien que Foundation : ce fichier se teste aussi sous
// Linux.

import Foundation

// MARK: - Les dates

/// L'année d'une date écrite comme sur un cartel : « 1888 », « vers 2090 av. J.-C. »,
/// « 1615–1705 », « XIIᵉ siècle », « IIIᵉ siècle – 1899 ». `nil` quand il n'y en a pas.
func yearValue(_ text: String) -> Int? {
    let lower = text.lowercased()
    if lower.contains("milliard") || lower.contains("million") {
        return -1_000_000
    }
    let before = lower.contains("av. j.-c") || lower.contains("av. j-c") || lower.contains("avant j")
    let chars = Array(text)

    // Le premier nombre écrit en chiffres, et sa place.
    var number: Int?
    var numberStart = chars.count
    var k = 0
    while k < chars.count {
        if chars[k].isASCII, chars[k].isNumber {
            var digits = ""
            numberStart = k
            while k < chars.count, chars[k].isASCII, chars[k].isNumber {
                digits.append(chars[k])
                k += 1
            }
            number = Int(digits)
            break
        }
        k += 1
    }

    // Un siècle en chiffres romains (« XIIᵉ », « IXe siècle »), s'il vient avant.
    if let century = romanCentury(in: chars), century.offset < numberStart {
        let middle = (century.value - 1) * 100 + 50
        return before ? -middle : middle
    }
    guard let number else { return nil }
    return before ? -number : number
}

/// Le premier siècle écrit en chiffres romains suivi de « ᵉ » (ou « e », « er »), et sa
/// place dans le texte.
private func romanCentury(in chars: [Character]) -> (value: Int, offset: Int)? {
    let values: [Character: Int] = ["I": 1, "V": 5, "X": 10, "L": 50, "C": 100]
    var start = 0
    while start < chars.count {
        guard values[chars[start]] != nil, start == 0 || !chars[start - 1].isLetter else {
            start += 1
            continue
        }
        var end = start
        while end < chars.count, values[chars[end]] != nil { end += 1 }
        var total = 0
        var previous = 0
        for c in chars[start..<end].reversed() {
            let v = values[c]!
            total += v < previous ? -v : v
            previous = max(previous, v)
        }
        // Le suffixe : « ᵉ », « ᵉʳ », « e », « er » (pour le premier siècle seulement).
        var after = end
        if after < chars.count, chars[after] == "ᵉ" || chars[after] == "e" {
            after += 1
            if after < chars.count, chars[after] == "ʳ" || chars[after] == "r" {
                after = total == 1 ? after + 1 : chars.count + 1
            }
            let ends = after == chars.count || (after < chars.count && !chars[after].isLetter)
            if ends, total > 0, total <= 30 { return (total, start) }
        }
        start = end
    }
    return nil
}

/// « XIXᵉ siècle », « Vᵉ siècle av. J.-C. », « Avant l'histoire ».
func centuryLabel(_ year: Int) -> String {
    if year <= -100_000 { return "Avant l'histoire" }
    let before = year < 0
    let n = before ? (-year - 1) / 100 + 1 : (year - 1) / 100 + 1
    let roman = romanNumeral(max(n, 1))
    let suffix = n == 1 ? "ᵉʳ" : "ᵉ"
    return "\(roman)\(suffix) siècle" + (before ? " av. J.-C." : "")
}

func romanNumeral(_ n: Int) -> String {
    let table: [(Int, String)] = [
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"), (90, "XC"),
        (50, "L"), (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
    ]
    var n = n
    var out = ""
    for (value, letters) in table {
        while n >= value {
            out += letters
            n -= value
        }
    }
    return out
}

/// Le rang d'un siècle pour le tri (`centuryLabel` d'une année).
func centuryRank(_ year: Int) -> Int {
    if year <= -100_000 { return Int.min / 2 }
    return year < 0 ? -((-year - 1) / 100 + 1) : (year - 1) / 100 + 1
}

// MARK: - Les index

/// Un artiste et toutes ses œuvres, dans toutes les expositions.
struct ArtistEntry: Identifiable, Hashable, Sendable {
    var name: String
    var life: String
    var style: String
    var biography: String
    var works: [WorkKey]
    var exhibitions: [String]
    /// Les années de ses œuvres (pour trouver ses contemporains).
    var years: [Int] = []

    var id: String { name }

    /// La lettre de l'index (« É » → « E »).
    var initial: String {
        let first = folded(name).first.map(String.init)?.uppercased() ?? "#"
        return first.first?.isLetter == true ? first : "#"
    }
}

/// Un groupe d'œuvres sous un titre (un siècle, un thème, une collection).
struct WorkGroup: Identifiable, Hashable, Sendable {
    var title: String
    var works: [WorkKey]

    var id: String { title }
}

/// Les index du musée, calculés une fois pour toutes.
struct MuseumIndex: Sendable {
    let artists: [ArtistEntry]
    let centuries: [WorkGroup]
    let undated: [WorkKey]
    let themes: [WorkGroup]
    let collections: [WorkGroup]
    let techniques: [WorkGroup]

    init(_ catalogue: Catalogue) {
        // Les artistes : ceux des notices et ceux des cartels.
        var byName: [String: ArtistEntry] = [:]
        var order: [String] = []
        for ex in catalogue.exhibitions {
            for a in ex.artists {
                if var entry = byName[a.name] {
                    if entry.biography.count < a.biography.count {
                        entry.biography = a.biography
                        entry.life = a.life.isEmpty ? entry.life : a.life
                        entry.style = a.style.isEmpty ? entry.style : a.style
                    }
                    byName[a.name] = entry
                } else {
                    byName[a.name] = ArtistEntry(
                        name: a.name, life: a.life, style: a.style, biography: a.biography,
                        works: [], exhibitions: []
                    )
                    order.append(a.name)
                }
            }
            for w in ex.works where !w.artist.isEmpty {
                if byName[w.artist] == nil {
                    byName[w.artist] = ArtistEntry(
                        name: w.artist, life: "", style: "", biography: "", works: [], exhibitions: []
                    )
                    order.append(w.artist)
                }
                byName[w.artist]!.works.append(ex.key(w))
                if let y = yearValue(w.year) { byName[w.artist]!.years.append(y) }
                if !byName[w.artist]!.exhibitions.contains(ex.slug) {
                    byName[w.artist]!.exhibitions.append(ex.slug)
                }
            }
        }
        artists = order.compactMap { byName[$0] }
            .filter { !$0.works.isEmpty }
            .sorted { folded($0.name).compare(folded($1.name)) == .orderedAscending }

        // La chronologie.
        var dated: [(year: Int, key: WorkKey)] = []
        var undated: [WorkKey] = []
        for (ex, w) in catalogue.allWorks {
            if let y = yearValue(w.year) {
                dated.append((y, ex.key(w)))
            } else {
                undated.append(ex.key(w))
            }
        }
        dated.sort { $0.year < $1.year }
        var centuries: [WorkGroup] = []
        var lastRank: Int?
        for (year, key) in dated {
            let rank = centuryRank(year)
            if rank != lastRank {
                centuries.append(WorkGroup(title: centuryLabel(year), works: []))
                lastRank = rank
            }
            centuries[centuries.count - 1].works.append(key)
        }
        self.centuries = centuries
        self.undated = undated

        themes = Self.groups(catalogue) { $0.notice.themes }
        collections = Self.groups(catalogue) { w in
            w.collection.isEmpty ? [] : [w.collection]
        }
        techniques = Self.groups(catalogue) { w in
            let t = Self.material(w.technique)
            return t.isEmpty ? [] : [t]
        }
    }

    /// Les groupes d'œuvres qui partagent une valeur, les plus fournis d'abord.
    private static func groups(_ catalogue: Catalogue, _ values: (Work) -> [String]) -> [WorkGroup] {
        var map: [String: [WorkKey]] = [:]
        var display: [String: String] = [:]
        for (ex, w) in catalogue.allWorks {
            for value in values(w) {
                let clean = value.trimmingCharacters(in: .whitespacesAndNewlines)
                guard !clean.isEmpty else { continue }
                let id = folded(clean)
                display[id] = display[id] ?? clean
                if map[id]?.contains(ex.key(w)) != true {
                    map[id, default: []].append(ex.key(w))
                }
            }
        }
        return map.map { WorkGroup(title: display[$0.key] ?? $0.key, works: $0.value) }
            .sorted {
                $0.works.count != $1.works.count
                    ? $0.works.count > $1.works.count
                    : folded($0.title) < folded($1.title)
            }
    }

    /// « Huile sur toile, 73 × 92 cm » → « Huile sur toile ».
    static func material(_ technique: String) -> String {
        var t = technique
        if let i = t.firstIndex(where: { $0 == "," || $0 == ";" || $0 == "(" }) {
            t = String(t[..<i])
        }
        t = t.trimmingCharacters(in: .whitespacesAndNewlines)
        // Pas de mesures seules (« 73 × 92 cm »).
        if t.first?.isNumber == true { return "" }
        guard let first = t.first else { return "" }
        return first.uppercased() + t.dropFirst()
    }

    func artist(named name: String) -> ArtistEntry? {
        artists.first { $0.name == name }
    }

    /// Les artistes dont le nom contient la recherche.
    func searchArtists(_ query: String) -> [ArtistEntry] {
        let q = folded(query.trimmingCharacters(in: .whitespacesAndNewlines))
        guard !q.isEmpty else { return [] }
        return artists.filter { folded("\($0.name) \($0.style)").contains(q) }
    }
}

extension Catalogue {
    /// L'œuvre du jour : la même toute la journée, une autre le lendemain.
    func workOfTheDay(_ date: Date = Date(), calendar: Calendar = .current) -> (Exhibition, Work)? {
        let works = allWorks.filter { $0.1.kind == .painting }
        guard !works.isEmpty else { return nil }
        let day = calendar.ordinality(of: .day, in: .era, for: date) ?? 0
        // Un pas premier avec le nombre d'œuvres : on fait le tour sans répétition.
        let index = (day &* 7919) % works.count
        return works[(index + works.count) % works.count]
    }

    /// Les autres œuvres d'un artiste, ailleurs dans le musée.
    func works(by artist: String, except key: WorkKey? = nil) -> [(Exhibition, Work)] {
        guard !artist.isEmpty else { return [] }
        return allWorks.filter { $0.1.artist == artist && $0.0.key($0.1) != key }
    }

    /// Les œuvres de ces clés, dans leur ordre.
    func works(_ keys: [WorkKey]) -> [(WorkKey, Exhibition, Work)] {
        keys.compactMap { key in work(key).map { (key, $0.0, $0.1) } }
    }
}

// MARK: - Le jeu : « Qui l'a fait ? »

struct QuizQuestion: Hashable, Sendable {
    var work: WorkKey
    var answer: String
    var choices: [String]
}

enum Quiz {
    /// Une question : une œuvre et quatre artistes, dont le bon. Seules comptent les
    /// œuvres d'artistes nommés (pas « Atelier anonyme »).
    static func question(
        from catalogue: Catalogue,
        index: MuseumIndex,
        using rng: inout some RandomNumberGenerator
    ) -> QuizQuestion? {
        let artists = index.artists.map(\.name).filter(isNamed)
        guard artists.count >= 4 else { return nil }
        let works = catalogue.allWorks.filter { isNamed($0.1.artist) }
        guard let (ex, work) = works.randomElement(using: &rng) else { return nil }
        var choices = [work.artist]
        // Des artistes de la même époque, de préférence : la question est moins facile.
        let year = yearValue(work.year)
        let near = index.artists.filter { entry in
            guard isNamed(entry.name), entry.name != work.artist else { return false }
            guard let year else { return true }
            return entry.years.contains { abs($0 - year) < 120 }
        }.map(\.name)
        var pool = (near.count >= 3 ? near : artists.filter { $0 != work.artist }).shuffled(using: &rng)
        while choices.count < 4, let next = pool.popLast() {
            if !choices.contains(next) { choices.append(next) }
        }
        guard choices.count == 4 else { return nil }
        return QuizQuestion(work: ex.key(work), answer: work.artist, choices: choices.shuffled(using: &rng))
    }

    /// Un artiste nommé, pas un atelier ni une culture.
    static func isNamed(_ name: String) -> Bool {
        let f = folded(name)
        let generic = ["anonyme", "atelier", "ateliers", "epoque", "dynastie", "empire", "royaume",
                       "manufacture", "fours", "tradition", "style", "moines", "scriptorium", "faiencerie",
                       "maison ", "mines", "gisement", "formation", "falaises", "recifs", "forets", "coulees",
                       "massif", "schistes", "orfevres", "tisserands", "couturieres", "armuriers", "peintres-",
                       "enlumineurs", "tapissiers", "mosaistes", "ceramistes", "studios"]
        if generic.contains(where: { f.contains($0) }) { return false }
        // Une culture ou un lieu (« Japon, époque d'Edo », « Inde du Sud »).
        return !name.contains(",") && name.split(separator: " ").count >= 2
    }
}

/// Un générateur de nombres pseudo-aléatoires reproductible (tests, tirages du jour).
struct SplitMix64: RandomNumberGenerator, Sendable {
    var state: UInt64

    init(seed: UInt64) { state = seed }

    mutating func next() -> UInt64 {
        state &+= 0x9E37_79B9_7F4A_7C15
        var z = state
        z = (z ^ (z >> 30)) &* 0xBF58_476D_1CE4_E5B9
        z = (z ^ (z >> 27)) &* 0x94D0_49BB_1331_11EB
        return z ^ (z >> 31)
    }
}

// MARK: - L'audioguide

extension Work {
    /// Le texte que lit l'audioguide : le cartel, puis la notice.
    func spokenText(in exhibition: Exhibition) -> String {
        var parts = ["\(title)."]
        if !artist.isEmpty { parts.append("\(artist).") }
        if !year.isEmpty { parts.append("\(year).") }
        if !note.isEmpty { parts.append(note) }
        for section in notice.sections {
            parts.append("\(section.title). \(section.text)")
        }
        if !notice.details.isEmpty {
            parts.append("À regarder de près. " + notice.details.joined(separator: " "))
        }
        if !notice.anecdote.isEmpty {
            parts.append("Le saviez-vous ? \(notice.anecdote)")
        }
        return parts.joined(separator: "\n")
    }
}

// MARK: - Le musée embarqué

/// Le site complet d'ARTEON, copié dans l'application (`Musee/` : catalogue, images,
/// pages et galerie 3D en WebAssembly) : tout se lit sans Internet.
enum LocalMuseum {
    /// Le schéma des adresses servies depuis l'application à la galerie 3D.
    static let scheme = "arteon"
    static let host = "musee"

    /// Le dossier `Musee` de l'application, s'il contient le catalogue : à la racine de
    /// l'application (Xcode) ou dans son paquet de ressources (Swift Playgrounds).
    static let root: URL? = {
        let fm = FileManager.default
        var places: [URL] = []
        for bundle in [Bundle.main] + Bundle.allBundles {
            guard let resources = bundle.resourceURL else { continue }
            places.append(resources)
            let inside = (try? fm.contentsOfDirectory(at: resources, includingPropertiesForKeys: nil)) ?? []
            for url in inside where url.pathExtension == "bundle" || url.pathExtension == "resources" {
                places.append(url)
                places.append(url.appendingPathComponent("Contents/Resources"))
            }
        }
        for place in places {
            let museum = place.appendingPathComponent("Musee", isDirectory: true)
            if fm.fileExists(atPath: museum.appendingPathComponent("catalogue.json").path) {
                return museum
            }
        }
        return nil
    }()

    static var isAvailable: Bool { root != nil }

    /// Le fichier d'un chemin du site (`tableaux/van-gogh/iris.jpg`, `van-gogh/galerie/`).
    static func file(_ path: String) -> URL? {
        root.flatMap { file(path, in: $0) }
    }

    /// Le fichier d'un chemin du site dans `root`, s'il existe : `/` et les dossiers
    /// donnent leur `index.html` ; rien ne sort du dossier.
    static func file(_ path: String, in root: URL) -> URL? {
        var clean = path.removingPercentEncoding ?? path
        if let i = clean.firstIndex(where: { $0 == "?" || $0 == "#" }) {
            clean = String(clean[..<i])
        }
        let parts = clean.split(separator: "/").map(String.init)
        guard !parts.contains(where: { $0 == ".." || $0 == "." }) else { return nil }
        var url = root
        for part in parts { url.appendPathComponent(part) }
        var isDirectory: ObjCBool = false
        guard FileManager.default.fileExists(atPath: url.path, isDirectory: &isDirectory) else {
            return nil
        }
        if isDirectory.boolValue {
            let index = url.appendingPathComponent("index.html")
            return FileManager.default.fileExists(atPath: index.path) ? index : nil
        }
        return url
    }

    /// L'adresse servie à la galerie 3D (`arteon://musee/van-gogh/galerie/`).
    static func url(_ path: String) -> URL {
        var trimmed = path
        while trimmed.hasPrefix("/") { trimmed.removeFirst() }
        return URL(string: "\(scheme)://\(host)/\(trimmed)")!
    }

    /// Le type MIME d'un fichier du site.
    static func mimeType(_ url: URL) -> String {
        switch url.pathExtension.lowercased() {
        case "html", "htm": "text/html; charset=utf-8"
        case "js", "mjs": "text/javascript; charset=utf-8"
        case "css": "text/css; charset=utf-8"
        case "json", "webmanifest": "application/json; charset=utf-8"
        case "wasm": "application/wasm"
        case "jpg", "jpeg": "image/jpeg"
        case "png": "image/png"
        case "webp": "image/webp"
        case "gif": "image/gif"
        case "svg": "image/svg+xml"
        case "ico": "image/x-icon"
        case "txt": "text/plain; charset=utf-8"
        case "woff2": "font/woff2"
        case "woff": "font/woff"
        default: "application/octet-stream"
        }
    }

    /// La taille du musée embarqué, en octets.
    static func size() -> Int64 {
        guard let root,
              let files = FileManager.default.enumerator(
                  at: root, includingPropertiesForKeys: [.fileSizeKey], options: []
              )
        else { return 0 }
        var total: Int64 = 0
        for case let url as URL in files {
            total += Int64((try? url.resourceValues(forKeys: [.fileSizeKey]))?.fileSize ?? 0)
        }
        return total
    }
}
