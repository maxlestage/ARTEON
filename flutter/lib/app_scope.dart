import 'package:flutter/widgets.dart';

import 'models.dart';
import 'repository.dart';

/// Le catalogue et les favoris, pour tous les écrans.
class AppScope extends InheritedWidget {
  const AppScope({
    super.key,
    required this.catalogue,
    required this.favorites,
    required this.offline,
    required super.child,
  });

  final Catalogue catalogue;
  final Favorites favorites;
  final bool offline;

  static AppScope of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<AppScope>()!;

  @override
  bool updateShouldNotify(AppScope old) =>
      old.catalogue != catalogue || old.offline != offline;
}
