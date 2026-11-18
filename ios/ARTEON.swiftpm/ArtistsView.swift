import SwiftUI

/// Un artiste de la liste : celui d'une exposition (avec sa biographie), ou celui des
/// collections du monde seulement ; les deux réunis par leur clé (`ArtistKey`).
struct ArtistItem: Identifiable, Hashable, Sendable {
    let id: String
    let name: String
    let line: String
    let museumWorks: Int
    let worldWorks: Int
    var total: Int { museumWorks + worldWorks }

    var initial: String {
        let first = folded(name).first { $0.isLetter || $0.isNumber }.map(String.init)?.uppercased() ?? "#"
        return first.first?.isLetter == true ? first : "#"
    }
}

/// Tous les artistes : ceux des expositions et ceux des collections du monde, de A à Z,
/// avec le nombre de toutes leurs œuvres.
struct ArtistsView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store
    @State private var filter = ""
    @State private var all: [ArtistItem] = []

    private var artists: [ArtistItem] {
        let q = folded(filter.trimmingCharacters(in: .whitespaces))
        guard q.count >= 2 else { return all }
        let byName = all.filter { folded($0.name).contains(q) }.sorted { $0.total > $1.total }
        let names = Set(byName.map(\.id))
        return byName + all.filter { !names.contains($0.id) && folded($0.line).contains(q) }
    }

    private var sections: [(letter: String, artists: [ArtistItem])] {
        if filter.trimmingCharacters(in: .whitespaces).count >= 2 { return [("Résultats", artists)] }
        var result: [(String, [ArtistItem])] = []
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
            if all.isEmpty {
                ProgressView()
            } else if artists.isEmpty, !filter.isEmpty {
                ContentUnavailableView.search(text: filter)
            }
        }
        .navigationTitle(all.isEmpty ? "Artistes" : "\(all.count.formatted()) artistes")
        .searchable(text: $filter, placement: .navigationBarDrawer(displayMode: .always), prompt: "Un nom, un style, un siècle…")
        .task(id: "\(store.index?.artists.count ?? 0)-\(store.collections?.count ?? 0)") { all = Self.merge(store.index?.artists ?? [], store.collections) }
    }

    /// Les artistes des expositions et des collections, réunis par clé, de A à Z.
    static func merge(_ museum: [ArtistEntry], _ world: CollectionIndex?) -> [ArtistItem] {
        var items: [String: ArtistItem] = [:]
        let counts = Dictionary((world?.artists ?? []).map { ($0.key, $0) }, uniquingKeysWith: { a, _ in a })
        for a in museum where !a.works.isEmpty {
            let key = ArtistKey.of(a.name)
            let previous = items[key]?.museumWorks ?? 0
            items[key] = ArtistItem(
                id: key, name: items[key]?.name ?? a.name,
                line: [a.life, a.style].filter { !$0.isEmpty }.joined(separator: " · "),
                museumWorks: previous + a.works.count, worldWorks: counts[key]?.count ?? 0
            )
        }
        for (key, w) in counts where items[key] == nil {
            items[key] = ArtistItem(id: key, name: w.name, line: "Dans les collections du monde", museumWorks: 0, worldWorks: w.count)
        }
        return items.values.sorted { folded($0.name).localizedStandardCompare(folded($1.name)) == .orderedAscending }
    }
}

/// Une ligne de la liste : la vignette d'une œuvre, le nom, les dates, le nombre d'œuvres.
struct ArtistListRow: View {
    let catalogue: Catalogue
    let artist: ArtistItem

    @Environment(CatalogueStore.self) private var store

    init(catalogue: Catalogue, artist: ArtistItem) {
        self.catalogue = catalogue
        self.artist = artist
    }

    /// Un artiste des expositions (la recherche).
    init(catalogue: Catalogue, artist entry: ArtistEntry) {
        self.init(catalogue: catalogue, artist: ArtistItem(
            id: ArtistKey.of(entry.name), name: entry.name,
            line: [entry.life, entry.style].filter { !$0.isEmpty }.joined(separator: " · "),
            museumWorks: entry.works.count, worldWorks: 0
        ))
    }

    var body: some View {
        HStack(spacing: 14) {
            Group {
                if let entry = store.index?.artist(named: artist.name), let key = entry.works.first {
                    RemoteImage(path: catalogue.work(key)?.1.image, maxPixels: 160)
                } else if let work = store.collections?.firstWork(ofArtist: artist.id) {
                    CollectionThumb(work: work)
                } else {
                    Color.secondary.opacity(0.15)
                }
            }
            .frame(width: 52, height: 52)
            .clipShape(.rect(cornerRadius: 10))
            VStack(alignment: .leading, spacing: 2) {
                Text(artist.name)
                    .font(.headline)
                if !artist.line.isEmpty {
                    Text(artist.line)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }
            Spacer(minLength: 8)
            Text("\(artist.total)")
                .font(.subheadline.monospacedDigit())
                .foregroundStyle(.secondary)
                .accessibilityLabel(artist.total == 1 ? "1 œuvre" : "\(artist.total) œuvres")
        }
    }
}

/// Un artiste : sa vie, son style, sa biographie, toutes ses œuvres, ses expositions.
struct ArtistView: View {
    let catalogue: Catalogue
    let name: String

    @Environment(CatalogueStore.self) private var store

    /// L'artiste des expositions, sous ce nom ou sous une autre forme du même nom.
    private var entry: ArtistEntry? {
        store.index?.artist(named: name)
            ?? store.index?.artists.first { !$0.works.isEmpty && ArtistKey.of($0.name) == ArtistKey.of(name) }
    }

