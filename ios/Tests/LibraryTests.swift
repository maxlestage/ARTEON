// Les index du musée (artistes, chronologie, thèmes, collections), le jeu, l'audioguide et
// le client de l'API. Sous Linux comme sur le simulateur.

import Foundation
import Testing

@testable import ARTEON

private func fixture() throws -> Catalogue {
    let url = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent()
        .appendingPathComponent("Fixtures/catalogue.json")
    return try Catalogue.decode(Data(contentsOf: url))
}

/// Le vrai catalogue, tel que l'API le sert (le site ARTEON), quand le réseau répond.
private let live: Catalogue? = {
    guard let data = try? Data(contentsOf: URL(string: "https://maxlestage.github.io/ARTEON/catalogue.json")!)
    else { return nil }
    return try? Catalogue.decode(data)
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
}

/// Le vrai musée, tel que l'API le sert : les index couvrent tout.
@Test(.enabled(if: live != nil))
func leVraiMuseeEnLigneEstComplet() throws {
    let c = try #require(live)
    #expect(c.exhibitions.count >= 50)
    #expect(c.workCount >= 500)

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

@Test func lesMuseesSeRangentParPays() throws {
    #expect(Building.country(of: "Paris, France (Jardin des Plantes)") == "France")
    #expect(Building.country(of: "Rochester, New York, États-Unis") == "États-Unis")
    #expect(Building.country(of: "Bombay (Mumbai), Inde") == "Inde")
    let c = try fixture()
    let index = MuseumIndex(c)
    // Chaque exposition qui a un bâtiment est dans un musée, une seule fois.
    let hosted = index.buildings.flatMap(\.exhibitions)
    #expect(hosted.count == c.exhibitions.filter { $0.museum != nil }.count)
    #expect(Set(hosted).count == hosted.count)
    #expect(index.countries.flatMap(\.buildings).count == index.buildings.count)
    // Le Grand Musée réunit toutes les expositions.
    #expect(index.wings.flatMap(\.exhibitions).sorted() == c.exhibitions.map(\.slug).sorted())
}

/// Le vrai catalogue porte le plan de chaque bâtiment.
@Test(.enabled(if: live != nil))
func lesPlansDesBatimentsSontLa() throws {
    let c = try #require(live)
    for ex in c.exhibitions {
        #expect(ex.hasPlan, "plan de \(ex.slug)")
        #expect(ex.entrance?.count == 4, "entrée de \(ex.slug)")
        #expect(ex.works.allSatisfy { [3, 4].contains($0.spot?.count ?? 0) }, "places de \(ex.slug)")
        #expect(ex.rooms.filter { !$0.corridor }.allSatisfy { $0.number != nil })
    }
    let index = MuseumIndex(c)
    #expect(index.buildings.count >= 45)
    #expect(index.building(named: "Musée d'Orsay")?.exhibitions.count == 2)
    #expect(index.countries.first?.name == "France")
    #expect(index.wings.count == 6)
    #expect(index.wings.first?.exhibitions.contains("tresors-de-la-terre") == true)
    #expect(index.wings.last?.exhibitions.contains("abstraction") == true)
}

// MARK: - Les collections du monde

private func worldFixture() throws -> WorldCollections {
    let url = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent()
        .appendingPathComponent("Fixtures/collections.json")
    return try WorldCollections.decode(Data(contentsOf: url))
}

@Test func lesCollectionsDuMondeSeLisent() throws {
    let c = try worldFixture()
    #expect(c.museums.count == 4)
    #expect(c.works.count == 12)
    let index = CollectionIndex(c)
    for museum in c.museums {
        #expect(index.works(.museum(museum.key)).count == 3)
        #expect(index.count(.museum(museum.key)) == 3)
    }
    let first = try #require(c.works.first)
    #expect(index.work(first.id)?.title == first.title)
    #expect(index.works(.artist(first.artist)).contains { $0.id == first.id })
    #expect(index.works(.search(first.title)).contains { $0.id == first.id })
    #expect(index.works(.search("   ")).isEmpty)
    #expect(index.categories.reduce(0) { $0 + $1.count } == 12)
    #expect(index.centuries.map(\.rank) == index.centuries.map(\.rank).sorted())
    #expect(index.picks(5, day: 3).map(\.id) == index.picks(5, day: 3).map(\.id))
    #expect(first.key.exhibition == WorldCollections.favoritePrefix)
    #expect(c.works.allSatisfy { $0.imageURL?.hasPrefix("https://") == true })
}

/// Les collections telles que le site les publie : l'index, la liste, les vignettes.
@Test func lesCollectionsDuSiteSeLisent() throws {
    let index = Data("""
    {"date":"2026-11-18","total":3,"parFichier":1000,
     "musees":[{"cle":"chicago","nom":"Art Institute of Chicago","ville":"Chicago","couleur":"#b5121b","licence":"CC0","oeuvres":2},
               {"cle":"met","nom":"Metropolitan Museum","ville":"New York","couleur":"#e4002b","licence":"CC0","oeuvres":1}],
     "ids":["chicago-1","chicago-2","met-3"]}
    """.utf8)
    let list = Data("""
    [["Les Iris","Vincent van Gogh","1889",1889,"Peinture",0,0.8],
     ["Nymphéas","Claude Monet","",null,"Peinture",0,1.2],
     ["La Vague","Katsushika Hokusai","vers 1831",1831,"Estampe",1,0.7,"https://images.metmuseum.org/v.jpg"]]
    """.utf8)
    let c = try WorldCollections.fromSite(index: index, list: list)
    #expect(c.museums.map(\.key) == ["chicago", "met"])
    #expect(c.museums.first?.site == "https://www.artic.edu")
    #expect(c.works.count == 3)
    #expect(c.works[0].thumbURL?.hasSuffix("monde/v/chicago-1.webp") == true)
    #expect(c.works[2].thumbURL == "https://images.metmuseum.org/v.jpg")
    #expect(c.works[1].date == nil && c.works[1].year == nil)
    #expect(c.works[2].museum == "met" && c.works[2].position == 2)
    let ix = CollectionIndex(c)
    #expect(ix.works(.artist("Claude Monet")).count == 1)
    #expect(ix.works(.search("vague")).first?.id == "met-3")
}

/// Le moteur générique des œuvres en direct : la règle du domaine public et les noms.
@Test func lesOeuvresEnDirectSuiventLaRegle() {
    #expect(LiveWorks.years("1606-07-15T00:00:00Z") == [1606])
    #expect(LiveWorks.years("1485/90–1576") == [1485, 1576])
    #expect(LiveWorks.isFree(date: "1888", end: 1888, death: 1890, birth: 1853))
    #expect(!LiveWorks.isFree(date: "1937", end: nil, death: 1940, birth: nil))
    #expect(!LiveWorks.isFree(date: "1907", end: 1907, death: 1973, birth: 1881))
    #expect(!LiveWorks.isFree(date: "1920", end: nil, death: nil, birth: 1902))
    #expect(LiveWorks.life("Dutch, 1606–1669").birth == 1606)
    #expect(LiveWorks.life("Dutch, 1606–1669").death == 1669)
    #expect(LiveWorks.sameArtist("Vilhelm Hammershøi", "Hammershøi, Vilhelm"))
    #expect(LiveWorks.sameArtist("Monet", "Claude Monet"))
    #expect(LiveWorks.sameArtist("Titien", "Titian"))
    #expect(!LiveWorks.sameArtist("Rembrandt", "Rembrandt Peale"))
    #expect(!LiveWorks.sameArtist("Claude Monet", "Follower of Claude Monet"))
    let record: [String: Any] = ["production": [["creator": "Rembrandt van Rijn"]], "public_domain": true]
    #expect(LiveWorks.text(LiveWorks.path(record, "production.0.creator")) == "Rembrandt van Rijn")
    #expect(LiveWorks.year(1669) == 1669 && LiveWorks.year(0) == nil && LiveWorks.year("9999") == nil)
}

/// La clé des artistes : la même table de variantes que le site, et les mêmes réunions.
@Test func lesNomsDUnMemeArtisteSontReunis() throws {
    let url = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        .appendingPathComponent("outils/monde/alias.json")
    let table = try JSONDecoder().decode([String: String].self, from: Data(contentsOf: url))
    #expect(table == ArtistKey.aliases)
    #expect(ArtistKey.of("Rembrandt (Rembrandt van Rijn)") == ArtistKey.of("Rembrandt"))
    #expect(ArtistKey.of("Rembrandt van Rijn") == "rembrandt")
    #expect(ArtistKey.of("Titian") == ArtistKey.of("Titien"))
    #expect(ArtistKey.of("Edouard Manet") == ArtistKey.of("Édouard Manet"))
    #expect(ArtistKey.of("Jacques Louis David") == ArtistKey.of("Jacques-Louis David"))
    #expect(ArtistKey.of("Henri Rousseau (le Douanier)") == "henri rousseau")
    #expect(ArtistKey.of("Goya (Francisco de Goya y Lucientes)") == "francisco de goya")
}
