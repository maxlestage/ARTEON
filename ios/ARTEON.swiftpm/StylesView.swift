import SwiftUI

// Les styles et mouvements, les genres, les techniques et les sujets représentés : le pendant
// de l'onglet « Styles et genres » du site (outils/monde/explorer.js). La liste vient de
// `monde/sujets.json`, les œuvres de chaque sujet de `api/sujets/` (calculées par l'API à
// chaque construction du site) : les plus connues d'abord, toutes du domaine public.

/// La liste des sujets : toutes les catégories confondues d'abord, des plus riches aux moins
/// riches ; un bouton par famille (styles, genres, techniques…) ne montre qu'elle. Avec une
/// recherche.
struct StylesView: View {
    @State private var families: [SubjectFamily] = []
    @State private var published: [String: PublishedSubject] = [:]
    @State private var loading = true
    @State private var search = ""
    /// Le bouton choisi : `nil`, toutes les catégories confondues ; sinon la clé d'une famille.
    @State private var chosen: String? = nil
    /// Toutes les catégories confondues, chacune une fois, des plus riches aux moins riches.
    @State private var everything: [(subject: Subject, family: String)] = []
    /// « Tout afficher » : sinon les 24 premières.
    @State private var unfolded = false
    private static let first = 24

    private static func folded(_ text: String) -> String {
        text.folding(options: [.diacriticInsensitive, .caseInsensitive], locale: nil)
    }

    /// Les sujets du bouton choisi, filtrés par la recherche.
    private var shown: [(subject: Subject, family: String)] {
        let all = chosen.flatMap { key in families.first { $0.key == key } }
            .map { family in family.subjects.map { (subject: $0, family: family.name) } } ?? everything
        let words = Self.folded(search).split(separator: " ")
        guard !words.isEmpty else { return all }
        return all.filter { item in
            let names = Self.folded("\(item.subject.fr) \(item.subject.en) \(item.subject.es)")
            return words.allSatisfy { names.contains($0) }
        }
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 14) {
                if loading {
                    HStack(spacing: 10) {
                        ProgressView()
                        Text(L("Ouverture des styles et des genres…", "Opening the styles and genres…", "Abriendo los estilos y los géneros…"))
                            .foregroundStyle(.secondary)
                    }
                } else if families.isEmpty {
                    Text(L("Le site n'a pas répondu. Réessayer plus tard.", "The site did not respond. Please try again later.", "El sitio no ha respondido. Vuelva a intentarlo más tarde."))
                        .foregroundStyle(.secondary)
                } else {
                    Text(L("De l'art gothique au Bauhaus, du portrait à la nature morte, de la fresque à l'estampe : pour chacun, ses œuvres libres de droits dans les musées du monde, les plus connues d'abord.",
                           "From Gothic art to the Bauhaus, from portraits to still lifes, from frescoes to prints: for each, its copyright-free works in the world's museums, best known first.",
                           "Del arte gótico a la Bauhaus, del retrato al bodegón, del fresco a la estampa: para cada uno, sus obras libres de derechos en los museos del mundo, las más conocidas primero."))
                        .foregroundStyle(.secondary)
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 8) {
                            familyButton(nil, L("Toutes", "All", "Todas"), everything.count)
                            ForEach(families) { familyButton($0.key, $0.name, $0.subjects.count) }
                        }
                    }
                    let all = shown
                    let open = !search.isEmpty || unfolded
                    let items = open ? all : Array(all.prefix(Self.first))
                    if all.isEmpty {
                        Text(L("Aucune catégorie de ce nom.", "No category by that name.", "Ninguna categoría con ese nombre."))
                            .foregroundStyle(.secondary)
                    } else if !search.isEmpty {
                        Text(all.count == 1 ? L("1 catégorie", "1 category", "1 categoría")
                                            : L("\(all.count.formatted()) catégories", "\(all.count.formatted()) categories", "\(all.count.formatted()) categorías"))
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }
                    LazyVGrid(columns: [GridItem(.adaptive(minimum: 150, maximum: 240), spacing: 10)], spacing: 10) {
                        ForEach(items, id: \.subject.key) { item in
                            NavigationLink(value: Route.subject(key: item.subject.key, name: item.subject.name, family: item.family)) {
                                SubjectTile(subject: item.subject, published: published[item.subject.key])
                            }
                            .buttonStyle(.plain)
                        }
                    }
                    if !open, all.count > Self.first {
                        Button(L("Tout afficher (\(all.count.formatted()))", "Show all (\(all.count.formatted()))", "Mostrar todo (\(all.count.formatted()))")) {
                            unfolded = true
                        }
                        .buttonStyle(.bordered)
                    }
                }
            }
            .padding()
            .padding(.bottom, 32)
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle(L("Styles et genres", "Styles and genres", "Estilos y géneros"))
        .searchable(text: $search, prompt: L("Un style, un genre, un sujet, un musée", "A style, a genre, a subject, a museum", "Un estilo, un género, un tema, un museo"))
        .task {
            async let ready = Subjects.published()
            families = await Subjects.families()
            published = await ready
            everything = SubjectFamily.merged(families, published)
            loading = false
        }
    }

    private func familyButton(_ key: String?, _ name: String, _ count: Int) -> some View {
        let selected = chosen == key
        return Button {
            withAnimation(.snappy) { chosen = key; unfolded = false }
        } label: {
            HStack(spacing: 5) {
                Text(name)
                Text(count.formatted()).opacity(0.7)
            }
            .font(.subheadline.weight(selected ? .semibold : .regular))
            .padding(.horizontal, 12)
            .padding(.vertical, 7)
            .background(selected ? Color.accentColor : Color(uiColor: .secondarySystemGroupedBackground), in: .capsule)
            .foregroundStyle(selected ? Color.white : Color.primary)
        }
        .buttonStyle(.plain)
        .accessibilityAddTraits(selected ? .isSelected : [])
    }
}

