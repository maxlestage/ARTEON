// D'où vient le catalogue : du site d'ARTEON, gardé sur l'appareil pour le relire hors
// ligne ; et les favoris, gardés dans les réglages de l'appareil.

import Foundation
import Observation
#if canImport(FoundationNetworking)
import FoundationNetworking
#endif

/// L'adresse du site : GitHub Pages, ou une autre (Heroku, un serveur local) donnée au
/// lancement par `-site https://…/` ou par la clé `ARTEONSite` de l'Info.plist.
enum Site {
    static let base: URL = {
        let args = ProcessInfo.processInfo.arguments
        if let i = args.firstIndex(of: "-site"), i + 1 < args.count,
           let url = URL(string: args[i + 1]) {
            return url
        }
        if let text = Bundle.main.object(forInfoDictionaryKey: "ARTEONSite") as? String,
           let url = URL(string: text) {
            return url
        }
        return URL(string: "https://maxlestage.github.io/ARTEON/")!
    }()

    /// L'adresse complète d'un chemin du site (`tableaux/van-gogh/iris.jpg`).
    static func url(_ path: String) -> URL {
        URL(string: path, relativeTo: base)?.absoluteURL ?? base
    }
}

@MainActor
@Observable
final class CatalogueStore {
    enum Phase {
        case loading
        case ready(Catalogue)
        case failed(String)
    }

    private(set) var phase: Phase = .loading
    /// Le catalogue vient de la copie gardée sur l'appareil (le site n'a pas répondu).
    private(set) var offline = false
    private(set) var favorites: [WorkKey] = []

    private let defaults: UserDefaults
    private static let favoritesKey = "favoris"

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        favorites = (defaults.stringArray(forKey: Self.favoritesKey) ?? []).compactMap(WorkKey.init)
    }

    var catalogue: Catalogue? {
        if case .ready(let c) = phase { c } else { nil }
    }

    /// Charge le catalogue du site, sinon la copie de l'appareil.
    func load() async {
        if catalogue == nil { phase = .loading }
        do {
            let data = try await Self.download()
            let catalogue = try await Self.decode(data)
            phase = .ready(catalogue)
            offline = false
            Self.save(data)
        } catch {
            if catalogue != nil {
                offline = true
            } else if let data = Self.saved(), let catalogue = try? await Self.decode(data) {
                phase = .ready(catalogue)
                offline = true
            } else {
                phase = .failed(error.localizedDescription)
            }
        }
    }

    // MARK: Favoris

    func isFavorite(_ key: WorkKey) -> Bool { favorites.contains(key) }

    func toggleFavorite(_ key: WorkKey) {
        if let i = favorites.firstIndex(of: key) {
            favorites.remove(at: i)
        } else {
            favorites.append(key)
        }
        defaults.set(favorites.map(\.text), forKey: Self.favoritesKey)
    }

    // MARK: Réseau et copie locale

    private nonisolated static func download() async throws -> Data {
        var request = URLRequest(url: Site.url("catalogue.json"))
        request.cachePolicy = .reloadIgnoringLocalCacheData
        request.timeoutInterval = 20
        let (data, response) = try await URLSession.shared.data(for: request)
        if let http = response as? HTTPURLResponse, http.statusCode != 200 {
            throw URLError(.badServerResponse)
        }
        return data
    }

    private nonisolated static func decode(_ data: Data) async throws -> Catalogue {
        try Catalogue.decode(data)
    }

    private nonisolated static var savedURL: URL? {
        guard let dir = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
        else { return nil }
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent("catalogue.json")
    }

    private nonisolated static func save(_ data: Data) {
        guard let url = savedURL else { return }
        try? data.write(to: url, options: .atomic)
    }

    private nonisolated static func saved() -> Data? {
        savedURL.flatMap { try? Data(contentsOf: $0) }
    }
}

/// Les images des œuvres, téléchargées une fois puis gardées sur l'appareil (elles se
/// revoient hors ligne).
actor ImageStore {
    static let shared = ImageStore()

    private var memory: [URL: Data] = [:]
    private var order: [URL] = []
    private var running: [URL: Task<Data?, Never>] = [:]
    private let folder: URL? = {
        guard let dir = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first
        else { return nil }
        let folder = dir.appendingPathComponent("images", isDirectory: true)
        try? FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        return folder
    }()

    /// Les octets de l'image : de la mémoire, du disque, ou du site.
    func data(for url: URL) async -> Data? {
        if let data = memory[url] { return data }
        if let task = running[url] { return await task.value }
        let file = folder?.appendingPathComponent(Self.name(of: url))
        if let file, let data = try? Data(contentsOf: file) {
            remember(url, data)
            return data
        }
        let task = Task<Data?, Never> {
            guard let (data, response) = try? await URLSession.shared.data(from: url),
                  (response as? HTTPURLResponse)?.statusCode ?? 200 == 200
            else { return nil }
            if let file { try? data.write(to: file, options: .atomic) }
            return data
        }
        running[url] = task
        let data = await task.value
        running[url] = nil
        if let data { remember(url, data) }
        return data
    }

    private func remember(_ url: URL, _ data: Data) {
        memory[url] = data
        order.append(url)
        // Une soixantaine d'images en mémoire ; les autres restent sur le disque.
        while order.count > 60 {
            memory[order.removeFirst()] = nil
        }
    }

    /// Un nom de fichier stable pour une adresse.
    private static func name(of url: URL) -> String {
        var hash: UInt64 = 0xcbf2_9ce4_8422_2325
        for byte in url.absoluteString.utf8 {
            hash = (hash ^ UInt64(byte)) &* 0x100_0000_01b3
        }
        return String(hash, radix: 16) + "-" + url.lastPathComponent
    }
}
