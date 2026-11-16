import SwiftUI

/// Le plan d'un bâtiment d'ARTEON, dessiné comme sur le site : les salles à la couleur de
/// leurs murs, les portes, les tableaux le long des murs, les socles des sculptures, les
/// numéros du parcours et l'entrée. Toucher une salle la choisit.
struct FloorPlanView: View {
    let exhibition: Exhibition
    @Binding var selected: Int?

    /// Le cadre du plan, en mètres, avec une marge.
    private var bounds: CGRect {
        let frames = exhibition.rooms.compactMap(\.frame)
        guard let first = frames.first else { return CGRect(x: 0, y: 0, width: 1, height: 1) }
        var minX = first.minX, minZ = first.minZ, maxX = first.maxX, maxZ = first.maxZ
        for f in frames {
            minX = min(minX, f.minX)
            minZ = min(minZ, f.minZ)
            maxX = max(maxX, f.maxX)
            maxZ = max(maxZ, f.maxZ)
        }
        let pad = 3.0
        return CGRect(x: minX - pad, y: minZ - pad, width: maxX - minX + 2 * pad, height: maxZ - minZ + 2 * pad)
    }

    var body: some View {
        let b = bounds
        GeometryReader { geo in
            let scale = min(geo.size.width / b.width, geo.size.height / b.height)
            let offset = CGPoint(
                x: (geo.size.width - b.width * scale) / 2 - b.minX * scale,
                y: (geo.size.height - b.height * scale) / 2 - b.minY * scale
            )
            Canvas { context, _ in
                draw(in: &context, scale: scale, offset: offset)
            }
            .contentShape(.rect)
            .gesture(
                SpatialTapGesture().onEnded { value in
                    let x = (value.location.x - offset.x) / scale
                    let z = (value.location.y - offset.y) / scale
                    let hit = exhibition.rooms.indices.last { i in
                        guard let f = exhibition.rooms[i].frame else { return false }
                        return x >= f.minX && x <= f.maxX && z >= f.minZ && z <= f.maxZ
                    }
                    withAnimation(.snappy) { selected = hit == selected ? nil : hit }
                }
            )
        }
        .aspectRatio(b.width / b.height, contentMode: .fit)
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("Le plan du bâtiment : \(exhibition.rooms.filter { !$0.corridor }.count) salles")
        .accessibilityHint("Les salles sont aussi listées sous le plan")
    }

    private func draw(in context: inout GraphicsContext, scale: Double, offset: CGPoint) {
        func point(_ x: Double, _ z: Double) -> CGPoint {
            CGPoint(x: x * scale + offset.x, y: z * scale + offset.y)
        }
        let accent = exhibition.accentColor
        let ink = Color(red: 0.2, green: 0.19, blue: 0.18)

        // Les salles.
        for (i, room) in exhibition.rooms.enumerated() {
            guard let f = room.frame else { continue }
            let rect = CGRect(origin: point(f.minX, f.minZ), size: CGSize(
                width: (f.maxX - f.minX) * scale, height: (f.maxZ - f.minZ) * scale
            ))
            let fill = room.corridor
                ? Color(red: 0.36, green: 0.33, blue: 0.3)
                : Color(hex: room.walls ?? "") ?? Color(red: 0.93, green: 0.92, blue: 0.89)
            context.fill(Path(rect), with: .color(fill))
            context.stroke(Path(rect), with: .color(ink), lineWidth: max(0.2 * scale, 1.2))
            if i == selected {
                context.fill(Path(rect), with: .color(accent.opacity(0.22)))
                context.stroke(Path(rect.insetBy(dx: 1.5, dy: 1.5)), with: .color(accent), lineWidth: 3)
            }
        }
        // Les portes : une ouverture dans le trait du mur.
        for room in exhibition.rooms {
            for door in room.doors ?? [] where door.count == 4 {
                var path = Path()
                path.move(to: point(door[0], door[1]))
                path.addLine(to: point(door[2], door[3]))
                context.stroke(path, with: .color(Color(red: 0.97, green: 0.96, blue: 0.93)), lineWidth: max(0.45 * scale, 3))
            }
        }
        // Les œuvres : les tableaux le long des murs, les socles des sculptures.
        for work in exhibition.works {
            guard let spot = work.spot else { continue }
            if spot.count == 4 {
                var path = Path()
                path.move(to: point(spot[0], spot[1]))
                path.addLine(to: point(spot[2], spot[3]))
                context.stroke(path, with: .color(accent), style: StrokeStyle(lineWidth: max(0.28 * scale, 2), lineCap: .round))
            } else if spot.count == 3 {
                let r = spot[2] * scale
                let c = point(spot[0], spot[1])
                let circle = Path(ellipseIn: CGRect(x: c.x - r, y: c.y - r, width: 2 * r, height: 2 * r))
                context.fill(circle, with: .color(accent.opacity(0.85)))
                context.stroke(circle, with: .color(ink), lineWidth: 1)
            }
        }
        // Les numéros du parcours.
        for room in exhibition.rooms {
            guard let n = room.number, let x = room.x, let z = room.z, let w = room.width, let d = room.depth else { continue }
            let size = min(max(min(w, d) * 0.4, 1.2), 3.2) * scale
            context.draw(
                Text("\(n)").font(.system(size: max(size, 9), weight: .bold, design: .serif)).foregroundStyle(ink.opacity(0.75)),
                at: point(x, z)
            )
        }
        // L'entrée : une flèche vers l'intérieur.
        if let e = exhibition.entrance, e.count == 4 {
            let at = point(e[0] - e[2] * 1.8, e[1] - e[3] * 1.8)
            var arrow = context
            arrow.translateBy(x: at.x, y: at.y)
            arrow.rotate(by: .radians(atan2(e[2], -e[3])))
            arrow.draw(
                Image(systemName: "arrowshape.up.fill"),
                in: CGRect(x: -9, y: -9, width: 18, height: 18)
            )
            context.draw(
                Text("Entrée").font(.caption2.bold()).foregroundStyle(ink),
                at: CGPoint(x: at.x, y: at.y + (e[3] < 0 ? 18 : -18))
            )
        }
    }
}

