import 'dart:io';

import 'package:arteon/main.dart';
import 'package:arteon/repository.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  final json = File('test/fixtures/catalogue_extrait.json').readAsBytesSync();

  Future<void> open(
    WidgetTester tester,
    Size size, {
    bool online = true,
  }) async {
    tester.view.physicalSize = size;
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final cache = File(
      '${Directory.systemTemp.createTempSync().path}/catalogue.json',
    );
    final client = MockClient((request) async {
      expect(request.url.path, endsWith('catalogue.json'));
      return online ? http.Response.bytes(json, 200) : http.Response('', 503);
    });
    await tester.pumpWidget(
      ArteonApp(
        repository: CatalogueRepository(
          client: client,
          cacheFile: () async => cache,
        ),
        favorites: Favorites(null),
      ),
    );
    // Le catalogue se lit dans un isolat : laisser le temps réel passer.
    for (var i = 0; i < 50; i++) {
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 100)),
      );
      await tester.pump();
      if (find.byType(Shell).evaluate().isNotEmpty ||
          find.text('Réessayer').evaluate().isNotEmpty) {
        break;
      }
    }
    await tester.pump();
  }

  testWidgets('sur iPad : barre latérale, expositions, puis une œuvre', (
    tester,
  ) async {
    await open(tester, const Size(1366, 1024));
    expect(find.byType(NavigationRail), findsOneWidget);
    expect(find.text('ARTEON'), findsWidgets);
    expect(find.text('Van Gogh'), findsWidgets);

    await tester.tap(find.text('Van Gogh').first);
    await tester.pumpAndSettle();
    expect(find.text('Visiter en 3D'), findsOneWidget);

    await tester.scrollUntilVisible(
      find.text('Les Tournesols'),
      300,
      scrollable: find.byType(Scrollable).first,
    );
    // Au milieu de l'écran, pas sous la barre du haut.
    await tester.runAsync(
      () => Scrollable.ensureVisible(
        tester.element(find.text('Les Tournesols')),
        alignment: .5,
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Les Tournesols'));
    await tester.pumpAndSettle();
    // Sur iPad à l'horizontale, l'image et la notice côte à côte.
    expect(find.text('Ce que l\'on voit'), findsOneWidget);
    await tester.tap(find.byTooltip('Ajouter aux favoris'));
    await tester.pump();
    expect(find.byTooltip('Retirer des favoris'), findsOneWidget);
  });

  testWidgets('sur iPhone : barre en bas, recherche', (tester) async {
    await open(tester, const Size(390, 844));
    expect(find.byType(NavigationBar), findsOneWidget);
    await tester.tap(find.text('Rechercher'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'tournesol');
    await tester.pump();
    expect(find.text('Les Tournesols'), findsWidgets);
  });

  testWidgets('sans Internet ni copie : un message et Réessayer', (
    tester,
  ) async {
    await open(tester, const Size(1024, 1366), online: false);
    expect(find.text('Réessayer'), findsOneWidget);
  });
}
