import 'dart:convert';
import 'dart:io';

import 'package:arteon/models.dart';
import 'package:flutter_test/flutter_test.dart';

Catalogue fixture() => Catalogue.fromJson(
  jsonDecode(File('test/fixtures/catalogue_extrait.json').readAsStringSync())
      as Map<String, dynamic>,
);

void main() {
  test('le catalogue du site se lit en entier', () {
    final c = fixture();
    expect(
      c.exhibitions.map((e) => e.slug),
      containsAll(['van-gogh', 'monet']),
    );
    final vanGogh = c.exhibition('van-gogh')!;
    expect(vanGogh.works, isNotEmpty);
    expect(vanGogh.rooms, isNotEmpty);
    expect(vanGogh.coverImage, startsWith('tableaux/'));
    final tournesols = vanGogh.works.firstWhere((w) => w.slug == 'tournesols');
    expect(tournesols.artist, 'Vincent van Gogh');
    expect(tournesols.notice.sections, isNotEmpty);
    expect(tournesols.aspect, greaterThan(0.5));
    // Chaque œuvre a sa salle, et chaque salle se retrouve.
    for (final ex in c.exhibitions) {
      final grouped = ex.worksByRoom.fold(0, (n, r) => n + r.$2.length);
      expect(grouped, ex.works.length);
    }
    // Les sculptures sont des sculptures.
    expect(
      c.exhibition('monet')!.works.any((w) => w.kind == WorkKind.sculpture),
      isTrue,
    );
  });

  test('la recherche ignore les accents et les majuscules', () {
    final c = fixture();
    expect(fold('Éléphant Œuvre'), 'elephant oeuvre'.replaceAll('oe', 'o'));
    final r = c.search('TOURNESOL');
    expect(r.map((x) => x.$2.slug), contains('tournesols'));
    expect(c.search('   '), isEmpty);
  });

  test('un catalogue trop récent est refusé', () {
    expect(
      () => Catalogue.fromJson({
        'version': catalogueVersion + 1,
        'exhibitions': [],
      }),
      throwsFormatException,
    );
  });
}
