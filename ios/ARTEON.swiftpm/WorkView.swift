import SwiftUI

/// Une œuvre : son image à agrandir, son cartel, sa notice complète, son artiste. Sur iPad
/// à l'horizontale, l'image à gauche et le texte à droite ; précédente et suivante dans la
/// barre.
struct WorkView: View {
    let exhibition: Exhibition

    @Environment(CatalogueStore.self) private var store
    @Environment(\.horizontalSizeClass) private var sizeClass
    @State private var index: Int
    @State private var zooming = false

    init(exhibition: Exhibition, work: Work) {
        self.exhibition = exhibition
        _index = State(initialValue: exhibition.works.firstIndex(of: work) ?? 0)
    }

    private var work: Work { exhibition.works[index] }
    private var key: WorkKey { exhibition.key(work) }

    var body: some View {
        GeometryReader { geo in
            if sizeClass == .regular && geo.size.width > geo.size.height * 1.1 {
                HStack(spacing: 0) {
                    picture
                        .padding(40)
                        .frame(width: geo.size.width * 0.55)
                        .frame(maxHeight: .infinity)
                        .background(Color.galleryWall)
                    ScrollView {
                        WorkText(exhibition: exhibition, work: work)
                            .padding(32)
                    }
                }
            } else {
                ScrollView {
                    VStack(spacing: 0) {
                        picture
                            .padding(24)
                            .frame(height: max(geo.size.height * 0.55, 280))
                            .frame(maxWidth: .infinity)
                            .background(Color.galleryWall)
                        WorkText(exhibition: exhibition, work: work)
                            .padding()
                            .padding(.bottom, 32)
                    }
                }
            }
        }
        .id(index)
        .navigationTitle(exhibition.name)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItemGroup(placement: .primaryAction) {
                let favorite = store.isFavorite(key)
                Button {
                    store.toggleFavorite(key)
                } label: {
                    Label(
                        favorite ? "Retirer des favoris" : "Ajouter aux favoris",
                        systemImage: favorite ? "heart.fill" : "heart"
                    )
                }
                .tint(favorite ? Color.red : nil)
                .sensoryFeedback(.selection, trigger: favorite)

                NavigationLink(value: Route.gallery(exhibition.slug, work: work.slug)) {
                    Label("Voir dans la galerie 3D", systemImage: "view.3d")
                }
            }
            ToolbarItemGroup(placement: .secondaryAction) {
                Button("Œuvre précédente", systemImage: "chevron.left") { step(-1) }
                    .disabled(index == 0)
                Button("Œuvre suivante", systemImage: "chevron.right") { step(1) }
                    .disabled(index >= exhibition.works.count - 1)
            }
        }
        .fullScreenCover(isPresented: $zooming) {
            ZoomView(path: work.image, title: work.title)
        }
    }

    private var picture: some View {
        RemoteImage(path: work.image, contentMode: .fit, tint: .clear)
            .aspectRatio(work.aspect, contentMode: .fit)
            .padding(8)
            .background(Color(red: 0.17, green: 0.14, blue: 0.13))
            .shadow(color: .black.opacity(0.35), radius: 20, y: 14)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .contentShape(.rect)
            .onTapGesture { zooming = true }
            .accessibilityLabel("\(work.title), toucher pour agrandir")
            .accessibilityAddTraits(.isButton)
    }

    private func step(_ delta: Int) {
        let next = index + delta
        guard exhibition.works.indices.contains(next) else { return }
        withAnimation(.easeInOut(duration: 0.2)) { index = next }
    }
}

/// Le cartel et la notice d'une œuvre.
struct WorkText: View {
    let exhibition: Exhibition
    let work: Work

    private var accent: Color { exhibition.accentColor }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(work.title)
                .font(.title.bold())
            Text(work.byline)
                .font(.title3)
                .foregroundStyle(.secondary)
                .padding(.top, 4)

            Grid(alignment: .leadingFirstTextBaseline, horizontalSpacing: 16, verticalSpacing: 6) {
                fact("Technique", work.technique)
                fact("Collection", work.collection)
                if exhibition.rooms.indices.contains(work.room) {
                    fact("Salle", exhibition.rooms[work.room].name)
                }
            }
            .padding(.top, 16)

