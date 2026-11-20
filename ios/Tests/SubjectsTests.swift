import Foundation
import Testing
@testable import ARTEON

/// Les styles, les genres, les techniques et les sujets : la liste du site et ce qu'il publie.
@Suite struct SubjectsTests {
    @Test func familiesAndSubjectsAreRead() throws {
        let json = """
        {"requetes": {}, "familles": [
          {"cle": "styles", "fr": "Styles et mouvements", "en": "Styles and movements", "es": "Estilos y movimientos",
           "sujets": [{"cle": "impressionnisme", "id": "wd:Q40415", "fr": "Impressionnisme", "en": "Impressionism", "es": "Impresionismo"},
                      {"cle": "baroque", "id": "wd:Q37853", "fr": "Baroque"}]},
          {"cle": "genres", "fr": "Genres", "en": "Genres", "es": "Géneros", "sujets": []}
        ]}
        """
        let families = SubjectFamily.parse(Data(json.utf8))
        #expect(families.map(\.key) == ["styles", "genres"])
        #expect(families[0].subjects.map(\.key) == ["impressionnisme", "baroque"])
        #expect(families[0].subjects[0].en == "Impressionism")
        // Sans traduction, le nom français sert.
        #expect(families[0].subjects[1].es == "Baroque")
        #expect(SubjectFamily.parse(Data("[]".utf8)).isEmpty)
    }

    @Test func publishedSubjectsAreRead() throws {
        let json = #"{"sujets": {"baroque": {"total": 2476, "image": "https://commons.wikimedia.org/wiki/Special:FilePath/a.jpg?width=250"}, "vide": {"image": null}}}"#
        let published = PublishedSubject.parse(Data(json.utf8))
        #expect(published["baroque"]?.total == 2476)
        #expect(published["baroque"]?.image.hasSuffix("width=250") == true)
        #expect(published["vide"] == nil)
    }

    @Test func aSubjectCatalogueGivesEachWorkItsArtist() throws {
        let json = #"{"cle": "baroque", "total": 1, "musees": {"wikidata": 1}, "manquants": [], "oeuvres": [{"id": "wikidata-http://www.wikidata.org/entity/Q185372", "titre": "La Jeune Fille à la perle", "artiste": "Johannes Vermeer", "date": "1665", "annee": 1665, "technique": "", "musee": "wikidata", "vignette": "https://x/v.jpg", "image": "https://x/g.jpg", "lien": "http://www.wikidata.org/entity/Q185372", "fournisseur": "Mauritshuis", "rang": 60}]}"#
        let catalogue = try JSONDecoder().decode(SubjectCatalogue.self, from: Data(json.utf8))
        #expect(catalogue.total == 1)
        #expect(catalogue.works.first?.artist == "Johannes Vermeer")
        #expect(catalogue.works.first?.provider == "Mauritshuis")
    }
}
