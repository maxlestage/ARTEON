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

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                VStack(alignment: .leading, spacing: 16) {
                    Text("Toute l'histoire de l'art, en 3D")
                        .font(.title3.italic())
                        .foregroundStyle(.tint)
                    HStack(spacing: 12) {
                        StatCard(value: catalogue.exhibitions.count, label: "expositions", color: .arteonBlue)
                        StatCard(value: catalogue.workCount, label: "œuvres", color: .arteonSun)
                        StatCard(value: catalogue.roomCount, label: "salles en 3D", color: .green)
                    }
                }
                .padding(.horizontal)

                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        chip("Tout", .all)
                        if catalogue.exhibitions.contains(where: \.ephemeral) {
                            chip("En ce moment", .now)
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
            if store.offline {
                ToolbarItem(placement: .primaryAction) {
                    Label("Hors ligne : le catalogue gardé sur l'appareil", systemImage: "icloud.slash")
                }
            }
        }
        .refreshable { await store.load() }
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
        .frame(maxWidth: 200)
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
                    Text("EN CE MOMENT")
                        .font(.caption2.bold())
                        .foregroundStyle(exhibition.accentColor)
                }
                Text(exhibition.name)
                    .font(.headline)
                    .lineLimit(2)
                Text([exhibition.movement, "\(exhibition.works.count) œuvres"]
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
