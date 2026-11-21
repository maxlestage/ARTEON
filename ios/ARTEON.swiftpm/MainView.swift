import SwiftUI

enum AppTab: String, Hashable {
    case exhibitions, museums, artists, explore, favorites, search
}

/// Les onglets : en barre latérale sur iPad, en bas sur iPhone. Sur iPhone (largeur
/// compacte), la barre n'a que cinq places : les artistes sont dans Explorer.
struct MainView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store
    @Environment(\.horizontalSizeClass) private var sizeClass
    @State private var tab: AppTab = .exhibitions
    @State private var exhibitionsPath = NavigationPath()
    @State private var museumsPath = NavigationPath()
    @State private var artistsPath = NavigationPath()
    @State private var explorePath = NavigationPath()
    @State private var favoritesPath = NavigationPath()
    @State private var searchPath = NavigationPath()
    @State private var query = ""

    var body: some View {
        TabView(selection: $tab) {
            Tab(L("Expositions", "Exhibitions", "Exposiciones"), systemImage: "photo.artframe", value: AppTab.exhibitions) {
                stack($exhibitionsPath) { HomeView(catalogue: catalogue) }
            }
            Tab(L("Musées", "Museums", "Museos"), systemImage: "building.columns", value: AppTab.museums) {
                stack($museumsPath) { MuseumsView(catalogue: catalogue) }
            }
            if sizeClass != .compact {
                Tab(L("Artistes", "Artists", "Artistas"), systemImage: "person.2", value: AppTab.artists) {
                    stack($artistsPath) { ArtistsView(catalogue: catalogue) }
                }
            }
            Tab(L("Explorer", "Explore", "Explorar"), systemImage: "safari", value: AppTab.explore) {
                stack($explorePath) { ExploreView(catalogue: catalogue) }
            }
            Tab(L("Favoris", "Favourites", "Favoritos"), systemImage: "heart", value: AppTab.favorites) {
                stack($favoritesPath) { FavoritesView(catalogue: catalogue) }
            }
            Tab(value: AppTab.search, role: .search) {
                stack($searchPath) { SearchView(catalogue: catalogue, query: $query) }
            }
        }
        .tabViewStyle(.sidebarAdaptable)
        .onChange(of: sizeClass) {
            // L'onglet Artistes disparaît en largeur compacte : on passe par Explorer.
            if sizeClass == .compact, tab == .artists {
                tab = .explore
                explorePath.append(Route.artists)
            }
        }
        .task { openFromLaunchArguments() }
    }

    private func stack<Content: View>(
        _ path: Binding<NavigationPath>,
        @ViewBuilder content: () -> Content
    ) -> some View {
        NavigationStack(path: path) {
            content()
                .navigationDestination(for: Route.self) { RouteView(route: $0, catalogue: catalogue) }
        }
    }

    /// Au lancement : `-ouvrir van-gogh`, `-ouvrir van-gogh/tournesols`,
    /// `-ouvrir 3d:van-gogh`, `-ouvrir styles`, `-ouvrir style:baroque` ouvrent cette page ;
    /// `-onglet artistes` cet onglet (pour les captures d'écran de la CI).
    private func openFromLaunchArguments() {
        let args = ProcessInfo.processInfo.arguments
        if let i = args.firstIndex(of: "-onglet"), i + 1 < args.count {
            switch args[i + 1] {
            case "musees": tab = .museums
            case "artistes":
                if sizeClass == .compact {
                    tab = .explore
                    explorePath.append(Route.artists)
                } else {
                    tab = .artists
                }
            case "explorer": tab = .explore
            case "favoris": tab = .favorites
            case "recherche": tab = .search
            default: break
            }
        }
        guard let i = args.firstIndex(of: "-ouvrir"), i + 1 < args.count else { return }
        let target = args[i + 1]
        if target.hasPrefix("3d:") {
            let slug = String(target.dropFirst(3))
            if catalogue.exhibition(slug) != nil {
                exhibitionsPath.append(Route.exhibition(slug))
                exhibitionsPath.append(Route.gallery(slug, work: nil))
            }
        } else if target == "jeu" {
            tab = .explore
            explorePath.append(Route.quiz)
        } else if target == "collections" {
            tab = .museums
            museumsPath.append(Route.worldCollections)
        } else if target.hasPrefix("collection:") {
            tab = .museums
            museumsPath.append(Route.worldCollections)
            museumsPath.append(Route.collection(.museum(String(target.dropFirst(11)))))
        } else if target == "grand-musee" {
            tab = .museums
            museumsPath.append(Route.grandMuseum)
        } else if target.hasPrefix("musee:") {
            tab = .museums
            museumsPath.append(Route.building(String(target.dropFirst(6))))
        } else if target == "chronologie" {
            tab = .explore
            explorePath.append(Route.timeline)
        } else if target == "styles" {
            tab = .explore
            explorePath.append(Route.styles)
        } else if target.hasPrefix("style:") {
            // `-ouvrir style:impressionnisme` (et `-voir 3` : sa 4ᵉ œuvre en grand).
            let key = String(target.dropFirst(6))
            tab = .explore
            explorePath.append(Route.styles)
            explorePath.append(Route.subject(key: key, name: key.replacingOccurrences(of: "-", with: " ").capitalized, family: ""))
        } else if let key = WorkKey(target), catalogue.work(key) != nil {
            exhibitionsPath.append(Route.exhibition(key.exhibition))
            exhibitionsPath.append(Route.work(key))
        } else if catalogue.exhibition(target) != nil {
            exhibitionsPath.append(Route.exhibition(target))
        } else if store.index?.artist(named: target) != nil {
            tab = .exhibitions
            exhibitionsPath.append(Route.artist(target))
        }
    }
}

