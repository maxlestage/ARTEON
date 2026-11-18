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
                LabeledContent(L("Expositions", "Exhibitions", "Exposiciones"), value: "\(catalogue.exhibitions.count)")
                LabeledContent(L("Œuvres des expositions", "Works in the exhibitions", "Obras de las exposiciones"), value: "\(catalogue.workCount)")
                if let collections = store.collections {
                    LabeledContent(L("Œuvres des collections du monde", "Works in the world collections", "Obras de las colecciones del mundo"), value: collections.count.formatted())
                }
                LabeledContent(L("Artistes", "Artists", "Artistas"), value: "\(store.index?.artists.count ?? 0)")
                LabeledContent(L("Salles en 3D", "3D rooms", "Salas en 3D"), value: "\(catalogue.roomCount)")
            } header: {
                Text(L("Le musée", "The museum", "El museo"))
            }

            Section {
                LabeledContent(L("Site", "Website", "Sitio"), value: Site.base.host ?? "")
                LabeledContent(L("Langue", "Language", "Idioma"), value: AppLanguage.current.name)
                LabeledContent(L("API Paquets", "Paquets API", "API Paquets"), value: Api.server.host ?? "")
                Button {
                    Task { await store.update() }
                } label: {
                    HStack {
                        Label(L("Recharger le musée", "Reload the museum", "Recargar el museo"), systemImage: "arrow.triangle.2.circlepath")
                        Spacer()
                        if store.updating { ProgressView() }
                    }
                }
                .disabled(store.updating)
                if let error = store.lastUpdateError {
                    Text(L("Le site n'a pas répondu.", "The website did not respond.", "El sitio no ha respondido.") + " (\(error))")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                LabeledContent(L("Mémoire de la visite", "Visit memory", "Memoria de la visita"), value: visitSize.formatted(.byteCount(style: .file)))
                Button(L("Effacer la mémoire de la visite", "Clear the visit memory", "Borrar la memoria de la visita"), role: .destructive) {
                    Task {
                        await SessionStore.shared.purge()
                        visitSize = await SessionStore.shared.size()
                    }
                }
            } header: {
                Text(L("L'API ARTEON", "The ARTEON API", "La API de ARTEON"))
            } footer: {
                Text(L("Tout passe par l'API : le musée, ses images, les galeries 3D et les collections viennent du site ARTEON, et toutes les œuvres de chaque artiste de l'API Paquets, en direct des musées. Pendant la visite, une copie compressée (SQLite) évite de tout redemander ; elle est effacée quand on quitte ARTEON. La langue est celle de l'appareil (ou celle choisie pour ARTEON dans les Réglages d'iOS).",
                       "Everything comes from the API: the museum, its images, the 3D galleries and the collections come from the ARTEON website, and all the works of each artist from the Paquets API, live from the museums. During the visit, a compressed copy (SQLite) avoids asking for everything again; it is erased when you leave ARTEON. The language is the device's (or the one chosen for ARTEON in iOS Settings).",
                       "Todo pasa por la API: el museo, sus imágenes, las galerías 3D y las colecciones vienen del sitio ARTEON, y todas las obras de cada artista de la API Paquets, en directo desde los museos. Durante la visita, una copia comprimida (SQLite) evita volver a pedirlo todo; se borra al salir de ARTEON. El idioma es el del dispositivo (o el elegido para ARTEON en los Ajustes de iOS)."))
            }

            Section {
                VStack(alignment: .leading) {
                    Text(L("Vitesse de la voix", "Voice speed", "Velocidad de la voz"))
                    Slider(value: $narrator.rate, in: 0.3...0.62) {
                        Text(L("Vitesse de la voix", "Voice speed", "Velocidad de la voz"))
                    } minimumValueLabel: {
                        Image(systemName: "tortoise")
                    } maximumValueLabel: {
                        Image(systemName: "hare")
                    }
                }
                ListenButton(id: "essai", text: {
                    L("Bienvenue au musée ARTEON. Voici la voix de l'audioguide.",
                      "Welcome to the ARTEON museum. This is the voice of the audio guide.",
                      "Bienvenido al museo ARTEON. Esta es la voz de la audioguía.")
                }, label: L("Essayer la voix", "Try the voice", "Probar la voz"))
            } header: {
                Text(L("Audioguide", "Audio guide", "Audioguía"))
            } footer: {
                Text(L("Pour une voix plus naturelle : Réglages de l'iPad → Accessibilité → Contenu énoncé → Voix → Français, et télécharger une voix « améliorée ».",
                       "For a more natural voice: iPad Settings → Accessibility → Spoken Content → Voices → English, and download an “Enhanced” voice.",
                       "Para una voz más natural: Ajustes del iPad → Accesibilidad → Contenido leído → Voces → Español, y descargue una voz «mejorada»."))
            }

            Section(L("Mes données", "My data", "Mis datos")) {
                LabeledContent(L("Favoris", "Favourites", "Favoritos"), value: "\(store.favorites.count)")
                LabeledContent(L("Œuvres vues", "Works viewed", "Obras vistas"), value: "\(store.history.count)")
                LabeledContent(L("Record au jeu", "Game record", "Récord del juego"), value: "\(store.quizBest)")
                Button(L("Effacer l'historique", "Clear history", "Borrar el historial"), role: .destructive) { store.clearHistory() }
                    .disabled(store.history.isEmpty)
                Button(L("Effacer les favoris", "Clear favourites", "Borrar los favoritos"), role: .destructive) { confirmFavorites = true }
                    .disabled(store.favorites.isEmpty)
            }

            Section {
                Text(L("Les œuvres exposées sont dans le domaine public : leurs artistes sont morts en 1955 ou avant, et elles datent d'avant 1931. Les œuvres encore protégées ne sont jamais reproduites, seulement racontées. Reproductions : les collections ouvertes des musées (CC0) et Wikimedia Commons, avec l'auteur et la licence de chaque photographie sous licence libre ; une mosaïque faite d'une photographie CC BY-SA est partagée aux mêmes conditions. Sculptures en 3D : Smithsonian Open Access (CC0).",
                       "The works on display are in the public domain: their artists died in 1955 or earlier, and they date from before 1931. Works still under copyright are never reproduced, only described. Reproductions: the museums' open collections (CC0) and Wikimedia Commons, with the author and licence of each freely licensed photograph; a mosaic made from a CC BY-SA photograph is shared under the same terms. 3D sculptures: Smithsonian Open Access (CC0).",
                       "Las obras expuestas son de dominio público: sus artistas murieron en 1955 o antes, y son anteriores a 1931. Las obras aún protegidas nunca se reproducen, solo se cuentan. Reproducciones: las colecciones abiertas de los museos (CC0) y Wikimedia Commons, con el autor y la licencia de cada fotografía con licencia libre; un mosaico hecho a partir de una fotografía CC BY-SA se comparte en las mismas condiciones. Esculturas en 3D: Smithsonian Open Access (CC0)."))
                    .font(.callout)
                Text(L("ARTEON est un projet indépendant : les musées réels qui inspirent ses bâtiments n'y sont pas associés et ne l'ont pas approuvé. L'application n'a ni publicité, ni traceur, ni compte, et ne garde rien du musée sur l'appareil.",
                       "ARTEON is an independent project: the real museums that inspire its buildings are not associated with it and have not endorsed it. The app has no ads, no trackers and no account, and keeps nothing of the museum on the device.",
                       "ARTEON es un proyecto independiente: los museos reales que inspiran sus edificios no están asociados a él ni lo han aprobado. La aplicación no tiene publicidad, ni rastreadores, ni cuenta, y no guarda nada del museo en el dispositivo."))
                    .font(.callout)
                Link(destination: URL(string: "https://github.com/maxlestage/ARTEON")!) {
                    Label(L("Le code d'ARTEON (licence MIT)", "ARTEON's source code (MIT licence)", "El código de ARTEON (licencia MIT)"), systemImage: "chevron.left.forwardslash.chevron.right")
                }
                Link(destination: URL(string: "https://commons.wikimedia.org/")!) {
                    Label("Wikimedia Commons", systemImage: "photo.on.rectangle")
                }
                Link(destination: URL(string: "https://3d.si.edu/")!) {
                    Label("Smithsonian 3D", systemImage: "cube")
                }
                LabeledContent(L("Version", "Version", "Versión"), value: Self.version)
            } header: {
                Text(L("À propos", "About", "Acerca de"))
            }
        }
        .navigationTitle(L("Réglages", "Settings", "Ajustes"))
        .task { visitSize = await SessionStore.shared.size() }
        .confirmationDialog(L("Effacer tous les favoris ?", "Clear all favourites?", "¿Borrar todos los favoritos?"), isPresented: $confirmFavorites, titleVisibility: .visible) {
            Button(L("Effacer les \(store.favorites.count) favoris", "Clear \(store.favorites.count) favourites", "Borrar \(store.favorites.count) favoritos"), role: .destructive) { store.clearFavorites() }
        }
    }

    private static var version: String {
        let info = Bundle.main.infoDictionary
        let short = info?["CFBundleShortVersionString"] as? String ?? "1.0"
        let build = info?["CFBundleVersion"] as? String ?? "1"
        return "\(short) (\(build))"
    }
}
