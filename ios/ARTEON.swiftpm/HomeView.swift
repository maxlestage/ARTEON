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
                VStack(alignment: .leading, spacing: 16) {
                    Text(L("Toute l'histoire de l'art, dans la poche", "All of art history, in your pocket", "Toda la historia del arte, en el bolsillo"))
                        .font(.title3.italic())
                        .foregroundStyle(.tint)
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 12) {
                            StatCard(value: catalogue.exhibitions.count, label: L("expositions", "exhibitions", "exposiciones"), color: .arteonBlue)
                            StatCard(value: catalogue.workCount + (store.collections?.count ?? 0), label: L("œuvres", "works", "obras"), color: .arteonSun)
                            StatCard(value: artistTotal, label: L("artistes", "artists", "artistas"), color: .purple)
                            StatCard(value: catalogue.roomCount, label: L("salles en 3D", "3D rooms", "salas en 3D"), color: .green)
                        }
                    }
                    .scrollClipDisabled()
                }
                .padding(.horizontal)

                if let daily = catalogue.workOfTheDay() {
                    DailyWorkCard(exhibition: daily.0, work: daily.1)
                        .padding(.horizontal)
                }

                let recent = catalogue.works(Array(store.history.prefix(12)))
                if !recent.isEmpty {
                    VStack(alignment: .leading, spacing: 10) {
                        Text(L("Reprendre la visite", "Resume your visit", "Continuar la visita"))
                            .font(.title2.bold())
                            .padding(.horizontal)
                        WorkRow(items: recent)
                    }
                }

                Text(L("Les expositions", "Exhibitions", "Las exposiciones"))
                    .font(.title2.bold())
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
        .toolbar {
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

struct StatCard: View {
    let value: Int
    let label: String
    let color: Color

    var body: some View {
        HStack(spacing: 12) {
            Capsule().fill(color).frame(width: 5)
            VStack(alignment: .leading, spacing: 2) {
                Text(value, format: .number)
                    .font(.title.bold())
                    .contentTransition(.numericText())
                Text(label)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
        }
        .padding(14)
        .frame(minWidth: 150, maxWidth: 200)
        .fixedSize(horizontal: false, vertical: true)
        .background(Color.card, in: .rect(cornerRadius: 16))
    }
}

/// La carte d'une exposition : sa couverture, son nom, son style.
struct ExhibitionCard: View {
    let exhibition: Exhibition

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            RemoteImage(path: exhibition.coverImage, tint: exhibition.accentColor.opacity(0.25))
                .frame(height: 230)
            Rectangle().fill(exhibition.accentColor).frame(height: 5)
            VStack(alignment: .leading, spacing: 4) {
                if exhibition.ephemeral {
                    Text(L("EN CE MOMENT", "NOW ON", "AHORA"))
                        .font(.caption2.bold())
                        .foregroundStyle(exhibition.accentColor)
                }
                Text(exhibition.name)
                    .font(.headline)
                    .lineLimit(2)
                Text([exhibition.movement, worksCount(exhibition.works.count)]
                    .filter { !$0.isEmpty }
                    .joined(separator: " · "))
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            .padding(14)
            .frame(maxWidth: .infinity, minHeight: 86, alignment: .topLeading)
        }
        .card()
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
                RemoteImage(path: work.image, tint: exhibition.accentColor.opacity(0.3), maxPixels: 1400)
                    .frame(height: 320)
                LinearGradient(colors: [.clear, .black.opacity(0.75)], startPoint: .center, endPoint: .bottom)
                VStack(alignment: .leading, spacing: 4) {
                    Label(L("L'œuvre du jour", "Work of the day", "La obra del día"), systemImage: "sun.max.fill")
                        .font(.caption.bold())
                        .textCase(.uppercase)
                        .foregroundStyle(Color.arteonSun)
                    Text(work.title)
                        .font(.title.bold())
                        .lineLimit(2)
                    Text("\(work.byline) · \(exhibition.name)")
                        .font(.subheadline)
                        .lineLimit(1)
                        .opacity(0.85)
                }
                .foregroundStyle(.white)
                .padding(20)
            }
            .clipShape(.rect(cornerRadius: 24))
            .contentShape(.rect(cornerRadius: 24))
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
