// Les index du musée (artistes, chronologie, thèmes, collections), le jeu, l'audioguide et
// le musée embarqué. Sous Linux comme sur le simulateur.

import Foundation
import Testing

@testable import ARTEON

private func fixture() throws -> Catalogue {
    let url = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent()
        .appendingPathComponent("Fixtures/catalogue.json")
    return try Catalogue.decode(Data(contentsOf: url))
}

/// Le musée embarqué (ios/ARTEON.swiftpm/Musee), s'il a été préparé.
private let embedded: URL? = {
    let url = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent()
        .deletingLastPathComponent()
        .appendingPathComponent("ARTEON.swiftpm/Musee")
    return FileManager.default.fileExists(atPath: url.appendingPathComponent("catalogue.json").path) ? url : nil
}()

@Test func lesDatesDesCartelsSeLisent() {
    #expect(yearValue("1888") == 1888)
    #expect(yearValue("vers 1503-1519") == 1503)
    #expect(yearValue("1615–1705") == 1615)
    #expect(yearValue("vers 2090 av. J.-C.") == -2090)
    #expect(yearValue("XIIᵉ siècle") == 1150)
    #expect(yearValue("IXe siècle") == 850)
    #expect(yearValue("Iᵉʳ siècle av. J.-C.") == -50)
    #expect(yearValue("IIIᵉ siècle – 1899") == 250)
    #expect(yearValue("Vers 1500") == 1500)
    #expect(yearValue("Il y a 4,5 milliards d'années") == -1_000_000)
    #expect(yearValue("") == nil)
    #expect(yearValue("date inconnue") == nil)
}

@Test func lesSieclesSEcriventEnChiffresRomains() {
    #expect(centuryLabel(1888) == "XIXᵉ siècle")
    #expect(centuryLabel(1900) == "XIXᵉ siècle")
    #expect(centuryLabel(1901) == "XXᵉ siècle")
    #expect(centuryLabel(50) == "Iᵉʳ siècle")
    #expect(centuryLabel(-2090) == "XXIᵉ siècle av. J.-C.")
    #expect(centuryLabel(-1_000_000) == "Avant l'histoire")
    #expect(romanNumeral(1494) == "MCDXCIV")
    #expect(centuryRank(-1_000_000) < centuryRank(-2090))
    #expect(centuryRank(-50) < centuryRank(50))
}

@Test func lIndexReunitArtistesSieclesEtThemes() throws {
    let c = try fixture()
    let index = MuseumIndex(c)
    let vanGogh = try #require(index.artist(named: "Vincent van Gogh"))
    #expect(!vanGogh.biography.isEmpty)
    #expect(vanGogh.works.count == c.exhibition("van-gogh")?.works.count)
    #expect(vanGogh.exhibitions == ["van-gogh"])
    // Chaque œuvre est une fois dans la chronologie (ou sans date).
    let placed = index.centuries.reduce(0) { $0 + $1.works.count } + index.undated.count
    #expect(placed == c.workCount)
    // La chronologie va du plus ancien au plus récent.
    let years = index.centuries.flatMap(\.works).compactMap { c.work($0).flatMap { yearValue($0.1.year) } }
    #expect(years == years.sorted())
    // Les thèmes et les collections, les plus fournis d'abord.
    #expect(!index.themes.isEmpty)
    #expect(zip(index.themes, index.themes.dropFirst()).allSatisfy { $0.works.count >= $1.works.count })
    #expect(!index.collections.isEmpty)
    #expect(MuseumIndex.material("Huile sur toile, 73 × 92 cm") == "Huile sur toile")
    #expect(MuseumIndex.material("73 × 92 cm") == "")
    #expect(index.searchArtists("GOGH").first?.name == "Vincent van Gogh")
}

@Test func laRechercheCombineLesMots() throws {
    let c = try fixture()
    let both = c.search("gogh tournesols")
    #expect(both.first?.1.slug == "tournesols")
    #expect(c.search("gogh nympheas").isEmpty)
    #expect(c.searchExhibitions("monet").contains { $0.slug == "monet" })
}

@Test func lOeuvreDuJourChangeChaqueJour() throws {
    let c = try fixture()
    var calendar = Calendar(identifier: .gregorian)
    calendar.timeZone = TimeZone(identifier: "UTC")!
    let day = Date(timeIntervalSince1970: 1_800_000_000)
    let today = try #require(c.workOfTheDay(day, calendar: calendar))
    #expect(c.workOfTheDay(day.addingTimeInterval(3600), calendar: calendar)?.1 == today.1)
    var seen = Set<String>()
    for d in 0..<10 {
        if let w = c.workOfTheDay(day.addingTimeInterval(Double(d) * 86_400), calendar: calendar) {
            seen.insert(w.0.key(w.1).text)
        }
    }
    #expect(seen.count > 5)
}

@Test func leJeuProposeQuatreArtistesDontLeBon() throws {
    let c = try fixture()
    let index = MuseumIndex(c)
    var rng = SplitMix64(seed: 42)
    for _ in 0..<20 {
        guard let q = Quiz.question(from: c, index: index, using: &rng) else {
            // L'extrait n'a peut-être pas quatre artistes nommés.
            #expect(index.artists.filter { Quiz.isNamed($0.name) }.count < 4)
            return
        }
        #expect(q.choices.count == 4)
        #expect(Set(q.choices).count == 4)
        #expect(q.choices.contains(q.answer))
        #expect(c.work(q.work)?.1.artist == q.answer)
    }
    #expect(Quiz.isNamed("Claude Monet"))
    #expect(!Quiz.isNamed("Japon, époque d'Edo"))
    #expect(!Quiz.isNamed("Atelier anonyme"))
}

