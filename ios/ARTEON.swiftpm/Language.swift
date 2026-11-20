// Les langues de l'application : le français (la langue d'origine), l'anglais et
// l'espagnol, comme le site (src/langue.rs). La langue suit celle de l'appareil (ou celle
// choisie pour ARTEON dans les Réglages d'iOS) ; toute autre langue donne le français.
//
// Les textes de l'interface s'écrivent dans les trois langues côte à côte, comme le `t()`
// du site : `L("Favoris", "Favourites", "Favoritos")`. Rien à empaqueter (ni catalogue de
// chaînes, ni dossiers .lproj) : le même code se compile avec Xcode, avec Swift Playgrounds
// et sous Linux pour les tests. Les données (titres, notices, biographies) arrivent déjà
// traduites dans le catalogue de la langue (`en/catalogue.json`, `es/catalogue.json`).
//
// Rien que Foundation : ce fichier se teste aussi sous Linux.

import Foundation

/// Une langue de l'application.
enum AppLanguage: String, CaseIterable, Sendable {
    case fr, en, es

    /// La langue de l'application, fixée au lancement (iOS relance l'application quand on
    /// change sa langue).
    static let current: AppLanguage = resolve(Locale.preferredLanguages)

    /// La première langue connue parmi les langues préférées (`en-GB`, `es-419`…), le
    /// français sinon.
    static func resolve(_ preferred: [String]) -> AppLanguage {
        for code in preferred {
            let language = code.lowercased().prefix { $0.isLetter }
            if let known = AppLanguage(rawValue: String(language)) { return known }
        }
        return .fr
    }

    /// Le début des adresses des pages du site : rien pour le français, `en/` et `es/`
    /// sinon. Les fichiers communs (images, données, collections) n'en ont pas.
    var pagePrefix: String { self == .fr ? "" : "\(rawValue)/" }

    /// Le catalogue du musée dans cette langue (`catalogue.json`, `en/catalogue.json`).
    var catalogueFile: String { pagePrefix + "catalogue.json" }

    /// La langue de l'audioguide (une voix de l'appareil).
    var speechLanguage: String {
        switch self {
        case .fr: "fr-FR"
        case .en: "en-GB"
        case .es: "es-ES"
        }
    }

    /// Le nom de la langue, dans cette langue.
    var name: String {
        switch self {
        case .fr: "Français"
        case .en: "English"
        case .es: "Español"
        }
    }

    /// La locale des textes (tri, accents).
    var locale: Locale { Locale(identifier: rawValue) }

    /// Le texte de cette langue.
    func pick(_ fr: String, _ en: String, _ es: String) -> String {
        switch self {
        case .fr: fr
        case .en: en
        case .es: es
        }
    }

    /// « 1 œuvre », « 12 œuvres » : le singulier pour 0 et 1 en français, pour 1 seulement
    /// en anglais et en espagnol. Le nombre est écrit à la manière de l'appareil (« 1 234 »).
    func count(_ n: Int, _ fr: (String, String), _ en: (String, String), _ es: (String, String)) -> String {
        let one = self == .fr ? abs(n) <= 1 : n == 1
        let words: (String, String) = switch self {
        case .fr: fr
        case .en: en
        case .es: es
        }
        return "\(n.formatted()) \(one ? words.0 : words.1)"
    }

    /// Une citation : « … » en français, “…” en anglais, «…» en espagnol.
    func quote(_ text: String) -> String {
        switch self {
        case .fr: "« \(text) »"
        case .en: "“\(text)”"
        case .es: "«\(text)»"
        }
    }
}

/// Un texte de l'interface dans la langue de l'application :
/// `L("Favoris", "Favourites", "Favoritos")`.
func L(_ fr: String, _ en: String, _ es: String) -> String {
    AppLanguage.current.pick(fr, en, es)
}

/// Un nombre et son nom, accordé : `Lcount(n, ("œuvre", "œuvres"), ("work", "works"), ("obra", "obras"))`.
func Lcount(_ n: Int, _ fr: (String, String), _ en: (String, String), _ es: (String, String)) -> String {
    AppLanguage.current.count(n, fr, en, es)
}

/// « 12 œuvres », « 12 works », « 12 obras ».
func worksCount(_ n: Int) -> String {
    Lcount(n, ("œuvre", "œuvres"), ("work", "works"), ("obra", "obras"))
}

/// Une citation dans la langue de l'application.
func Lquote(_ text: String) -> String {
    AppLanguage.current.quote(text)
}

extension Site {
    /// L'adresse d'une PAGE du site dans la langue de l'application
    /// (`van-gogh/galerie/` → `…/en/van-gogh/galerie/`). Les fichiers (images, données)
    /// passent par `Site.url`, sans la langue.
    static func page(_ path: String, in language: AppLanguage = .current) -> URL {
        url(language.pagePrefix + path)
    }
}

