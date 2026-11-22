import SwiftUI

/// L'accueil : les expositions, filtrées par style ou « en ce moment ».
struct HomeView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store
    @State private var filter: Filter = .all

    enum Filter: Hashable {
        case all, now
        case movement(String)
    }

    private var shown: [Exhibition] {
        catalogue.exhibitions.filter { ex in
            switch filter {
            case .all: true
            case .now: ex.ephemeral
            case .movement(let m): ex.movement == m
            }
        }
    }

    /// Tous les artistes : ceux des expositions et ceux des collections du monde (le même
    /// nombre que sur le site).
    private var artistTotal: Int {
        let museum = Set((store.index?.artists ?? []).filter { !$0.works.isEmpty }.map { ArtistKey.of($0.name) })
        return museum.union(store.collections?.artistKeys ?? []).count
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                // L'en-tête : le nom d'ARTEON comme celui d'un musée, gravé.
                VStack(alignment: .leading, spacing: 6) {
                    Text("ARTEON")
                        .font(.system(size: 44, weight: .semibold, design: .serif))
                        .tracking(8)
                        .accessibilityAddTraits(.isHeader)
                    Text(L("Toute l'histoire de l'art, dans la poche", "All of art history, in your pocket", "Toda la historia del arte, en el bolsillo"))
                        .font(.system(.title3, design: .serif).italic())
                        .foregroundStyle(.secondary)
                }
                .padding(.horizontal)

                if let daily = catalogue.workOfTheDay() {
                    DailyWorkCard(exhibition: daily.0, work: daily.1)
                        .padding(.horizontal)
                }

                StatsPanel(items: [
                    (catalogue.exhibitions.count, L("expositions", "exhibitions", "exposiciones"), Color.arteonBlue),
                    (catalogue.workCount + (store.collections?.count ?? 0), L("œuvres", "works", "obras"), Color.arteonSun),
                    (artistTotal, L("artistes", "artists", "artistas"), Color.purple),
                    (catalogue.roomCount, L("salles en 3D", "3D rooms", "salas en 3D"), Color.green),
                ])
                .padding(.horizontal)

                let recent = catalogue.works(Array(store.history.prefix(12)))
                if !recent.isEmpty {
                    VStack(alignment: .leading, spacing: 10) {
                        SectionTitle(L("Reprendre la visite", "Resume your visit", "Continuar la visita"))
                            .padding(.horizontal)
                        WorkRow(items: recent)
                    }
                }

                SectionTitle(L("Les expositions", "Exhibitions", "Las exposiciones"))
                    .padding(.horizontal)

                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        chip(L("Tout", "All", "Todo"), .all)
                        if catalogue.exhibitions.contains(where: \.ephemeral) {
                            chip(L("En ce moment", "Now on", "Ahora"), .now)
                        }
                        ForEach(catalogue.movements, id: \.self) { chip($0, .movement($0)) }
                    }
                    .padding(.horizontal)
                }

                LazyVGrid(
                    columns: [GridItem(.adaptive(minimum: 260, maximum: 420), spacing: 16)],
                    spacing: 16
                ) {
                    ForEach(shown) { ex in
                        NavigationLink(value: Route.exhibition(ex.slug)) {
                            ExhibitionCard(exhibition: ex)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal)
            }
            .padding(.vertical)
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle("ARTEON")
        // Le nom est dans l'en-tête de la page : la barre ne garde que les réglages.
        .toolbarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .principal) { Color.clear.frame(width: 1, height: 1) }
            ToolbarItem(placement: .primaryAction) {
                NavigationLink(value: Route.settings) {
                    Label(L("Réglages", "Settings", "Ajustes"), systemImage: "gearshape")
                }
            }
            if store.updating {
                ToolbarItem(placement: .status) {
                    ProgressView().controlSize(.small)
                }
            }
        }
        .refreshable { await store.update() }
    }

    @ViewBuilder
    private func chip(_ title: String, _ value: Filter) -> some View {
        if filter == value {
            Button(title) {}
                .buttonStyle(.glassProminent)
        } else {
            Button(title) {
                withAnimation(.snappy) { filter = value }
            }
            .buttonStyle(.glass)
        }
    }
}

/// Les chiffres d'ARTEON : un seul panneau, deux colonnes sur iPhone, quatre sur iPad ;
/// des chiffres à empattements, comme sur un cartel.
struct StatsPanel: View {
    let items: [(Int, String, Color)]
    @Environment(\.horizontalSizeClass) private var sizeClass

    var body: some View {
        let columns = Array(repeating: GridItem(.flexible(), spacing: 0, alignment: .leading), count: sizeClass == .regular ? 4 : 2)
        LazyVGrid(columns: columns, alignment: .leading, spacing: 18) {
            ForEach(items.indices, id: \.self) { i in
                let (value, label, color) = items[i]
                VStack(alignment: .leading, spacing: 3) {
                    Text(value, format: .number)
                        .font(.system(.title, design: .serif).weight(.semibold))
                        .monospacedDigit()
                        .contentTransition(.numericText())
                        .minimumScaleFactor(0.7)
                        .lineLimit(1)
                    HStack(spacing: 6) {
                        Circle().fill(color).frame(width: 7, height: 7)
                        Text(label)
                            .font(.caption.weight(.semibold))
                            .textCase(.uppercase)
                            .tracking(1.2)
                            .foregroundStyle(.secondary)
                    }
                }
                .accessibilityElement(children: .combine)
            }
        }
        .padding(20)
        .background(Color.card, in: .rect(cornerRadius: 24))
        .overlay(RoundedRectangle(cornerRadius: 24).strokeBorder(.quaternary))
    }
}

