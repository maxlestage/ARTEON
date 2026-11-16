import SwiftUI

/// Les musées : les vrais musées du monde dont ARTEON reproduit les bâtiments, pays par
/// pays, et le Grand Musée ARTEON, un bâtiment imaginaire qui les réunit tous.
struct MuseumsView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store
    @State private var filter = ""

    private var countries: [CountryGroup] {
        let all = store.index?.countries ?? []
        let q = folded(filter.trimmingCharacters(in: .whitespaces))
        guard !q.isEmpty else { return all }
        return all.compactMap { group in
            let found = group.buildings.filter {
                folded("\($0.museum.name) \($0.museum.city) \($0.museum.architect)").contains(q)
            }
            return found.isEmpty ? nil : CountryGroup(name: group.name, buildings: found)
        }
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                if filter.isEmpty {
                    Text("Chaque exposition d'ARTEON est installée dans la reproduction d'un vrai musée : sa façade, ses enfilades, ses verrières, sa lumière. Le Grand Musée ARTEON, lui, n'existe nulle part ailleurs.")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: 760, alignment: .leading)

                    GrandMuseumCard(catalogue: catalogue)
                    WorldCollectionsCard()

                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 12) {
                            StatCard(value: store.index?.buildings.count ?? 0, label: "musées reproduits", color: .arteonBlue)
                            StatCard(value: store.index?.countries.count ?? 0, label: "pays", color: .teal)
                            StatCard(value: 1, label: "musée original", color: .arteonSun)
                        if let collections = store.collections {
                            StatCard(value: collections.count, label: "œuvres des collections", color: .orange)
                        }
                        }
                    }
                    .scrollClipDisabled()
                }

                ForEach(countries) { country in
                    VStack(alignment: .leading, spacing: 12) {
                        Text(country.name)
                            .font(.title2.bold())
                        LazyVGrid(
                            columns: [GridItem(.adaptive(minimum: 260, maximum: 420), spacing: 16)],
                            spacing: 16
                        ) {
                            ForEach(country.buildings) { building in
                                NavigationLink(value: Route.building(building.museum.name)) {
                                    BuildingCard(catalogue: catalogue, building: building)
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }
                }
            }
            .padding()
        }
        .overlay {
            if countries.isEmpty, !filter.isEmpty {
                ContentUnavailableView.search(text: filter)
            }
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle("Musées")
        .searchable(text: $filter, prompt: "Un musée, une ville, un architecte…")
    }
}

/// La carte d'un vrai musée : l'œuvre de couverture, son nom, sa ville, son architecte.
struct BuildingCard: View {
    let catalogue: Catalogue
    let building: Building

    private var exhibitions: [Exhibition] { building.exhibitions.compactMap(catalogue.exhibition) }

    var body: some View {
        let first = exhibitions.first
        VStack(alignment: .leading, spacing: 0) {
            ZStack(alignment: .topTrailing) {
                RemoteImage(path: first?.coverImage, tint: (first?.accentColor ?? .gray).opacity(0.25))
                    .frame(height: 180)
                if first?.hasPlan == true, let first {
                    MiniPlan(exhibition: first)
                        .frame(width: 92, height: 92)
                        .padding(6)
                        .background(.regularMaterial, in: .rect(cornerRadius: 12))
                        .padding(10)
                }
            }
            Rectangle().fill(first?.accentColor ?? .gray).frame(height: 5)
            VStack(alignment: .leading, spacing: 4) {
                Text(building.museum.name)
                    .font(.headline)
                    .lineLimit(2)
                Text(building.museum.city)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                let line = [building.museum.architect, building.museum.built].filter { !$0.isEmpty }
                if !line.isEmpty {
                    Text(line.joined(separator: " · "))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
                Text(exhibitions.map(\.name).joined(separator: " · "))
                    .font(.caption.bold())
                    .foregroundStyle(first?.accentColor ?? .secondary)
                    .lineLimit(1)
            }
            .padding(14)
            .frame(maxWidth: .infinity, minHeight: 108, alignment: .topLeading)
        }
        .card()
        .accessibilityElement(children: .combine)
    }
}

/// Le plan en miniature, sans interaction.
struct MiniPlan: View {
    let exhibition: Exhibition

