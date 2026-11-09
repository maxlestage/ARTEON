/// D'où vient le catalogue : du site d'ARTEON, gardé sur l'appareil pour le relire hors
/// ligne ; et les favoris, gardés dans les préférences.
library;

import 'dart:convert';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'models.dart';

/// L'adresse du site (GitHub Pages par défaut ; une autre au build :
/// `flutter build ios --dart-define=ARTEON_URL=https://mon-app.herokuapp.com/`).
const siteUrl = String.fromEnvironment(
  'ARTEON_URL',
  defaultValue: 'https://maxlestage.github.io/ARTEON/',
);

/// L'adresse complète d'un chemin du site (`tableaux/van-gogh/iris.jpg`).
Uri siteUri(String path) => Uri.parse(siteUrl).resolve(path);

/// Le catalogue et d'où il vient.
class Loaded {
  Loaded(this.catalogue, {required this.offline});

  final Catalogue catalogue;

  /// Lu sur l'appareil, le site n'ayant pas répondu.
  final bool offline;
}

/// Charge le catalogue : du site, sinon de la copie gardée sur l'appareil.
class CatalogueRepository {
  CatalogueRepository({http.Client? client, Future<File> Function()? cacheFile})
    : _client = client ?? http.Client(),
      _cacheFile = cacheFile ?? _defaultCacheFile;

  final http.Client _client;
  final Future<File> Function() _cacheFile;

  static Future<File> _defaultCacheFile() async {
    final dir = await getApplicationSupportDirectory();
    return File('${dir.path}/catalogue.json');
  }

  Future<Loaded> load() async {
    try {
      final res = await _client
          .get(siteUri('catalogue.json'))
          .timeout(const Duration(seconds: 20));
      if (res.statusCode != 200) {
        throw HttpException('HTTP ${res.statusCode}');
      }
      final text = utf8.decode(res.bodyBytes);
      final catalogue = await compute(_parse, text);
      try {
        await (await _cacheFile()).writeAsString(text);
      } catch (_) {
        // Pas de copie hors ligne cette fois : le catalogue est quand même là.
      }
      return Loaded(catalogue, offline: false);
    } catch (error) {
      final file = await _cacheFile();
      if (await file.exists()) {
        return Loaded(
          await compute(_parse, await file.readAsString()),
          offline: true,
        );
      }
      rethrow;
    }
  }
}

Catalogue _parse(String text) =>
    Catalogue.fromJson(jsonDecode(text) as Map<String, dynamic>);

/// Les œuvres mises de côté (« exposition/œuvre »).
class Favorites extends ChangeNotifier {
  Favorites(this._prefs) : _keys = {...?_prefs?.getStringList(_key)};

  static const _key = 'favoris';
  final SharedPreferences? _prefs;
  final Set<String> _keys;

  static Future<Favorites> open() async {
    try {
      return Favorites(await SharedPreferences.getInstance());
    } catch (_) {
      return Favorites(null);
    }
  }

  static String keyOf(Exhibition ex, Work w) => '${ex.slug}/${w.slug}';

  bool contains(Exhibition ex, Work w) => _keys.contains(keyOf(ex, w));

  int get length => _keys.length;

  void toggle(Exhibition ex, Work w) {
    final key = keyOf(ex, w);
    if (!_keys.remove(key)) _keys.add(key);
    _prefs?.setStringList(_key, _keys.toList());
    notifyListeners();
  }

  /// Les favoris encore au catalogue, dans l'ordre du musée.
  List<(Exhibition, Work)> inCatalogue(Catalogue c) => [
    for (final (ex, w) in c.allWorks)
      if (contains(ex, w)) (ex, w),
  ];
}
