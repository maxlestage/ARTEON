import 'package:flutter/material.dart';

import '../app_scope.dart';
import '../theme.dart';
import 'exhibition.dart';

/// Chercher une œuvre dans tout le musée : titre, artiste, technique, collection, année.
class SearchScreen extends StatefulWidget {
  const SearchScreen({super.key});

  @override
  State<SearchScreen> createState() => _SearchScreenState();
}

class _SearchScreenState extends State<SearchScreen> {
  var _query = '';

  @override
  Widget build(BuildContext context) {
    final c = AppScope.of(context).catalogue;
    final results = c.search(_query);
    final padding = pagePadding(context);
    return CustomScrollView(
      slivers: [
        const SliverAppBar.large(title: Text('Rechercher')),
        SliverPadding(
          padding: padding.copyWith(bottom: 16),
          sliver: SliverToBoxAdapter(
            child: SearchBar(
              hintText: 'Monet, estampe, Louvre, 1889…',
              leading: const Icon(Icons.search),
              onChanged: (q) => setState(() => _query = q),
            ),
          ),
        ),
        if (_query.trim().isNotEmpty)
          SliverPadding(
            padding: padding.copyWith(bottom: 12),
            sliver: SliverToBoxAdapter(
              child: Text(
                results.isEmpty
                    ? 'Aucune œuvre ne correspond.'
                    : '${results.length} œuvre${results.length > 1 ? 's' : ''}',
              ),
            ),
          ),
        SliverPadding(
          padding: padding.copyWith(bottom: 32),
          sliver: SliverGrid.builder(
            gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
              maxCrossAxisExtent: 260,
              mainAxisSpacing: 14,
              crossAxisSpacing: 14,
              childAspectRatio: .72,
            ),
            itemCount: results.length,
            itemBuilder: (context, i) =>
                WorkTile(results[i].$1, results[i].$2, showExhibition: true),
          ),
        ),
      ],
    );
  }
}
