import SwiftUI

enum AppTab: Hashable {
    case exhibitions, favorites, search
}

/// Les trois onglets : en barre latérale sur iPad, en bas sur iPhone.
struct MainView: View {
    let catalogue: Catalogue

    @State private var tab: AppTab = .exhibitions
    @State private var exhibitionsPath = NavigationPath()
    @State private var favoritesPath = NavigationPath()
    @State private var searchPath = NavigationPath()
    @State private var query = ""

    var body: some View {
        TabView(selection: $tab) {
            Tab("Expositions", systemImage: "building.columns", value: AppTab.exhibitions) {
                NavigationStack(path: $exhibitionsPath) {
                    HomeView(catalogue: catalogue)
                        .navigationDestination(for: Route.self) { RouteView(route: $0, catalogue: catalogue) }
                }
            }
            Tab("Favoris", systemImage: "heart", value: AppTab.favorites) {
                NavigationStack(path: $favoritesPath) {
                    FavoritesView(catalogue: catalogue)
                        .navigationDestination(for: Route.self) { RouteView(route: $0, catalogue: catalogue) }
                }
            }
            Tab(value: AppTab.search, role: .search) {
                NavigationStack(path: $searchPath) {
                    SearchView(catalogue: catalogue, query: $query)
                        .navigationDestination(for: Route.self) { RouteView(route: $0, catalogue: catalogue) }
                }
            }
        }
        .tabViewStyle(.sidebarAdaptable)
        .task { openFromLaunchArguments() }
    }

    /// `-ouvrir van-gogh` ou `-ouvrir van-gogh/tournesols` au lancement : ouvre cette page
    /// (pour les captures d'écran de la CI, et les liens).
    private func openFromLaunchArguments() {
        let args = ProcessInfo.processInfo.arguments
        guard let i = args.firstIndex(of: "-ouvrir"), i + 1 < args.count else { return }
        let target = args[i + 1]
        if let key = WorkKey(target), catalogue.work(key) != nil {
            exhibitionsPath.append(Route.exhibition(key.exhibition))
            exhibitionsPath.append(Route.work(key))
        } else if catalogue.exhibition(target) != nil {
            exhibitionsPath.append(Route.exhibition(target))
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