    var body: some View {
        FloorPlanView(exhibition: exhibition, selected: .constant(nil))
            .allowsHitTesting(false)
            .environment(\.colorScheme, .light)
            .accessibilityHidden(true)
    }
}

/// Un vrai musée : son histoire, ce qu'ARTEON lui emprunte, et pour chaque exposition
/// qu'il abrite, le plan interactif de ses salles et l'entrée en 3D.
struct BuildingView: View {
    let catalogue: Catalogue
    let name: String

    @Environment(CatalogueStore.self) private var store
    @Environment(Narrator.self) private var narrator

    var body: some View {
        if let building = store.index?.building(named: name) {
            let exhibitions = building.exhibitions.compactMap(catalogue.exhibition)
            let accent = exhibitions.first?.accentColor ?? .arteonBlue
            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    RemoteImage(path: exhibitions.first?.coverImage, tint: accent.opacity(0.3), maxPixels: 1600)
                        .frame(height: 300)
                        .overlay {
                            LinearGradient(colors: [.clear, .black.opacity(0.7)], startPoint: .center, endPoint: .bottom)
                        }
                        .overlay(alignment: .bottomLeading) {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(building.museum.name)
                                    .font(.largeTitle.bold())
                                Text(building.museum.city)
                                    .font(.headline)
                                    .opacity(0.85)
                            }
                            .foregroundStyle(.white)
                            .shadow(radius: 6)
                            .padding()
                        }
                        .overlay(alignment: .bottom) { Rectangle().fill(accent).frame(height: 5) }

                    VStack(alignment: .leading, spacing: 18) {
                        Grid(alignment: .leadingFirstTextBaseline, horizontalSpacing: 16, verticalSpacing: 6) {
                            fact("Architecte", building.museum.architect)
                            fact("Construction", building.museum.built)
                            fact("Pays", building.country)
                        }

                        ListenButton(
                            id: "musee/\(building.museum.name)",
                            text: { "\(building.museum.name), \(building.museum.city). \(building.museum.presentation)\n\(building.museum.here)" },
                            label: "Écouter la visite du bâtiment"
                        )
                        .buttonStyle(.glass)

                        if !building.museum.presentation.isEmpty {
                            section("Le vrai musée", building.museum.presentation, accent: accent)
                        }
                        if !building.museum.here.isEmpty {
                            section("Ce qu'ARTEON lui emprunte", building.museum.here, accent: accent)
                        }

                        ForEach(exhibitions) { ex in
                            VStack(alignment: .leading, spacing: 12) {
                                Text(exhibitions.count > 1 ? ex.name : "Le plan")
                                    .font(.title2.bold())
                                    .padding(.top, 10)
                                if exhibitions.count > 1 {
                                    Text("Une des \(exhibitions.count) expositions installées dans ce bâtiment.")
                                        .font(.subheadline)
                                        .foregroundStyle(.secondary)
                                }
                                if ex.hasPlan {
                                    BuildingPlan(exhibition: ex)
                                }
                                HStack(spacing: 10) {
                                    NavigationLink(value: Route.gallery(ex.slug, work: nil)) {
                                        Label("Entrer en 3D", systemImage: "view.3d")
                                            .font(.headline)
                                    }
                                    .buttonStyle(.glassProminent)
                                    .tint(ex.accentColor)
                                    NavigationLink(value: Route.exhibition(ex.slug)) {
                                        Label("L'exposition", systemImage: "photo.on.rectangle")
                                    }
                                    .buttonStyle(.glass)
                                }
                            }
                        }
                    }
                    .padding()
                    .padding(.bottom, 32)
                    .frame(maxWidth: 900, alignment: .leading)
                }
            }
            .background(Color(uiColor: .systemGroupedBackground))
            .navigationTitle(building.museum.name)
            .navigationBarTitleDisplayMode(.inline)
            .onDisappear { narrator.stop() }
        } else {
            ContentUnavailableView("Musée introuvable", systemImage: "building.columns")
        }
    }

    private func section(_ title: String, _ text: String, accent: Color) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.headline)
                .foregroundStyle(accent)
            Text(text)
                .lineSpacing(4)
                .textSelection(.enabled)
        }
        .frame(maxWidth: 760, alignment: .leading)
    }

    @ViewBuilder
    private func fact(_ label: String, _ value: String) -> some View {
        if !value.isEmpty {
            GridRow {
                Text(label).font(.caption).foregroundStyle(.secondary)
                Text(value).font(.subheadline)
            }
        }
    }
}

