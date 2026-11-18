// Les langues de l'application : celle de l'appareil (le français par défaut), le catalogue
// et les pages du site dans cette langue, les fichiers communs sans la langue.

import Foundation
import Testing

@testable import ARTEON

@Test func laLangueSuitCelleDeLAppareil() {
    #expect(AppLanguage.resolve(["fr-FR"]) == .fr)
    #expect(AppLanguage.resolve(["en-GB", "fr-FR"]) == .en)
    #expect(AppLanguage.resolve(["es-419"]) == .es)
    #expect(AppLanguage.resolve(["ES_es"]) == .es)
    #expect(AppLanguage.resolve(["en"]) == .en)
    // Une autre langue : la suivante qu'ARTEON parle, sinon le français.
    #expect(AppLanguage.resolve(["de-DE", "en-US"]) == .en)
    #expect(AppLanguage.resolve(["de-DE"]) == .fr)
    #expect(AppLanguage.resolve(["zh-Hans-CN", "ja-JP"]) == .fr)
    #expect(AppLanguage.resolve([]) == .fr)
}

@Test func leCatalogueEtLesPagesSontDansLaLangue() {
    #expect(AppLanguage.fr.catalogueFile == "catalogue.json")
    #expect(AppLanguage.en.catalogueFile == "en/catalogue.json")
    #expect(AppLanguage.es.catalogueFile == "es/catalogue.json")
    let base = "https://maxlestage.github.io/ARTEON/"
    #expect(Site.page("van-gogh/galerie/", in: .fr).absoluteString == base + "van-gogh/galerie/")
    #expect(Site.page("van-gogh/galerie/", in: .en).absoluteString == base + "en/van-gogh/galerie/")
    #expect(Site.page("van-gogh/oeuvres/tournesols/", in: .es).absoluteString == base + "es/van-gogh/oeuvres/tournesols/")
    // Les fichiers (images, données) restent communs à toutes les langues.
    #expect(Site.url("tableaux/van-gogh/tournesols.jpg").absoluteString == base + "tableaux/van-gogh/tournesols.jpg")
    #expect(Site.url("monde/donnees/index.json").absoluteString == base + "monde/donnees/index.json")
}

@Test func lesTextesSontDansLesTroisLangues() {
    #expect(AppLanguage.fr.pick("Favoris", "Favourites", "Favoritos") == "Favoris")
    #expect(AppLanguage.en.pick("Favoris", "Favourites", "Favoritos") == "Favourites")
    #expect(AppLanguage.es.pick("Favoris", "Favourites", "Favoritos") == "Favoritos")
    let works = (("œuvre", "œuvres"), ("work", "works"), ("obra", "obras"))
    #expect(AppLanguage.fr.count(0, works.0, works.1, works.2) == "0 œuvre")
    #expect(AppLanguage.fr.count(1, works.0, works.1, works.2) == "1 œuvre")
    #expect(AppLanguage.fr.count(2, works.0, works.1, works.2) == "2 œuvres")
    #expect(AppLanguage.en.count(0, works.0, works.1, works.2) == "0 works")
    #expect(AppLanguage.en.count(1, works.0, works.1, works.2) == "1 work")
    #expect(AppLanguage.es.count(3, works.0, works.1, works.2) == "3 obras")
    #expect(AppLanguage.en.quote("monet") == "“monet”")
    #expect(CollectionCategory.display("Peinture", in: .en) == "Painting")
    #expect(CollectionCategory.display("Autres objets", in: .es) == "Otros objetos")
    #expect(CollectionCategory.display("Peinture", in: .fr) == "Peinture")
    #expect(CollectionCategory.display("Inconnue", in: .en) == "Inconnue")
}
