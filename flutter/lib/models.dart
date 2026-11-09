/// Le catalogue d'ARTEON, tel que le site le publie dans `catalogue.json` : les
/// expositions, leurs salles, leurs artistes et leurs œuvres avec leur notice.
library;

import 'dart:ui' show Color;

String _s(Object? v) => v is String ? v : '';
List<String> _strings(Object? v) =>
    v is List ? v.whereType<String>().toList() : const [];
List<Map<String, dynamic>> _maps(Object? v) =>
    v is List ? v.whereType<Map<String, dynamic>>().toList() : const [];
double _d(Object? v) => v is num ? v.toDouble() : 0;

/// La version du format que l'application sait lire.
const catalogueVersion = 1;

class Catalogue {
  Catalogue({required this.exhibitions});

  final List<Exhibition> exhibitions;

  factory Catalogue.fromJson(Map<String, dynamic> json) {
    final version = json['version'];
    if (version is! int || version > catalogueVersion) {
      throw FormatException(
        'Catalogue de version $version : mettez l\'application à jour.',
      );
    }
    return Catalogue(
      exhibitions: _maps(json['exhibitions']).map(Exhibition.fromJson).toList(),
    );
  }

  /// Toutes les œuvres, avec leur exposition.
  Iterable<(Exhibition, Work)> get allWorks sync* {
    for (final ex in exhibitions) {
      for (final w in ex.works) {
        yield (ex, w);
      }
    }
  }

  int get workCount => exhibitions.fold(0, (n, e) => n + e.works.length);

  /// Les mouvements, dans l'ordre des expositions, sans doublon.
  List<String> get movements {
    final seen = <String>{};
    return [
      for (final e in exhibitions)
        if (e.movement.isNotEmpty && seen.add(e.movement)) e.movement,
    ];
  }

  Exhibition? exhibition(String slug) {
    for (final e in exhibitions) {
      if (e.slug == slug) return e;
    }
    return null;
  }

  /// Les œuvres dont le titre, l'artiste, la technique ou la collection contient `query`
  /// (sans tenir compte des accents ni des majuscules).
  List<(Exhibition, Work)> search(String query) {
    final q = fold(query.trim());
    if (q.isEmpty) return const [];
    return [
      for (final (ex, w) in allWorks)
        if (fold(
          '${w.title} ${w.artist} ${w.technique} ${w.collection} ${w.year}',
        ).contains(q))
          (ex, w),
    ];
  }
}

/// Le texte sans accents, en minuscules, pour chercher.
String fold(String s) {
  const from = 'àâäáãåçéèêëíìîïñóòôöõúùûüýÿœæ';
  const to = 'aaaaaaceeeeiiiinooooouuuuyyoa';
  final out = StringBuffer();
  for (final ch in s.toLowerCase().split('')) {
    final i = from.indexOf(ch);
    out.write(i < 0 ? ch : to[i]);
  }
  return out.toString();
}

class Museum {
  Museum({
    required this.name,
    required this.city,
    required this.architect,
    required this.built,
    required this.presentation,
    required this.here,
  });

  final String name, city, architect, built, presentation, here;

  factory Museum.fromJson(Map<String, dynamic> j) => Museum(
    name: _s(j['name']),
    city: _s(j['city']),
    architect: _s(j['architect']),
    built: _s(j['built']),
    presentation: _s(j['presentation']),
    here: _s(j['here']),
  );
}

class Room {
  Room({required this.name, required this.theme, required this.corridor});

  final String name, theme;
  final bool corridor;

  factory Room.fromJson(Map<String, dynamic> j) => Room(
    name: _s(j['name']),
    theme: _s(j['theme']),
    corridor: j['corridor'] == true,
  );
}

class Artist {
  Artist({
    required this.name,
    required this.life,
    required this.biography,
    required this.style,
  });

  final String name, life, biography, style;

  factory Artist.fromJson(Map<String, dynamic> j) => Artist(
    name: _s(j['name']),
    life: _s(j['life']),
    biography: _s(j['biography']),
    style: _s(j['style']),
  );
}

class Notice {
  Notice({
    required this.description,
    required this.evocation,
    required this.story,
    required this.manner,
    required this.details,
    required this.themes,
    required this.anecdote,
  });

  final String description, evocation, story, manner, anecdote;
  final List<String> details, themes;

  factory Notice.fromJson(Map<String, dynamic>? j) {
    j ??= const {};
    return Notice(
      description: _s(j['description']),
      evocation: _s(j['evocation']),
      story: _s(j['story']),
      manner: _s(j['manner']),
      details: _strings(j['details']),
      themes: _strings(j['themes']),
      anecdote: _s(j['anecdote']),
    );
  }

