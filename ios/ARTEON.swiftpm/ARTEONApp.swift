// ARTEON sur iPad et iPhone : le musée virtuel en SwiftUI. Les expositions, les œuvres
// et leurs notices en natif, la recherche, les favoris, les images en grand, et la
// galerie 3D du site.

import SwiftUI

@main
struct ARTEONApp: App {
    @State private var store = CatalogueStore()

    var body: some Scene {
        WindowGroup {
            RootView()
                .environment(store)
                .tint(.arteonBlue)
        }
    }
}

/// L'écran d'ouverture, puis le musée.
struct RootView: View {
    @Environment(CatalogueStore.self) private var store

    var body: some View {
        Group {
            switch store.phase {
            case .loading:
                SplashView(message: "Ouverture du musée…", retry: nil)
            case .failed(let message):
                SplashView(
                    message: "Le musée n'a pas pu ouvrir : il faut Internet la première fois.\n\(message)",
                    retry: { Task { await store.load() } }
                )
            case .ready(let catalogue):
                MainView(catalogue: catalogue)
            }
        }
        .task { await store.load() }
    }
}

struct SplashView: View {
    let message: String
    let retry: (() -> Void)?

    var body: some View {
        ZStack {
            Color.arteonNight.ignoresSafeArea()
            VStack(spacing: 22) {
                ArteonLogo(size: 88)
                Text("ARTEON")
                    .font(.system(size: 40, weight: .heavy))
                    .tracking(10)
                    .foregroundStyle(.white)
                Text(message)
                    .multilineTextAlignment(.center)
                    .foregroundStyle(.white.opacity(0.75))
                    .frame(maxWidth: 460)
                if let retry {
                    Button("Réessayer", systemImage: "arrow.clockwise", action: retry)
                        .buttonStyle(.glassProminent)
                } else {
                    ProgressView().tint(.arteonSun)
                }
            }
            .padding(32)
        }
    }
}

/// Le logo du site : un soleil et une étoile sur un ciel bleu nuit.
struct ArteonLogo: View {
    var size: CGFloat = 44

    var body: some View {
        RoundedRectangle(cornerRadius: size * 0.25)
            .fill(
                RadialGradient(
                    colors: [Color(red: 0.24, green: 0.39, blue: 0.79), .arteonBlue, .arteonNight],
                    center: UnitPoint(x: 0.7, y: 0.3),
                    startRadius: 0,
                    endRadius: size
                )
            )
            .overlay(alignment: .topLeading) {
                Circle().fill(Color.arteonSun)
                    .frame(width: size * 0.46, height: size * 0.46)
                    .shadow(color: .arteonSun.opacity(0.6), radius: size * 0.12)
                    .offset(x: size * 0.18, y: size * 0.18)
            }
            .overlay(alignment: .bottomTrailing) {
                Circle().fill(Color(red: 1, green: 0.96, blue: 0.79))
                    .frame(width: size * 0.18, height: size * 0.18)
                    .offset(x: -size * 0.14, y: -size * 0.16)
            }
            .frame(width: size, height: size)
            .accessibilityHidden(true)
    }
}
