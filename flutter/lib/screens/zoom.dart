import 'package:flutter/material.dart';

import '../widgets/art_image.dart';

/// L'image en grand, sur fond noir : pincer pour agrandir, glisser pour se déplacer,
/// toucher deux fois pour revenir.
class ZoomScreen extends StatefulWidget {
  const ZoomScreen({super.key, required this.path, required this.title});

  final String path, title;

  @override
  State<ZoomScreen> createState() => _ZoomScreenState();
}

class _ZoomScreenState extends State<ZoomScreen> {
  final _controller = TransformationController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.black,
        foregroundColor: Colors.white,
        title: Text(widget.title, overflow: TextOverflow.ellipsis),
      ),
      body: GestureDetector(
        onDoubleTap: () => _controller.value = Matrix4.identity(),
        child: InteractiveViewer(
          transformationController: _controller,
          minScale: 1,
          maxScale: 8,
          child: SizedBox.expand(
            child: ArtImage(
              widget.path,
              fit: BoxFit.contain,
              tint: Colors.black,
            ),
          ),
        ),
      ),
    );
  }
}