    var body: some View {
        if let artist = entry {
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

                    let world = store.collections?.works(.artist(artist.name)) ?? []
                    let total = artist.works.count + world.count
                    Text(total == 1 ? "Le catalogue complet : 1 œuvre" : "Le catalogue complet : \(total.formatted()) œuvres")
                        .font(.title2.bold())
                        .padding(.top, 8)
                    Text(artist.works.count == 1 ? "Son œuvre dans les expositions" : "Ses \(artist.works.count) œuvres dans les expositions")
                        .font(.headline)
                        .foregroundStyle(.secondary)
                    WorkGrid(items: catalogue.works(artist.works).map { ($0.0, $0.1, $0.2) }, showExhibition: true)

                    if !world.isEmpty {
                        Text(world.count == 1 ? "Son œuvre dans les collections du monde" : "Ses \(world.count.formatted()) œuvres dans les collections du monde")
                            .font(.headline)
                            .foregroundStyle(.secondary)
                            .padding(.top, 8)
                        CollectionGrid(works: world.sorted { ($0.year ?? .max) < ($1.year ?? .max) })
                    }

                    LiveWorksSection(name: artist.name, known: Set(world.map(\.id)))
                        .padding(.top, 8)

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
        } else if let world = store.collections?.works(.artist(name)), !world.isEmpty {
            // Un artiste des collections du monde seulement : tout son catalogue, par date.
            ScrollView {
                VStack(alignment: .leading, spacing: 14) {
                    Text(name)
                        .font(.largeTitle.bold())
                    if let life = world.first?.life, !life.isEmpty {
                        Text(life).font(.headline).foregroundStyle(.secondary)
                    }
                    Text(world.count == 1 ? "Le catalogue complet : 1 œuvre" : "Le catalogue complet : \(world.count.formatted()) œuvres")
                        .font(.title2.bold())
                        .padding(.top, 8)
                    Text("Dans les collections du monde, de la plus ancienne à la plus récente.")
                        .foregroundStyle(.secondary)
                    CollectionGrid(works: world.sorted { ($0.year ?? .max) < ($1.year ?? .max) })
                    LiveWorksSection(name: name, known: Set(world.map(\.id)))
                        .padding(.top, 8)
                }
                .padding()
                .padding(.bottom, 32)
            }
            .background(Color(uiColor: .systemGroupedBackground))
            .navigationTitle(name)
            .navigationBarTitleDisplayMode(.inline)
        } else {
            // Un artiste qu'ARTEON ne montre pas encore : tout son catalogue vient des musées.
            ScrollView {
                VStack(alignment: .leading, spacing: 14) {
                    Text(name)
                        .font(.largeTitle.bold())
                    LiveWorksSection(name: name, known: [])
                }
                .padding()
                .padding(.bottom, 32)
            }
            .background(Color(uiColor: .systemGroupedBackground))
            .navigationTitle(name)
            .navigationBarTitleDisplayMode(.inline)
        }
    }
}

/// Toutes les autres œuvres libres de l'artiste, en direct des musées : par l'API ARTEON
/// (le serveur Paquets), ou directement chez les musées s'il ne répond pas. Rien n'est gardé.
struct LiveWorksSection: View {
    let name: String
    /// Les œuvres déjà montrées (les collections du monde), pour ne pas les répéter.
    let known: Set<String>

    @State private var result: LiveCatalogue?
    @State private var museums: [String: String] = [:]
    @State private var loading = true

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            if loading {
                HStack(spacing: 10) {
                    ProgressView()
                    Text("Recherche de ses autres œuvres dans les musées…")
                        .foregroundStyle(.secondary)
                }
            } else if let result {
                let more = result.works.filter { !known.contains($0.id) }
                Text(more.isEmpty ? "Dans les musées, en direct" : "Dans les musées, en direct : \(more.count.formatted()) œuvres de plus")
                    .font(.title2.bold())
                if more.isEmpty {
                    Text("Aucune autre œuvre libre de droits dans les collections ouvertes des musées.")
                        .foregroundStyle(.secondary)
                } else {
                    Text(result.source == "musees"
                         ? "Domaine public (artiste mort en 1955 ou avant, œuvre d'avant 1931) · directement chez les musées"
                         : "Domaine public (artiste mort en 1955 ou avant, œuvre d'avant 1931) · par l'API ARTEON")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    LazyVGrid(columns: [GridItem(.adaptive(minimum: 150, maximum: 240), spacing: 12)], spacing: 12) {
                        ForEach(more) { work in
                            Link(destination: URL(string: work.link) ?? Api.server) {
                                LiveWorkCard(work: work, museum: work.provider.map { "\($0) · \((museums[work.museum] ?? work.museum).components(separatedBy: " (").first ?? "")" } ?? museums[work.museum] ?? work.museum)
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
            } else {
                Text("Les musées n'ont pas répondu.")
                    .foregroundStyle(.secondary)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .task(id: name) {
            loading = true
            async let names = LiveWorks.museumNames()
            result = await LiveWorks.of(name)
            museums = await names
            loading = false
        }
    }
}

/// Une œuvre trouvée en direct : sa vignette (chez le musée), son titre, sa date, son musée.
struct LiveWorkCard: View {
    let work: LiveWork
    let museum: String

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            RemoteImage(path: work.thumbnail, maxPixels: 480)
                .frame(height: 170)
                .frame(maxWidth: .infinity)
                .clipped()
            VStack(alignment: .leading, spacing: 2) {
                Text(work.title)
                    .font(.subheadline.bold())
                    .lineLimit(2)
                Text([work.date, museum].filter { !$0.isEmpty }.joined(separator: " · "))
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            .padding(10)
            .frame(maxWidth: .infinity, minHeight: 62, alignment: .topLeading)
        }
        .card()
        .accessibilityElement(children: .combine)
    }
}
