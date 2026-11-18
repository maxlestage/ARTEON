import SwiftUI
import WebKit

/// La galerie 3D de l'exposition (WebGL 2, écrite en Rust), en plein écran : visite
/// guidée ou libre, plan, notices ; ouverte devant une œuvre si `work`. Elle vient du site
/// ARTEON, comme tout le reste : rien n'est gardé sur l'appareil.
struct GalleryView: View {
    let exhibition: Exhibition
    let work: String?

    @State private var reload = 0
    @State private var failed: String?

    private var url: URL {
        let page = Site.url(exhibition.gallery + "/")
        var components = URLComponents(url: page, resolvingAgainstBaseURL: true)
        components?.fragment = work
        return components?.url ?? page
    }

    var body: some View {
        ZStack {
            Color(red: 0.04, green: 0.04, blue: 0.06).ignoresSafeArea()
            GalleryWebView(url: url, reload: reload) { failed = $0 }
                .ignoresSafeArea(edges: .bottom)
            if let failed {
                ContentUnavailableView {
                    Label("La galerie n'a pas pu s'ouvrir", systemImage: "view.3d")
                } description: {
                    Text(failed)
                } actions: {
                    Button("Réessayer", systemImage: "arrow.clockwise") {
                        self.failed = nil
                        reload += 1
                    }
                    .buttonStyle(.glassProminent)
                }
                .foregroundStyle(.white)
            }
        }
        .navigationTitle("Galerie 3D · \(exhibition.name)")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button("Recharger", systemImage: "arrow.clockwise") {
                    failed = nil
                    reload += 1
                }
            }
        }
        .toolbarVisibility(.hidden, for: .tabBar)
        .persistentSystemOverlays(.hidden)
    }
}

/// La page de la galerie dans un WKWebView, réduite à la scène 3D.
struct GalleryWebView: UIViewRepresentable {
    let url: URL
    let reload: Int
    let onError: (String) -> Void

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

    /// Pas de service worker dans l'application : rien n'est gardé hors ligne.
    static let noServiceWorker = """
    (() => {
      try {
        const sw = navigator.serviceWorker;
        if (sw) { sw.register = () => new Promise(() => {}); }
        else {
          Object.defineProperty(navigator, 'serviceWorker', {
            value: { register: () => new Promise(() => {}), ready: new Promise(() => {}),
                     addEventListener() {}, controller: null },
            configurable: true });
        }
      } catch (e) {}
    })();
    """

    @MainActor
    final class Coordinator: NSObject, WKNavigationDelegate {
        var loaded = 0
        var onError: (String) -> Void = { _ in }

        func webView(
            _ webView: WKWebView,
            decidePolicyFor action: WKNavigationAction
        ) async -> WKNavigationActionPolicy {
            // Les liens vers d'autres sites (sources, crédits) s'ouvrent dans Safari.
            if let url = action.request.url, let scheme = url.scheme,
               scheme == "http" || scheme == "https",
               action.navigationType == .linkActivated, url.host != Site.base.host {
                Task { await UIApplication.shared.open(url) }
                return .cancel
            }
            return .allow
        }

        func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: any Error) {
            report(error)
        }

        func webView(
            _ webView: WKWebView,
            didFailProvisionalNavigation navigation: WKNavigation!,
            withError error: any Error
        ) {
            report(error)
        }

        func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
            // La scène 3D a demandé trop de mémoire : on la recharge.
            webView.reload()
        }

        private func report(_ error: any Error) {
            let code = (error as NSError).code
            guard code != NSURLErrorCancelled else { return }
            onError(error.localizedDescription)
        }
    }

    func makeCoordinator() -> Coordinator { Coordinator() }

    func makeUIView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.allowsInlineMediaPlayback = true
        // Rien n'est gardé : ni cache, ni stockage du site sur l'appareil.
        configuration.websiteDataStore = .nonPersistent()
        configuration.userContentController.addUserScript(
            WKUserScript(source: Self.noServiceWorker, injectionTime: .atDocumentStart, forMainFrameOnly: true)
        )
        configuration.userContentController.addUserScript(
            WKUserScript(source: Self.fullScreen, injectionTime: .atDocumentEnd, forMainFrameOnly: true)
        )
        let view = WKWebView(frame: .zero, configuration: configuration)
        view.navigationDelegate = context.coordinator
        context.coordinator.onError = onError
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
        context.coordinator.onError = onError
        guard context.coordinator.loaded != reload else { return }
        context.coordinator.loaded = reload
        view.load(URLRequest(url: url))
    }
}
