// Le catalogue que publie le site (un extrait : Van Gogh et Monet) se lit en entier, et la
// recherche trouve les œuvres. Ces tests tournent sur le simulateur iPad (Xcode) et aussi
// sous Linux (le modèle n'utilise que Foundation).

import Foundation
import Testing

@testable import ARTEON

private func fixture() throws -> Catalogue {
    let url = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent()
        .appendingPathComponent("Fixtures/catalogue.json")
    return try Catalogue.decode(Data(contentsOf: url))
}

@Test func leCatalogueDuSiteSeLitEnEntier() throws {
    let c = try fixture()
    #expect(c.exhibitions.map(\.slug).contains("van-gogh"))
    let vanGogh = try #require(c.exhibition("van-gogh"))
    #expect(!vanGogh.works.isEmpty)
    #expect(!vanGogh.rooms.isEmpty)
    #expect(vanGogh.coverImage?.hasPrefix("tableaux/") == true)

    let tournesols = try #require(vanGogh.works.first { $0.slug == "tournesols" })
    #expect(tournesols.artist == "Vincent van Gogh")
    #expect(tournesols.notice.sections.count == 4)
    #expect(!tournesols.notice.details.isEmpty)
    #expect(tournesols.aspect > 0.5 && tournesols.aspect < 1.2)
    #expect(tournesols.credit != nil)

    // Chaque œuvre est dans une salle, et les salles les rendent toutes.
    for ex in c.exhibitions {
        #expect(ex.worksByRoom.reduce(0) { $0 + $1.works.count } == ex.works.count)
    }
    // Les sculptures de Monet sont des sculptures.
    let monet = try #require(c.exhibition("monet"))
    #expect(monet.works.contains { $0.kind == .sculpture })
    // La couleur de l'exposition.
    let rgb = vanGogh.accentRGB
    #expect((0...1).contains(rgb.red) && (0...1).contains(rgb.blue))
}

@Test func laRechercheIgnoreAccentsEtMajuscules() throws {
    let c = try fixture()
    #expect(folded("Éléphant ŒUVRE") == "elephant oeuvre")
    #expect(c.search("TOURNESOL").contains { $0.1.slug == "tournesols" })
    #expect(c.search("nympheas").contains { $0.0.slug == "monet" })
    #expect(c.search("   ").isEmpty)
}

@Test func uneOeuvreSeRetrouveParSaCle() throws {
    let c = try fixture()
    let key = try #require(WorkKey("van-gogh/tournesols"))
    #expect(c.work(key)?.1.title == "Les Tournesols")
    #expect(WorkKey("sans-barre") == nil)
    #expect(key.text == "van-gogh/tournesols")
}

@Test func unCatalogueTropRecentEstRefuse() {
    let json = Data(#"{"version": 99, "exhibitions": []}"#.utf8)
    #expect(throws: CatalogueError.tooNew(99)) {
        try Catalogue.decode(json)
    }
}

@MainActor
@Test func lesFavorisSeGardentSurLAppareil() throws {
    let defaults = try #require(UserDefaults(suiteName: "arteon-tests-\(UUID().uuidString)"))
    let store = CatalogueStore(defaults: defaults)
    let key = WorkKey(exhibition: "van-gogh", work: "tournesols")
    #expect(!store.isFavorite(key))
    store.toggleFavorite(key)
    #expect(store.isFavorite(key))
    // Relus au lancement suivant.
    #expect(CatalogueStore(defaults: defaults).isFavorite(key))
    store.toggleFavorite(key)
    #expect(!CatalogueStore(defaults: defaults).isFavorite(key))
}

@Test func lesImagesSeTrouventSurLeSite() {
    #expect(
        Site.url("tableaux/van-gogh/tournesols.jpg").absoluteString
            == "https://maxlestage.github.io/ARTEON/tableaux/van-gogh/tournesols.jpg"
    )
    #expect(Site.url("catalogue.json").lastPathComponent == "catalogue.json")
}
