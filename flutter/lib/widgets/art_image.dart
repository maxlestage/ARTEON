import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';

import '../repository.dart';

/// L'image d'une œuvre, téléchargée une fois puis gardée sur l'appareil.
class ArtImage extends StatelessWidget {
  const ArtImage(
    this.path, {
    super.key,
    this.fit = BoxFit.cover,
    this.tint,
    this.semanticLabel,
  });

  final String? path;
  final BoxFit fit;
  final Color? tint;
  final String? semanticLabel;

  @override
  Widget build(BuildContext context) {
    final background =
        tint ?? Theme.of(context).colorScheme.surfaceContainerHighest;
    final path = this.path;
    if (path == null || path.isEmpty) {
      return ColoredBox(color: background);
    }
    return Semantics(
      label: semanticLabel,
      image: true,
      child: CachedNetworkImage(
        imageUrl: siteUri(path).toString(),
        fit: fit,
        fadeInDuration: const Duration(milliseconds: 250),
        placeholder: (_, _) =>
            ColoredBox(color: background.withValues(alpha: .35)),
        errorWidget: (_, _, _) => ColoredBox(
          color: background.withValues(alpha: .35),
          child: const Center(child: Icon(Icons.image_not_supported_outlined)),
        ),
      ),
    );
  }
}