/// La tuile d'un sujet : son œuvre la plus connue derrière son nom.
struct SubjectTile: View {
    let subject: Subject
    let published: PublishedSubject?

    var body: some View {
        ZStack(alignment: .bottomLeading) {
            RemoteImage(path: published?.image.isEmpty == false ? published?.image : subject.image, tint: Color(white: 0.2), maxPixels: 480)
                .frame(height: 120)
                .frame(maxWidth: .infinity)
                .clipped()
            LinearGradient(colors: [.black.opacity(0.1), .black.opacity(0.8)], startPoint: .center, endPoint: .bottom)
            VStack(alignment: .leading, spacing: 2) {
                Text(subject.name)
                    .font(.subheadline.bold())
                    .lineLimit(2)
                    .multilineTextAlignment(.leading)
                if let total = published.map(\.total) ?? (subject.count > 0 ? subject.count : nil), total > 0 {
                    Text(worksCount(total))
                        .font(.caption)
                        .opacity(0.9)
                }
            }
            .foregroundStyle(.white)
            .padding(10)
        }
        .frame(height: 120)
        .clipShape(.rect(cornerRadius: 14))
        .contentShape(.rect(cornerRadius: 14))
        .hoverEffect(.lift)
        .accessibilityElement(children: .combine)
    }
}

/// Les œuvres d'un sujet : en mosaïque ou avec les titres, et en grand d'un toucher
/// (LiveGallery.swift), comme le catalogue d'un artiste.
struct SubjectView: View {
    let key: String
    let name: String
    let family: String

    @State private var works: [LiveWork]?
    @State private var museums: [String: String] = [:]
    @State private var loading = true
    @State private var shown: LiveWork?
    @AppStorage("vue-mosaique") private var mosaic = true

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                Text(name)
                    .font(.largeTitle.bold())
                Text(family)
                    .font(.headline)
                    .foregroundStyle(.secondary)
                if loading {
                    HStack(spacing: 10) {
                        ProgressView()
                        Text(L("Recherche de ses œuvres dans les musées…", "Looking for its works in the museums…", "Buscando sus obras en los museos…"))
                            .foregroundStyle(.secondary)
                    }
                } else if let works, !works.isEmpty {
                    Text(worksCount(works.count))
                        .font(.title2.bold())
                        .padding(.top, 6)
                    Text(L("Les plus connues d'abord · Domaine public (artiste mort en 1955 ou avant, œuvre d'avant 1931)",
                           "Best known first · Public domain (artist died in 1955 or earlier, work from before 1931)",
                           "Las más conocidas primero · Dominio público (artista fallecido en 1955 o antes, obra anterior a 1931)"))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Picker(L("Affichage", "View", "Vista"), selection: $mosaic) {
                        Text(L("Mosaïque", "Mosaic", "Mosaico")).tag(true)
                        Text(L("Avec les titres", "With titles", "Con los títulos")).tag(false)
                    }
                    .pickerStyle(.segmented)
                    .frame(maxWidth: 360)
                    if mosaic {
                        LazyVGrid(columns: [GridItem(.adaptive(minimum: 84, maximum: 150), spacing: 2)], spacing: 2) {
                            ForEach(works) { work in
                                Button {
                                    shown = work
                                } label: {
                                    RemoteImage(path: work.thumbnail, maxPixels: 320)
                                        .aspectRatio(1, contentMode: .fit)
                                }
                                .buttonStyle(.plain)
                                .accessibilityLabel(work.title)
                            }
                        }
                    } else {
                        LazyVGrid(columns: [GridItem(.adaptive(minimum: 150, maximum: 240), spacing: 12)], spacing: 12) {
                            ForEach(works) { work in
                                Button {
                                    shown = work
                                } label: {
                                    // Sous le titre : la date et l'auteur (le musée est dans la fiche).
                                    LiveWorkCard(work: work, museum: work.artist ?? "")
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }
                } else {
                    Text(L("Wikidata n'a pas répondu. Réessayer un peu plus tard.", "Wikidata did not respond. Please try again a little later.", "Wikidata no ha respondido. Vuelva a intentarlo un poco más tarde."))
                        .foregroundStyle(.secondary)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding()
            .padding(.bottom, 32)
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .navigationTitle(name)
        .navigationBarTitleDisplayMode(.inline)
        .fullScreenCover(item: $shown) { work in
            LiveWorkViewer(works: works ?? [], start: work, museums: museums, artist: "")
        }
        .task(id: key) {
            loading = true
            async let names = LiveWorks.museumNames()
            works = await Subjects.works(key)
            museums = await names
            loading = false
            // `-voir 3` au lancement : sa 4ᵉ œuvre en grand (pour les captures d'écran de la CI).
            let args = ProcessInfo.processInfo.arguments
            if let i = args.firstIndex(of: "-voir"), i + 1 < args.count, let n = Int(args[i + 1]),
               let works, works.indices.contains(n) {
                shown = works[n]
            }
        }
    }
}
