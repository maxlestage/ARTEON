import SwiftUI

/// Tout télécharger depuis Nuage pour visiter sans Internet : le musée (catalogue, images,
/// galeries 3D) ou les collections du monde (fiches et vignettes). Ce qui est déjà sur
/// l'appareil n'est pas retéléchargé.
@MainActor
@Observable
final class OfflineDownloader {
    enum Part: String, CaseIterable, Identifiable {
        case museum, collections

        var id: String { rawValue }

        var title: String {
            switch self {
            case .museum: "Le musée"
            case .collections: "Les collections du monde"
            }
        }

        var detail: String {
            switch self {
            case .museum: "Catalogue, images des expositions, galeries 3D"
            case .collections: "Fiches et vignettes de toutes les œuvres"
            }
        }

        /// Les fichiers de Nuage qui en font partie.
        func contains(_ name: String) -> Bool {
            switch self {
            case .collections: name.hasPrefix("collections/")
            case .museum: !name.hasPrefix("collections/") && !name.hasPrefix("modeles/")
            }
        }
    }

    struct Progress {
        var done = 0
        var total = 0
        var bytes = 0
        var totalBytes = 0
    }

    private(set) var running: Part?
    private(set) var progress = Progress()
    private(set) var failed: String?
    /// La taille de chaque partie sur Nuage, et ce qui est déjà sur l'appareil.
    private(set) var sizes: [Part: (total: Int, stored: Int)] = [:]
    @ObservationIgnored private var task: Task<Void, Never>?

    /// Mesure chaque partie : sur Nuage, et déjà sur l'appareil.
    func measure() async {
        guard Nuage.isConfigured, let files = try? await NuageFiles.shared.list() else { return }
        var result: [Part: (total: Int, stored: Int)] = [:]
        for part in Part.allCases {
            let mine = files.filter { part.contains($0.name) }
            let stored = mine.filter { isStored($0) }.reduce(0) { $0 + $1.size }
            result[part] = (total: mine.reduce(0) { $0 + $1.size }, stored: stored)
        }
        sizes = result
    }

    private func isStored(_ entry: NuageFiles.Entry) -> Bool {
        let file = LocalMuseum.downloads.appendingPathComponent(entry.name)
        let size = (try? file.resourceValues(forKeys: [.fileSizeKey]))?.fileSize ?? -1
        return size == entry.size
    }

    func start(_ part: Part) {
        guard running == nil else { return }
        running = part
        failed = nil
        progress = Progress()
        task = Task {
            defer { running = nil }
            do {
                let files = try await NuageFiles.shared.list().filter { part.contains($0.name) && !isStored($0) }
                progress.total = files.count
                progress.totalBytes = files.reduce(0) { $0 + $1.size }
                try await withThrowingTaskGroup(of: Int.self) { group in
                    var next = 0
                    func add() {
                        guard next < files.count else { return }
                        let entry = files[next]
                        next += 1
                        group.addTask {
                            try await NuageFiles.shared.download(
                                entry.name, to: LocalMuseum.downloads.appendingPathComponent(entry.name)
                            )
                            return entry.size
                        }
                    }
                    for _ in 0..<6 { add() }
                    while let size = try await group.next() {
                        if Task.isCancelled { group.cancelAll(); break }
                        progress.done += 1
                        progress.bytes += size
                        add()
                    }
                }
            } catch {
                failed = "Le téléchargement s'est arrêté (\(error.localizedDescription)) : il reprendra où il en était."
            }
            await measure()
        }
    }

    func cancel() {
        task?.cancel()
        running = nil
    }

    /// Efface ce qui a été téléchargé (le musée reste visitable avec Internet).
    func clear() async {
        let folder = LocalMuseum.downloads
        await Task.detached(priority: .utility) {
            let fm = FileManager.default
            for item in (try? fm.contentsOfDirectory(at: folder, includingPropertiesForKeys: nil)) ?? [] {
                try? fm.removeItem(at: item)
            }
        }.value
        await measure()
    }
}

/// La section « Hors ligne » des réglages.
struct OfflineSection: View {
    @State private var downloader = OfflineDownloader()
    @State private var confirmClear = false

    var body: some View {
        Section {
            if LocalMuseum.isAvailable {
                Label {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Tout est dans l'application")
                        Text("Catalogue, images, galeries 3D et collections du monde se visitent sans Internet, sans rien télécharger.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                } icon: {
                    Image(systemName: "checkmark.seal.fill").foregroundStyle(.green)
                }
            } else if !Nuage.isConfigured {
                Label("Cette copie de l'app n'a pas de jeton Nuage : les images viennent du site.", systemImage: "exclamationmark.icloud")
                    .font(.callout)
            } else {
                ForEach(OfflineDownloader.Part.allCases) { part in
                    row(part)
                }
                if let failed = downloader.failed {
                    Text(failed).font(.caption).foregroundStyle(.orange)
                }
                Button("Effacer ce qui est téléchargé", role: .destructive) { confirmClear = true }
                    .disabled(downloader.running != nil)
            }
        } header: {
            Text("Hors ligne")
        } footer: {
            Text("L'application est légère : chaque image se télécharge depuis Nuage la première fois qu'on la regarde, puis reste sur l'appareil. Tout télécharger permet de visiter sans Internet.")
        }
        .task { if !LocalMuseum.isAvailable { await downloader.measure() } }
        .confirmationDialog("Effacer le musée téléchargé ?", isPresented: $confirmClear, titleVisibility: .visible) {
            Button("Effacer", role: .destructive) { Task { await downloader.clear() } }
        } message: {
            Text("Il se retéléchargera au fil de la visite.")
        }
    }

    @ViewBuilder
    private func row(_ part: OfflineDownloader.Part) -> some View {
        let size = downloader.sizes[part]
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text(part.title).font(.headline)
                    Text(part.detail).font(.caption).foregroundStyle(.secondary)
                }
                Spacer()
                if let size {
                    if size.total > 0, size.stored >= size.total {
                        Label("Sur l'appareil", systemImage: "checkmark.circle.fill")
                            .labelStyle(.iconOnly)
                            .foregroundStyle(.green)
                    }
                    Text("\(Int64(size.stored).formatted(.byteCount(style: .file))) / \(Int64(size.total).formatted(.byteCount(style: .file)))")
                        .font(.caption.monospacedDigit())
                        .foregroundStyle(.secondary)
                }
            }
            if downloader.running == part {
                ProgressView(
                    value: Double(downloader.progress.bytes),
                    total: Double(max(downloader.progress.totalBytes, 1))
                )
                HStack {
                    Text("\(downloader.progress.done) / \(downloader.progress.total) fichiers")
                        .font(.caption.monospacedDigit())
                    Spacer()
                    Button("Arrêter", role: .cancel) { downloader.cancel() }
                        .font(.caption)
                }
            } else if size.map({ $0.stored < $0.total }) ?? true {
                Button("Tout télécharger", systemImage: "arrow.down.circle") { downloader.start(part) }
                    .disabled(downloader.running != nil)
            }
        }
        .padding(.vertical, 4)
    }
}
