import SwiftUI

/// Les réglages et l'à-propos : d'où vient le musée (l'API), l'audioguide, les choix du
/// visiteur, les crédits.
struct SettingsView: View {
    let catalogue: Catalogue

    @Environment(CatalogueStore.self) private var store
    @Environment(Narrator.self) private var narrator
    @State private var confirmFavorites = false
    @State private var visitSize: Int64 = 0

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
                LabeledContent("Site", value: Site.base.host ?? "")
                LabeledContent("API Paquets", value: Api.server.host ?? "")
                Button {
                    Task { await store.update() }
                } label: {
                    HStack {
                        Label("Recharger le musée", systemImage: "arrow.triangle.2.circlepath")
                        Spacer()
                        if store.updating { ProgressView() }
                    }
                }
                .disabled(store.updating)
                if let error = store.lastUpdateError {
                    Text("Le site n'a pas répondu. (\(error))")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                LabeledContent("Mémoire de la visite", value: visitSize.formatted(.byteCount(style: .file)))
                Button("Effacer la mémoire de la visite", role: .destructive) {
                    Task {
                        await SessionStore.shared.purge()
                        visitSize = await SessionStore.shared.size()
                    }
                }
            } header: {
                Text("L'API ARTEON")
            } footer: {
                Text("Tout passe par l'API : le musée, ses images, les galeries 3D et les collections viennent du site ARTEON, et toutes les œuvres de chaque artiste de l'API Paquets, en direct des musées. Pendant la visite, une copie compressée (SQLite) évite de tout redemander ; elle est effacée quand on quitte ARTEON.")
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
                Text("ARTEON est un projet indépendant : les musées réels qui inspirent ses bâtiments n'y sont pas associés et ne l'ont pas approuvé. L'application n'a ni publicité, ni traceur, ni compte, et ne garde rien du musée sur l'appareil.")
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
        .task { visitSize = await SessionStore.shared.size() }
        .confirmationDialog("Effacer tous les favoris ?", isPresented: $confirmFavorites, titleVisibility: .visible) {
            Button("Effacer les \(store.favorites.count) favoris", role: .destructive) { store.clearFavorites() }
        }
    }

    private static var version: String {
        let info = Bundle.main.infoDictionary
        let short = info?["CFBundleShortVersionString"] as? String ?? "1.0"
        let build = info?["CFBundleVersion"] as? String ?? "1"
        return "\(short) (\(build))"
    }
}
