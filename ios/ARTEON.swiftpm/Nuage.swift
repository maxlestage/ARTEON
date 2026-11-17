// Le musée sur Nuage : les fichiers d'ARTEON (catalogue, images, galeries 3D, collections
// du monde) sont gardés dans les fichiers de l'application `arteon` sur Nuage DB. L'app
// n'embarque rien de lourd : elle demande un fichier la première fois qu'il sert, le garde
// sur l'appareil (Application Support/Musee), et peut tout télécharger d'un coup pour le
// hors-ligne. Rien que Foundation : ce fichier se teste aussi sous Linux.
//
// Les octets ne passent pas par le serveur Nuage : il signe un lien vers son bucket (une
// heure), l'app télécharge depuis le bucket (avec `Range` pour une vignette dans un paquet).

import Foundation
#if canImport(FoundationNetworking)
import FoundationNetworking
#endif

enum Nuage {
    /// Le serveur Nuage d'ARTEON ; `-nuage https://…` au lancement ou la clé `ARTEONNuage`
    /// de l'Info.plist en donnent un autre.
    static let base: URL = {
        let args = ProcessInfo.processInfo.arguments
        if let i = args.firstIndex(of: "-nuage"), i + 1 < args.count, let url = URL(string: args[i + 1]) {
            return url
        }
        if let text = Bundle.main.object(forInfoDictionaryKey: "ARTEONNuage") as? String,
           let url = URL(string: text) {
            return url
        }
        return URL(string: "https://nuage-f48a8945f31c.herokuapp.com")!
    }()

    /// Un jeton de l'application `arteon` en lecture seule (il lit, il n'écrit jamais) :
    /// la clé `ARTEONNuageToken` de l'Info.plist, que la CI remplit avec le secret GitHub
    /// `NUAGE_READ_TOKEN`. Il n'est jamais dans le code. Sans lui, l'app passe par le site.
    static var token: String? {
        // Un jeton collé dans les réglages de l'app (Swift Playgrounds n'a pas de CI).
        if let saved = UserDefaults.standard.string(forKey: "jetonNuage")?
            .trimmingCharacters(in: .whitespacesAndNewlines), !saved.isEmpty {
            return saved
        }
        let text = (Bundle.main.object(forInfoDictionaryKey: "ARTEONNuageToken") as? String ?? "")
            .trimmingCharacters(in: .whitespaces)
        return text.isEmpty || text.hasPrefix("$(") ? nil : text
    }

    /// Nuage ne sert que si le musée n'est pas embarqué : l'app embarquée est 100 % hors ligne.
    static var isConfigured: Bool { token != nil && !Site.neverOnline && !LocalMuseum.isAvailable }

    /// Un nom de fichier Nuage valide : lettres, chiffres, `.`, `_`, `-`, parties séparées
    /// par `/`. Les chemins du site (`/van-gogh/galerie/`, `pkg/arteon.js`) y deviennent
    /// `van-gogh/galerie/index.html`, `pkg/arteon.js`.
    static func name(_ path: String) -> String? {
        var clean = path.removingPercentEncoding ?? path
        if let i = clean.firstIndex(where: { $0 == "?" || $0 == "#" }) {
            clean = String(clean[..<i])
        }
        var parts = clean.split(separator: "/").map(String.init)
        if clean.hasSuffix("/") || parts.isEmpty { parts.append("index.html") }
        if let last = parts.last, !last.contains(".") { parts.append("index.html") }
        let allowed = CharacterSet.alphanumerics.union(CharacterSet(charactersIn: "._-"))
        for part in parts {
            guard !part.isEmpty, part != ".", part != "..",
                  part.unicodeScalars.allSatisfy({ allowed.contains($0) && $0.isASCII })
            else { return nil }
        }
        return parts.joined(separator: "/")
    }
}