// MARK: - Les noms des données communes

/// Les catégories des collections du monde (outils/collections_monde.py) restent en
/// français dans les données et les filtres ; seul leur nom affiché change de langue
/// (comme outils/monde/app.js).
enum CollectionCategory {
    static let names: [String: (en: String, es: String)] = [
        "Peinture": ("Painting", "Pintura"), "Dessin": ("Drawing", "Dibujo"), "Estampe": ("Print", "Estampa"),
        "Photographie": ("Photography", "Fotografía"), "Sculpture": ("Sculpture", "Escultura"),
        "Céramique": ("Ceramics", "Cerámica"), "Textile et costume": ("Textiles and costume", "Textiles e indumentaria"),
        "Mobilier": ("Furniture", "Mobiliario"), "Verre": ("Glass", "Vidrio"), "Bijoux": ("Jewellery", "Joyería"),
        "Armes et armures": ("Arms and armour", "Armas y armaduras"),
        "Livres et manuscrits": ("Books and manuscripts", "Libros y manuscritos"),
        "Monnaies et médailles": ("Coins and medals", "Monedas y medallas"),
        "Instruments de musique": ("Musical instruments", "Instrumentos musicales"),
        "Orfèvrerie et métal": ("Metalwork", "Orfebrería y metal"), "Autres objets": ("Other objects", "Otros objetos"),
        // Les catégories plus fines (outils/categories.py).
        "Estampe japonaise": ("Japanese prints", "Estampa japonesa"),
        "Affiche": ("Posters", "Carteles"),
        "Dessin d'architecture": ("Architectural drawings", "Dibujo de arquitectura"),
        "Miniature et enluminure": ("Miniatures and illumination", "Miniatura e iluminación"),
        "Rouleaux et paravents": ("Scrolls and screens", "Rollos y biombos"),
        "Icônes et retables": ("Icons and altarpieces", "Iconos y retablos"),
        "Aquarelle": ("Watercolour", "Acuarela"),
        "Pastel": ("Pastel", "Pastel"),
        "Éventails": ("Fans", "Abanicos"),
        "Masques": ("Masks", "Máscaras"),
        "Vannerie": ("Basketry", "Cestería"),
        "Mosaïques et vitraux": ("Mosaics and stained glass", "Mosaicos y vidrieras"),
        "Horlogerie": ("Clocks and watches", "Relojería"),
        "Art funéraire égyptien": ("Egyptian funerary art", "Arte funerario egipcio"),
        "Sceaux et amulettes": ("Seals and amulets", "Sellos y amuletos"),
        "Tapisseries et tapis": ("Tapestries and carpets", "Tapices y alfombras"),
        "Costume et mode": ("Costume and fashion", "Indumentaria y moda"),
        "Dentelle et broderie": ("Lace and embroidery", "Encaje y bordado"),
        "Vases grecs": ("Greek vases", "Vasos griegos"),
        "Porcelaine": ("Porcelain", "Porcelana"),
        "Laque": ("Lacquer", "Laca"),
        "Ivoire": ("Ivory", "Marfil"),
        "Jade et pierres dures": ("Jade and hardstones", "Jade y piedras duras"),
        "Sculpture en bronze": ("Bronze sculpture", "Escultura en bronce"),
        "Sculpture en marbre et en pierre": ("Marble and stone sculpture", "Escultura en mármol y piedra"),
        "Sculpture en bois": ("Wood sculpture", "Escultura en madera"),
        "Sculpture en terre cuite": ("Terracotta sculpture", "Escultura en terracota"),
    ]

    /// Le nom affiché d'une catégorie (`Peinture` → `Painting`).
    static func display(_ name: String, in language: AppLanguage = .current) -> String {
        guard let names = names[name] else { return name }
        return language.pick(name, names.en, names.es)
    }
}

/// « Artiste inconnu » : la valeur des données (et la clé des recherches), affichée dans la
/// langue de l'application.
func displayArtist(_ name: String) -> String {
    name == "Artiste inconnu" ? L(name, "Unknown artist", "Artista desconocido") : name
}

/// Les villes et licences des musées des collections (`monde/donnees/index.json`), écrites
/// en français.
func localizedCollectionText(_ text: String) -> String {
    guard AppLanguage.current != .fr else { return text }
    return text
        .replacingOccurrences(of: "États-Unis", with: L("États-Unis", "United States", "Estados Unidos"))
        .replacingOccurrences(of: "Images et données", with: L("Images et données", "Images and data", "Imágenes y datos"))
        .replacingOccurrences(of: " ; textes de présentation", with: L(" ; textes de présentation", "; descriptive texts", "; textos de presentación"))
}
