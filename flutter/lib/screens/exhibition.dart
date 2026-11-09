import 'package:flutter/material.dart';

import '../models.dart';
import '../theme.dart';
import '../widgets/art_image.dart';
import 'gallery.dart';
import 'work.dart';

/// Une exposition : sa présentation, son bâtiment, ses salles et leurs œuvres, ses artistes.
class ExhibitionScreen extends StatelessWidget {
  const ExhibitionScreen(this.ex, {super.key});

  final Exhibition ex;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final padding = pagePadding(context);
    final wide = MediaQuery.sizeOf(context).width >= 900;
    return Scaffold(
      body: CustomScrollView(
        slivers: [
          SliverAppBar(
            pinned: true,
            expandedHeight: wide ? 380 : 280,
            backgroundColor: ex.accent,
            foregroundColor: Colors.white,
            flexibleSpace: FlexibleSpaceBar(
              title: Text(
                ex.shortName.isEmpty ? ex.title : ex.shortName,
                style: const TextStyle(
                  color: Colors.white,
                  shadows: [Shadow(blurRadius: 8, color: Colors.black54)],
                ),
              ),
              background: Stack(
                fit: StackFit.expand,
                children: [
                  Hero(
                    tag: 'cover-${ex.slug}',
                    child: ArtImage(ex.coverImage, tint: ex.accent),
                  ),
                  const DecoratedBox(
                    decoration: BoxDecoration(
                      gradient: LinearGradient(
                        begin: Alignment.topCenter,
                        end: Alignment.bottomCenter,
                        colors: [Colors.transparent, Colors.black54],
                        stops: [.45, 1],
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
          SliverPadding(
            padding: padding.copyWith(top: 24),
            sliver: SliverToBoxAdapter(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (ex.kicker.isNotEmpty)
                    Text(
                      ex.kicker.toUpperCase(),
                      style: theme.textTheme.labelLarge?.copyWith(
                        color: ex.accent,
                        letterSpacing: 1.5,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                  const SizedBox(height: 6),
                  Text(
                    ex.title,
                    style: theme.textTheme.headlineMedium?.copyWith(
                      fontWeight: FontWeight.w800,
                    ),
                  ),
                  if (ex.movement.isNotEmpty || ex.artistDates.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.only(top: 4),
                      child: Text(
                        [
                          ex.movement,
                          ex.artistDates,
                        ].where((s) => s.isNotEmpty).join(' · '),
                        style: theme.textTheme.titleMedium,
                      ),
                    ),
                  const SizedBox(height: 16),
                  Wrap(
                    spacing: 12,
                    runSpacing: 12,
                    children: [
                      FilledButton.icon(
                        style: FilledButton.styleFrom(
                          backgroundColor: ex.accent,
                        ),
                        onPressed: () => Navigator.of(context).push(
                          MaterialPageRoute(builder: (_) => GalleryScreen(ex)),
                        ),
                        icon: const Icon(Icons.view_in_ar),
                        label: const Text('Visiter en 3D'),
                      ),
                    ],
                  ),
                  const SizedBox(height: 20),
                  ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 760),
                    child: Text(
                      ex.introduction,
                      style: theme.textTheme.bodyLarge?.copyWith(height: 1.5),
                    ),
                  ),
                  if (ex.museum != null) _MuseumSection(ex.museum!, ex.accent),
                ],
              ),
            ),
          ),
          for (final (i, (room, works)) in ex.worksByRoom.indexed)
            if (works.isNotEmpty) ...[
              SliverPadding(
                padding: padding.copyWith(top: 32, bottom: 12),
                sliver: SliverToBoxAdapter(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        room.name.isEmpty ? 'Salle ${i + 1}' : room.name,
                        style: theme.textTheme.titleLarge?.copyWith(
                          fontWeight: FontWeight.w700,
                        ),
                      ),
                      if (room.theme.isNotEmpty)
                        Padding(
                          padding: const EdgeInsets.only(top: 4),
                          child: Text(
                            room.theme,
                            style: theme.textTheme.bodyMedium,
                          ),
                        ),
                    ],
                  ),
                ),
              ),
              SliverPadding(
                padding: padding,
                sliver: SliverGrid.builder(
                  gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                    maxCrossAxisExtent: 260,
                    mainAxisSpacing: 14,
                    crossAxisSpacing: 14,
                    childAspectRatio: .72,
                  ),
                  itemCount: works.length,
                  itemBuilder: (context, j) => WorkTile(ex, works[j]),
                ),
              ),
            ],
          if (ex.artists.isNotEmpty) ...[
            SliverPadding(
              padding: padding.copyWith(top: 36, bottom: 12),
              sliver: SliverToBoxAdapter(
                child: Text(
                  ex.artists.length == 1 ? 'L\'artiste' : 'Les artistes',
                  style: theme.textTheme.titleLarge?.copyWith(
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ),
            ),
            SliverPadding(
              padding: padding,
              sliver: SliverList.separated(
                itemCount: ex.artists.length,
                separatorBuilder: (_, _) => const Divider(height: 1),
                itemBuilder: (context, i) => ArtistTile(ex.artists[i]),
              ),
            ),
          ],
          const SliverToBoxAdapter(child: SizedBox(height: 48)),
        ],
      ),
    );
  }
}

class _MuseumSection extends StatelessWidget {
  const _MuseumSection(this.museum, this.accent);

  final Museum museum;
  final Color accent;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.only(top: 28),
      child: Container(
        constraints: const BoxConstraints(maxWidth: 760),
        padding: const EdgeInsets.all(20),
        decoration: BoxDecoration(
          color: theme.colorScheme.surface,
          borderRadius: BorderRadius.circular(18),
          border: Border(left: BorderSide(color: accent, width: 5)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.account_balance, color: accent),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(
                    'Le bâtiment',
                    style: theme.textTheme.titleMedium?.copyWith(
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              [museum.name, museum.city].where((s) => s.isNotEmpty).join(', '),
              style: theme.textTheme.titleSmall,
            ),
            if (museum.architect.isNotEmpty || museum.built.isNotEmpty)
              Text(
                [
                  museum.architect,
                  museum.built,
                ].where((s) => s.isNotEmpty).join(' · '),
                style: theme.textTheme.bodySmall,
              ),
            for (final text in [museum.presentation, museum.here])
              if (text.isNotEmpty)
                Padding(
                  padding: const EdgeInsets.only(top: 10),
                  child: Text(
                    text,
                    style: theme.textTheme.bodyMedium?.copyWith(height: 1.5),
                  ),
                ),
          ],
        ),
      ),
    );
  }
}