  /// Les parties rédigées, dans l'ordre du site.
  List<(String, String)> get sections => [
    ('Ce que l\'on voit', description),
    ('Ce qu\'elle évoque', evocation),
    ('Son histoire', story),
    ('La manière', manner),
  ].where((s) => s.$2.isNotEmpty).toList();

  bool get isEmpty =>
      sections.isEmpty && details.isEmpty && themes.isEmpty && anecdote.isEmpty;
}

class Credit {
  Credit({required this.author, required this.licence, required this.source});

  final String author, licence, source;

  static Credit? fromJson(Object? j) => j is Map<String, dynamic>
      ? Credit(
          author: _s(j['author']),
          licence: _s(j['licence']),
          source: _s(j['source']),
        )
      : null;
}

enum WorkKind { painting, sculpture }

class Work {
  Work({
    required this.kind,
    required this.slug,
    required this.title,
    required this.artist,
    required this.year,
    required this.technique,
    required this.collection,
    required this.note,
    required this.image,
    required this.room,
    required this.width,
    required this.height,
    required this.credit,
    required this.notice,
  });

  final WorkKind kind;
  final String slug, title, artist, year, technique, collection, note, image;
  final int room;
  final double width, height;
  final Credit? credit;
  final Notice notice;

  /// Largeur / hauteur de l'œuvre (pour réserver la place de son image).
  double get aspect => width > 0 && height > 0 ? width / height : 0.8;

  factory Work.fromJson(Map<String, dynamic> j) => Work(
    kind: j['kind'] == 'sculpture' ? WorkKind.sculpture : WorkKind.painting,
    slug: _s(j['slug']),
    title: _s(j['title']),
    artist: _s(j['artist']),
    year: _s(j['year']),
    technique: _s(j['technique']),
    collection: _s(j['collection']),
    note: _s(j['note']),
    image: _s(j['image']),
    room: j['room'] is int ? j['room'] as int : 0,
    width: _d(j['width']),
    height: _d(j['height']),
    credit: Credit.fromJson(j['credit']),
    notice: Notice.fromJson(j['notice'] as Map<String, dynamic>?),
  );
}

class Exhibition {
  Exhibition({
    required this.slug,
    required this.title,
    required this.shortName,
    required this.kicker,
    required this.artist,
    required this.artistDates,
    required this.movement,
    required this.ephemeral,
    required this.introduction,
    required this.accent,
    required this.cover,
    required this.gallery,
    required this.museum,
    required this.rooms,
    required this.artists,
    required this.works,
  });

  final String slug, title, shortName, kicker, artist, artistDates, movement;
  final bool ephemeral;
  final String introduction, gallery;
  final Color accent;
  final String? cover;
  final Museum? museum;
  final List<Room> rooms;
  final List<Artist> artists;
  final List<Work> works;

  factory Exhibition.fromJson(Map<String, dynamic> j) {
    final hex = _s(j['accent']).replaceFirst('#', '');
    final rgb = int.tryParse(hex, radix: 16) ?? 0x1d3f8f;
    return Exhibition(
      slug: _s(j['slug']),
      title: _s(j['title']),
      shortName: _s(j['short_name']),
      kicker: _s(j['kicker']),
      artist: _s(j['artist']),
      artistDates: _s(j['artist_dates']),
      movement: _s(j['movement']),
      ephemeral: j['ephemeral'] == true,
      introduction: _s(j['introduction']),
      accent: Color(0xff000000 | rgb),
      cover: j['cover'] is String ? j['cover'] as String : null,
      gallery: _s(j['gallery']),
      museum: j['museum'] is Map<String, dynamic>
          ? Museum.fromJson(j['museum'] as Map<String, dynamic>)
          : null,
      rooms: _maps(j['rooms']).map(Room.fromJson).toList(),
      artists: _maps(j['artists']).map(Artist.fromJson).toList(),
      works: _maps(j['works']).map(Work.fromJson).toList(),
    );
  }

  /// Les œuvres de chaque salle, salles vides comprises, dans l'ordre.
  List<(Room, List<Work>)> get worksByRoom => [
    for (var i = 0; i < rooms.length; i++)
      (rooms[i], works.where((w) => w.room == i).toList()),
  ];

  Artist? artistNamed(String name) {
    for (final a in artists) {
      if (a.name == name) return a;
    }
    return null;
  }

  String? get coverImage => cover ?? (works.isEmpty ? null : works.first.image);
}