/// Les liens signés de Nuage et les téléchargements.
actor NuageFiles {
    static let shared = NuageFiles()

    struct Entry: Decodable, Sendable {
        var name: String
        var size: Int
    }

    private var links: [String: (url: URL, until: Date)] = [:]
    private let session: URLSession = {
        let configuration = URLSessionConfiguration.default
        configuration.timeoutIntervalForRequest = 60
        configuration.httpMaximumConnectionsPerHost = 8
        return URLSession(configuration: configuration)
    }()

    enum Failure: Error {
        case notConfigured, refused(Int), invalidName
    }

    /// Le lien signé d'un fichier (gardé 50 minutes : Nuage les signe pour une heure).
    func link(_ name: String) async throws -> URL {
        if let cached = links[name], cached.until > Date() { return cached.url }
        guard let token = Nuage.token else { throw Failure.notConfigured }
        var components = URLComponents(url: Nuage.base.appendingPathComponent("v1/files/" + name),
                                       resolvingAgainstBaseURL: false)!
        components.queryItems = [URLQueryItem(name: "link", value: nil)]
        var request = URLRequest(url: components.url!)
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        let (data, response) = try await session.data(for: request)
        let status = (response as? HTTPURLResponse)?.statusCode ?? 0
        guard status == 200 else { throw Failure.refused(status) }
        struct Link: Decodable { var url: URL }
        let url = try JSONDecoder().decode(Link.self, from: data).url
        links[name] = (url, Date().addingTimeInterval(50 * 60))
        return url
    }

    /// Télécharge un fichier dans `destination` (créée au besoin, remplacée en une fois).
    func download(_ name: String, to destination: URL) async throws {
        let url = try await link(name)
        let (temporary, response) = try await session.download(from: url)
        let status = (response as? HTTPURLResponse)?.statusCode ?? 0
        guard status == 200 else {
            links[name] = nil
            throw Failure.refused(status)
        }
        let fm = FileManager.default
        try fm.createDirectory(at: destination.deletingLastPathComponent(), withIntermediateDirectories: true)
        try? fm.removeItem(at: destination)
        try fm.moveItem(at: temporary, to: destination)
    }

    /// Quelques octets d'un fichier (une vignette dans un paquet).
    func bytes(_ name: String, offset: Int, length: Int) async throws -> Data {
        let url = try await link(name)
        var request = URLRequest(url: url)
        request.setValue("bytes=\(offset)-\(offset + length - 1)", forHTTPHeaderField: "Range")
        let (data, response) = try await session.data(for: request)
        let status = (response as? HTTPURLResponse)?.statusCode ?? 0
        guard status == 206 || status == 200 else {
            links[name] = nil
            throw Failure.refused(status)
        }
        // Un serveur qui ignore `Range` renvoie tout le fichier.
        return status == 200 ? data.subdata(in: offset..<min(offset + length, data.count)) : data
    }

    /// Les fichiers dont le nom commence par `prefix`.
    func list(prefix: String = "") async throws -> [Entry] {
        guard let token = Nuage.token else { throw Failure.notConfigured }
        var components = URLComponents(url: Nuage.base.appendingPathComponent("v1/files"),
                                       resolvingAgainstBaseURL: false)!
        if !prefix.isEmpty { components.queryItems = [URLQueryItem(name: "prefix", value: prefix)] }
        var request = URLRequest(url: components.url!)
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        let (data, response) = try await session.data(for: request)
        let status = (response as? HTTPURLResponse)?.statusCode ?? 0
        guard status == 200 else { throw Failure.refused(status) }
        struct Listing: Decodable { var files: [Entry] }
        return try JSONDecoder().decode(Listing.self, from: data).files
    }
}

extension LocalMuseum {
    /// Le musée téléchargé depuis Nuage, sur l'appareil (Application Support/Musee).
    static let downloads: URL = {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? FileManager.default.temporaryDirectory
        let folder = base.appendingPathComponent("Musee", isDirectory: true)
        try? FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        return folder
    }()

    /// Le fichier d'un chemin du site sur l'appareil : dans l'application (si elle a été
    /// compilée avec le musée), sinon déjà téléchargé.
    static func stored(_ path: String) -> URL? {
        if let bundled = root.flatMap({ file(path, in: $0) }) { return bundled }
        return file(path, in: downloads)
    }

    /// Le fichier d'un chemin du site : sur l'appareil, sinon téléchargé depuis Nuage
    /// (puis gardé). `nil` sans Internet ni copie.
    static func fetch(_ path: String) async -> URL? {
        if let local = stored(path) { return local }
        guard Nuage.isConfigured, let name = Nuage.name(path) else { return nil }
        let destination = downloads.appendingPathComponent(name)
        do {
            try await NuageFiles.shared.download(name, to: destination)
            return destination
        } catch {
            return nil
        }
    }
}
