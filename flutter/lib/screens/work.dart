import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import '../app_scope.dart';
import '../models.dart';
import '../theme.dart';
import '../widgets/art_image.dart';
import 'exhibition.dart';
import 'gallery.dart';
import 'zoom.dart';

/// Une œuvre : son image à agrandir, son cartel, sa notice complète, son artiste. Sur
/// iPad à l'horizontale, l'image à gauche et le texte à droite.
class WorkScreen extends StatelessWidget {
  const WorkScreen(this.ex, this.work, {super.key});

  final Exhibition ex;
  final Work work;

  void _go(BuildContext context, int step) {
    final i = ex.works.indexOf(work) + step;
    if (i < 0 || i >= ex.works.length) return;
    Navigator.of(context).pushReplacement(
      PageRouteBuilder(
        pageBuilder: (_, _, _) => WorkScreen(ex, ex.works[i]),
        transitionsBuilder: (_, animation, _, child) =>
            FadeTransition(opacity: animation, child: child),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final favorites = AppScope.of(context).favorites;
    final index = ex.works.indexOf(work);
    final size = MediaQuery.sizeOf(context);
    final wide = size.width >= 900 && size.width > size.height * 1.1;
    final image = _Picture(ex, work);
    final text = _Text(ex, work);
    return Scaffold(
      appBar: AppBar(
        title: Text(ex.shortName.isEmpty ? ex.title : ex.shortName),
        actions: [
          ListenableBuilder(
            listenable: favorites,
            builder: (context, _) {
              final on = favorites.contains(ex, work);
              return IconButton(
                tooltip: on ? 'Retirer des favoris' : 'Ajouter aux favoris',
                icon: Icon(on ? Icons.favorite : Icons.favorite_border),
                color: on ? Colors.redAccent : null,
                onPressed: () => favorites.toggle(ex, work),
              );
            },
          ),
          IconButton(
            tooltip: 'Voir dans la galerie 3D',
            icon: const Icon(Icons.view_in_ar),
            onPressed: () => Navigator.of(context).push(
              MaterialPageRoute(builder: (_) => GalleryScreen(ex, work: work)),
            ),
          ),
          IconButton(
            tooltip: 'Œuvre précédente',
            icon: const Icon(Icons.chevron_left),
            onPressed: index > 0 ? () => _go(context, -1) : null,
          ),
          IconButton(
            tooltip: 'Œuvre suivante',
            icon: const Icon(Icons.chevron_right),
            onPressed: index < ex.works.length - 1
                ? () => _go(context, 1)
                : null,
          ),
        ],
      ),
      body: wide
          ? Row(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Expanded(
                  flex: 11,
                  child: Container(
                    color: const Color(0xffeceae4),
                    padding: const EdgeInsets.all(32),
                    child: image,
                  ),
                ),
                Expanded(
                  flex: 9,
                  child: SingleChildScrollView(
                    padding: const EdgeInsets.fromLTRB(32, 24, 40, 48),
                    child: text,
                  ),
                ),
              ],
            )
          : SingleChildScrollView(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Container(
                    color: const Color(0xffeceae4),
                    height: size.height * .5,
                    padding: const EdgeInsets.all(20),
                    child: image,
                  ),
                  Padding(
                    padding: pagePadding(context).copyWith(top: 24, bottom: 48),
                    child: text,
                  ),
                ],
              ),
            ),
    );
  }
}

/// L'image de l'œuvre, dans un cadre sombre ; un toucher l'ouvre en grand.
class _Picture extends StatelessWidget {
  const _Picture(this.ex, this.work);

