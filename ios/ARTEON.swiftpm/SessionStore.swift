// La mémoire de la visite : tout ce que l'application reçoit de l'API (le catalogue, les
// collections, le catalogue de chaque artiste, les images) est gardé pendant la visite dans
// une base SQLite, compressé (LZFSE), pour ne pas le redemander. La base est dans le
// dossier temporaire de l'application et elle est effacée quand on quitte ARTEON (et, par
// précaution, à chaque ouverture) : rien ne reste sur l'appareil d'une visite à l'autre.

import Foundation
#if canImport(SQLite3)
import SQLite3

actor SessionStore {
    static let shared = SessionStore()

    private var db: OpaquePointer?
    private let path: String = FileManager.default.temporaryDirectory
        .appendingPathComponent("arteon-visite.sqlite").path

    private init() {}

    private func open() -> OpaquePointer? {
        if let db { return db }
        var handle: OpaquePointer?
        guard sqlite3_open(path, &handle) == SQLITE_OK else { return nil }
        sqlite3_exec(handle, "PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF;", nil, nil, nil)
        sqlite3_exec(handle, "CREATE TABLE IF NOT EXISTS reponses (cle TEXT PRIMARY KEY, donnees BLOB, compresse INTEGER, quand REAL)", nil, nil, nil)
        db = handle
        return handle
    }

    /// Les octets gardés pour une adresse, s'il y en a.
    func get(_ key: String) -> Data? {
        guard let db = open() else { return nil }
        var statement: OpaquePointer?
        defer { sqlite3_finalize(statement) }
        guard sqlite3_prepare_v2(db, "SELECT donnees, compresse FROM reponses WHERE cle = ?", -1, &statement, nil) == SQLITE_OK
        else { return nil }
        sqlite3_bind_text(statement, 1, key, -1, Self.transient)
        guard sqlite3_step(statement) == SQLITE_ROW, let bytes = sqlite3_column_blob(statement, 0) else { return nil }
        let data = Data(bytes: bytes, count: Int(sqlite3_column_bytes(statement, 0)))
        if sqlite3_column_int(statement, 1) == 1 {
            return try? (data as NSData).decompressed(using: .lzfse) as Data
        }
        return data
    }

    /// Garde les octets d'une adresse, compressés quand ils s'y prêtent (le JSON).
    func put(_ key: String, _ data: Data) {
        guard let db = open() else { return }
        let packed = (try? (data as NSData).compressed(using: .lzfse) as Data).flatMap { $0.count < data.count * 9 / 10 ? $0 : nil }
        let stored = packed ?? data
        var statement: OpaquePointer?
        defer { sqlite3_finalize(statement) }
        guard sqlite3_prepare_v2(db, "INSERT OR REPLACE INTO reponses VALUES (?, ?, ?, ?)", -1, &statement, nil) == SQLITE_OK
        else { return }
        sqlite3_bind_text(statement, 1, key, -1, Self.transient)
        _ = stored.withUnsafeBytes { sqlite3_bind_blob(statement, 2, $0.baseAddress, Int32(stored.count), Self.transient) }
        sqlite3_bind_int(statement, 3, packed == nil ? 0 : 1)
        sqlite3_bind_double(statement, 4, Date().timeIntervalSince1970)
        sqlite3_step(statement)
    }

    /// Efface toute la mémoire de la visite.
    func purge() {
        if let db { sqlite3_close(db) }
        db = nil
        try? FileManager.default.removeItem(atPath: path)
    }

    /// La place prise par la visite, en octets.
    func size() -> Int64 {
        (try? FileManager.default.attributesOfItem(atPath: path)[.size] as? Int64) ?? 0
    }

    /// Efface la base sans attendre (à la fermeture de l'application, quand il n'y a plus le
    /// temps d'une tâche).
    nonisolated static func purgeNow() {
        let path = FileManager.default.temporaryDirectory.appendingPathComponent("arteon-visite.sqlite").path
        try? FileManager.default.removeItem(atPath: path)
    }

    private static var transient: sqlite3_destructor_type { unsafeBitCast(-1, to: sqlite3_destructor_type.self) }
}
#endif
