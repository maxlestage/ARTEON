import SwiftUI

/// Un artiste de la liste : celui d'une exposition (avec sa biographie), celui des
/// collections du monde, ou celui de l'annuaire (tous les artistes dont un musée montre une
/// œuvre) ; réunis par leur clé (`ArtistKey`).
struct ArtistItem: Identifiable, Hashable, Sendable {
    let id: String
    let name: String
    let line: String
    let museumWorks: Int
    let worldWorks: Int
    /// Les œuvres que les musées montrent ailleurs (l'annuaire).
    let elsewhere: Int
    /// Le nom sans accents ni majuscules (le tri, la recherche), et sa lettre.
    let sort: String
    let initial: String

    var total: Int { max(museumWorks + worldWorks, elsewhere) }

    init(id: String, name: String, line: String, museumWorks: Int, worldWorks: Int, elsewhere: Int = 0, sort: String? = nil) {
        self.id = id
        self.name = name
        self.line = line
        self.museumWorks = museumWorks
        self.worldWorks = worldWorks
        self.elsewhere = elsewhere
        let key = sort ?? folded(name)
        self.sort = key
        let first = key.first { $0.isLetter || $0.isNumber }.map(String.init)?.uppercased() ?? "#"
        self.initial = first.first?.isLetter == true && first.first?.isASCII == true ? first : "#"
    }
}

/// Tous les artistes : ceux des expositions, ceux des collections du monde et ceux de
/// l'annuaire, de A à Z, avec le nombre de leurs œuvres. Un nom absent de la liste se cherche
/// tel quel dans tous les musées.
struct ArtistsView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store
    @State private var filter = ""
    @State private var all: [ArtistItem] = []
    /// Les artistes de A à Z, lettre par lettre (calculé une fois : ils sont des dizaines de milliers).
    @State private var letters: [(letter: String, artists: [ArtistItem])] = []

    private var query: String { filter.trimmingCharacters(in: .whitespaces) }

    /// Les artistes trouvés : d'abord par le nom (les plus riches en œuvres en premier), puis
    /// par le style et les dates ; 300 au plus.
    private var found: [ArtistItem] {
        let q = folded(query)
        let byName = all.filter { $0.sort.contains(q) }.sorted { $0.total > $1.total }
        if byName.count >= 300 { return Array(byName.prefix(300)) }
        let names = Set(byName.map(\.id))
        return Array((byName + all.filter { !names.contains($0.id) && ($0.museumWorks > 0 ? folded($0.line) : $0.line).contains(q) }).prefix(300))
    }

    private var sections: [(letter: String, artists: [ArtistItem])] {
        query.count >= 2 ? [("Résultats", found)] : letters
    }

    /// N'importe quel artiste : un nom écrit qui n'est pas dans la liste se cherche dans les musées.
    private var elsewhere: String? {
        let q = folded(query)
        return q.count >= 3 && !all.contains { $0.sort == q } ? query : nil
    }

    var body: some View {
        List {
            if let name = elsewhere {
                Section {
                    NavigationLink(value: Route.artist(name)) {
                        Label {
                            VStack(alignment: .leading, spacing: 2) {
                                Text("Chercher « \(name) » dans tous les musées").font(.headline)
                                Text("Son catalogue, en direct des musées du monde").font(.caption).foregroundStyle(.secondary)
                            }
                        } icon: {
                            Image(systemName: "globe")
                        }
                    }
                }
            }
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
            } else if query.count >= 2, found.isEmpty, elsewhere == nil {
                ContentUnavailableView.search(text: filter)
            }
        }
        .navigationTitle(all.isEmpty ? "Artistes" : "\(all.count.formatted()) artistes")
        .searchable(text: $filter, placement: .navigationBarDrawer(displayMode: .always), prompt: "Un nom, un style, un siècle…")
        .task { await store.loadDirectory() }
        .task(id: "\(store.index?.artists.count ?? 0)-\(store.collections?.count ?? 0)-\(store.directory.count)") {
            all = Self.merge(store.index?.artists ?? [], store.collections, store.directory)
            var result: [(letter: String, artists: [ArtistItem])] = []
            for artist in all {
                if result.last?.letter == artist.initial {
                    result[result.count - 1].artists.append(artist)
                } else {
                    result.append((artist.initial, [artist]))
                }
            }
            letters = result
        }
    }

    /// Les artistes des expositions, des collections et de l'annuaire, réunis par clé, de A à Z.
    static func merge(_ museum: [ArtistEntry], _ world: CollectionIndex?, _ directory: [String: DirectoryArtist] = [:]) -> [ArtistItem] {
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
        for (key, d) in directory {
            if let known = items[key] {
                items[key] = ArtistItem(
                    id: key, name: known.name, line: known.line.hasPrefix("Dans les") && !d.life.isEmpty ? d.life : known.line,
                    museumWorks: known.museumWorks, worldWorks: known.worldWorks, elsewhere: d.works, sort: known.sort
                )
            } else {
                items[key] = ArtistItem(
                    id: key, name: d.name, line: d.life.isEmpty ? "Dans les musées du monde" : d.life,
                    museumWorks: 0, worldWorks: 0, elsewhere: d.works, sort: d.sort
                )
            }
        }
        // Les lettres d'abord (A à Z), puis les noms qui commencent autrement.
        return items.values.sorted { ($0.initial == "#" ? 1 : 0, $0.sort) < ($1.initial == "#" ? 1 : 0, $1.sort) }
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
                    if let life = store.directory[ArtistKey.of(name)]?.life, !life.isEmpty {
                        Text(life).font(.headline).foregroundStyle(.secondary)
                    }
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