// MARK: - Le Grand Musée ARTEON

/// L'entrée du Grand Musée ARTEON, sur la page des musées.
struct GrandMuseumCard: View {
    let catalogue: Catalogue

    var body: some View {
        NavigationLink(value: Route.grandMuseum) {
            HStack(spacing: 18) {
                ArteonLogo(size: 64)
                VStack(alignment: .leading, spacing: 4) {
                    Text("MUSÉE ORIGINAL")
                        .font(.caption.bold())
                        .tracking(1.5)
                        .foregroundStyle(Color.arteonSun)
                    Text("Le Grand Musée ARTEON")
                        .font(.title2.bold())
                    Text("Un bâtiment imaginaire : \(catalogue.exhibitions.count) pavillons en six ailes, de la Terre au street art.")
                        .font(.subheadline)
                        .opacity(0.85)
                }
                Spacer(minLength: 0)
                Image(systemName: "chevron.right")
                    .font(.headline)
                    .opacity(0.6)
            }
            .foregroundStyle(.white)
            .padding(20)
            .background(
                LinearGradient(colors: [.arteonNight, .arteonBlue], startPoint: .topLeading, endPoint: .bottomTrailing),
                in: .rect(cornerRadius: 24)
            )
            .contentShape(.rect(cornerRadius: 24))
            .hoverEffect(.lift)
        }
        .buttonStyle(.plain)
    }
}

