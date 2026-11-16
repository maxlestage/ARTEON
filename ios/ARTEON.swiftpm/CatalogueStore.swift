// D'où vient le catalogue : du musée embarqué dans l'application (tout marche sans
// Internet), mis à jour depuis le site quand il répond ; et ce que l'on garde sur
// l'appareil : favoris, œuvres vues, réglages, score du jeu.

import Foundation
import Observation
#if canImport(FoundationNetworking)
import FoundationNetworking
#endif

/// L'adresse du site, pour les mises à jour : GitHub Pages, ou une autre donnée au
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

    /// `-horsligne` au lancement : jamais de réseau (captures d'écran, essais).
    static let neverOnline = ProcessInfo.processInfo.arguments.contains("-horsligne")
}

@MainActor
@Observable
final class CatalogueStore {
    enum Phase {
        case loading
        case ready(Catalogue)
        case failed(String)
    }

    /// D'où vient le catalogue affiché.
    enum Source: Equatable {
        /// Le musée embarqué dans l'application.
        case bundled
        /// Le site, à l'instant.
        case online
        /// La dernière copie téléchargée, gardée sur l'appareil.
        case saved
    }

    private(set) var phase: Phase = .loading
    private(set) var source: Source = .bundled
    private(set) var index: MuseumIndex?
    private(set) var favorites: [WorkKey] = []
    /// Les œuvres regardées, la plus récente d'abord.
    private(set) var history: [WorkKey] = []
    private(set) var updating = false
    private(set) var lastUpdateError: String?

    /// Chercher les nouveautés sur le site au lancement.
    var updatesEnabled: Bool {
        didSet { defaults.set(updatesEnabled, forKey: Keys.updates) }
    }
    /// Le meilleur score du jeu.
    var quizBest: Int {
        didSet { defaults.set(quizBest, forKey: Keys.quizBest) }
    }

    private let defaults: UserDefaults

    private enum Keys {
        static let favorites = "favoris"
        static let history = "historique"
        static let updates = "majEnLigne"
        static let quizBest = "meilleurScore"
    }

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        favorites = (defaults.stringArray(forKey: Keys.favorites) ?? []).compactMap(WorkKey.init)
        history = (defaults.stringArray(forKey: Keys.history) ?? []).compactMap(WorkKey.init)
        updatesEnabled = defaults.object(forKey: Keys.updates) as? Bool ?? true
        quizBest = defaults.integer(forKey: Keys.quizBest)
    }

    var catalogue: Catalogue? {
        if case .ready(let c) = phase { c } else { nil }
    }

    /// Le musée embarqué est-il dans l'application ?
    var isBundled: Bool { LocalMuseum.isAvailable }

    /// Ouvre le musée : le catalogue embarqué (ou la dernière copie téléchargée), tout de
    /// suite ; puis, si c'est permis, les nouveautés du site, sans attendre.
    func load() async {
        if catalogue == nil {
            if let data = Self.bundled(), let c = try? await Self.decode(data) {
                await show(c, from: .bundled)
            }
            if let data = Self.saved(), let c = try? await Self.decode(data),
               catalogue == nil || c.workCount > (catalogue?.workCount ?? 0) {
                await show(c, from: .saved)
            }
        }
        if updatesEnabled, !Site.neverOnline {
            await update()
        }
        if catalogue == nil {
            phase = .failed(lastUpdateError ?? "Le catalogue est introuvable.")
        }
    }

    /// Télécharge le catalogue du site ; garde l'actuel si le site ne répond pas.
    func update() async {
        guard !updating else { return }
        updating = true
        defer { updating = false }
        do {
            let data = try await Self.download()
            let c = try await Self.decode(data)
            // Une copie du site qui aurait moins d'œuvres que le musée embarqué (un site
            // en retard sur l'application) ne le remplace pas.
            if let current = catalogue, source == .bundled, c.workCount < current.workCount {
                lastUpdateError = nil
                return
            }
            await show(c, from: .online)
            Self.save(data)
            lastUpdateError = nil
        } catch {
            lastUpdateError = error.localizedDescription
        }
    }

    private func show(_ catalogue: Catalogue, from source: Source) async {
        let index = await Task.detached(priority: .userInitiated) { MuseumIndex(catalogue) }.value
        self.index = index
        self.source = source
        phase = .ready(catalogue)
    }

    // MARK: Favoris

    func isFavorite(_ key: WorkKey) -> Bool { favorites.contains(key) }

    func toggleFavorite(_ key: WorkKey) {
        if let i = favorites.firstIndex(of: key) {
            favorites.remove(at: i)
        } else {
            favorites.append(key)
        }
        defaults.set(favorites.map(\.text), forKey: Keys.favorites)
    }

    func clearFavorites() {
        favorites = []
        defaults.set([String](), forKey: Keys.favorites)
    }

    // MARK: Historique

    /// Note qu'une œuvre a été regardée (les 40 dernières sont gardées).
    func visited(_ key: WorkKey) {
        history.removeAll { $0 == key }
        history.insert(key, at: 0)
        if history.count > 40 { history.removeLast(history.count - 40) }
        defaults.set(history.map(\.text), forKey: Keys.history)
    }

    func clearHistory() {
        history = []
        defaults.set([String](), forKey: Keys.history)
    }

    // MARK: Fichiers

    private nonisolated static func bundled() -> Data? {
        LocalMuseum.file("catalogue.json").flatMap { try? Data(contentsOf: $0) }
    }

    private nonisolated static func download() async throws -> Data {
        var request = URLRequest(url: Site.url("catalogue.json"))
        request.cachePolicy = .reloadIgnoringLocalCacheData
        request.timeoutInterval = 15
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

/// Les octets des images : du musée embarqué, sinon du site (téléchargées une fois puis
/// gardées sur l'appareil, pour les œuvres arrivées après l'application).
actor ImageStore {
    static let shared = ImageStore()

    private var running: [URL: Task<Data?, Never>] = [:]
    private let folder: URL? = {
        guard let dir = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first
        else { return nil }
        let folder = dir.appendingPathComponent("images", isDirectory: true)
        try? FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        return folder
    }()

    /// Le fichier de l'image sur l'appareil : dans l'application, ou téléchargé.
    func file(for path: String) async -> URL? {
        if let local = LocalMuseum.file(path) { return local }
        let url = Site.url(path)
        guard let file = folder?.appendingPathComponent(Self.name(of: url)) else { return nil }
        if FileManager.default.fileExists(atPath: file.path) { return file }
        guard !Site.neverOnline else { return nil }
        if let task = running[url] {
            return await task.value == nil ? nil : file
        }
        let task = Task<Data?, Never> {
            guard let (data, response) = try? await URLSession.shared.data(from: url),
                  (response as? HTTPURLResponse)?.statusCode ?? 200 == 200
            else { return nil }
            try? data.write(to: file, options: .atomic)
            return data
        }
        running[url] = task
        let data = await task.value
        running[url] = nil
        return data == nil ? nil : file
    }

    /// La place prise par les images téléchargées.
    func downloadedSize() -> Int64 {
        guard let folder,
              let files = try? FileManager.default.contentsOfDirectory(
                  at: folder, includingPropertiesForKeys: [.fileSizeKey]
              )
        else { return 0 }
        return files.reduce(0) {
            $0 + Int64((try? $1.resourceValues(forKeys: [.fileSizeKey]))?.fileSize ?? 0)
        }
    }

    func clearDownloads() {
        guard let folder,
              let files = try? FileManager.default.contentsOfDirectory(at: folder, includingPropertiesForKeys: nil)
        else { return }
        for file in files { try? FileManager.default.removeItem(at: file) }
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
