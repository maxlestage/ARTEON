// swift-tools-version: 6.0

// Les tests du modèle d'ARTEON sans appareil Apple : `swift test` dans ios/, sous Linux
// comme sur un Mac. Seuls les fichiers qui n'utilisent que Foundation sont compilés ici
// (le catalogue, les index, le client de l'API) ; l'application entière se compile avec
// Xcode (project.yml) ou Swift Playgrounds (ARTEON.swiftpm).

import PackageDescription

let package = Package(
    name: "ARTEONModele",
    targets: [
        .target(
            name: "ARTEON",
            path: "ARTEON.swiftpm",
            exclude: [
                "Musee", "Assets.xcassets", "Package.swift", "ARTEONApp.swift", "ArtistsView.swift",
                "ExhibitionView.swift", "ExploreView.swift", "GalleryView.swift", "HomeView.swift",
                "MainView.swift", "Narrator.swift", "FloorPlanView.swift", "MuseumsView.swift", "CollectionsView.swift", "QuizView.swift", "SearchView.swift",
                "SettingsView.swift", "SlideshowView.swift", "Theme.swift", "WorkView.swift", "LiveGallery.swift",
            ],
            sources: ["Models.swift", "Library.swift", "CatalogueStore.swift", "Collections.swift", "Api.swift", "SessionStore.swift", "Language.swift"]
        ),
        .testTarget(
            name: "ARTEONTests",
            dependencies: ["ARTEON"],
            path: "Tests",
            exclude: ["Fixtures"]
        ),
    ],
    swiftLanguageModes: [.v6]
)
