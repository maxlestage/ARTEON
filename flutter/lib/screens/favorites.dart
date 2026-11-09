import 'package:flutter/material.dart';

import '../app_scope.dart';
import '../theme.dart';
import 'exhibition.dart';

/// Les œuvres mises de côté avec le cœur.
class FavoritesScreen extends StatelessWidget {
  const FavoritesScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final padding = pagePadding(context);
    return ListenableBuilder(
      listenable: scope.favorites,
      builder: (context, _) {
        final list = scope.favorites.inCatalogue(scope.catalogue);
        return CustomScrollView(
          slivers: [
            const SliverAppBar.large(title: Text('Favoris')),
            if (list.isEmpty)
              SliverFillRemaining(
                hasScrollBody: false,
                child: Center(
                  child: Padding(
                    padding: const EdgeInsets.all(32),
                    child: Text(
                      'Touchez le cœur d\'une œuvre pour la retrouver ici.',
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                  ),
                ),
              )
            else
              SliverPadding(
                padding: padding.copyWith(bottom: 32),
                sliver: SliverGrid.builder(
                  gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                    maxCrossAxisExtent: 260,
                    mainAxisSpacing: 14,
                    crossAxisSpacing: 14,
                    childAspectRatio: .72,
                  ),
                  itemCount: list.length,
                  itemBuilder: (context, i) =>
                      WorkTile(list[i].$1, list[i].$2, showExhibition: true),
                ),
              ),
          ],
        );
      },
    );
  }
}
