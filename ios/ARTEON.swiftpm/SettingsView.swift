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
    @State private var nuageToken = UserDefaults.standard.string(forKey: "jetonNuage") ?? ""

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

            OfflineSection()

            Section {
                SecureField("Jeton en lecture seule", text: $nuageToken)
                    .textContentType(.password)
                    .autocorrectionDisabled()
                    .textInputAutocapitalization(.never)
                    .onSubmit { UserDefaults.standard.set(nuageToken, forKey: "jetonNuage") }
                    .onChange(of: nuageToken) { UserDefaults.standard.set(nuageToken, forKey: "jetonNuage") }
                LabeledContent("Serveur", value: Nuage.base.host ?? "")
            } header: {
                Text("Nuage")
            } footer: {
                Text("Facultatif : l'app compilée par la CI a déjà son jeton. Pour une copie ouverte dans Swift Playgrounds, coller ici un jeton en lecture seule de l'application arteon (console Nuage → Tokens).")
            }

            Section {
                LabeledContent("Catalogue", value: sourceLabel)
                if let bundledSize, bundledSize > 0 {
                    LabeledContent("Musée embarqué dans l'app", value: bundledSize.formatted(.byteCount(style: .file)))
                }
                if let downloadedSize, downloadedSize > 0 {
                    LabeledContent("Autres images gardées", value: downloadedSize.formatted(.byteCount(style: .file)))
                    Button("Effacer ces images", role: .destructive) {
                        Task {
                            await ImageStore.shared.clearDownloads()
                            self.downloadedSize = await ImageStore.shared.downloadedSize()
                        }
                    }
                }
            } header: {
                Text("Sur l'appareil")
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
                Text("Les œuvres exposées sont dans le domaine public : leurs artistes sont morts en 1955 ou avant, et elles datent d'avant 1931. Les œuvres encore protégées ne sont jamais reproduites, seulement racontées. Reproductions : les collections ouvertes des musées (CC0) et Wikimedia Commons, avec l'auteur et la licence de chaque photographie sous licence libre ; une mosaïque faite d'une photographie CC BY-SA est partagée aux mêmes conditions. Sculptures en 3D : Smithsonian Open Access (CC0).")
                    .font(.callout)
                Text("ARTEON est un projet indépendant : les musées réels qui inspirent ses bâtiments n'y sont pas associés et ne l'ont pas approuvé. L'application n'a ni publicité, ni traceur, ni compte ; tout le musée est dans l'appareil.")
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
