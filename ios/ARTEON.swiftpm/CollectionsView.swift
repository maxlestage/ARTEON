import SwiftUI

/// Les collections du monde : des dizaines de milliers d'œuvres du domaine public, par
/// musée, par catégorie, par siècle ; tout vient du site ARTEON, rien n'est gardé.
struct CollectionsView: View {
    @Environment(CatalogueStore.self) private var store
    @State private var query = ""

    var body: some View {
        Group {
            if let index = store.collections {
                content(index)
            } else if store.collectionsError && !store.loadingCollections {
                ContentUnavailableView {
                    Label(L("Les collections n'ont pas répondu", "The collections did not respond", "Las colecciones no han respondido"), systemImage: "wifi.exclamationmark")
                } description: {
                    Text(L("Tout passe par l'API ARTEON : il faut Internet.", "Everything comes from the ARTEON API: an Internet connection is needed.", "Todo pasa por la API de ARTEON: hace falta Internet."))
                } actions: {
                    Button(L("Réessayer", "Try again", "Reintentar"), systemImage: "arrow.clockwise") {
                        Task { await store.loadCollections() }
                    }
                    .buttonStyle(.glassProminent)
                }
            } else {
                ProgressView(L("Ouverture des collections…", "Opening the collections…", "Abriendo las colecciones…"))
                    .task { await store.loadCollections() }
            }
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle(L("Collections du monde", "World collections", "Colecciones del mundo"))
    }

    @ViewBuilder
    private func content(_ index: CollectionIndex) -> some View {
        let trimmed = query.trimmingCharacters(in: .whitespaces)
        ScrollView {
            if !trimmed.isEmpty {
                let found = index.works(.search(trimmed), limit: 600)
                VStack(alignment: .leading, spacing: 12) {
                    Text(found.count >= 600 ? L("Plus de 600 œuvres", "More than 600 works", "Más de 600 obras") : worksCount(found.count))
                        .foregroundStyle(.secondary)
                    CollectionGrid(works: found)
                }
                .padding()
            } else {
                VStack(alignment: .leading, spacing: 24) {
                    Text(L("\(index.count.formatted()) œuvres du domaine public, choisies dans les collections ouvertes de \(index.collections.museums.count) grands musées. Tout vient du site ARTEON ; l'image en grand vient du musée. Rien n'est gardé sur l'appareil.",
                           "\(index.count.formatted()) public-domain works, chosen from the open collections of \(index.collections.museums.count) major museums. Everything comes from the ARTEON website; the large image comes from the museum. Nothing is kept on the device.",
                           "\(index.count.formatted()) obras de dominio público, elegidas en las colecciones abiertas de \(index.collections.museums.count) grandes museos. Todo viene del sitio ARTEON; la imagen grande viene del museo. Nada se guarda en el dispositivo."))
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: 760, alignment: .leading)

                    section(L("Les musées", "The museums", "Los museos")) {
                        LazyVGrid(columns: [GridItem(.adaptive(minimum: 260, maximum: 420), spacing: 14)], spacing: 14) {
                            ForEach(index.collections.museums) { museum in
                                NavigationLink(value: Route.collection(.museum(museum.key))) {
                                    CollectionMuseumCard(index: index, museum: museum)
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }

                    section(L("Aujourd'hui", "Today", "Hoy")) {
                        let day = Calendar.current.ordinality(of: .day, in: .era, for: Date()) ?? 0
                        CollectionRow(works: index.picks(16, day: day))
                    }

                    section(L("Par catégorie", "By category", "Por categoría")) {
                        FlowLayout(spacing: 10) {
                            ForEach(index.categories, id: \.name) { item in
                                NavigationLink(value: Route.collection(.category(item.name))) {
                                    chip(CollectionCategory.display(item.name), item.count)
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }

                    section(L("Par siècle", "By century", "Por siglo")) {
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
        .searchable(text: $query, prompt: L("Dans \(index.count.formatted()) œuvres : un titre, un artiste…", "In \(index.count.formatted()) works: a title, an artist…", "En \(index.count.formatted()) obras: un título, un artista…"))
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
                Text(localizedCollectionText(museum.city)).font(.subheadline).foregroundStyle(.secondary)
                Text(worksCount(museum.count))
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

/// La vignette d'une œuvre des collections, demandée au site ARTEON (ou au musée).
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
                image = await ImagePipeline.shared.collectionThumbnail(work)
            }
    }
}

/// L'image d'une œuvre des collections : la vignette tout de suite, puis l'image en grand,
/// demandée au musée.
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
            // L'image en grand, chez le musée (son adresse est dans le détail de l'œuvre).
            let detailed = work.imageURL == nil ? await store.details(work) : work
            guard let url = detailed.imageURL else { return }
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

/// Les œuvres d'un musée, d'une catégorie, d'un siècle ou d'un artiste.
struct CollectionListView: View {
    let filter: CollectionFilter

    @Environment(CatalogueStore.self) private var store
    @State private var category: String?
    @State private var shown = 300

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
                            Text(localizedCollectionText(museum.city)).font(.headline).foregroundStyle(.secondary)
                            Text(localizedCollectionText(museum.licence)).font(.caption).foregroundStyle(.secondary)
                            if let url = URL(string: museum.site) {
                                Link(destination: url) { Label(L("Le site du musée", "The museum's website", "El sitio del museo"), systemImage: "arrow.up.right.square") }
                                    .font(.subheadline)
                            }
                        }
                    }
                    if categories.count > 1, !isCategory {
                        ScrollView(.horizontal, showsIndicators: false) {
                            HStack(spacing: 8) {
                                categoryButton(L("Tout", "All", "Todo"), nil, all.count)
                                ForEach(categories, id: \.0) { categoryButton(CollectionCategory.display($0.0), $0.0, $0.1) }
                            }
                        }
                        .scrollClipDisabled()
                    }
                    HStack {
                        Text(worksCount(works.count))
                            .foregroundStyle(.secondary)
                        Spacer()
                    }
                    CollectionGrid(works: Array(works.prefix(shown)))
                    if works.count > shown {
                        Button(L("Afficher plus (\((works.count - shown).formatted()) de plus)", "Show more (\((works.count - shown).formatted()) more)", "Mostrar más (\((works.count - shown).formatted()) más)")) {
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
    /// Le détail de l'œuvre (technique, texte du musée, image en grand), lu sur le site.
    @State private var detailed: CollectionWork?

    var body: some View {
        if let index = store.collections, let listed = index.work(id) {
            let work = detailed?.id == id ? detailed! : listed
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
            .navigationTitle(museum?.name ?? L("Collections", "Collections", "Colecciones"))
            .navigationBarTitleDisplayMode(.inline)
            .onAppear { store.visited(work.key) }
            .task(id: id) { detailed = await store.details(listed) }
            .toolbar {
                ToolbarItemGroup(placement: .primaryAction) {
                    let favorite = store.isFavorite(work.key)
                    Button {
                        store.toggleFavorite(work.key)
                    } label: {
                        Label(favorite ? L("Retirer des favoris", "Remove from favourites", "Quitar de favoritos") : L("Ajouter aux favoris", "Add to favourites", "Añadir a favoritos"),
                              systemImage: favorite ? "heart.fill" : "heart")
                    }
                    .tint(favorite ? Color.red : nil)
                    .sensoryFeedback(.selection, trigger: favorite)
                    ListenButton(id: work.key.text, text: { spoken(work, museum: museum) })
                    if let page = work.page, let url = URL(string: page) {
                        ShareLink(item: url, subject: Text(work.title), message: Text(work.byline)) {
                            Label(L("Partager", "Share", "Compartir"), systemImage: "square.and.arrow.up")
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
            ContentUnavailableView(L("Œuvre introuvable", "Work not found", "Obra no encontrada"), systemImage: "questionmark.square.dashed")
        }
    }

    private func picture(_ work: CollectionWork) -> some View {
        CollectionImage(work: work)
            .aspectRatio(work.aspect, contentMode: .fit)
            .shadow(color: .black.opacity(0.3), radius: 16, y: 10)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .contentShape(.rect)
            .onTapGesture { zooming = true }
            .accessibilityLabel(work.title + L(", toucher pour agrandir", ", tap to enlarge", ", toque para ampliar"))
            .accessibilityAddTraits(.isButton)
    }

    private func spoken(_ work: CollectionWork, museum: CollectionMuseum?) -> String {
        var parts = ["\(work.title).", "\(displayArtist(work.artist))."]
        if let d = work.date { parts.append("\(d).") }
        if let m = work.medium { parts.append(m) }
        if let museum { parts.append("\(museum.name), \(localizedCollectionText(museum.city)).") }
        if let x = work.text { parts.append(x) }
        return parts.joined(separator: "\n")
    }

    @ViewBuilder
    private func text(_ work: CollectionWork, museum: CollectionMuseum?, index: CollectionIndex) -> some View {
        let accent = museum.flatMap { Color(hex: $0.color) } ?? .arteonBlue
        VStack(alignment: .leading, spacing: 0) {
            Text(work.title).font(.title.bold())
            NavigationLink(value: Route.collection(.artist(work.artist))) {
                Text(displayArtist(work.artist)).font(.title3)
            }
            .padding(.top, 4)
            if let life = work.life {
                Text(life).font(.subheadline).foregroundStyle(.secondary)
            }

            Grid(alignment: .leadingFirstTextBaseline, horizontalSpacing: 16, verticalSpacing: 6) {
                fact(L("Date", "Date", "Fecha"), work.date)
                fact(L("Technique", "Medium", "Técnica"), work.medium)
                fact(L("Mesures", "Dimensions", "Medidas"), work.dimensions)
                fact(L("Catégorie", "Category", "Categoría"), work.category.map { CollectionCategory.display($0) })
                fact(L("Origine", "Origin", "Origen"), work.origin)
                fact(L("Musée", "Museum", "Museo"), museum.map { "\($0.name), \(localizedCollectionText($0.city))" })
                fact(L("Où la voir", "Where to see it", "Dónde verla"), work.gallery)
            }
            .padding(.top, 16)

            if let x = work.text {
                Text(L("Le musée la présente ainsi", "How the museum presents it", "Así la presenta el museo"))
                    .font(.headline)
                    .foregroundStyle(accent)
                    .padding(.top, 24)
                    .padding(.bottom, 8)
                Text(x).lineSpacing(4)
                if work.museum == "chicago" {
                    Text(L("Texte : Art Institute of Chicago, CC BY 4.0.", "Text: Art Institute of Chicago, CC BY 4.0.", "Texto: Art Institute of Chicago, CC BY 4.0."))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .padding(.top, 4)
                }
            }

            let others = index.works(.artist(work.artist), limit: 30).filter { $0.id != work.id }
            if !others.isEmpty, work.artist != "Artiste inconnu" {
                Text(L("Du même artiste", "By the same artist", "Del mismo artista"))
                    .font(.headline)
                    .foregroundStyle(accent)
                    .padding(.top, 24)
                    .padding(.bottom, 8)
                CollectionRow(works: others)
            }
            if store.index?.artist(named: work.artist) != nil {
                NavigationLink(value: Route.artist(work.artist)) {
                    Label(L("\(work.artist) dans les expositions d'ARTEON", "\(work.artist) in the ARTEON exhibitions", "\(work.artist) en las exposiciones de ARTEON"), systemImage: "person.crop.rectangle.stack")
                }
                .padding(.top, 12)
            }

            if let page = work.page, let url = URL(string: page) {
                Link(destination: url) {
                    Label(L("La fiche sur le site du musée", "The record on the museum's website", "La ficha en el sitio del museo"), systemImage: "arrow.up.right.square")
                }
                .padding(.top, 20)
            }
            if let museum {
                Text(localizedCollectionText(museum.licence))
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
                        Button(L("Fermer", "Close", "Cerrar"), systemImage: "xmark") { dismiss() }
                    }
                }
        }
        .preferredColorScheme(.dark)
    }
}
