/// ARTEON sur iPad (et iPhone) : le catalogue du musée virtuel en natif — expositions,
/// œuvres et notices, recherche, favoris, images en grand — et sa galerie 3D.
library;

import 'package:flutter/material.dart';

import 'app_scope.dart';
import 'repository.dart';
import 'screens/favorites.dart';
import 'screens/home.dart';
import 'screens/search.dart';
import 'theme.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(ArteonApp(repository: CatalogueRepository()));
}

class ArteonApp extends StatefulWidget {
  const ArteonApp({super.key, required this.repository, this.favorites});

  final CatalogueRepository repository;

  /// Les favoris (ouverts depuis l'appareil si absents).
  final Favorites? favorites;

  @override
  State<ArteonApp> createState() => _ArteonAppState();
}

/// Charge le catalogue et les favoris, puis ouvre le musée. Le catalogue est posé
/// au-dessus du navigateur : toutes les pages ouvertes ensuite le voient.
class _ArteonAppState extends State<ArteonApp> {
  late Future<(Loaded, Favorites)> _loading = _load();

  Future<(Loaded, Favorites)> _load() async {
    final favorites = widget.favorites ?? await Favorites.open();
    return (await widget.repository.load(), favorites);
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder(
      future: _loading,
      builder: (context, snapshot) {
        final Widget home;
        TransitionBuilder? builder;
        if (snapshot.hasData) {
          final (loaded, favorites) = snapshot.data!;
          home = const Shell();
          builder = (context, child) => AppScope(
            catalogue: loaded.catalogue,
            favorites: favorites,
            offline: loaded.offline,
            child: child!,
          );
        } else if (snapshot.hasError) {
          home = _Message(
            icon: Icons.wifi_off,
            text:
                'Le musée n\'a pas pu ouvrir : il faut Internet la première fois.\n'
                '${snapshot.error}',
            retry: () => setState(() => _loading = _load()),
          );
        } else {
          home = const _Message(
            icon: Icons.museum_outlined,
            text: 'Ouverture du musée…',
          );
        }
        return MaterialApp(
          // Une nouvelle application quand le catalogue arrive : on repart de l'accueil.
          key: ValueKey(snapshot.hasData),
          title: 'ARTEON',
          debugShowCheckedModeBanner: false,
          theme: arteonTheme(Brightness.light),
          darkTheme: arteonTheme(Brightness.dark),
          builder: builder,
          home: home,
        );
      },
    );
  }
}

class _Message extends StatelessWidget {
  const _Message({required this.icon, required this.text, this.retry});

  final IconData icon;
  final String text;
  final VoidCallback? retry;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: arteonNight,
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, color: arteonSun, size: 56),
              const SizedBox(height: 20),
              const Text(
                'ARTEON',
                style: TextStyle(
                  color: Colors.white,
                  fontSize: 32,
                  letterSpacing: 8,
                  fontWeight: FontWeight.w800,
                ),
              ),
              const SizedBox(height: 16),
              Text(
                text,
                textAlign: TextAlign.center,
                style: const TextStyle(color: Colors.white70, fontSize: 16),
              ),
              if (retry != null) ...[
                const SizedBox(height: 20),
                FilledButton(onPressed: retry, child: const Text('Réessayer')),
              ],
              if (retry == null) ...[
                const SizedBox(height: 24),
                const CircularProgressIndicator(color: arteonSun),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

/// Les trois onglets : une barre latérale sur iPad, en bas sur iPhone.
class Shell extends StatefulWidget {
  const Shell({super.key});

  @override
  State<Shell> createState() => _ShellState();
}

class _ShellState extends State<Shell> {
  var _tab = 0;

  static const _destinations = [
    (Icons.museum_outlined, Icons.museum, 'Expositions'),
    (Icons.search, Icons.search, 'Rechercher'),
    (Icons.favorite_border, Icons.favorite, 'Favoris'),
  ];

  @override
  Widget build(BuildContext context) {
    final pages = [
      const HomeScreen(),
      const SearchScreen(),
      const FavoritesScreen(),
    ];
    final body = IndexedStack(index: _tab, children: pages);
    if (MediaQuery.sizeOf(context).width >= 700) {
      return Scaffold(
        body: SafeArea(
          child: Row(
            children: [
              NavigationRail(
                selectedIndex: _tab,
                onDestinationSelected: (i) => setState(() => _tab = i),
                labelType: NavigationRailLabelType.all,
                leading: const Padding(
                  padding: EdgeInsets.symmetric(vertical: 16),
                  child: _Logo(),
                ),
                destinations: [
                  for (final (icon, selected, label) in _destinations)
                    NavigationRailDestination(
                      icon: Icon(icon),
                      selectedIcon: Icon(selected),
                      label: Text(label),
                    ),
                ],
              ),
              const VerticalDivider(width: 1),
              Expanded(child: body),
            ],
          ),
        ),
      );
    }
    return Scaffold(
      body: SafeArea(bottom: false, child: body),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _tab,
        onDestinationSelected: (i) => setState(() => _tab = i),
        destinations: [
          for (final (icon, selected, label) in _destinations)
            NavigationDestination(
              icon: Icon(icon),
              selectedIcon: Icon(selected),
              label: label,
            ),
        ],
      ),
    );
  }
}

/// Le logo du site : un soleil et une étoile sur un ciel bleu nuit.
class _Logo extends StatelessWidget {
  const _Logo();

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 44,
      height: 44,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(12),
        gradient: const RadialGradient(
          center: Alignment(.4, -.4),
          radius: 1.1,
          colors: [Color(0xff3d63c9), arteonBlue, arteonNight],
        ),
      ),
      child: Stack(
        children: [
          Positioned(
            left: 8,
            top: 8,
            child: Container(
              width: 20,
              height: 20,
              decoration: const BoxDecoration(
                color: arteonSun,
                shape: BoxShape.circle,
              ),
            ),
          ),
          Positioned(
            right: 7,
            bottom: 7,
            child: Container(
              width: 8,
              height: 8,
              decoration: const BoxDecoration(
                color: Color(0xfffff6c9),
                shape: BoxShape.circle,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
