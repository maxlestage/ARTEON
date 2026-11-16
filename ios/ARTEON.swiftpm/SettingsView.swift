import SwiftUI

/// Les réglages et l'à-propos : le musée sur l'appareil, les mises à jour, l'audioguide,
/// les données gardées, les crédits.
struct SettingsView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store
    @Environment(Narrator.self) private var narrator
    @State private var bundledSize: Int64?
    @State private var downloadedSize: Int64?
    @State private var confirmFavorites = false

    var body: some View {
        @Bindable var store = store
        @Bindable var narrator = narrator
        Form {
            Section {
                LabeledContent("Expositions", value: "\(catalogue.exhibitions.count)")
                LabeledContent("Œuvres des expositions", value: "\(catalogue.workCount)")
                if let collections = store.collections {
                    LabeledContent("Œuvres des collections du monde", value: collections.count.formatted())
                }
                LabeledContent("Artistes", value: "\(store.index?.artists.count ?? 0)")
                LabeledContent("Salles en 3D", value: "\(catalogue.roomCount)")
            } header: {
                Text("Le musée")
            }

            Section {
                if store.isBundled {
                    Label {
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Tout le musée est sur l'appareil")
                            Text("Catalogue, images et galeries 3D se visitent sans Internet.")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    } icon: {
                        Image(systemName: "checkmark.seal.fill").foregroundStyle(.green)
                    }
                    if let bundledSize {
                        LabeledContent("Place occupée", value: bundledSize.formatted(.byteCount(style: .file)))
                    }
                } else {
                    Label {
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Musée téléchargé au fil de la visite")
                            Text("Les images vues restent sur l'appareil ; la 3D demande Internet.")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    } icon: {
                        Image(systemName: "icloud.and.arrow.down").foregroundStyle(.orange)
                    }
                }
                LabeledContent("Catalogue", value: sourceLabel)
                if let downloadedSize, downloadedSize > 0 {
                    LabeledContent("Images téléchargées", value: downloadedSize.formatted(.byteCount(style: .file)))
                    Button("Effacer les images téléchargées", role: .destructive) {
                        Task {
                            await ImageStore.shared.clearDownloads()
                            self.downloadedSize = await ImageStore.shared.downloadedSize()
                        }
                    }
                }
            } header: {
                Text("Hors ligne")
            }

            Section {
                Toggle("Chercher les nouveautés sur le site", isOn: $store.updatesEnabled)
                Button {
                    Task { await store.update() }
                } label: {
                    HStack {
                        Label("Mettre à jour maintenant", systemImage: "arrow.triangle.2.circlepath")
                        Spacer()
                        if store.updating { ProgressView() }
                    }
                }
                .disabled(store.updating)
                if let error = store.lastUpdateError {
                    Text("Le site n'a pas répondu : le musée de l'appareil reste affiché. (\(error))")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            } header: {
                Text("Mises à jour")
            } footer: {
                Text("Facultatif : les nouvelles expositions publiées sur \(Site.base.host ?? "le site") arrivent dans l'application quand Internet est là.")
            }

            Section {
                VStack(alignment: .leading) {
                    Text("Vitesse de la voix")
                    Slider(value: $narrator.rate, in: 0.3...0.62) {
                        Text("Vitesse de la voix")
                    } minimumValueLabel: {
                        Image(systemName: "tortoise")
                    } maximumValueLabel: {
                        Image(systemName: "hare")
                    }
                }
                ListenButton(id: "essai", text: {
                    "Bienvenue au musée ARTEON. Voici la voix de l'audioguide."
                }, label: "Essayer la voix")
            } header: {
                Text("Audioguide")
            } footer: {
                Text("Pour une voix plus naturelle : Réglages de l'iPad → Accessibilité → Contenu énoncé → Voix → Français, et télécharger une voix « améliorée ».")
            }

            Section("Mes données") {
                LabeledContent("Favoris", value: "\(store.favorites.count)")
                LabeledContent("Œuvres vues", value: "\(store.history.count)")
                LabeledContent("Record au jeu", value: "\(store.quizBest)")
                Button("Effacer l'historique", role: .destructive) { store.clearHistory() }
                    .disabled(store.history.isEmpty)
                Button("Effacer les favoris", role: .destructive) { confirmFavorites = true }
                    .disabled(store.favorites.isEmpty)
            }

            Section {
                Text("Les œuvres exposées sont dans le domaine public, ou montrées par des photographies sous licence libre d'œuvres installées dans l'espace public. Reproductions : Wikimedia Commons et les collections ouvertes des musées. Sculptures en 3D : Smithsonian Open Access (CC0).")
                    .font(.callout)
                Link(destination: URL(string: "https://github.com/maxlestage/ARTEON")!) {
                    Label("Le code d'ARTEON (licence MIT)", systemImage: "chevron.left.forwardslash.chevron.right")
                }
                Link(destination: URL(string: "https://commons.wikimedia.org/")!) {
                    Label("Wikimedia Commons", systemImage: "photo.on.rectangle")
                }
                Link(destination: URL(string: "https://3d.si.edu/")!) {
                    Label("Smithsonian 3D", systemImage: "cube")
                }
                LabeledContent("Version", value: Self.version)
            } header: {
                Text("À propos")
            }
        }
        .navigationTitle("Réglages")
        .confirmationDialog("Effacer tous les favoris ?", isPresented: $confirmFavorites, titleVisibility: .visible) {
            Button("Effacer les \(store.favorites.count) favoris", role: .destructive) { store.clearFavorites() }
        }
        .task {
            bundledSize = await Task.detached(priority: .utility) { LocalMuseum.size() }.value
            downloadedSize = await ImageStore.shared.downloadedSize()
        }
    }

    private var sourceLabel: String {
        switch store.source {
        case .bundled: "Embarqué dans l'application"
        case .online: "À jour avec le site"
        case .saved: "Dernière mise à jour gardée"
        }
    }

    private static var version: String {
        let info = Bundle.main.infoDictionary
        let short = info?["CFBundleShortVersionString"] as? String ?? "1.0"
        let build = info?["CFBundleVersion"] as? String ?? "1"
        return "\(short) (\(build))"
    }
}
