import SwiftUI

/// Explorer le musée autrement : par siècle, par thème, par collection, par technique, au
/// hasard, ou en jouant.
struct ExploreView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store
    @State private var random: WorkKey?

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                WorldCollectionsCard()
                LazyVGrid(
                    columns: [GridItem(.adaptive(minimum: 300, maximum: 500), spacing: 16)],
                    spacing: 16
                ) {
                    ExploreCard(
                        title: "La chronologie",
                        subtitle: "\(store.index?.centuries.count ?? 0) siècles, des fossiles aux avant-gardes",
                        systemImage: "calendar",
                        color: .arteonBlue,
                        image: store.index?.centuries.first?.works.first,
                        route: .timeline
                    )
                    ExploreCard(
                        title: "Qui l'a fait ?",
                        subtitle: store.quizBest > 0 ? "Le jeu · record : \(store.quizBest)" : "Le jeu : reconnaître l'artiste",
                        systemImage: "questionmark.bubble",
                        color: .orange,
                        image: catalogue.workOfTheDay().map { $0.0.key($0.1) },
                        route: .quiz
                    )
                    ExploreCard(
                        title: "Les artistes",
                        subtitle: "\(store.index?.artists.count ?? 0) artistes, ateliers et cultures, de A à Z",
                        systemImage: "person.2",
                        color: .indigo,
                        image: store.index?.artist(named: "Vincent van Gogh")?.works.first ?? store.index?.artists.first?.works.first,
                        route: .artists
                    )
                    ExploreCard(
                        title: "Les thèmes",
                        subtitle: "\(store.index?.themes.count ?? 0) thèmes : la nuit, la mer, l'amour…",
                        systemImage: "tag",
                        color: .pink,
                        image: store.index?.themes.first?.works.first,
                        route: .themes
                    )
                    ExploreCard(
                        title: "Les musées du monde",
                        subtitle: "\(store.index?.collections.count ?? 0) collections où voir les œuvres en vrai",
                        systemImage: "building.2",
                        color: .teal,
                        image: store.index?.collections.first?.works.first,
                        route: .collections
                    )
                    ExploreCard(
                        title: "Les techniques",
                        subtitle: "Huile, tempera, estampe, bronze, porcelaine…",
                        systemImage: "paintbrush.pointed",
                        color: .purple,
                        image: store.index?.techniques.first?.works.dropFirst(3).first,
                        route: .techniques
                    )
                }

                Button {
                    random = catalogue.allWorks.randomElement().map { $0.0.key($0.1) }
                } label: {
                    Label("Une œuvre au hasard", systemImage: "dice")
                        .font(.headline)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 6)
                }
                .buttonStyle(.glassProminent)
                .sensoryFeedback(.impact, trigger: random)
            }
            .padding()
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle("Explorer")
        .navigationDestination(item: $random) { key in
            RouteView(route: .work(key), catalogue: catalogue)
        }
    }
}

struct ExploreCard: View {
    let title: String
    let subtitle: String
    let systemImage: String
    let color: Color
    let image: WorkKey?
    let route: Route

    @Environment(CatalogueStore.self) private var store

    var body: some View {
        NavigationLink(value: route) {
            ZStack(alignment: .bottomLeading) {
                RemoteImage(path: image.flatMap { store.catalogue?.work($0)?.1.image }, tint: color.opacity(0.3))
                    .frame(height: 190)
                LinearGradient(colors: [color.opacity(0.1), color.opacity(0.92)], startPoint: .top, endPoint: .bottom)
                VStack(alignment: .leading, spacing: 4) {
                    Image(systemName: systemImage)
                        .font(.title2.bold())
                    Text(title)
                        .font(.title2.bold())
                    Text(subtitle)
                        .font(.subheadline)
                        .opacity(0.9)
                        .lineLimit(2)
                }
                .foregroundStyle(.white)
                .padding(18)
            }
            .clipShape(.rect(cornerRadius: 22))
            .contentShape(.rect(cornerRadius: 22))
            .hoverEffect(.lift)
        }
        .buttonStyle(.plain)
        .accessibilityElement(children: .combine)
    }
}