@Test func lAudioguideLitLeCartelPuisLaNotice() throws {
    let c = try fixture()
    let ex = try #require(c.exhibition("van-gogh"))
    let work = try #require(ex.works.first { $0.slug == "tournesols" })
    let text = work.spokenText(in: ex)
    #expect(text.hasPrefix("Les Tournesols."))
    #expect(text.contains("Ce que l'on voit."))
    #expect(text.contains(work.notice.description))
}

@MainActor
@Test func lHistoriqueGardeLesDernieresOeuvres() throws {
    let defaults = try #require(UserDefaults(suiteName: "arteon-tests-\(UUID().uuidString)"))
    let store = CatalogueStore(defaults: defaults)
    for i in 0..<50 {
        store.visited(WorkKey(exhibition: "x", work: "w\(i)"))
    }
    store.visited(WorkKey(exhibition: "x", work: "w10"))
    #expect(store.history.count == 40)
    #expect(store.history.first?.work == "w10")
    #expect(store.history.filter { $0.work == "w10" }.count == 1)
    #expect(CatalogueStore(defaults: defaults).history.first?.work == "w10")
    store.quizBest = 7
    #expect(CatalogueStore(defaults: defaults).quizBest == 7)
    store.updatesEnabled = false
    #expect(CatalogueStore(defaults: defaults).updatesEnabled == false)
}

@Test func leMuseeEmbarqueSertSesFichiers() throws {
    let root = FileManager.default.temporaryDirectory.appendingPathComponent("musee-\(UUID().uuidString)")
    defer { try? FileManager.default.removeItem(at: root) }
    let fm = FileManager.default
    try fm.createDirectory(at: root.appendingPathComponent("van-gogh/galerie"), withIntermediateDirectories: true)
    try fm.createDirectory(at: root.appendingPathComponent("tableaux/van-gogh"), withIntermediateDirectories: true)
    try Data("<html>".utf8).write(to: root.appendingPathComponent("van-gogh/galerie/index.html"))
    try Data("jpg".utf8).write(to: root.appendingPathComponent("tableaux/van-gogh/nuit étoilée.jpg"))

    #expect(LocalMuseum.file("/van-gogh/galerie/", in: root)?.lastPathComponent == "index.html")
    #expect(LocalMuseum.file("van-gogh/galerie", in: root)?.lastPathComponent == "index.html")
    #expect(LocalMuseum.file("tableaux/van-gogh/nuit%20%C3%A9toil%C3%A9e.jpg", in: root) != nil)
    #expect(LocalMuseum.file("van-gogh/galerie/#nuit", in: root) != nil)
    #expect(LocalMuseum.file("tableaux/van-gogh/absente.jpg", in: root) == nil)
    #expect(LocalMuseum.file("van-gogh", in: root) == nil)
    #expect(LocalMuseum.file("../etc/passwd", in: root) == nil)
    #expect(LocalMuseum.url("/van-gogh/galerie/").absoluteString == "arteon://musee/van-gogh/galerie/")
    #expect(LocalMuseum.mimeType(URL(fileURLWithPath: "/a/arteon_bg.wasm")) == "application/wasm")
    #expect(LocalMuseum.mimeType(URL(fileURLWithPath: "/a/index.html")).hasPrefix("text/html"))
}

/// Le vrai musée embarqué, quand ios/preparer-musee.sh l'a rempli : chaque œuvre a son
/// image, chaque exposition sa galerie, et les index couvrent tout.
@Test(.enabled(if: embedded != nil))
func leVraiMuseeEmbarqueEstComplet() throws {
    let root = try #require(embedded)
    let c = try Catalogue.decode(Data(contentsOf: root.appendingPathComponent("catalogue.json")))
    #expect(c.exhibitions.count >= 50)
    #expect(c.workCount >= 500)
    for ex in c.exhibitions {
        #expect(LocalMuseum.file(ex.gallery + "/", in: root) != nil, "galerie de \(ex.slug)")
        #expect(LocalMuseum.file("donnees/\(ex.slug).bin", in: root) != nil, "données de \(ex.slug)")
        for w in ex.works {
            #expect(LocalMuseum.file(w.image, in: root) != nil, "image de \(ex.slug)/\(w.slug)")
        }
    }
    #expect(LocalMuseum.file("pkg/arteon_bg.wasm", in: root) != nil)

    let index = MuseumIndex(c)
    #expect(index.artists.count > 300)
    // Presque toutes les œuvres ont une date lisible.
    #expect(index.undated.count * 20 < c.workCount, "sans date : \(index.undated.map(\.text))")
    #expect(index.centuries.count >= 20)
    var rng = SplitMix64(seed: 7)
    for _ in 0..<200 {
        let q = try #require(Quiz.question(from: c, index: index, using: &rng))
        #expect(Set(q.choices).count == 4 && q.choices.contains(q.answer))
    }
    #expect(c.search("nuit etoilee").contains { $0.1.slug == "nuit-etoilee" })
}
