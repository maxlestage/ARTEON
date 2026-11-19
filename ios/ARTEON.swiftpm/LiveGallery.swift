import SwiftUI

// Comme une galerie de photos : une œuvre des musées du monde ouverte en grand, sans quitter
// l'application (le pendant de outils/monde/visionneuse.js sur le site). On glisse d'une
// œuvre à l'autre ; « Fiche » déplie la technique, la collection, le musée et le lien vers
// sa fiche.

/// L'image en grand : la vignette tout de suite, puis la grande image quand elle arrive
/// (certains musées la refusent parfois : la vignette reste).
struct LiveWorkPicture: View {
    let work: LiveWork

    @State private var image: UIImage?

    var body: some View {
        ZStack {
            if let image {
                Image(uiImage: image)
                    .resizable()
                    .scaledToFit()
            } else {
                ProgressView()
                    .tint(.white)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .task(id: work.id) { await load() }
        .accessibilityLabel(work.title)
    }

    private func load() async {
        image = nil
        if let small = await ImagePipeline.shared.image(work.thumbnail, maxPixels: 640) {
            image = small
        }
        if !work.image.isEmpty, work.image != work.thumbnail,
           let large = await ImagePipeline.shared.image(work.image, maxPixels: 2000) {
            image = large
        }
    }
}

/// La visionneuse : les œuvres une à une, en plein écran, sur fond noir.
struct LiveWorkViewer: View {
    let works: [LiveWork]
    let museums: [String: String]
    let artist: String

    @State private var current: String
    @State private var details = false
    @Environment(\.dismiss) private var dismiss

    init(works: [LiveWork], start: LiveWork, museums: [String: String], artist: String) {
        self.works = works
        self.museums = museums
        self.artist = artist
        _current = State(initialValue: start.id)
    }

    private var index: Int {
        works.firstIndex { $0.id == current } ?? 0
    }

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()
            VStack(spacing: 0) {
                HStack {
                    Text(verbatim: "\(index + 1) / \(works.count)")
                        .font(.subheadline.monospacedDigit())
                        .foregroundStyle(Color.white.opacity(0.75))
                    Spacer()
                    Button {
                        dismiss()
                    } label: {
                        Image(systemName: "xmark")
                            .font(.headline)
                            .foregroundStyle(Color.white)
                            .frame(width: 44, height: 44)
                            .background(Color.white.opacity(0.15), in: Circle())
                    }
                    .accessibilityLabel(L("Fermer", "Close", "Cerrar"))
                }
                .padding(.horizontal, 16)
                .padding(.top, 8)
                TabView(selection: $current) {
                    ForEach(works) { work in
                        LiveWorkPicture(work: work)
                            .tag(work.id)
                    }
                }
                .tabViewStyle(.page(indexDisplayMode: .never))
                if works.indices.contains(index) {
                    caption(works[index])
                }
            }
        }
        .preferredColorScheme(.dark)
    }

    private func caption(_ work: LiveWork) -> some View {
        let museum = (museums[work.museum] ?? work.museum).components(separatedBy: " (").first ?? work.museum
        let byline = [artist, work.date].filter { !$0.isEmpty }.joined(separator: " · ")
        let linkTitle = work.museum == "wikidata"
            ? L("La fiche sur Wikidata", "On Wikidata", "La ficha en Wikidata")
            : L("La fiche du musée", "On the museum's website", "La ficha del museo")
        return VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .top, spacing: 12) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(work.title.isEmpty ? L("Sans titre", "Untitled", "Sin título") : work.title)
                        .font(.headline)
                        .foregroundStyle(Color.white)
                        .lineLimit(details ? nil : 2)
                    Text(byline)
                        .font(.subheadline)
                        .foregroundStyle(Color.white.opacity(0.7))
                }
                Spacer(minLength: 0)
                Button {
                    withAnimation { details.toggle() }
                } label: {
                    Label(L("Fiche", "Details", "Ficha"), systemImage: "info.circle")
                        .font(.subheadline)
                        .foregroundStyle(Color.white)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 8)
                        .background(Color.white.opacity(details ? 0.3 : 0.15), in: Capsule())
                }
            }
            if details {
                fact(L("Technique", "Medium", "Técnica"), work.medium)
                fact(L("Collection", "Collection", "Colección"), work.provider ?? "")
                fact(L("Source", "Source", "Fuente"), museum)
                if let url = URL(string: work.link) {
                    Link(destination: url) {
                        Label(linkTitle, systemImage: "arrow.up.right.square")
                            .font(.subheadline.bold())
                    }
                    .tint(Color(red: 0.62, green: 0.76, blue: 1))
                    .padding(.top, 4)
                }
                Text(L("Domaine public · image et fiche chez le musée",
                       "Public domain · image and record hosted by the museum",
                       "Dominio público · imagen y ficha en el museo"))
                    .font(.caption)
                    .foregroundStyle(Color.white.opacity(0.6))
            }
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 12)
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    @ViewBuilder
    private func fact(_ name: String, _ value: String) -> some View {
        if !value.isEmpty {
            VStack(alignment: .leading, spacing: 1) {
                Text(name)
                    .font(.caption.bold())
                    .foregroundStyle(Color.white.opacity(0.7))
                Text(value)
                    .font(.subheadline)
                    .foregroundStyle(Color.white)
            }
        }
    }
}
