import 'package:flutter/material.dart';

import '../app_scope.dart';
import '../models.dart';
import '../theme.dart';
import '../widgets/art_image.dart';
import 'exhibition.dart';

/// L'accueil : toutes les expositions, filtrées par style.
class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

/// Le filtre « en ce moment dans les musées du monde ».
const _now = '\u0000maintenant';

class _HomeScreenState extends State<HomeScreen> {
  String? _filter;

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final c = scope.catalogue;
    final theme = Theme.of(context);
    final shown = c.exhibitions.where((e) {
      if (_filter == null) return true;
      if (_filter == _now) return e.ephemeral;
      return e.movement == _filter;
    }).toList();
    final hasEphemeral = c.exhibitions.any((e) => e.ephemeral);
    final padding = pagePadding(context);
    return CustomScrollView(
      slivers: [
        SliverAppBar.large(
          title: const Text('ARTEON'),
          actions: [
            if (scope.offline)
              const Padding(
                padding: EdgeInsets.only(right: 16),
                child: Tooltip(
                  message: 'Hors ligne : le catalogue gardé sur l\'appareil',
                  child: Icon(Icons.cloud_off_outlined),
                ),
              ),
          ],
        ),
        SliverPadding(
          padding: padding,
          sliver: SliverToBoxAdapter(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Toute l\'histoire de l\'art, en 3D',
                  style: theme.textTheme.titleLarge?.copyWith(
                    fontStyle: FontStyle.italic,
                    color: theme.colorScheme.primary,
                  ),
                ),
                const SizedBox(height: 16),
                Wrap(
                  spacing: 12,
                  runSpacing: 12,
                  children: [
                    _Stat('${c.exhibitions.length}', 'expositions', arteonBlue),
                    _Stat('${c.workCount}', 'œuvres', arteonSun),
                    _Stat(
                      '${c.exhibitions.fold(0, (n, e) => n + e.rooms.length)}',
                      'salles en 3D',
                      const Color(0xff3d7a5a),
                    ),
                  ],
                ),
                const SizedBox(height: 24),
                SizedBox(
                  height: 40,
                  child: ListView(
                    scrollDirection: Axis.horizontal,
                    children: [
                      _chip('Tout', null),
                      if (hasEphemeral) _chip('En ce moment', _now),
                      for (final m in c.movements) _chip(m, m),
                    ],
                  ),
                ),
                const SizedBox(height: 16),
              ],
            ),
          ),
        ),
        SliverPadding(
          padding: padding.copyWith(bottom: 32),
          sliver: SliverGrid.builder(
            gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
              maxCrossAxisExtent: 380,
              mainAxisSpacing: 16,
              crossAxisSpacing: 16,
              childAspectRatio: .82,
            ),
            itemCount: shown.length,
            itemBuilder: (context, i) => ExhibitionCard(shown[i]),
          ),
        ),
      ],
    );
  }

  Widget _chip(String label, String? value) => Padding(
    padding: const EdgeInsets.only(right: 8),
    child: ChoiceChip(
      label: Text(label),
      selected: _filter == value,
      onSelected: (_) => setState(() => _filter = value),
    ),
  );
}

class _Stat extends StatelessWidget {
  const _Stat(this.value, this.label, this.color);

  final String value, label;
  final Color color;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Container(
      width: 150,
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
      decoration: BoxDecoration(
        color: theme.colorScheme.surface,
        borderRadius: BorderRadius.circular(14),
        border: Border(left: BorderSide(color: color, width: 5)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            value,
            style: theme.textTheme.headlineMedium?.copyWith(
              fontWeight: FontWeight.w800,
            ),
          ),
          Text(label, style: theme.textTheme.bodyMedium),
        ],
      ),
    );
  }
}

/// La carte d'une exposition : sa couverture, son nom, son style.
class ExhibitionCard extends StatelessWidget {
  const ExhibitionCard(this.ex, {super.key});

  final Exhibition ex;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      child: InkWell(
        onTap: () =>
            Navigator.of(context)
                .push(MaterialPageRoute(builder: (_) => ExhibitionScreen(ex))),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Expanded(
              child: Hero(
                tag: 'cover-${ex.slug}',
                child: ArtImage(
                  ex.coverImage,
                  tint: ex.accent,
                  semanticLabel: ex.title,
                ),
              ),
            ),
            Container(height: 5, color: ex.accent),
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 12, 16, 16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (ex.ephemeral)
                    Text(
                      'EN CE MOMENT',
                      style: theme.textTheme.labelSmall?.copyWith(
                        color: ex.accent,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                  Text(
                    ex.shortName.isEmpty ? ex.title : ex.shortName,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: theme.textTheme.titleMedium?.copyWith(
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    [
                      if (ex.movement.isNotEmpty) ex.movement,
                      '${ex.works.length} œuvres',
                    ].join(' · '),
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
