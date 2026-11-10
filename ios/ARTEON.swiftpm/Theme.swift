import SwiftUI

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

/// Où mène un toucher : une exposition, une œuvre, ou la galerie 3D (devant une œuvre).
enum Route: Hashable {
    case exhibition(String)
    case work(WorkKey)
    case gallery(String, work: String?)
}

/// L'image d'une œuvre, téléchargée une fois puis gardée sur l'appareil.
struct RemoteImage: View {
    let path: String?
    var contentMode: ContentMode = .fill
    var tint: Color = Color.secondary.opacity(0.15)

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
        if let data = await ImageStore.shared.data(for: Site.url(path)),
           let loaded = UIImage(data: data) {
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
