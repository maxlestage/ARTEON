import SwiftUI
import ImageIO

extension Color {
    /// Le bleu d'ARTEON, la nuit et le soleil du logo.
    static let arteonBlue = Color(red: 0x1d / 255, green: 0x3f / 255, blue: 0x8f / 255)
    static let arteonNight = Color(red: 0x0f / 255, green: 0x1a / 255, blue: 0x3a / 255)
    static let arteonSun = Color(red: 0xf2 / 255, green: 0xb7 / 255, blue: 0x05 / 255)
    /// Le mur clair derrière une œuvre.
    static let galleryWall = Color(red: 0.93, green: 0.92, blue: 0.89)
    /// Le fond des cartes.
    static let card = Color(uiColor: .secondarySystemGroupedBackground)
}

extension Exhibition {
    var accentColor: Color {
        let c = accentRGB
        return Color(red: c.red, green: c.green, blue: c.blue)
    }
}

/// Où mène un toucher.
enum Route: Hashable {
    case exhibition(String)
    case work(WorkKey)
    case gallery(String, work: String?)
    case artist(String)
    case works(title: String, subtitle: String, keys: [WorkKey])
    case timeline
    case themes
    case collections
    case techniques
    case quiz
    case settings
    case building(String)
    case grandMuseum
    case artists
    case worldCollections
    case collection(CollectionFilter)
    case collectionWork(String)
}

/// Les images, lues sur l'appareil et décodées à la taille où elles s'affichent : les
/// vignettes restent légères, l'image en grand garde tous ses pixels.
actor ImagePipeline {
    static let shared = ImagePipeline()

    private let cache: NSCache<NSString, UIImage> = {
        let cache = NSCache<NSString, UIImage>()
        cache.totalCostLimit = 160 * 1024 * 1024
        return cache
    }()

    /// `maxPixels` : le plus grand côté voulu, en pixels (`nil` : l'image entière).
    func image(_ path: String, maxPixels: Int?) async -> UIImage? {
        let key = "\(maxPixels ?? 0)|\(path)" as NSString
        if let image = cache.object(forKey: key) { return image }
        guard let file = await ImageStore.shared.file(for: path),
              let source = CGImageSourceCreateWithURL(file as CFURL, nil)
        else { return nil }
        var options: [CFString: Any] = [
            kCGImageSourceCreateThumbnailFromImageAlways: true,
            kCGImageSourceCreateThumbnailWithTransform: true,
            kCGImageSourceShouldCacheImmediately: true,
        ]
        if let maxPixels { options[kCGImageSourceThumbnailMaxPixelSize] = maxPixels }
        guard let cg = CGImageSourceCreateThumbnailAtIndex(source, 0, options as CFDictionary) else {
            return nil
        }
        let image = UIImage(cgImage: cg)
        cache.setObject(image, forKey: key, cost: cg.bytesPerRow * cg.height)
        return image
    }

    /// Les octets d'une vignette : dans le paquet sur l'appareil, sinon déjà reçue, sinon
    /// demandée à Nuage (quelques Ko du paquet, par `Range`), puis gardée.
    private static func thumbnailBytes(_ work: CollectionWork, pack: String) async -> Data? {
        if let data = LocalMuseum.thumbnail(of: work, pack: pack) { return data }
        let file = LocalMuseum.downloads.appendingPathComponent("collections/vignettes/\(work.id).jpg")
        if let data = try? Data(contentsOf: file) { return data }
        guard Nuage.isConfigured, let range = work.thumb, range.count == 2,
              let data = try? await NuageFiles.shared.bytes("collections/\(pack)", offset: range[0], length: range[1])
        else { return nil }
        try? FileManager.default.createDirectory(at: file.deletingLastPathComponent(), withIntermediateDirectories: true)
        try? data.write(to: file, options: .atomic)
        return data
    }

    /// L'image en grand (700 px) d'une œuvre des collections, embarquée dans l'app.
    func collectionLarge(_ work: CollectionWork, pack: String?) async -> UIImage? {
        let key = "grande|\(work.id)" as NSString
        if let image = cache.object(forKey: key) { return image }
        guard let data = LocalMuseum.largeImage(of: work, pack: pack),
              let image = UIImage(data: data)?.preparingForDisplay()
        else { return nil }
        cache.setObject(image, forKey: key, cost: data.count * 10)
        return image
    }

    /// La vignette d'une œuvre des collections du monde, lue dans le paquet de son musée.
    func collectionThumbnail(_ work: CollectionWork, pack: String) async -> UIImage? {
        let key = "vignette|\(work.id)" as NSString
        if let image = cache.object(forKey: key) { return image }
        guard let data = await Self.thumbnailBytes(work, pack: pack),
              let image = UIImage(data: data)?.preparingForDisplay()
        else { return nil }
        cache.setObject(image, forKey: key, cost: data.count * 8)
        return image
    }
}

