import SwiftUI

/// Une exposition : sa présentation, son bâtiment, ses salles et leurs œuvres, ses artistes.
struct ExhibitionView: View {
    let exhibition: Exhibition

    private var accent: Color { exhibition.accentColor }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                cover
                VStack(alignment: .leading, spacing: 18) {
                    header
                    NavigationLink(value: Route.gallery(exhibition.slug, work: nil)) {
                        Label("Visiter en 3D", systemImage: "view.3d")
                            .font(.headline)
                            .padding(.horizontal, 6)
                    }
                    .buttonStyle(.glassProminent)
                    .tint(accent)

                    Text(exhibition.introduction)
                        .font(.body)
                        .lineSpacing(4)
                        .frame(maxWidth: 760, alignment: .leading)

                    if let museum = exhibition.museum {
                        MuseumCard(museum: museum, accent: accent)
                    }

                    ForEach(exhibition.worksByRoom, id: \.index) { group in
                        VStack(alignment: .leading, spacing: 10) {
                            Text(group.room.name.isEmpty ? "Salle \(group.index + 1)" : group.room.name)
                                .font(.title2.bold())
                            if !group.room.theme.isEmpty {
                                Text(group.room.theme)
                                    .foregroundStyle(.secondary)
                            }
                            LazyVGrid(
                                columns: [GridItem(.adaptive(minimum: 170, maximum: 260), spacing: 14)],
                                spacing: 14
                            ) {
                                ForEach(group.works) { work in
                                    NavigationLink(value: Route.work(exhibition.key(work))) {
                                        WorkCard(work: work)
                                    }
                                    .buttonStyle(.plain)
                                }
                            }
                        }
                        .padding(.top, 18)
                    }

                    if !exhibition.artists.isEmpty {
                        Text(exhibition.artists.count == 1 ? "L'artiste" : "Les artistes")
                            .font(.title2.bold())
                            .padding(.top, 18)
                        VStack(spacing: 0) {
                            ForEach(exhibition.artists) { artist in
                                ArtistRow(artist: artist)
                                Divider()
                            }
                        }
                    }
                }
                .padding()
                .padding(.bottom, 32)
            }
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle(exhibition.name)
        .navigationBarTitleDisplayMode(.inline)
    }

    private var cover: some View {
        RemoteImage(path: exhibition.coverImage, tint: accent.opacity(0.3))
            .frame(height: 340)
            .overlay {
                LinearGradient(
                    colors: [.clear, .black.opacity(0.65)],
                    startPoint: .center,
                    endPoint: .bottom
                )
            }
            .overlay(alignment: .bottomLeading) {
                Text(exhibition.name)
                    .font(.largeTitle.bold())
                    .foregroundStyle(.white)
                    .shadow(radius: 6)
                    .padding()
            }
            .overlay(alignment: .bottom) {
                Rectangle().fill(accent).frame(height: 5)
            }
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 6) {
            if !exhibition.kicker.isEmpty {
                Text(exhibition.kicker.uppercased())
                    .font(.caption.bold())
                    .tracking(1.5)
                    .foregroundStyle(accent)
            }
            Text(exhibition.title)
                .font(.title.bold())
            let line = [exhibition.movement, exhibition.artistDates].filter { !$0.isEmpty }
            if !line.isEmpty {
                Text(line.joined(separator: " · "))
                    .font(.headline)
                    .foregroundStyle(.secondary)
            }
        }
    }
}

struct MuseumCard: View {
    let museum: Museum
    let accent: Color

    var body: some View {
        HStack(alignment: .top, spacing: 14) {
            Capsule().fill(accent).frame(width: 5)
            VStack(alignment: .leading, spacing: 8) {
                Label("Le bâtiment", systemImage: "building.columns.fill")
                    .font(.headline)
                    .foregroundStyle(accent)
                Text([museum.name, museum.city].filter { !$0.isEmpty }.joined(separator: ", "))
                    .font(.subheadline.bold())
                let made = [museum.architect, museum.built].filter { !$0.isEmpty }
                if !made.isEmpty {
                    Text(made.joined(separator: " · "))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                ForEach([museum.presentation, museum.here].filter { !$0.isEmpty }, id: \.self) {
                    Text($0).lineSpacing(3)
                }
            }
        }
        .padding()
        .frame(maxWidth: 760, alignment: .leading)
        .background(Color.card, in: .rect(cornerRadius: 18))
    }
}

/// Un artiste : ses dates, son style, sa biographie à déplier.
struct ArtistRow: View {
    let artist: Artist
    @State private var open = false

    var body: some View {
        DisclosureGroup(isExpanded: $open) {
            Text(artist.biography)
                .lineSpacing(3)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.vertical, 8)
        } label: {
            VStack(alignment: .leading, spacing: 2) {
                Text(artist.name).font(.headline)
                let line = [artist.life, artist.style].filter { !$0.isEmpty }
                if !line.isEmpty {
                    Text(line.joined(separator: " · "))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .padding(.vertical, 8)
    }
}
