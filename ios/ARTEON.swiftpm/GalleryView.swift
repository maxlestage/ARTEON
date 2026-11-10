import SwiftUI
import WebKit

/// La galerie 3D de l'exposition, celle du site (WebGL 2, écrite en Rust), en plein
/// écran : visite guidée ou libre, plan, notices ; ouverte devant une œuvre si `work`.
struct GalleryView: View {
    let exhibition: Exhibition
    let work: String?

    @State private var reload = 0

    private var url: URL {
        var components = URLComponents(
            url: Site.url(exhibition.gallery),
            resolvingAgainstBaseURL: true
        )
        components?.fragment = work
        return components?.url ?? Site.url(exhibition.gallery)
    }

    var body: some View {
        GalleryWebView(url: url, reload: reload)
            .ignoresSafeArea(edges: .bottom)
            .background(Color(red: 0.04, green: 0.04, blue: 0.06))
            .navigationTitle("Galerie 3D · \(exhibition.name)")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button("Recharger", systemImage: "arrow.clockwise") { reload += 1 }
                }
            }
            .toolbarVisibility(.hidden, for: .tabBar)
    }
}

/// La page de la galerie dans un WKWebView, réduite à la scène 3D.
struct GalleryWebView: UIViewRepresentable {
    let url: URL
    let reload: Int

    /// Ne garde de la page que la galerie, sur toute la vue.
    static let fullScreen = """
    (() => {
      if (document.getElementById('arteon-app')) return;
      const style = document.createElement('style');
      style.id = 'arteon-app';
      style.textContent = `
        html, body { overflow: hidden !important; background: #0b0b10 !important; }
        .viewer { position: fixed !important; inset: 0 !important; z-index: 2147483000;
          border: 0 !important; border-radius: 0 !important; }
        .viewer .scene-gallery { height: 100vh !important; height: 100dvh !important; }
        .viewer-full { display: none !important; }
      `;
      document.head.appendChild(style);
      window.dispatchEvent(new Event('resize'));
    })();
    """

    final class Coordinator {
        var loaded = 0
    }

    func makeCoordinator() -> Coordinator { Coordinator() }

    func makeUIView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.allowsInlineMediaPlayback = true
        configuration.userContentController.addUserScript(
            WKUserScript(source: Self.fullScreen, injectionTime: .atDocumentEnd, forMainFrameOnly: true)
        )
        let view = WKWebView(frame: .zero, configuration: configuration)
        view.isOpaque = false
        view.backgroundColor = .black
        view.scrollView.backgroundColor = .black
        view.scrollView.contentInsetAdjustmentBehavior = .never
        view.scrollView.isScrollEnabled = false
        view.isInspectable = true
        view.load(URLRequest(url: url))
        context.coordinator.loaded = reload
        return view
    }

    func updateUIView(_ view: WKWebView, context: Context) {
        guard context.coordinator.loaded != reload else { return }
        context.coordinator.loaded = reload
        view.load(URLRequest(url: url))
    }
}
