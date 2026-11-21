// D'où vient le catalogue : tout passe par l'API (le site ARTEON et le serveur Paquets,
// Api.swift), à chaque ouverture : rien du musée n'est gardé sur l'appareil. Seuls les
// choix du visiteur y restent : favoris, œuvres vues, réglages, score du jeu.

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

    /// Tout passe par l'API : l'application est toujours en ligne.
    static let neverOnline = false
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
        /// Pas encore ouvert.
        case bundled
        /// L'API, à l'instant.
        case online
        /// (plus utilisé : rien n'est gardé sur l'appareil)
        case saved
    }

    private(set) var phase: Phase = .loading
    private(set) var source: Source = .bundled
    private(set) var index: MuseumIndex?
    /// Les collections du monde (des dizaines de milliers d'œuvres), lues après le musée.
    private(set) var collections: CollectionIndex?
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
        updatesEnabled = true
        quizBest = defaults.integer(forKey: Keys.quizBest)
    }

    var catalogue: Catalogue? {
        if case .ready(let c) = phase { c } else { nil }
    }

    /// Rien du musée n'est gardé sur l'appareil.
    var isBundled: Bool { false }

    /// Ouvre le musée : le catalogue, demandé au site à chaque ouverture.
    func load() async {
        if catalogue == nil {
            phase = .loading
            await update()
        }
        if collections == nil {
            // Les collections du monde (3,6 Mo) : un peu après, pour laisser le réseau aux
            // images de l'accueil. Une page qui en a besoin plus tôt les demande elle-même.
            Task {
                try? await Task.sleep(nanoseconds: 1_500_000_000)
                await loadCollections()
            }
        }
        if catalogue == nil {
            let message = L("ARTEON a besoin d'Internet : tout passe par l'API, rien n'est gardé sur l'appareil.",
                            "ARTEON needs the Internet: everything comes from the API, nothing is kept on the device.",
                            "ARTEON necesita Internet: todo pasa por la API, nada se guarda en el dispositivo.")
            phase = .failed(lastUpdateError.map { "\(message) (\($0))" } ?? message)
        }
    }

    /// Demande le catalogue au site, dans la langue de l'application (`catalogue.json`,
    /// `en/catalogue.json`, `es/catalogue.json`) ; garde l'actuel si le site ne répond pas.
    func update() async {
        guard !updating else { return }
        updating = true
        defer { updating = false }
        do {
            let data = try await Api.site(AppLanguage.current.catalogueFile)
            let c = try await Self.decode(data)
            await show(c, from: .online)
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

    /// Les collections du monde, demandées au site (en arrière-plan : la liste est longue).
    func loadCollections() async {
        guard collections == nil, !loadingCollections else { return }
        loadingCollections = true
        defer { loadingCollections = false }
        // Les deux fichiers ensemble, pas l'un après l'autre.
        async let indexFile = try? Api.site("monde/donnees/index.json")
        async let listFile = try? Api.site("monde/donnees/liste.json")
        guard let index = await indexFile, let list = await listFile
        else { collectionsError = true; return }
        collectionsError = false
        collections = await Task.detached(priority: .utility) {
            (try? WorldCollections.fromSite(index: index, list: list)).map(CollectionIndex.init)
        }.value
    }

    /// L'annuaire de tous les artistes (des dizaines de milliers), demandé au site.
    private(set) var directory: [String: DirectoryArtist] = [:]
    private var loadingDirectory = false

    func loadDirectory() async {
        guard directory.isEmpty, !loadingDirectory else { return }
        loadingDirectory = true
        defer { loadingDirectory = false }
        guard let data = try? await Api.site("monde/annuaire.json") else { return }
        directory = await Task.detached(priority: .utility) {
            Dictionary(DirectoryArtist.parse(data).map { ($0.key, $0) }, uniquingKeysWith: { a, b in a.works >= b.works ? a : b })
        }.value
    }

    /// Les collections sont-elles en train d'arriver, ou le site n'a-t-il pas répondu ?
    private(set) var loadingCollections = false
    private(set) var collectionsError = false

    /// Le détail d'une œuvre des collections (technique, texte, image en grand…).
    func details(_ work: CollectionWork) async -> CollectionWork {
        await CollectionDetails.shared.complete(work, perFile: collections?.collections.perFile ?? 1000)
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

    private nonisolated static func decode(_ data: Data) async throws -> Catalogue {
        try Catalogue.decode(data)
    }
}

/// Les octets des images : demandés au site (ou au musée), gardés en mémoire le temps de
/// la visite seulement. Rien n'est écrit sur l'appareil.
actor ImageStore {
    static let shared = ImageStore()

    private var running: [URL: Task<Data?, Never>] = [:]
    /// Les octets reçus, peu de temps (l'image décodée est gardée par ImagePipeline, et les
    /// octets par la mémoire de la visite) : de quoi redécoder à une autre taille.
    private let memory: NSCache<NSURL, NSData> = {
        let cache = NSCache<NSURL, NSData>()
        cache.totalCostLimit = 32 * 1024 * 1024
        return cache
    }()
    /// Huit téléchargements à la fois : dans une galerie de 2 000 vignettes qu'on fait défiler,
    /// des centaines de demandes partaient ensemble, celles de l'écran derrière toutes les
    /// autres (et Wikimedia en refusait une partie).
    private let turns = Turns(limit: 8)

    /// Les octets d'une image du site (`tableaux/van-gogh/iris.jpg`) ou d'une adresse complète.
    /// `nil` aussi quand la tâche est annulée avant son tour (l'image a quitté l'écran).
    func data(for path: String) async -> Data? {
        let absolute = path.hasPrefix("https://") || path.hasPrefix("http://")
        let url = absolute ? (URL(string: path) ?? Site.url(path)) : Site.url(path)
        if let data = memory.object(forKey: url as NSURL) { return data as Data }
        if let task = running[url] { return await task.value }
        guard await turns.enter() else { return nil }
        // Pendant l'attente, une autre vue a pu demander la même image.
        if let data = memory.object(forKey: url as NSURL) {
            await turns.leave()
            return data as Data
        }
        if let task = running[url] {
            await turns.leave()
            return await task.value
        }
        // Le téléchargement commencé va au bout, même si la vue disparaît : il sert au retour.
        let task = Task<Data?, Never> {
            if let data = try? await Api.data(url) { return data }
            // Un refus passager (Wikimedia limite les rafales) : un second essai, un peu après.
            try? await Task.sleep(nanoseconds: 1_200_000_000)
            return try? await Api.data(url)
        }
        running[url] = task
        let data = await task.value
        running[url] = nil
        await turns.leave()
        if let data { memory.setObject(data as NSData, forKey: url as NSURL, cost: data.count) }
        return data
    }
}

/// Des tours : au plus `limit` travaux à la fois. Quand ils sont tous pris, le dernier arrivé
/// passe le premier (dans une galerie qu'on fait défiler, c'est ce qui est à l'écran), et un
/// demandeur annulé en attendant (sa vue a quitté l'écran) sort de la file.
actor Turns {
    private let limit: Int
    private var active = 0
    private var next: UInt64 = 0
    private var waiting: [(id: UInt64, continuation: CheckedContinuation<Bool, Never>)] = []

    init(limit: Int) {
        self.limit = max(1, limit)
    }

    /// Attend son tour. `false` : la tâche a été annulée en attendant, et n'a pas de tour à
    /// rendre ; `true` : à elle de jouer, puis d'appeler `leave()`.
    func enter() async -> Bool {
        if active < limit {
            active += 1
            return true
        }
        let id = next
        next += 1
        return await withTaskCancellationHandler {
            await withCheckedContinuation { continuation in
                if Task.isCancelled {
                    continuation.resume(returning: false)
                } else {
                    waiting.append((id, continuation))
                }
            }
        } onCancel: {
            Task { await self.abandon(id) }
        }
    }

    /// Rend son tour : il passe au dernier arrivé, s'il y en a un.
    func leave() {
        if let last = waiting.popLast() {
            last.continuation.resume(returning: true)
        } else {
            active -= 1
        }
    }

    private func abandon(_ id: UInt64) {
        if let i = waiting.firstIndex(where: { $0.id == id }) {
            waiting.remove(at: i).continuation.resume(returning: false)
        }
    }

    /// Combien attendent leur tour (pour les tests).
    var waitingCount: Int { waiting.count }
}
