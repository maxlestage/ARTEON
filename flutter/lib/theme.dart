import 'package:flutter/material.dart';

/// Les couleurs du site : le bleu d'ARTEON, la nuit, le soleil.
const arteonBlue = Color(0xff1d3f8f);
const arteonNight = Color(0xff0f1a3a);
const arteonSun = Color(0xfff2b705);

ThemeData arteonTheme(Brightness brightness) {
  final scheme = ColorScheme.fromSeed(
    seedColor: arteonBlue,
    brightness: brightness,
    primary: brightness == Brightness.light
        ? arteonBlue
        : const Color(0xff9db4ff),
    tertiary: arteonSun,
  );
  final base = ThemeData(colorScheme: scheme, useMaterial3: true);
  return base.copyWith(
    scaffoldBackgroundColor: brightness == Brightness.light
        ? const Color(0xfff8f9fc)
        : arteonNight,
    textTheme: base.textTheme.apply(
      bodyColor: scheme.onSurface,
      displayColor: scheme.onSurface,
    ),
    cardTheme: CardThemeData(
      clipBehavior: Clip.antiAlias,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(18),
        side: BorderSide(color: scheme.outlineVariant),
      ),
    ),
    chipTheme: base.chipTheme.copyWith(
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(999)),
    ),
  );
}

/// Les marges d'une page : plus larges sur iPad.
EdgeInsets pagePadding(BuildContext context) {
  final width = MediaQuery.sizeOf(context).width;
  final side = width >= 1100
      ? 48.0
      : width >= 700
      ? 32.0
      : 16.0;
  return EdgeInsets.symmetric(horizontal: side);
}
