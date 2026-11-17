import SwiftUI

/// Les collections du monde : des dizaines de milliers d'œuvres du domaine public, par
/// musée, par catégorie, par siècle ; leurs fiches et leurs vignettes sont sur l'appareil.
struct CollectionsView: View {
    @Environment(CatalogueStore.self) private var store
    @State private var query = ""

    var body: some View {
        Group {
            if let index = store.collections {
                content(index)
            } else if LocalMuseum.hasCollections {
                ProgressView("Ouverture des collections…")
                    .task { await store.loadCollections() }
            } else {
                ContentUnavailableView(
                    "Les collections ne sont pas dans cette copie",
                    systemImage: "square.stack.3d.up.slash",
                    description: Text("Cette application a été compilée sans les collections du monde (outils/collections_monde.py).")
                )
            }
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle("Collections du monde")
    }

    @ViewBuilder
    private func content(_ index: CollectionIndex) -> some View {
        let trimmed = query.trimmingCharacters(in: .whitespaces)
        ScrollView {
            if !trimmed.isEmpty {
                let found = index.works(.search(trimmed), limit: 600)
                VStack(alignment: .leading, spacing: 12) {
                    Text(found.count >= 600 ? "Plus de 600 œuvres" : (found.count == 1 ? "1 œuvre" : "\(found.count) œuvres"))
                        .foregroundStyle(.secondary)
                    CollectionGrid(works: found)
                }
                .padding()
            } else {
                VStack(alignment: .leading, spacing: 24) {
                    Text("\(index.count.formatted()) œuvres du domaine public, choisies dans les collections ouvertes de \(index.collections.museums.count) grands musées. Leurs fiches et leurs vignettes sont sur l'appareil ; l'image en grand se télécharge au premier regard, puis reste.")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: 760, alignment: .leading)

                    section("Les musées") {
                        LazyVGrid(columns: [GridItem(.adaptive(minimum: 260, maximum: 420), spacing: 14)], spacing: 14) {
                            ForEach(index.collections.museums) { museum in
                                NavigationLink(value: Route.collection(.museum(museum.key))) {
                                    CollectionMuseumCard(index: index, museum: museum)
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }

                    section("Aujourd'hui") {
                        let day = Calendar.current.ordinality(of: .day, in: .era, for: Date()) ?? 0
                        CollectionRow(works: index.picks(16, day: day))
                    }

                    section("Par catégorie") {
                        FlowLayout(spacing: 10) {
                            ForEach(index.categories, id: \.name) { item in
                                NavigationLink(value: Route.collection(.category(item.name))) {
                                    chip(item.name, item.count)
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }

                    section("Par siècle") {
                        FlowLayout(spacing: 10) {
                            ForEach(index.centuries, id: \.rank) { item in
                                NavigationLink(value: Route.collection(.century(item.rank))) {
                                    chip(item.title, item.count)
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }
                }
                .padding()
            }
        }
        .searchable(text: $query, prompt: "Dans \(index.count.formatted()) œuvres : un titre, un artiste…")
    }

    private func section<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(title).font(.title2.bold())
            content()
        }
    }

    private func chip(_ title: String, _ count: Int) -> some View {
        HStack(spacing: 6) {
            Text(title)
            Text(count.formatted())
                .foregroundStyle(.secondary)
                .monospacedDigit()
        }
        .font(.callout)
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(Color.card, in: .capsule)
    }
}

/// La carte d'un musée des collections : quatre vignettes et son nom.
struct CollectionMuseumCard: View {
    let index: CollectionIndex
    let museum: CollectionMuseum

    var body: some View {
        let samples = index.works(.museum(museum.key), limit: 4)
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 2) {
                ForEach(samples) { work in
                    CollectionThumb(work: work)
                        .frame(maxWidth: .infinity)
                        .frame(height: 120)
                        .clipped()
                }
            }
            Rectangle().fill(Color(hex: museum.color) ?? .arteonBlue).frame(height: 5)
            VStack(alignment: .leading, spacing: 4) {
                Text(museum.name).font(.headline)
                Text(museum.city).font(.subheadline).foregroundStyle(.secondary)
                Text("\(museum.count.formatted()) œuvres")
                    .font(.caption.bold())
                    .foregroundStyle(Color(hex: museum.color) ?? .secondary)
            }
            .padding(14)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .card()
        .accessibilityElement(children: .combine)
    }
}

/// La vignette d'une œuvre des collections, lue dans le paquet de son musée.
struct CollectionThumb: View {
    let work: CollectionWork
    var contentMode: ContentMode = .fill

    @Environment(CatalogueStore.self) private var store
    @State private var image: UIImage?

    var body: some View {
        Rectangle()
            .fill(Color.secondary.opacity(0.15))
            .overlay {
                if let image {
                    Image(uiImage: image)
                        .resizable()
                        .aspectRatio(contentMode: contentMode)
                }
            }
            .clipped()
            .task(id: work.id) {
                guard let pack = store.collections?.collections.museum(work.museum)?.pack else { return }
                image = await ImagePipeline.shared.collectionThumbnail(work, pack: pack)
            }
    }
}

/// L'image d'une œuvre des collections : la vignette tout de suite, puis l'image en grand,
/// embarquée dans l'app (700 px, hors ligne).
struct CollectionImage: View {
    let work: CollectionWork
    var contentMode: ContentMode = .fit

    @Environment(CatalogueStore.self) private var store

    @State private var full: UIImage?
    @State private var loading = false

    var body: some View {
        ZStack {
            CollectionThumb(work: work, contentMode: contentMode)
                .opacity(full == nil ? 1 : 0)
            if let full {
                Image(uiImage: full)
                    .resizable()
                    .aspectRatio(contentMode: contentMode)
                    .transition(.opacity)
            }
            if loading && full == nil {
                ProgressView()
                    .padding(8)
                    .background(.ultraThinMaterial, in: .circle)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottomTrailing)
                    .padding(10)
            }
        }
        .task(id: work.id) {
            // L'image en grand embarquée dans l'app (hors ligne), sinon celle du musée.
            let pack = store.collections?.collections.museum(work.museum)?.largePack
            if let local = await ImagePipeline.shared.collectionLarge(work, pack: pack) {
                full = local
                return
            }
            guard let url = work.imageURL, !LocalMuseum.isAvailable else { return }
            loading = true
            let loaded = await ImagePipeline.shared.image(url, maxPixels: 2400)
            loading = false
            if let loaded {
                withAnimation(.easeOut(duration: 0.3)) { full = loaded }
            }
        }
    }
}

/// La vignette et le cartel d'une œuvre des collections.
struct CollectionCard: View {
    let work: CollectionWork

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            CollectionThumb(work: work)
                .frame(height: 170)
            VStack(alignment: .leading, spacing: 2) {
                Text(work.title)
                    .font(.subheadline.bold())
                    .lineLimit(2)
                Text(work.byline)
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

/// Des œuvres des collections, en grille.
struct CollectionGrid: View {
    let works: [CollectionWork]

    var body: some View {
        LazyVGrid(columns: [GridItem(.adaptive(minimum: 150, maximum: 240), spacing: 12)], spacing: 12) {
            ForEach(works) { work in
                NavigationLink(value: Route.collectionWork(work.id)) {
                    CollectionCard(work: work)
                }
                .buttonStyle(.plain)
            }
        }
    }
}

/// Des œuvres des collections sur une ligne qui défile.
struct CollectionRow: View {
    let works: [CollectionWork]

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            LazyHStack(spacing: 12) {
                ForEach(Array(works.enumerated()), id: \.offset) { _, work in
                    NavigationLink(value: Route.collectionWork(work.id)) {
                        CollectionCard(work: work).frame(width: 170)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
        .scrollClipDisabled()
    }
}

/// Les œuvres d'un musée, d'une catégorie, d'un siècle ou d'un artiste ; tout télécharger
/// pour les voir en grand sans Internet.
struct CollectionListView: View {
    let filter: CollectionFilter

    @Environment(CatalogueStore.self) private var store
    @State private var category: String?
    @State private var shown = 300
    @State private var downloader = Downloader()

    var body: some View {
        if let index = store.collections {
            let all = index.works(filter)
            let works = category.map { c in all.filter { ($0.category ?? "Autres objets") == c } } ?? all
            let categories = Dictionary(grouping: all, by: { $0.category ?? "Autres objets" })
                .map { ($0.key, $0.value.count) }
                .sorted { $0.1 > $1.1 }
            ScrollView {
                VStack(alignment: .leading, spacing: 14) {
                    if case .museum(let key) = filter, let museum = index.collections.museum(key) {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(museum.city).font(.headline).foregroundStyle(.secondary)
                            Text(museum.licence).font(.caption).foregroundStyle(.secondary)
                            if let url = URL(string: museum.site) {
                                Link(destination: url) { Label("Le site du musée", systemImage: "arrow.up.right.square") }
                                    .font(.subheadline)
                            }
                        }
                    }
                    if categories.count > 1, !isCategory {
                        ScrollView(.horizontal, showsIndicators: false) {
                            HStack(spacing: 8) {
                                categoryButton("Tout", nil, all.count)
                                ForEach(categories, id: \.0) { categoryButton($0.0, $0.0, $0.1) }
                            }
                        }
                        .scrollClipDisabled()
                    }
                    HStack {
                        Text(works.count == 1 ? "1 œuvre" : "\(works.count.formatted()) œuvres")
                            .foregroundStyle(.secondary)
                        Spacer()
                        DownloadButton(downloader: downloader, works: works)
                    }
                    CollectionGrid(works: Array(works.prefix(shown)))
                    if works.count > shown {
                        Button("Afficher plus (\((works.count - shown).formatted()) de plus)") {
                            shown += 300
                        }
                        .buttonStyle(.glass)
                        .frame(maxWidth: .infinity)
                    }
                }
                .padding()
            }
            .background(Color(uiColor: .systemGroupedBackground))
            .navigationTitle(index.title(filter))
            .onChange(of: category) { shown = 300 }
        } else {
            ProgressView()
        }
    }

    private var isCategory: Bool {
        if case .category = filter { true } else { false }
    }

    @ViewBuilder
    private func categoryButton(_ title: String, _ value: String?, _ count: Int) -> some View {
        if category == value {
            Button("\(title) \(count.formatted())") {}
                .buttonStyle(.glassProminent)
        } else {
            Button("\(title) \(count.formatted())") {
                withAnimation(.snappy) { category = value }
            }
            .buttonStyle(.glass)
        }
    }
}

/// Une œuvre des collections : l'image (vignette, puis en grand), le cartel, le texte du
/// musée, les autres œuvres de l'artiste, le lien vers le musée.
struct CollectionWorkView: View {
    let id: String

    @Environment(CatalogueStore.self) private var store
    @Environment(\.horizontalSizeClass) private var sizeClass
    @State private var zooming = false

    var body: some View {
        if let index = store.collections, let work = index.work(id) {
            let museum = index.collections.museum(work.museum)
            GeometryReader { geo in
                if sizeClass == .regular && geo.size.width > geo.size.height * 1.1 {
                    HStack(spacing: 0) {
                        picture(work)
                            .padding(40)
                            .frame(width: geo.size.width * 0.55)
                            .frame(maxHeight: .infinity)
                            .background(Color.galleryWall)
                        ScrollView { text(work, museum: museum, index: index).padding(32) }
                    }
                } else {
                    ScrollView {
                        VStack(spacing: 0) {
                            picture(work)
                                .padding(20)
                                .frame(height: max(geo.size.height * 0.55, 280))
                                .frame(maxWidth: .infinity)
                                .background(Color.galleryWall)
                            text(work, museum: museum, index: index)
                                .padding()
                                .padding(.bottom, 32)
                        }
                    }
                }
            }
            .navigationTitle(museum?.name ?? "Collections")
            .navigationBarTitleDisplayMode(.inline)
            .onAppear { store.visited(work.key) }
            .toolbar {
                ToolbarItemGroup(placement: .primaryAction) {
                    let favorite = store.isFavorite(work.key)
                    Button {
                        store.toggleFavorite(work.key)
                    } label: {
                        Label(favorite ? "Retirer des favoris" : "Ajouter aux favoris",
                              systemImage: favorite ? "heart.fill" : "heart")
                    }
                    .tint(favorite ? Color.red : nil)
                    .sensoryFeedback(.selection, trigger: favorite)
                    ListenButton(id: work.key.text, text: { spoken(work, museum: museum) })
                    if let page = work.page, let url = URL(string: page) {
                        ShareLink(item: url, subject: Text(work.title), message: Text(work.byline)) {
                            Label("Partager", systemImage: "square.and.arrow.up")
                        }
                    }
                }
            }
            .fullScreenCover(isPresented: $zooming) {
                CollectionZoom(work: work)
            }
        } else if store.collections == nil {
            ProgressView().task { await store.loadCollections() }
        } else {
            ContentUnavailableView("Œuvre introuvable", systemImage: "questionmark.square.dashed")
        }
    }

    private func picture(_ work: CollectionWork) -> some View {
        CollectionImage(work: work)
            .aspectRatio(work.aspect, contentMode: .fit)
            .shadow(color: .black.opacity(0.3), radius: 16, y: 10)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .contentShape(.rect)
            .onTapGesture { zooming = true }
            .accessibilityLabel("\(work.title), toucher pour agrandir")
            .accessibilityAddTraits(.isButton)
    }

    private func spoken(_ work: CollectionWork, museum: CollectionMuseum?) -> String {
        var parts = ["\(work.title).", "\(work.artist)."]
        if let d = work.date { parts.append("\(d).") }
        if let m = work.medium { parts.append(m) }
        if let museum { parts.append("\(museum.name), \(museum.city).") }
        if let x = work.text { parts.append(x) }
        return parts.joined(separator: "\n")
    }

    @ViewBuilder
    private func text(_ work: CollectionWork, museum: CollectionMuseum?, index: CollectionIndex) -> some View {
        let accent = museum.flatMap { Color(hex: $0.color) } ?? .arteonBlue
        VStack(alignment: .leading, spacing: 0) {
            Text(work.title).font(.title.bold())
            NavigationLink(value: Route.collection(.artist(work.artist))) {
                Text(work.artist).font(.title3)
            }
            .padding(.top, 4)
            if let life = work.life {
                Text(life).font(.subheadline).foregroundStyle(.secondary)
            }

            Grid(alignment: .leadingFirstTextBaseline, horizontalSpacing: 16, verticalSpacing: 6) {
                fact("Date", work.date)
                fact("Technique", work.medium)
                fact("Mesures", work.dimensions)
                fact("Catégorie", work.category)
                fact("Origine", work.origin)
                fact("Musée", museum.map { "\($0.name), \($0.city)" })
                fact("Où la voir", work.gallery)
            }
            .padding(.top, 16)

            if let x = work.text {
                Text("Le musée la présente ainsi")
                    .font(.headline)
                    .foregroundStyle(accent)
                    .padding(.top, 24)
                    .padding(.bottom, 8)
                Text(x).lineSpacing(4)
                if work.museum == "chicago" {
                    Text("Texte : Art Institute of Chicago, CC BY 4.0.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .padding(.top, 4)
                }
            }

            let others = index.works(.artist(work.artist), limit: 30).filter { $0.id != work.id }
            if !others.isEmpty, work.artist != "Artiste inconnu" {
                Text("Du même artiste")
                    .font(.headline)
                    .foregroundStyle(accent)
                    .padding(.top, 24)
                    .padding(.bottom, 8)
                CollectionRow(works: others)
            }
            if store.index?.artist(named: work.artist) != nil {
                NavigationLink(value: Route.artist(work.artist)) {
                    Label("\(work.artist) dans les expositions d'ARTEON", systemImage: "person.crop.rectangle.stack")
                }
                .padding(.top, 12)
            }

            if let page = work.page, let url = URL(string: page) {
                Link(destination: url) {
                    Label("La fiche sur le site du musée", systemImage: "arrow.up.right.square")
                }
                .padding(.top, 20)
            }
            if let museum {
                Text(museum.licence)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .padding(.top, 8)
            }
        }
        .frame(maxWidth: 720, alignment: .leading)
        .textSelection(.enabled)
    }

    @ViewBuilder
    private func fact(_ label: String, _ value: String?) -> some View {
        if let value, !value.isEmpty {
            GridRow {
                Text(label).font(.caption).foregroundStyle(.secondary)
                Text(value).font(.subheadline)
            }
        }
    }
}

/// L'image en grand d'une œuvre des collections : pincer, glisser, toucher deux fois.
struct CollectionZoom: View {
    let work: CollectionWork

    @Environment(\.dismiss) private var dismiss
    @State private var scale: CGFloat = 1
    @State private var lastScale: CGFloat = 1
    @State private var offset: CGSize = .zero
    @State private var lastOffset: CGSize = .zero

    var body: some View {
        NavigationStack {
            CollectionImage(work: work)
                .scaleEffect(scale)
                .offset(offset)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .contentShape(.rect)
                .gesture(
                    MagnifyGesture()
                        .onChanged { scale = min(max(lastScale * $0.magnification, 1), 8) }
                        .onEnded { _ in lastScale = scale }
                        .simultaneously(with: DragGesture()
                            .onChanged { v in
                                guard scale > 1 else { return }
                                offset = CGSize(width: lastOffset.width + v.translation.width,
                                                height: lastOffset.height + v.translation.height)
                            }
                            .onEnded { _ in lastOffset = offset })
                )
                .onTapGesture(count: 2) {
                    withAnimation(.spring) {
                        scale = 1; lastScale = 1; offset = .zero; lastOffset = .zero
                    }
                }
                .background(Color.black.ignoresSafeArea())
                .navigationTitle(work.title)
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .confirmationAction) {
                        Button("Fermer", systemImage: "xmark") { dismiss() }
                    }
                }
        }
        .preferredColorScheme(.dark)
    }
}

// MARK: - Tout télécharger

/// Télécharge les images en grand d'une liste d'œuvres, pour les voir sans Internet.
@MainActor
@Observable
final class Downloader {
    private(set) var done = 0
    private(set) var total = 0
    private(set) var running = false
    @ObservationIgnored private var task: Task<Void, Never>?

    func start(_ urls: [String]) {
        guard !running else { return }
        running = true
        done = 0
        total = urls.count
        task = Task {
            await withTaskGroup(of: Void.self) { group in
                var next = 0
                func add() {
                    guard next < urls.count else { return }
                    let url = urls[next]
                    next += 1
                    group.addTask { _ = await ImageStore.shared.file(for: url) }
                }
                for _ in 0..<6 { add() }
                for await _ in group {
                    if Task.isCancelled { group.cancelAll(); break }
                    done += 1
                    add()
                }
            }
            running = false
        }
    }

    func cancel() {
        task?.cancel()
        running = false
    }
}

/// « Tout télécharger » : les images en grand d'une liste, avec la progression.
struct DownloadButton: View {
    let downloader: Downloader
    let works: [CollectionWork]

    @State private var confirm = false

    var body: some View {
        if LocalMuseum.isAvailable {
            // Tout est dans l'app, les images en grand comprises.
            EmptyView()
        } else if downloader.running {
            HStack(spacing: 8) {
                ProgressView(value: Double(downloader.done), total: Double(max(downloader.total, 1)))
                    .frame(width: 90)
                Text("\(downloader.done)/\(downloader.total)")
                    .font(.caption.monospacedDigit())
                Button("Arrêter", systemImage: "stop.fill") { downloader.cancel() }
                    .labelStyle(.iconOnly)
            }
        } else if downloader.total > 0, downloader.done >= downloader.total {
            Label("Tout est sur l'appareil", systemImage: "checkmark.circle.fill")
                .font(.caption)
                .foregroundStyle(.green)
        } else {
            Button("Tout télécharger", systemImage: "arrow.down.circle") { confirm = true }
                .font(.subheadline)
                .confirmationDialog(
                    "Télécharger les \(works.count.formatted()) images en grand ?",
                    isPresented: $confirm,
                    titleVisibility: .visible
                ) {
                    Button("Télécharger (environ \(max(1, works.count * 250 / 1000)) Mo)") {
                        downloader.start(works.compactMap(\.imageURL))
                    }
                } message: {
                    Text("Elles resteront sur l'appareil pour être vues sans Internet.")
                }
        }
    }
}