  final Exhibition ex;
  final Work work;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: GestureDetector(
        onTap: () => Navigator.of(context).push(
          MaterialPageRoute(
            fullscreenDialog: true,
            builder: (_) => ZoomScreen(path: work.image, title: work.title),
          ),
        ),
        child: Hero(
          tag: 'work-${ex.slug}-${work.slug}',
          child: AspectRatio(
            aspectRatio: work.aspect,
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: const Color(0xff2a2420),
                boxShadow: const [
                  BoxShadow(
                    blurRadius: 30,
                    offset: Offset(0, 18),
                    color: Colors.black38,
                  ),
                ],
                border: Border.all(color: const Color(0xff2a2420), width: 8),
              ),
              child: ArtImage(
                work.image,
                fit: BoxFit.contain,
                semanticLabel: work.title,
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _Text extends StatelessWidget {
  const _Text(this.ex, this.work);

  final Exhibition ex;
  final Work work;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final n = work.notice;
    final artist = ex.artistNamed(work.artist);
    final body = theme.textTheme.bodyLarge?.copyWith(height: 1.55);
    Widget heading(String text) => Padding(
      padding: const EdgeInsets.only(top: 28, bottom: 8),
      child: Text(
        text,
        style: theme.textTheme.titleMedium?.copyWith(
          fontWeight: FontWeight.w700,
          color: ex.accent,
        ),
      ),
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          work.title,
          style: theme.textTheme.headlineSmall?.copyWith(
            fontWeight: FontWeight.w800,
          ),
        ),
        const SizedBox(height: 6),
        Text(
          [work.artist, work.year].where((s) => s.isNotEmpty).join(' · '),
          style: theme.textTheme.titleMedium,
        ),
        const SizedBox(height: 14),
        _Facts([
          if (work.technique.isNotEmpty) ('Technique', work.technique),
          if (work.collection.isNotEmpty) ('Collection', work.collection),
          if (work.width > 0 && work.kind == WorkKind.painting)
            (
              'Accrochage',
              '${work.width.toStringAsFixed(2)} × ${work.height.toStringAsFixed(2)} m dans la galerie',
            ),
          if (work.room < ex.rooms.length) ('Salle', ex.rooms[work.room].name),
        ]),
        if (work.note.isNotEmpty) ...[
          const SizedBox(height: 18),
          Text(work.note, style: body?.copyWith(fontStyle: FontStyle.italic)),
        ],
        for (final (title, text) in n.sections) ...[
          heading(title),
          Text(text, style: body),
        ],
        if (n.details.isNotEmpty) ...[
          heading('À regarder de près'),
          for (final d in n.details)
            Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Padding(
                    padding: const EdgeInsets.only(top: 9, right: 10),
                    child: Icon(Icons.circle, size: 7, color: ex.accent),
                  ),
                  Expanded(child: Text(d, style: body)),
                ],
              ),
            ),
        ],
        if (n.themes.isNotEmpty) ...[
          heading('Thèmes'),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [for (final t in n.themes) Chip(label: Text(t))],
          ),
        ],
        if (n.anecdote.isNotEmpty) ...[
          heading('Le saviez-vous ?'),
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: ex.accent.withValues(alpha: .08),
              borderRadius: BorderRadius.circular(14),
            ),
            child: Text(n.anecdote, style: body),
          ),
        ],
        if (artist != null) ...[heading('L\'artiste'), ArtistTile(artist)],
        if (work.credit case final credit? when credit.author.isNotEmpty) ...[
          heading('Reproduction'),
          Text(
            '${credit.author} · ${credit.licence}',
            style: theme.textTheme.bodyMedium,
          ),
          if (credit.source.startsWith('https://'))
            TextButton.icon(
              style: TextButton.styleFrom(padding: EdgeInsets.zero),
              onPressed: () => launchUrl(Uri.parse(credit.source)),
              icon: const Icon(Icons.open_in_new, size: 18),
              label: const Text('Source'),
            ),
        ],
      ],
    );
  }
}

class _Facts extends StatelessWidget {
  const _Facts(this.facts);

  final List<(String, String)> facts;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Column(
      children: [
        for (final (label, value) in facts)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 3),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                SizedBox(
                  width: 110,
                  child: Text(
                    label,
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
                ),
                Expanded(child: Text(value, style: theme.textTheme.bodyMedium)),
              ],
            ),
          ),
      ],
    );
  }
}