/// Un artiste : ses dates, son style, sa biographie à déplier.
class ArtistTile extends StatelessWidget {
  const ArtistTile(this.artist, {super.key});

  final Artist artist;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return ExpansionTile(
      tilePadding: EdgeInsets.zero,
      title: Text(
        artist.name,
        style: const TextStyle(fontWeight: FontWeight.w600),
      ),
      subtitle: Text(
        [artist.life, artist.style].where((s) => s.isNotEmpty).join(' · '),
      ),
      childrenPadding: const EdgeInsets.only(bottom: 16),
      expandedAlignment: Alignment.topLeft,
      children: [
        Text(
          artist.biography,
          style: theme.textTheme.bodyMedium?.copyWith(height: 1.5),
        ),
      ],
    );
  }
}

/// La vignette d'une œuvre : son image, son titre, son artiste.
class WorkTile extends StatelessWidget {
  const WorkTile(this.ex, this.work, {super.key, this.showExhibition = false});

  final Exhibition ex;
  final Work work;
  final bool showExhibition;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      child: InkWell(
        onTap: () =>
            Navigator.of(context)
                .push(MaterialPageRoute(builder: (_) => WorkScreen(ex, work))),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Expanded(
              child: Hero(
                tag: 'work-${ex.slug}-${work.slug}',
                child: ArtImage(work.image, semanticLabel: work.title),
              ),
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 10, 12, 12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    work.title,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: theme.textTheme.titleSmall?.copyWith(
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                  const SizedBox(height: 2),
                  Text(
                    showExhibition
                        ? '${work.artist} · ${ex.shortName}'
                        : [
                            work.artist,
                            work.year,
                          ].where((s) => s.isNotEmpty).join(' · '),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: theme.textTheme.bodySmall,
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