/// La page d'une route.
struct RouteView: View {
    let route: Route
    let catalogue: Catalogue

    var body: some View {
        switch route {
        case .exhibition(let slug):
            if let ex = catalogue.exhibition(slug) {
                ExhibitionView(exhibition: ex)
            } else {
                missing
            }
        case .work(let key):
            if let found = catalogue.work(key) {
                WorkView(exhibition: found.0, work: found.1)
            } else {
                missing
            }
        case .gallery(let slug, let work):
            if let ex = catalogue.exhibition(slug) {
                GalleryView(exhibition: ex, work: work)
            } else {
                missing
            }
        case .artist(let name):
            ArtistView(catalogue: catalogue, name: name)
        case .works(let title, let subtitle, let keys):
            WorkListView(catalogue: catalogue, title: title, subtitle: subtitle, keys: keys)
        case .timeline:
            TimelineView(catalogue: catalogue)
        case .themes:
            GroupsView(catalogue: catalogue, kind: .themes)
        case .collections:
            GroupsView(catalogue: catalogue, kind: .collections)
        case .techniques:
            GroupsView(catalogue: catalogue, kind: .techniques)
        case .styles:
            StylesView()
        case .subject(let key, let name, let family):
            SubjectView(key: key, name: name, family: family)
        case .quiz:
            QuizView(catalogue: catalogue)
        case .settings:
            SettingsView(catalogue: catalogue)
        case .artists:
            ArtistsView(catalogue: catalogue)
        case .worldCollections:
            CollectionsView()
        case .collection(let filter):
            CollectionListView(filter: filter)
        case .collectionWork(let id):
            CollectionWorkView(id: id)
        case .building(let name):
            BuildingView(catalogue: catalogue, name: name)
        case .grandMuseum:
            GrandMuseumView(catalogue: catalogue)
        }
    }

    private var missing: some View {
        ContentUnavailableView(
            L("Introuvable", "Not found", "No encontrado"),
            systemImage: "questionmark.square.dashed",
            description: Text(L("Cette page n'est plus au catalogue.", "This page is no longer in the catalogue.", "Esta página ya no está en el catálogo."))
        )
    }
}
