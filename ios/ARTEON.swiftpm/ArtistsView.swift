import SwiftUI

/// Tous les artistes du musée, de A à Z, avec le nombre de leurs œuvres.
struct ArtistsView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store
    @State private var filter = ""

    private var artists: [ArtistEntry] {
        let all = store.index?.artists ?? []
        let q = folded(filter.trimmingCharacters(in: .whitespaces))
        return q.isEmpty ? all : all.filter { folded("\($0.name) \($0.style) \($0.life)").contains(q) }
    }

    private var sections: [(letter: String, artists: [ArtistEntry])] {
        var result: [(String, [ArtistEntry])] = []
        for artist in artists {
            if result.last?.0 == artist.initial {
                result[result.count - 1].1.append(artist)
            } else {
                result.append((artist.initial, [artist]))
            }
        }
        return result
    }

    var body: some View {
        List {
            ForEach(sections, id: \.letter) { section in
                Section(section.letter) {
                    ForEach(section.artists) { artist in
                        NavigationLink(value: Route.artist(artist.name)) {
                            ArtistListRow(catalogue: catalogue, artist: artist)
                        }
                    }
                }
                .sectionIndexLabel(section.letter)
            }
        }
        .listStyle(.insetGrouped)
        .listSectionIndexVisibility(.visible)
        .overlay {
            if artists.isEmpty, !filter.isEmpty {
                ContentUnavailableView.search(text: filter)
            }
        }
        .navigationTitle("Artistes")
        .searchable(text: $filter, placement: .navigationBarDrawer(displayMode: .always), prompt: "Un nom, un style, un siècle…")
    }
}

/// Une ligne de la liste : la vignette d'une œuvre, le nom, les dates, le nombre d'œuvres.
struct ArtistListRow: View {
    let catalogue: Catalogue
    let artist: ArtistEntry

    var body: some View {
        HStack(spacing: 14) {
            RemoteImage(path: artist.works.first.flatMap { catalogue.work($0)?.1.image }, maxPixels: 160)
                .frame(width: 52, height: 52)
                .clipShape(.rect(cornerRadius: 10))
            VStack(alignment: .leading, spacing: 2) {
                Text(artist.name)
                    .font(.headline)
                let line = [artist.life, artist.style].filter { !$0.isEmpty }.joined(separator: " · ")
                if !line.isEmpty {
                    Text(line)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }
            Spacer(minLength: 8)
            Text("\(artist.works.count)")
                .font(.subheadline.monospacedDigit())
                .foregroundStyle(.secondary)
                .accessibilityLabel(artist.works.count == 1 ? "1 œuvre" : "\(artist.works.count) œuvres")
        }
    }
}

/// Un artiste : sa vie, son style, sa biographie, toutes ses œuvres, ses expositions.
struct ArtistView: View {
    let catalogue: Catalogue
    let name: String

    @Environment(CatalogueStore.self) private var store

    var body: some View {
        if let artist = store.index?.artist(named: name) {
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    VStack(alignment: .leading, spacing: 6) {
                        Text(artist.name)
                            .font(.largeTitle.bold())
                        let line = [artist.life, artist.style].filter { !$0.isEmpty }
                        if !line.isEmpty {
                            Text(line.joined(separator: " · "))
                                .font(.headline)
                                .foregroundStyle(.secondary)
                        }
                    }

                    if !artist.biography.isEmpty {
                        HStack(spacing: 12) {
                            ListenButton(
                                id: "artiste/\(artist.name)",
                                text: { "\(artist.name). \(artist.life). \(artist.biography)" },
                                label: "Écouter la biographie"
                            )
                            .buttonStyle(.glass)
                        }
                        Text(artist.biography)
                            .lineSpacing(4)
                            .frame(maxWidth: 760, alignment: .leading)
                            .textSelection(.enabled)
                    }

                    Text(artist.works.count == 1 ? "Son œuvre au musée" : "Ses \(artist.works.count) œuvres au musée")
                        .font(.title2.bold())
                        .padding(.top, 8)
                    WorkGrid(items: catalogue.works(artist.works).map { ($0.0, $0.1, $0.2) }, showExhibition: true)

                    let exhibitions = artist.exhibitions.compactMap(catalogue.exhibition)
                    if !exhibitions.isEmpty {
                        Text(exhibitions.count == 1 ? "L'exposition" : "Les expositions")
                            .font(.title2.bold())
                            .padding(.top, 8)
                        LazyVGrid(
                            columns: [GridItem(.adaptive(minimum: 260, maximum: 420), spacing: 16)],
                            spacing: 16
                        ) {
                            ForEach(exhibitions) { ex in
                                NavigationLink(value: Route.exhibition(ex.slug)) {
                                    ExhibitionCard(exhibition: ex)
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }
                }
                .padding()
                .padding(.bottom, 32)
            }
            .background(Color(uiColor: .systemGroupedBackground))
            .navigationTitle(artist.name)
            .navigationBarTitleDisplayMode(.inline)
        } else {
            ContentUnavailableView("Artiste introuvable", systemImage: "person.slash")
        }
    }
}
