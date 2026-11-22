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

/// Les catégories découvertes dans Wikidata, et leurs œuvres demandées en direct.
@Suite struct SubjectsLiveTests {
    @Test func countsImagesAndQueriesAreRead() throws {
        let json = #"""
        {"requetes": {"wikidata": {"sujet": "https://qlever.dev/api/wikidata?query={s}", "sujet_repli": "https://query-main.wikidata.org/sparql?format=json&query={s}"}},
         "familles": [{"cle": "motifs", "fr": "Sujets et motifs", "wikidata": "VALUES ?s { {id} } ?oeuvre wdt:P180 ?s .",
           "sujets": [{"cle": "narguile", "id": "wd:Q102151", "fr": "Narguilé", "en": "Hookah", "es": "Narguile", "n": 22, "image": "https://x/a.jpg?width=250"},
                      {"cle": "quai", "id": "wd:Q1", "fr": "Quai", "n": 15}]}]}
        """#
        let families = SubjectFamily.parse(Data(json.utf8))
        #expect(families[0].pattern.contains("{id}"))
        #expect(families[0].subjects[0].count == 22)
        #expect(families[0].subjects[0].ids == "wd:Q102151")
        #expect(families[0].subjects[0].image?.hasSuffix("width=250") == true)
        #expect(families[0].subjects[1].image == nil)
        #expect(SubjectFamily.queries(Data(json.utf8)).count == 2)
    }

    @Test func aLiveAnswerIsRankedAndFiltered() throws {
        let json = #"""
        {"results": {"bindings": [
          {"oeuvre": {"value": "http://www.wikidata.org/entity/Q1"}, "titre": {"value": "Peu connue"}, "image": {"value": "https://c/a.jpg"}, "annee": {"value": "1800"}, "nom": {"value": "A"}, "liens": {"value": "1"}},
          {"oeuvre": {"value": "http://www.wikidata.org/entity/Q2"}, "titre": {"value": "Célèbre"}, "image": {"value": "https://c/b.jpg"}, "annee": {"value": "1880"}, "nom": {"value": "B"}, "liens": {"value": "40"}, "collection": {"value": "Louvre"}},
          {"oeuvre": {"value": "http://www.wikidata.org/entity/Q3"}, "titre": {"value": "Trop récente"}, "image": {"value": "https://c/c.jpg"}, "annee": {"value": "1925"}, "fin": {"value": "1940"}, "liens": {"value": "90"}},
          {"oeuvre": {"value": "http://www.wikidata.org/entity/Q4"}, "titre": {"value": "Sans image"}, "liens": {"value": "99"}}
        ]}}
        """#
        let works = try #require(Subjects.parse(Data(json.utf8)))
        #expect(works.map(\.title) == ["Célèbre", "Peu connue"])
        #expect(works[0].provider == "Louvre")
        #expect(works[0].artist == "B")
        #expect(works[0].thumbnail == "https://c/b.jpg?width=250")
        #expect(works[0].year == 1880)
    }

    @Test func toutesLesCategoriesConfondues() {
        let a = SubjectFamily(key: "styles", fr: "Styles", en: "Styles", es: "Estilos", subjects: [
            Subject(key: "baroque", fr: "Baroque", en: "Baroque", es: "Barroco", count: 50),
            Subject(key: "gothique", fr: "Gothique", en: "Gothic", es: "Gótico", count: 10),
        ])
        let b = SubjectFamily(key: "genres", fr: "Genres", en: "Genres", es: "Géneros", subjects: [
            Subject(key: "portrait", fr: "Portrait", en: "Portrait", es: "Retrato", count: 10),
            Subject(key: "baroque", fr: "Baroque", en: "Baroque", es: "Barroco", count: 50),
            Subject(key: "paysage", fr: "Paysage", en: "Landscape", es: "Paisaje", count: 5),
        ])
        let tout = SubjectFamily.merged([a, b], ["paysage": PublishedSubject(total: 900, image: "")])
        // Chacune une fois, des plus riches aux moins riches (les œuvres publiées d'abord) ; à
        // égalité, l'ordre des familles.
        #expect(tout.map(\.subject.key) == ["paysage", "baroque", "gothique", "portrait"])
        #expect(tout[1].family == "Styles")
    }
}
