import SwiftUI

enum AppTab: String, Hashable {
    case exhibitions, artists, explore, favorites, search
}

/// Les cinq onglets : en barre latérale sur iPad, en bas sur iPhone.
struct MainView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store
    @State private var tab: AppTab = .exhibitions
    @State private var exhibitionsPath = NavigationPath()
    @State private var artistsPath = NavigationPath()
    @State private var explorePath = NavigationPath()
    @State private var favoritesPath = NavigationPath()
    @State private var searchPath = NavigationPath()
    @State private var query = ""

    var body: some View {
        TabView(selection: $tab) {
            Tab("Musée", systemImage: "building.columns", value: AppTab.exhibitions) {
                stack($exhibitionsPath) { HomeView(catalogue: catalogue) }
            }
            Tab("Artistes", systemImage: "person.2", value: AppTab.artists) {
                stack($artistsPath) { ArtistsView(catalogue: catalogue) }
            }
            Tab("Explorer", systemImage: "safari", value: AppTab.explore) {
                stack($explorePath) { ExploreView(catalogue: catalogue) }
            }
            Tab("Favoris", systemImage: "heart", value: AppTab.favorites) {
                stack($favoritesPath) { FavoritesView(catalogue: catalogue) }
            }
            Tab(value: AppTab.search, role: .search) {
                stack($searchPath) { SearchView(catalogue: catalogue, query: $query) }
            }
        }
        .tabViewStyle(.sidebarAdaptable)
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
    /// `-ouvrir 3d:van-gogh` ouvrent cette page ; `-onglet artistes` cet onglet (pour les
    /// captures d'écran de la CI).
    private func openFromLaunchArguments() {
        let args = ProcessInfo.processInfo.arguments
        if let i = args.firstIndex(of: "-onglet"), i + 1 < args.count {
            switch args[i + 1] {
            case "artistes": tab = .artists
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
        } else if target == "chronologie" {
            tab = .explore
            explorePath.append(Route.timeline)
        } else if let key = WorkKey(target), catalogue.work(key) != nil {
            exhibitionsPath.append(Route.exhibition(key.exhibition))
            exhibitionsPath.append(Route.work(key))
        } else if catalogue.exhibition(target) != nil {
            exhibitionsPath.append(Route.exhibition(target))
        } else if store.index?.artist(named: target) != nil {
            tab = .artists
            artistsPath.append(Route.artist(target))
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
        case .quiz:
            QuizView(catalogue: catalogue)
        case .settings:
            SettingsView(catalogue: catalogue)
        }
    }

    private var missing: some View {
        ContentUnavailableView(
            "Introuvable",
            systemImage: "questionmark.square.dashed",
            description: Text("Cette page n'est plus au catalogue.")
        )
    }
}
