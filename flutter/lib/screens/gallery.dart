import 'package:flutter/material.dart';
import 'package:webview_flutter/webview_flutter.dart';

import '../models.dart';
import '../repository.dart';

/// La galerie 3D de l'exposition, celle du site (WebGL 2, écrite en Rust), en plein
/// écran : visite guidée ou libre, plan, notices. Ouverte devant une œuvre si `work`.
class GalleryScreen extends StatefulWidget {
  const GalleryScreen(this.ex, {super.key, this.work});

  final Exhibition ex;
  final Work? work;

  @override
  State<GalleryScreen> createState() => _GalleryScreenState();
}

/// Ne garde de la page que la galerie, sur tout l'écran.
const _fullScreen = r'''
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
''';

class _GalleryScreenState extends State<GalleryScreen> {
  late final WebViewController _controller;
  var _progress = 0;
  String? _error;

  Uri get _url {
    final uri = siteUri(widget.ex.gallery);
    final work = widget.work;
    return work == null ? uri : uri.replace(fragment: work.slug);
  }

  @override
  void initState() {
    super.initState();
    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..setBackgroundColor(const Color(0xff0b0b10))
      ..setNavigationDelegate(
        NavigationDelegate(
          onProgress: (p) => setState(() => _progress = p),
          onPageStarted: (_) =>
              _controller.runJavaScript(_fullScreen).catchError((_) {}),
          onPageFinished: (_) {
            _controller.runJavaScript(_fullScreen).catchError((_) {});
            setState(() => _progress = 100);
          },
          onWebResourceError: (e) {
            if (e.isForMainFrame ?? true) {
              setState(() => _error = e.description);
            }
          },
          // Les liens vers d'autres pages du site restent dans la galerie.
          onNavigationRequest: (request) =>
              request.url.startsWith(siteUrl) || request.url == 'about:blank'
              ? NavigationDecision.navigate
              : NavigationDecision.prevent,
        ),
      )
      ..loadRequest(_url);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xff0b0b10),
      appBar: AppBar(
        backgroundColor: const Color(0xff0b0b10),
        foregroundColor: Colors.white,
        title: Text('Galerie 3D · ${widget.ex.shortName}'),
        actions: [
          IconButton(
            tooltip: 'Recharger',
            icon: const Icon(Icons.refresh),
            onPressed: () {
              setState(() => _error = null);
              _controller.loadRequest(_url);
            },
          ),
        ],
        bottom: _progress < 100
            ? PreferredSize(
                preferredSize: const Size.fromHeight(3),
                child: LinearProgressIndicator(
                  value: _progress / 100,
                  minHeight: 3,
                ),
              )
            : null,
      ),
      body: _error == null
          ? WebViewWidget(controller: _controller)
          : Center(
              child: Padding(
                padding: const EdgeInsets.all(32),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const Icon(Icons.wifi_off, color: Colors.white70, size: 48),
                    const SizedBox(height: 16),
                    const Text(
                      'La galerie 3D a besoin d\'Internet.',
                      style: TextStyle(color: Colors.white, fontSize: 18),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      _error!,
                      style: const TextStyle(color: Colors.white60),
                    ),
                  ],
                ),
              ),
            ),
    );
  }
}
