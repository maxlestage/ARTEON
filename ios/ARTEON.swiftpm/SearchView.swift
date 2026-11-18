import SwiftUI

/// Chercher dans tout le musée : œuvres (titre, artiste, technique, collection, année,
/// thèmes), artistes et expositions.
struct SearchView: View {
    let catalogue: Catalogue
    @Binding var query: String

    @Environment(CatalogueStore.self) private var store

    var body: some View {
        let trimmed = query.trimmingCharacters(in: .whitespaces)
        let works = catalogue.search(query).map { ($0.0.key($0.1), $0.0, $0.1) }
        let artists = store.index?.searchArtists(query) ?? []
        let exhibitions = catalogue.searchExhibitions(query)
        let world = trimmed.isEmpty ? [] : (store.collections?.works(.search(trimmed), limit: 61) ?? [])
        ScrollView {
            if trimmed.isEmpty {
                VStack(alignment: .leading, spacing: 16) {
                    ContentUnavailableView(
                        "Chercher dans tout le musée",
                        systemImage: "magnifyingglass",
                        description: Text("Un titre, un artiste, un thème, une technique, un musée, une année…")
                    )
                    .padding(.top, 30)
                    if let themes = store.index?.themes.prefix(18), !themes.isEmpty {
                        Text("Idées")
                            .font(.headline)
                        FlowLayout {
                            ForEach(Array(themes)) { theme in
                                Button(theme.title) { query = theme.title }
                                    .buttonStyle(.glass)
                            }
                        }
                    }
                }
                .padding()
            } else if works.isEmpty && artists.isEmpty && exhibitions.isEmpty && world.isEmpty {
                ContentUnavailableView.search(text: query)
                    .padding(.top, 60)
            } else {
                VStack(alignment: .leading, spacing: 14) {
                    if !exhibitions.isEmpty {
                        Text(exhibitions.count == 1 ? "1 exposition" : "\(exhibitions.count) expositions")
                            .font(.headline)
                        ScrollView(.horizontal, showsIndicators: false) {
                            LazyHStack(spacing: 14) {
                                ForEach(exhibitions) { ex in
                                    NavigationLink(value: Route.exhibition(ex.slug)) {
                                        ExhibitionCard(exhibition: ex).frame(width: 280)
                                    }
                                    .buttonStyle(.plain)
                                }
                            }
                        }
                        .scrollClipDisabled()
                    }
                    if !artists.isEmpty {
                        Text(artists.count == 1 ? "1 artiste" : "\(artists.count) artistes")
                            .font(.headline)
                        VStack(spacing: 0) {
                            ForEach(artists.prefix(12)) { artist in
                                NavigationLink(value: Route.artist(artist.name)) {
                                    ArtistListRow(catalogue: catalogue, artist: artist)
                                        .padding(.horizontal)
                                        .padding(.vertical, 8)
                                        .contentShape(.rect)
                                }
                                .buttonStyle(.plain)
                                Divider().padding(.leading, 80)
                            }
                        }
                        .background(Color.card, in: .rect(cornerRadius: 18))
                    }
                    if !works.isEmpty {
                        Text(works.count == 1 ? "1 œuvre dans les expositions" : "\(works.count) œuvres dans les expositions")
                            .font(.headline)
                        WorkGrid(items: works, showExhibition: true)
                    }
                    if !world.isEmpty {
                        HStack {
                            Text(world.count > 60 ? "Dans les collections du monde" : (world.count == 1 ? "1 œuvre dans les collections du monde" : "\(world.count) œuvres dans les collections du monde"))
                                .font(.headline)
                            Spacer()
                            if world.count > 60 {
                                NavigationLink("Tout voir", value: Route.collection(.search(trimmed)))
                            }
                        }
                        .padding(.top, 8)
                        CollectionGrid(works: Array(world.prefix(60)))
                    }
                }
                .padding()
            }
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle("Rechercher")
        .searchable(text: $query, prompt: "Monet, nuit, estampe, Louvre, 1889…")
    }
}

/// Les œuvres mises de côté avec le cœur, et les dernières regardées.
struct FavoritesView: View {
    let catalogue: Catalogue
    @Environment(CatalogueStore.self) private var store
    @State private var slideshow = false

    var body: some View {
        let favorites = catalogue.works(store.favorites)
        let history = catalogue.works(store.history)
        let worldFavorites = store.favorites.compactMap { key in
            key.exhibition == WorldCollections.favoritePrefix ? store.collections?.work(key.work) : nil
        }
        let worldHistory = store.history.compactMap { key in
            key.exhibition == WorldCollections.favoritePrefix ? store.collections?.work(key.work) : nil
        }
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if favorites.isEmpty && worldFavorites.isEmpty {
                    ContentUnavailableView(
                        "Pas encore de favoris",
                        systemImage: "heart",
                        description: Text("Touchez le cœur d'une œuvre pour la retrouver ici.")
                    )
                    .padding(.top, 40)
                } else {
                    WorkGrid(items: favorites, showExhibition: true)
                    if !worldFavorites.isEmpty {
                        Text("Dans les collections du monde")
                            .font(.headline)
                            .padding(.top, 8)
                        CollectionGrid(works: worldFavorites)
                    }
                }
                if !history.isEmpty || !worldHistory.isEmpty {
                    HStack {
                        Text("Vues récemment")
                            .font(.title2.bold())
                        Spacer()
                        Button("Effacer", role: .destructive) {
                            withAnimation { store.clearHistory() }
                        }
                        .font(.subheadline)
                    }
                    .padding(.top, 20)
                    WorkGrid(items: Array(history.prefix(24)), showExhibition: true)
                    if !worldHistory.isEmpty {
                        CollectionGrid(works: Array(worldHistory.prefix(24)))
                    }
                }
            }
            .padding()
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle("Favoris")
        .toolbar {
            if !favorites.isEmpty {
                ToolbarItem(placement: .primaryAction) {
                    Button("Diaporama", systemImage: "play.rectangle") { slideshow = true }
                }
            }
        }
        .fullScreenCover(isPresented: $slideshow) {
            SlideshowView(title: "Mes favoris", items: favorites.map { ($0.1, $0.2) })
        }
    }
}

/// Des œuvres de toutes les expositions, en grille.
struct WorkGrid: View {
    let items: [(WorkKey, Exhibition, Work)]
    var showExhibition = false

    var body: some View {
        LazyVGrid(
            columns: [GridItem(.adaptive(minimum: 160, maximum: 260), spacing: 14)],
            spacing: 14
        ) {
            ForEach(items, id: \.0) { item in
                NavigationLink(value: Route.work(item.0)) {
                    WorkCard(
                        work: item.2,
                        subtitle: showExhibition
                            ? [item.2.artist, item.1.name].filter { !$0.isEmpty }.joined(separator: " · ")
                            : nil
                    )
                }
                .buttonStyle(.plain)
            }
        }
    }
}
