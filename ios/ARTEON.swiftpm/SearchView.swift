import SwiftUI

/// Chercher une œuvre dans tout le musée : titre, artiste, technique, collection, année.
struct SearchView: View {
    let catalogue: Catalogue
    @Binding var query: String

    var body: some View {
        let results = catalogue.search(query).map { (key: $0.0.key($0.1), exhibition: $0.0, work: $0.1) }
        ScrollView {
            if query.trimmingCharacters(in: .whitespaces).isEmpty {
                ContentUnavailableView(
                    "Chercher dans tout le musée",
                    systemImage: "magnifyingglass",
                    description: Text("Un titre, un artiste, une technique, un musée, une année…")
                )
                .padding(.top, 60)
            } else if results.isEmpty {
                ContentUnavailableView.search(text: query)
                    .padding(.top, 60)
            } else {
                VStack(alignment: .leading, spacing: 12) {
                    Text(results.count == 1 ? "1 œuvre" : "\(results.count) œuvres")
                        .foregroundStyle(.secondary)
                    WorkGrid(items: results.map { ($0.key, $0.exhibition, $0.work) })
                }
                .padding()
            }
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle("Rechercher")
        .searchable(text: $query, prompt: "Monet, estampe, Louvre, 1889…")
    }
}

/// Les œuvres mises de côté avec le cœur.
struct FavoritesView: View {
    let catalogue: Catalogue
    @Environment(CatalogueStore.self) private var store

    var body: some View {
        let items = store.favorites.compactMap { key in
            catalogue.work(key).map { (key, $0.0, $0.1) }
        }
        ScrollView {
            if items.isEmpty {
                ContentUnavailableView(
                    "Pas encore de favoris",
                    systemImage: "heart",
                    description: Text("Touchez le cœur d'une œuvre pour la retrouver ici.")
                )
                .padding(.top, 60)
            } else {
                WorkGrid(items: items).padding()
            }
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle("Favoris")
    }
}

/// Des œuvres de toutes les expositions, en grille.
struct WorkGrid: View {
    let items: [(WorkKey, Exhibition, Work)]

    var body: some View {
        LazyVGrid(
            columns: [GridItem(.adaptive(minimum: 170, maximum: 260), spacing: 14)],
            spacing: 14
        ) {
            ForEach(items, id: \.0) { item in
                NavigationLink(value: Route.work(item.0)) {
                    WorkCard(work: item.2, subtitle: "\(item.2.artist) · \(item.1.name)")
                }
                .buttonStyle(.plain)
            }
        }
    }
}