            if !work.note.isEmpty {
                Text(work.note)
                    .italic()
                    .lineSpacing(3)
                    .padding(.top, 18)
            }

            ForEach(work.notice.sections, id: \.title) { section in
                heading(section.title)
                Text(section.text).lineSpacing(4)
            }

            if !work.notice.details.isEmpty {
                heading("À regarder de près")
                VStack(alignment: .leading, spacing: 10) {
                    ForEach(work.notice.details, id: \.self) { detail in
                        HStack(alignment: .firstTextBaseline, spacing: 10) {
                            Circle().fill(accent).frame(width: 7, height: 7)
                            Text(detail).lineSpacing(3)
                        }
                    }
                }
            }

            if !work.notice.themes.isEmpty {
                heading("Thèmes")
                FlowLayout {
                    ForEach(work.notice.themes, id: \.self) { theme in
                        Text(theme)
                            .font(.subheadline)
                            .padding(.horizontal, 12)
                            .padding(.vertical, 6)
                            .background(accent.opacity(0.12), in: .capsule)
                    }
                }
            }

            if !work.notice.anecdote.isEmpty {
                heading("Le saviez-vous ?")
                Text(work.notice.anecdote)
                    .lineSpacing(4)
                    .padding()
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(accent.opacity(0.1), in: .rect(cornerRadius: 14))
            }

            if let artist = exhibition.artist(named: work.artist) {
                heading("L'artiste")
                ArtistRow(artist: artist)
            }

            if let credit = work.credit, !credit.author.isEmpty {
                heading("Reproduction")
                Text("\(credit.author) · \(credit.licence)")
                    .font(.subheadline)
                if credit.source.hasPrefix("https://"), let url = URL(string: credit.source) {
                    Link(destination: url) {
                        Label("Source", systemImage: "arrow.up.right.square")
                    }
                    .padding(.top, 4)
                }
            }
        }
        .frame(maxWidth: 720, alignment: .leading)
        .textSelection(.enabled)
    }

    private func heading(_ text: String) -> some View {
        Text(text)
            .font(.headline)
            .foregroundStyle(accent)
            .padding(.top, 26)
            .padding(.bottom, 8)
    }

    @ViewBuilder
    private func fact(_ label: String, _ value: String) -> some View {
        if !value.isEmpty {
            GridRow {
                Text(label)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text(value)
                    .font(.subheadline)
            }
        }
    }
}

/// L'image en grand, sur fond noir : pincer pour agrandir, glisser pour se déplacer,
/// toucher deux fois pour revenir.
struct ZoomView: View {
    let path: String
    let title: String

    @Environment(\.dismiss) private var dismiss
    @State private var scale: CGFloat = 1
    @State private var lastScale: CGFloat = 1
    @State private var offset: CGSize = .zero
    @State private var lastOffset: CGSize = .zero

    var body: some View {
        NavigationStack {
            RemoteImage(path: path, contentMode: .fit, tint: .clear)
                .scaleEffect(scale)
                .offset(offset)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .contentShape(.rect)
                .gesture(magnify.simultaneously(with: drag))
                .onTapGesture(count: 2) { reset() }
                .background(Color.black.ignoresSafeArea())
                .navigationTitle(title)
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .confirmationAction) {
                        Button("Fermer", systemImage: "xmark") { dismiss() }
                    }
                }
        }
        .preferredColorScheme(.dark)
    }

    private var magnify: some Gesture {
        MagnifyGesture()
            .onChanged { value in
                scale = min(max(lastScale * value.magnification, 1), 8)
            }
            .onEnded { _ in
                lastScale = scale
                if scale <= 1.01 { reset() }
            }
    }

    private var drag: some Gesture {
        DragGesture()
            .onChanged { value in
                guard scale > 1 else { return }
                offset = CGSize(
                    width: lastOffset.width + value.translation.width,
                    height: lastOffset.height + value.translation.height
                )
            }
            .onEnded { _ in lastOffset = offset }
    }

    private func reset() {
        withAnimation(.spring) {
            scale = 1
            lastScale = 1
            offset = .zero
            lastOffset = .zero
        }
    }
}