/// L'image d'une œuvre, du musée embarqué (ou du site, pour une nouveauté).
struct RemoteImage: View {
    let path: String?
    var contentMode: ContentMode = .fill
    var tint: Color = Color.secondary.opacity(0.15)
    /// Le plus grand côté, en pixels ; `nil` pour l'image entière (zoom).
    var maxPixels: Int? = 700

    @State private var image: UIImage?
    @State private var failed = false

    var body: some View {
        Rectangle()
            .fill(tint)
            .overlay {
                if let image {
                    Image(uiImage: image)
                        .resizable()
                        .aspectRatio(contentMode: contentMode)
                        .transition(.opacity)
                } else if failed {
                    Image(systemName: "photo")
                        .font(.title)
                        .foregroundStyle(.secondary)
                }
            }
            .clipped()
            .task(id: path) { await load() }
    }

    private func load() async {
        guard let path, !path.isEmpty else { return }
        if let loaded = await ImagePipeline.shared.image(path, maxPixels: maxPixels) {
            withAnimation(.easeOut(duration: 0.25)) { image = loaded }
        } else {
            failed = true
        }
    }
}

/// Une carte : fond, coins arrondis, liseré.
struct CardStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .background(Color.card)
            .clipShape(.rect(cornerRadius: 20))
            .overlay(RoundedRectangle(cornerRadius: 20).strokeBorder(.quaternary))
            .contentShape(.rect(cornerRadius: 20))
            .hoverEffect(.lift)
    }
}

extension View {
    func card() -> some View { modifier(CardStyle()) }
}

/// Des étiquettes qui passent à la ligne quand la place manque (les thèmes d'une œuvre).
struct FlowLayout: Layout {
    var spacing: CGFloat = 8

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let rows = arrange(width: proposal.width ?? .infinity, subviews: subviews)
        return CGSize(
            width: rows.map(\.width).max() ?? 0,
            height: rows.map(\.height).reduce(0, +) + spacing * CGFloat(max(rows.count - 1, 0))
        )
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        var y = bounds.minY
        for row in arrange(width: bounds.width, subviews: subviews) {
            var x = bounds.minX
            for index in row.items {
                let size = subviews[index].sizeThatFits(.unspecified)
                subviews[index].place(at: CGPoint(x: x, y: y), proposal: ProposedViewSize(size))
                x += size.width + spacing
            }
            y += row.height + spacing
        }
    }

    private struct Row {
        var items: [Int] = []
        var width: CGFloat = 0
        var height: CGFloat = 0
    }

    private func arrange(width: CGFloat, subviews: Subviews) -> [Row] {
        var rows = [Row()]
        for (index, subview) in subviews.enumerated() {
            let size = subview.sizeThatFits(.unspecified)
            if !rows[rows.count - 1].items.isEmpty, rows[rows.count - 1].width + spacing + size.width > width {
                rows.append(Row())
            }
            var row = rows[rows.count - 1]
            row.width += (row.items.isEmpty ? 0 : spacing) + size.width
            row.height = max(row.height, size.height)
            row.items.append(index)
            rows[rows.count - 1] = row
        }
        return rows
    }
}