/// Toutes les œuvres, du plus ancien au plus récent, siècle par siècle.
struct TimelineView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store

    var body: some View {
        let centuries = store.index?.centuries ?? []
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 26, pinnedViews: [.sectionHeaders]) {
                    ForEach(centuries) { century in
                        Section {
                            WorkRow(items: catalogue.works(century.works))
                        } header: {
                            HStack(alignment: .firstTextBaseline) {
                                Text(century.title)
                                    .font(.title2.bold())
                                Spacer()
                                Text(century.works.count == 1 ? "1 œuvre" : "\(century.works.count) œuvres")
                                    .font(.subheadline)
                                    .foregroundStyle(.secondary)
                            }
                            .padding(.horizontal)
                            .padding(.vertical, 8)
                            .background(.bar)
                            .id(century.id)
                        }
                    }
                    if let undated = store.index?.undated, !undated.isEmpty {
                        Section {
                            WorkRow(items: catalogue.works(undated))
                        } header: {
                            Text("Sans date précise")
                                .font(.title2.bold())
                                .padding(.horizontal)
                        }
                    }
                }
                .padding(.vertical)
            }
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Menu {
                        ForEach(centuries) { century in
                            Button(century.title) {
                                withAnimation { proxy.scrollTo(century.id, anchor: .top) }
                            }
                        }
                    } label: {
                        Label("Aller au siècle", systemImage: "list.bullet")
                    }
                }
            }
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle("Chronologie")
        .navigationBarTitleDisplayMode(.inline)
    }
}

/// Des groupes d'œuvres : les thèmes, les collections ou les techniques.
struct GroupsView: View {
    enum Kind {
        case themes, collections, techniques

        var title: String {
            switch self {
            case .themes: "Les thèmes"
            case .collections: "Les musées du monde"
            case .techniques: "Les techniques"
            }
        }

        var prompt: String {
            switch self {
            case .themes: "Un thème : nuit, mer, portrait…"
            case .collections: "Un musée, une ville…"
            case .techniques: "Huile, bronze, estampe…"
            }
        }
    }

    let catalogue: Catalogue
    let kind: Kind

    @Environment(CatalogueStore.self) private var store
    @State private var filter = ""

    private var groups: [WorkGroup] {
        let all: [WorkGroup] = switch kind {
        case .themes: store.index?.themes ?? []
        case .collections: store.index?.collections ?? []
        case .techniques: store.index?.techniques ?? []
        }
        let q = folded(filter.trimmingCharacters(in: .whitespaces))
        return q.isEmpty ? all : all.filter { folded($0.title).contains(q) }
    }

    var body: some View {
        ScrollView {
            if kind == .themes {
                FlowLayout(spacing: 10) {
                    ForEach(groups) { group in
                        NavigationLink(value: route(group)) {
                            HStack(spacing: 6) {
                                Text(group.title)
                                Text("\(group.works.count)")
                                    .foregroundStyle(.secondary)
                                    .monospacedDigit()
                            }
                            .font(.callout)
                            .padding(.horizontal, 14)
                            .padding(.vertical, 8)
                            .background(Color.card, in: .capsule)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding()
            } else {
                LazyVStack(spacing: 0) {
                    ForEach(groups) { group in
                        NavigationLink(value: route(group)) {
                            HStack(spacing: 14) {
                                RemoteImage(path: group.works.first.flatMap { catalogue.work($0)?.1.image }, maxPixels: 160)
                                    .frame(width: 56, height: 56)
                                    .clipShape(.rect(cornerRadius: 10))
                                Text(group.title)
                                    .multilineTextAlignment(.leading)
                                Spacer(minLength: 8)
                                Text("\(group.works.count)")
                                    .foregroundStyle(.secondary)
                                    .monospacedDigit()
                                Image(systemName: "chevron.right")
                                    .font(.caption.bold())
                                    .foregroundStyle(.tertiary)
                            }
                            .padding(.horizontal)
                            .padding(.vertical, 8)
                            .contentShape(.rect)
                        }
                        .buttonStyle(.plain)
                        Divider().padding(.leading, 86)
                    }
                }
                .background(Color.card, in: .rect(cornerRadius: 18))
                .padding()
            }
        }
        .overlay {
            if groups.isEmpty, !filter.isEmpty {
                ContentUnavailableView.search(text: filter)
            }
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle(kind.title)
        .searchable(text: $filter, prompt: kind.prompt)
    }

    private func route(_ group: WorkGroup) -> Route {
        .works(
            title: group.title,
            subtitle: group.works.count == 1 ? "1 œuvre" : "\(group.works.count) œuvres",
            keys: group.works
        )
    }
}

/// Une liste d'œuvres de toutes les expositions.
struct WorkListView: View {
    let catalogue: Catalogue
    let title: String
    let subtitle: String
    let keys: [WorkKey]

    @State private var slideshow = false

    var body: some View {
        let items = catalogue.works(keys)
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                Text(subtitle)
                    .foregroundStyle(.secondary)
                WorkGrid(items: items, showExhibition: true)
            }
            .padding()
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle(title)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button("Diaporama", systemImage: "play.rectangle") { slideshow = true }
                    .disabled(items.isEmpty)
            }
        }
        .fullScreenCover(isPresented: $slideshow) {
            SlideshowView(title: title, items: items.map { ($0.1, $0.2) })
        }
    }
}