/// Le Grand Musée ARTEON : un musée qui n'existe nulle part ailleurs. Un hall, puis six
/// ailes en enfilade, de la plus ancienne à la plus récente ; chaque pavillon est une
/// exposition, dans la reproduction d'un vrai musée.
struct GrandMuseumView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store

    private let paper = Color(red: 0.97, green: 0.96, blue: 0.93)
    private let ink = Color(red: 0.2, green: 0.19, blue: 0.18)

    var body: some View {
        let wings = store.index?.wings ?? []
        ScrollView {
            VStack(spacing: 0) {
                hall(wings: wings)
                ForEach(Array(wings.enumerated()), id: \.element.id) { i, wing in
                    corridor
                    wingView(wing, number: i + 1)
                }
                corridor
                Text("Sortie · Jardin des sculptures")
                    .font(.caption.bold())
                    .foregroundStyle(ink.opacity(0.7))
                    .padding(.vertical, 12)
            }
            .padding()
            .frame(maxWidth: 980)
            .frame(maxWidth: .infinity)
        }
        .background(paper)
        .environment(\.colorScheme, .light)
        .navigationTitle("Le Grand Musée ARTEON")
        .navigationBarTitleDisplayMode(.inline)
    }

    private func hall(wings: [Wing]) -> some View {
        VStack(spacing: 10) {
            ArteonLogo(size: 56)
            Text("Le Grand Musée ARTEON")
                .font(.system(.largeTitle, design: .serif).bold())
            Text("Un musée original, qui n'existe que dans l'application : \(catalogue.exhibitions.count) pavillons, chacun dans la reproduction d'un vrai musée, réunis en \(wings.count) ailes. On y entre par le hall, on suit le temps.")
                .multilineTextAlignment(.center)
                .foregroundStyle(ink.opacity(0.75))
                .frame(maxWidth: 620)
            Label("Hall d'entrée", systemImage: "door.left.hand.open")
                .font(.caption.bold())
                .padding(.horizontal, 14)
                .padding(.vertical, 6)
                .overlay(Capsule().strokeBorder(ink.opacity(0.5)))
        }
        .foregroundStyle(ink)
        .padding(24)
        .frame(maxWidth: .infinity)
        .overlay(RoundedRectangle(cornerRadius: 6).strokeBorder(ink, lineWidth: 2))
    }

    /// Le couloir entre deux ailes.
    private var corridor: some View {
        Rectangle()
            .fill(ink.opacity(0.35))
            .frame(width: 3, height: 34)
    }

    private func wingView(_ wing: Wing, number: Int) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                Text("AILE \(romanNumeral(number))")
                    .font(.caption.bold())
                    .tracking(1.2)
                    .foregroundStyle(ink.opacity(0.6))
                Text(wing.title)
                    .font(.system(.title2, design: .serif).bold())
                Text(wing.subtitle)
                    .font(.subheadline)
                    .foregroundStyle(ink.opacity(0.65))
            }
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 160, maximum: 320), spacing: 0)], spacing: 0) {
                ForEach(wing.exhibitions, id: \.self) { slug in
                    if let ex = catalogue.exhibition(slug) {
                        NavigationLink(value: ex.museum.map { Route.building($0.name) } ?? Route.exhibition(ex.slug)) {
                            pavilion(ex)
                        }
                        .buttonStyle(.plain)
                    }
                }
            }
            .overlay(Rectangle().strokeBorder(ink, lineWidth: 2))
        }
        .foregroundStyle(ink)
        .padding(14)
        .background(.white.opacity(0.55))
        .overlay(RoundedRectangle(cornerRadius: 4).strokeBorder(ink.opacity(0.6), lineWidth: 1))
    }

    /// Un pavillon : une salle du plan, à la couleur de l'exposition.
    private func pavilion(_ ex: Exhibition) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .top) {
                RemoteImage(path: ex.coverImage, tint: ex.accentColor.opacity(0.3), maxPixels: 300)
                    .frame(width: 54, height: 54)
                    .clipShape(.rect(cornerRadius: 4))
                Spacer()
                if ex.hasPlan {
                    MiniPlan(exhibition: ex)
                        .frame(width: 54, height: 54)
                }
            }
            Text(ex.name)
                .font(.subheadline.bold())
                .lineLimit(2)
            Text(ex.museum?.name ?? ex.movement)
                .font(.caption)
                .opacity(0.7)
                .lineLimit(2)
        }
        .foregroundStyle(ink)
        .padding(12)
        .frame(maxWidth: .infinity, minHeight: 150, alignment: .topLeading)
        .background(ex.accentColor.opacity(0.12))
        .overlay(Rectangle().strokeBorder(ink.opacity(0.35), lineWidth: 0.75))
        .contentShape(.rect)
        .hoverEffect(.highlight)
        .accessibilityElement(children: .combine)
    }
}

/// L'entrée des collections du monde, sur la page des musées et dans Explorer.
struct WorldCollectionsCard: View {
    @Environment(CatalogueStore.self) private var store

    var body: some View {
        if LocalMuseum.hasCollections {
            NavigationLink(value: Route.worldCollections) {
                HStack(spacing: 18) {
                    Image(systemName: "square.stack.3d.up.fill")
                        .font(.system(size: 34))
                        .frame(width: 64, height: 64)
                        .background(.white.opacity(0.15), in: .rect(cornerRadius: 16))
                    VStack(alignment: .leading, spacing: 4) {
                        Text("DOMAINE PUBLIC")
                            .font(.caption.bold())
                            .tracking(1.5)
                            .opacity(0.8)
                        Text("Les collections du monde")
                            .font(.title2.bold())
                        Text(store.collections.map {
                            "\($0.count.formatted()) œuvres de \($0.collections.museums.count) grands musées, sur l'appareil."
                        } ?? "Des dizaines de milliers d'œuvres, sur l'appareil.")
                            .font(.subheadline)
                            .opacity(0.85)
                    }
                    Spacer(minLength: 0)
                    Image(systemName: "chevron.right").font(.headline).opacity(0.6)
                }
                .foregroundStyle(.white)
                .padding(20)
                .background(
                    LinearGradient(colors: [Color(red: 0.42, green: 0.12, blue: 0.16), Color(red: 0.75, green: 0.32, blue: 0.18)],
                                   startPoint: .topLeading, endPoint: .bottomTrailing),
                    in: .rect(cornerRadius: 24)
                )
                .contentShape(.rect(cornerRadius: 24))
                .hoverEffect(.lift)
            }
            .buttonStyle(.plain)
        }
    }
}
