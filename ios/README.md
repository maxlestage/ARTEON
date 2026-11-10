# ARTEON pour iPad et iPhone, en Swift

L'application native d'ARTEON, écrite entièrement en **Swift 6** et **SwiftUI** :

- **Expositions** : les cinquante et quelques expositions du musée, filtrées par style ou
  « en ce moment » ; chaque exposition avec sa couverture, son bâtiment, ses salles et leurs
  œuvres, ses artistes.
- **Œuvres** : sur iPad à l'horizontale, l'image à gauche et la notice complète à droite
  (ce que l'on voit, ce qu'elle évoque, son histoire, la manière, les détails, les thèmes,
  l'anecdote, l'artiste, la reproduction et sa licence) ; œuvre précédente et suivante,
  texte sélectionnable.
- **Image en grand** : pincer pour agrandir, glisser, toucher deux fois pour revenir.
- **Galerie 3D** : la visite du site (WebGL 2, écrite en Rust) en plein écran dans un
  `WKWebView`, ouverte devant l'œuvre choisie.
- **Recherche** dans tout le musée sans tenir compte des accents, **favoris** gardés sur
  l'appareil, catalogue et images **hors ligne** (seule la 3D a besoin d'Internet).
- Le style d'iOS 26 : onglets en barre latérale sur iPad (`TabView` `.sidebarAdaptable`),
  boutons en verre (Liquid Glass), mode sombre, Dynamic Type, VoiceOver.

Le contenu vient du site : `catalogue.json` (route `/catalogue.json`, `src/catalogue.rs`).
Une exposition ajoutée au site apparaît dans l'application sans la republier.

## Versions

- **Xcode 27.2** (sorti le 29 octobre 2026) et le **SDK iOS 27.2**, sur l'image `xcode-27`
  de GitHub Actions (macOS 27) ; **Swift 6.4**, en mode de langage Swift 6 (concurrence
  stricte).
- L'application demande **iOS / iPadOS 26** ou plus récent (Liquid Glass, onglets
  adaptatifs).
- XcodeGen 2.46.0 génère le projet Xcode ; les tests sont écrits avec Swift Testing.

## L'ouvrir sur l'iPad, sans Mac et sans compte payant : Swift Playgrounds

`ARTEON.swiftpm` est un *App Playground* : l'app **Swift Playgrounds** (gratuite, App Store)
le compile et le lance sur l'iPad.

1. Sur l'iPad, ouvrir le dernier run de **Actions → Application iOS (Swift)** sur GitHub,
   télécharger l'artefact **ARTEON-swift-playgrounds** (ou tout le dépôt : Code → Download
   ZIP).
2. Dans l'app Fichiers, toucher le zip pour l'ouvrir, puis toucher **ARTEON.swiftpm** :
   Swift Playgrounds l'ouvre.
3. Toucher ▶ : ARTEON se lance. « Installer sur l'écran d'accueil » (menu ⋯ de Swift
   Playgrounds, selon les versions) la garde comme une app.

## L'installer comme une vraie app : TestFlight

Avec un compte Apple Developer (99 € par an), le workflow
[`ios.yml`](../.github/workflows/ios.yml) signe l'application sur un Mac de GitHub et
l'envoie sur TestFlight ; les étapes, toutes faisables depuis un téléphone, sont en tête du
fichier. L'app TestFlight l'installe ensuite sur l'iPad, avec chaque nouvelle version.

## Développer (sur un Mac)

```sh
cd ios
brew install xcodegen        # ou la version 2.46.0 de github.com/yonaskolb/XcodeGen
xcodegen generate            # ARTEON.xcodeproj
open ARTEON.xcodeproj        # ou : open ARTEON.swiftpm (le même code)
# Un autre site que GitHub Pages : argument de lancement -site http://localhost:3000/
# Ouvrir une page au lancement : -ouvrir van-gogh, -ouvrir van-gogh/tournesols
```

Le modèle (`Models.swift`, `CatalogueStore.swift`) n'utilise que Foundation : ses tests
(`Tests/`) tournent aussi sous Linux avec la chaîne Swift officielle.
