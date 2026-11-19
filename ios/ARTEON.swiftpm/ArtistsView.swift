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
        query.count >= 2 ? [(L("Résultats", "Results", "Resultados"), found)] : letters
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
                                Text(L("Chercher « \(name) » dans tous les musées", "Search for “\(name)” in all museums", "Buscar «\(name)» en todos los museos")).font(.headline)
                                Text(L("Son catalogue, en direct des musées du monde", "Their catalogue, live from the world's museums", "Su catálogo, en directo desde los museos del mundo")).font(.caption).foregroundStyle(.secondary)
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
        .navigationTitle(all.isEmpty ? L("Artistes", "Artists", "Artistas") : Lcount(all.count, ("artiste", "artistes"), ("artist", "artists"), ("artista", "artistas")))
        .searchable(text: $filter, placement: .navigationBarDrawer(displayMode: .always), prompt: L("Un nom, un style, un siècle…", "A name, a style, a century…", "Un nombre, un estilo, un siglo…"))
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

    /// La ligne d'un artiste des collections du monde, ou de l'annuaire, sans dates connues.
    static let inWorldCollections = L("Dans les collections du monde", "In the world collections", "En las colecciones del mundo")
    static let inWorldMuseums = L("Dans les musées du monde", "In the world's museums", "En los museos del mundo")

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
            items[key] = ArtistItem(id: key, name: w.name, line: inWorldCollections, museumWorks: 0, worldWorks: w.count)
        }
        for (key, d) in directory {
            if let known = items[key] {
                items[key] = ArtistItem(
                    id: key, name: known.name, line: known.line == inWorldCollections && !d.life.isEmpty ? d.life : known.line,
                    museumWorks: known.museumWorks, worldWorks: known.worldWorks, elsewhere: d.works, sort: known.sort
                )
            } else {
                items[key] = ArtistItem(
                    id: key, name: d.name, line: d.life.isEmpty ? inWorldMuseums : d.life,
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
                .accessibilityLabel(worksCount(artist.total))
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
                                label: L("Écouter la biographie", "Listen to the biography", "Escuchar la biografía")
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
                    Text(L("Le catalogue complet : ", "The complete catalogue: ", "El catálogo completo: ") + worksCount(total))
                        .font(.title2.bold())
                        .padding(.top, 8)
                    Text(artist.works.count == 1
                         ? L("Son œuvre dans les expositions", "Their work in the exhibitions", "Su obra en las exposiciones")
                         : L("Ses \(artist.works.count) œuvres dans les expositions", "Their \(artist.works.count) works in the exhibitions", "Sus \(artist.works.count) obras en las exposiciones"))
                        .font(.headline)
                        .foregroundStyle(.secondary)
                    WorkGrid(items: catalogue.works(artist.works).map { ($0.0, $0.1, $0.2) }, showExhibition: true)

                    if !world.isEmpty {
                        Text(world.count == 1
                             ? L("Son œuvre dans les collections du monde", "Their work in the world collections", "Su obra en las colecciones del mundo")
                             : L("Ses \(world.count.formatted()) œuvres dans les collections du monde", "Their \(world.count.formatted()) works in the world collections", "Sus \(world.count.formatted()) obras en las colecciones del mundo"))
                            .font(.headline)
                            .foregroundStyle(.secondary)
                            .padding(.top, 8)
                        CollectionGrid(works: world.sorted { ($0.year ?? .max) < ($1.year ?? .max) })
                    }

                    LiveWorksSection(name: artist.name, known: Set(world.map(\.id)))
                        .padding(.top, 8)

                    let exhibitions = artist.exhibitions.compactMap(catalogue.exhibition)
                    if !exhibitions.isEmpty {
                        Text(exhibitions.count == 1 ? L("L'exposition", "The exhibition", "La exposición") : L("Les expositions", "The exhibitions", "Las exposiciones"))
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
                    Text(L("Le catalogue complet : ", "The complete catalogue: ", "El catálogo completo: ") + worksCount(world.count))
                        .font(.title2.bold())
                        .padding(.top, 8)
                    Text(L("Dans les collections du monde, de la plus ancienne à la plus récente.", "In the world collections, from the oldest to the most recent.", "En las colecciones del mundo, de la más antigua a la más reciente."))
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
    /// L'œuvre ouverte en grand (LiveGallery.swift).
    @State private var shown: LiveWork?
    /// Mosaïque (des vignettes serrées, comme une galerie de photos) ou cartes avec les titres.
    @AppStorage("vue-mosaique") private var mosaic = true

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            if loading {
                HStack(spacing: 10) {
                    ProgressView()
                    Text(L("Recherche de ses autres œuvres dans les musées…", "Looking for their other works in the museums…", "Buscando sus otras obras en los museos…"))
                        .foregroundStyle(.secondary)
                }
            } else if let result {
                let more = result.works.filter { !known.contains($0.id) }
                Text(more.isEmpty
                     ? L("Dans les musées, en direct", "In the museums, live", "En los museos, en directo")
                     : L("Dans les musées, en direct : ", "In the museums, live: ", "En los museos, en directo: ")
                        + Lcount(more.count, ("œuvre de plus", "œuvres de plus"), ("more work", "more works"), ("obra más", "obras más")))
                    .font(.title2.bold())
                if more.isEmpty {
                    Text(L("Aucune autre œuvre libre de droits dans les collections ouvertes des musées.", "No other copyright-free work in the museums' open collections.", "Ninguna otra obra libre de derechos en las colecciones abiertas de los museos."))
                        .foregroundStyle(.secondary)
                } else {
                    Text(L("Domaine public (artiste mort en 1955 ou avant, œuvre d'avant 1931)",
                           "Public domain (artist died in 1955 or earlier, work from before 1931)",
                           "Dominio público (artista fallecido en 1955 o antes, obra anterior a 1931)")
                         + " · " + (result.source == "musees"
                            ? L("directement chez les musées", "directly from the museums", "directamente de los museos")
                            : L("par l'API ARTEON", "through the ARTEON API", "a través de la API de ARTEON")))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Picker(L("Affichage", "View", "Vista"), selection: $mosaic) {
                        Text(L("Mosaïque", "Mosaic", "Mosaico")).tag(true)
                        Text(L("Avec les titres", "With titles", "Con los títulos")).tag(false)
                    }
                    .pickerStyle(.segmented)
                    .frame(maxWidth: 360)
                    // Toucher une œuvre l'ouvre en grand dans l'application (et non plus sur le
                    // site du musée) ; on glisse ensuite de l'une à l'autre.
                    if mosaic {
                        LazyVGrid(columns: [GridItem(.adaptive(minimum: 84, maximum: 150), spacing: 2)], spacing: 2) {
                            ForEach(more) { work in
                                Button {
                                    shown = work
                                } label: {
                                    RemoteImage(path: work.thumbnail, maxPixels: 320)
                                        .aspectRatio(1, contentMode: .fit)
                                }
                                .buttonStyle(.plain)
                                .accessibilityLabel(work.title)
                            }
                        }
                    } else {
                        LazyVGrid(columns: [GridItem(.adaptive(minimum: 150, maximum: 240), spacing: 12)], spacing: 12) {
                            ForEach(more) { work in
                                Button {
                                    shown = work
                                } label: {
                                    LiveWorkCard(work: work, museum: work.provider.map { "\($0) · \((museums[work.museum] ?? work.museum).components(separatedBy: " (").first ?? "")" } ?? museums[work.museum] ?? work.museum)
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }
                }
            } else {
                Text(L("Les musées n'ont pas répondu.", "The museums did not respond.", "Los museos no han respondido."))
                    .foregroundStyle(.secondary)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .fullScreenCover(item: $shown) { work in
            LiveWorkViewer(works: (result?.works ?? []).filter { !known.contains($0.id) }, start: work, museums: museums, artist: name)
        }
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