extension Color {
    /// « #e8e2d6 ».
    init?(hex: String) {
        let text = hex.trimmingCharacters(in: CharacterSet(charactersIn: "#"))
        guard text.count == 6, let value = UInt32(text, radix: 16) else { return nil }
        self.init(
            red: Double((value >> 16) & 0xff) / 255,
            green: Double((value >> 8) & 0xff) / 255,
            blue: Double(value & 0xff) / 255
        )
    }
}

/// Le plan, la salle choisie et ses œuvres : le bâtiment d'une exposition.
struct BuildingPlan: View {
    let exhibition: Exhibition

    @Environment(CatalogueStore.self) private var store
    @State private var selected: Int?

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            FloorPlanView(exhibition: exhibition, selected: $selected)
                .frame(maxHeight: 520)
                .padding(12)
                .background(Color(red: 0.97, green: 0.96, blue: 0.93), in: .rect(cornerRadius: 18))
                .environment(\.colorScheme, .light)

            HStack(spacing: 14) {
                legend(color: exhibition.accentColor, text: "Tableaux", systemImage: "line.diagonal")
                if exhibition.works.contains(where: { $0.kind == .sculpture }) {
                    legend(color: exhibition.accentColor, text: "Sculptures", systemImage: "circle.fill")
                }
                legend(color: .secondary, text: "Toucher une salle", systemImage: "hand.tap")
            }
            .font(.caption)
            .foregroundStyle(.secondary)

            if let selected, exhibition.rooms.indices.contains(selected) {
                let room = exhibition.rooms[selected]
                let works = exhibition.works.filter { $0.room == selected }
                VStack(alignment: .leading, spacing: 8) {
                    HStack(alignment: .firstTextBaseline) {
                        if let n = room.number {
                            Text("Salle \(n)")
                                .font(.caption.bold())
                                .foregroundStyle(exhibition.accentColor)
                        }
                        Text(room.name.isEmpty ? (room.corridor ? "Un couloir" : "Une salle") : room.name)
                            .font(.headline)
                    }
                    if !room.theme.isEmpty {
                        Text(room.theme)
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }
                    if works.isEmpty {
                        Text(room.corridor ? "Un passage entre les salles." : "Pas d'œuvre dans cette salle.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    } else {
                        ScrollView(.horizontal, showsIndicators: false) {
                            LazyHStack(spacing: 12) {
                                ForEach(works) { work in
                                    NavigationLink(value: Route.work(exhibition.key(work))) {
                                        WorkCard(work: work).frame(width: 160)
                                    }
                                    .buttonStyle(.plain)
                                }
                            }
                        }
                        .scrollClipDisabled()
                    }
                    NavigationLink(value: Route.gallery(exhibition.slug, work: works.first?.slug)) {
                        Label("Entrer dans cette salle en 3D", systemImage: "view.3d")
                    }
                    .buttonStyle(.glass)
                }
                .padding()
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.card, in: .rect(cornerRadius: 16))
                .transition(.opacity.combined(with: .move(edge: .top)))
            }

            // Les salles en liste (et pour VoiceOver).
            DisclosureGroup("Les \(exhibition.rooms.filter { !$0.corridor }.count) salles") {
                VStack(alignment: .leading, spacing: 0) {
                    ForEach(Array(exhibition.rooms.enumerated()).filter { !$0.element.corridor }, id: \.offset) { item in
                        Button {
                            withAnimation(.snappy) { selected = item.offset }
                        } label: {
                            HStack {
                                Text(item.element.number.map { "\($0)." } ?? "•")
                                    .monospacedDigit()
                                    .foregroundStyle(exhibition.accentColor)
                                    .frame(width: 30, alignment: .leading)
                                Text(item.element.name)
                                Spacer()
                                let count = exhibition.works.filter { $0.room == item.offset }.count
                                Text(count == 1 ? "1 œuvre" : "\(count) œuvres")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                            .padding(.vertical, 8)
                            .contentShape(.rect)
                        }
                        .buttonStyle(.plain)
                        Divider()
                    }
                }
            }
        }
    }

    private func legend(color: Color, text: String, systemImage: String) -> some View {
        Label {
            Text(text)
        } icon: {
            Image(systemName: systemImage).foregroundStyle(color)
        }
    }
}