/// Le titre d'une partie de l'accueil, à empattements.
struct SectionTitle: View {
    let text: String
    init(_ text: String) { self.text = text }

    var body: some View {
        Text(text)
            .font(.system(.title2, design: .serif).weight(.semibold))
            .accessibilityAddTraits(.isHeader)
    }
}

/// La carte d'une exposition : sa couverture en grand, son nom posé dessus.
struct ExhibitionCard: View {
    let exhibition: Exhibition

    var body: some View {
        ZStack(alignment: .bottomLeading) {
            RemoteImage(path: exhibition.coverImage, tint: exhibition.accentColor.opacity(0.25))
                .frame(height: 300)
                .frame(maxWidth: .infinity)
                .clipped()
            LinearGradient(stops: [.init(color: .clear, location: 0.35), .init(color: .black.opacity(0.85), location: 1)],
                           startPoint: .top, endPoint: .bottom)
            VStack(alignment: .leading, spacing: 6) {
                HStack(spacing: 6) {
                    if exhibition.ephemeral {
                        Text(L("En ce moment", "Now on", "Ahora"))
                            .padding(.horizontal, 8).padding(.vertical, 3)
                            .background(exhibition.accentColor, in: .capsule)
                    }
                    if !exhibition.movement.isEmpty {
                        Text(exhibition.movement)
                            .lineLimit(1)
                            .opacity(0.85)
                    }
                }
                .font(.caption2.weight(.bold))
                .textCase(.uppercase)
                .tracking(1)
                Text(exhibition.name)
                    .font(.system(.title2, design: .serif).weight(.semibold))
                    .lineLimit(2)
                    .multilineTextAlignment(.leading)
                Text(worksCount(exhibition.works.count))
                    .font(.caption)
                    .opacity(0.8)
            }
            .foregroundStyle(.white)
            .padding(18)
            Rectangle().fill(exhibition.accentColor).frame(height: 4)
        }
        .frame(height: 300)
        .clipShape(.rect(cornerRadius: 24))
        .contentShape(.rect(cornerRadius: 24))
        .hoverEffect(.lift)
        .accessibilityElement(children: .combine)
    }
}

/// La vignette d'une œuvre.
struct WorkCard: View {
    let work: Work
    var subtitle: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            RemoteImage(path: work.image)
                .frame(height: 190)
            VStack(alignment: .leading, spacing: 2) {
                Text(work.title)
                    .font(.subheadline.bold())
                    .lineLimit(2)
                Text(subtitle ?? work.byline)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            .padding(10)
            .frame(maxWidth: .infinity, minHeight: 66, alignment: .topLeading)
        }
        .card()
        .accessibilityElement(children: .combine)
    }
}

/// L'œuvre du jour, en grand.
struct DailyWorkCard: View {
    let exhibition: Exhibition
    let work: Work

    var body: some View {
        NavigationLink(value: Route.work(exhibition.key(work))) {
            ZStack(alignment: .bottomLeading) {
                RemoteImage(path: work.image, tint: exhibition.accentColor.opacity(0.3), maxPixels: 1600)
                    .frame(height: 460)
                    .frame(maxWidth: .infinity)
                    .clipped()
                LinearGradient(stops: [.init(color: .clear, location: 0.4), .init(color: .black.opacity(0.88), location: 1)],
                               startPoint: .top, endPoint: .bottom)
                VStack(alignment: .leading, spacing: 8) {
                    // L'étiquette sur un verre sombre : lisible sur toutes les œuvres, même claires.
                    HStack(spacing: 6) {
                        Image(systemName: "sun.max.fill").foregroundStyle(Color.arteonSun)
                        Text(L("L'œuvre du jour", "Work of the day", "La obra del día"))
                            .textCase(.uppercase)
                            .tracking(1.2)
                    }
                    .font(.caption.weight(.bold))
                    .padding(.horizontal, 10).padding(.vertical, 5)
                    .background(.black.opacity(0.35), in: .capsule)
                    .overlay(Capsule().strokeBorder(.white.opacity(0.25)))
                    Text(work.title)
                        .font(.system(size: 34, weight: .semibold, design: .serif))
                        .lineLimit(3)
                        .minimumScaleFactor(0.75)
                    Text("\(work.byline) · \(exhibition.name)")
                        .font(.subheadline)
                        .lineLimit(1)
                        .opacity(0.85)
                }
                .foregroundStyle(.white)
                .padding(22)
            }
            .frame(height: 460)
            .clipShape(.rect(cornerRadius: 28))
            .shadow(color: .black.opacity(0.18), radius: 18, y: 10)
            .contentShape(.rect(cornerRadius: 28))
            .hoverEffect(.lift)
        }
        .buttonStyle(.plain)
        .accessibilityElement(children: .combine)
        .accessibilityHint(L("Ouvre l'œuvre du jour", "Opens the work of the day", "Abre la obra del día"))
    }
}

/// Des œuvres sur une ligne qui défile.
struct WorkRow: View {
    let items: [(WorkKey, Exhibition, Work)]

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            LazyHStack(spacing: 14) {
                ForEach(items, id: \.0) { item in
                    NavigationLink(value: Route.work(item.0)) {
                        WorkCard(work: item.2, subtitle: item.2.artist.isEmpty ? item.1.name : item.2.artist)
                            .frame(width: 190)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal)
        }
        .scrollClipDisabled()
    }
}
