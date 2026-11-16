# ARTEON pour iPad et iPhone, en Swift

L'application native d'ARTEON, écrite entièrement en **Swift 6** et **SwiftUI**, avec
**tout le musée embarqué** : les 52 expositions, leurs 532 œuvres et leurs notices, les
images et les galeries 3D se visitent **sans Internet**, sans serveur (ni Heroku, ni
GitHub Pages).

## Ce qu'elle fait

- **Musée** : l'œuvre du jour, « reprendre la visite » (les dernières œuvres vues), les
  expositions filtrées par style ou « en ce moment ».
- **Exposition** : sa couverture, son bâtiment, ses salles et leurs œuvres, ses artistes ;
  la **visite en 3D**, le **diaporama** et l'**introduction lue à voix haute**.
- **Œuvre** : sur iPad à l'horizontale, l'image à gauche et la notice complète à droite (ce
  que l'on voit, ce qu'elle évoque, son histoire, la manière, les détails, les thèmes,
  l'anecdote, l'artiste, la reproduction et sa licence) ; l'**audioguide** (la voix
  française de l'appareil lit le cartel et la notice), les autres œuvres du même artiste,
  le **partage** (l'image et son cartel), favori, précédente et suivante, image en grand
  (pincer, glisser, toucher deux fois).
- **Musées** : les vrais musées dont ARTEON reproduit les bâtiments (Orsay, le Prado, le
  Rijksmuseum, le Guggenheim, le British Museum, la Tate Modern…), pays par pays, chacun
  avec son histoire, ce qu'ARTEON lui emprunte, son **plan interactif** dessiné en natif
  (les salles à la couleur de leurs murs, les portes, chaque tableau le long de son mur,
  les socles, l'entrée : toucher une salle montre ses œuvres) et l'entrée en 3D. Et un
  musée **original**, le **Grand Musée ARTEON** : un bâtiment imaginaire où toutes les
  expositions sont des pavillons, en six ailes qui suivent le temps.
- **Collections du monde** : environ **50 000 œuvres du domaine public** choisies dans les
  collections ouvertes (CC0) de l'Art Institute of Chicago, du Cleveland Museum of Art, du
  Metropolitan Museum et de la National Gallery of Art de Washington : peintures d'abord,
  puis sculpture, dessin, photographie, objets… Fiches et vignettes sont dans
  l'application (on parcourt, on cherche, on met en favori sans Internet) ; l'image en
  grand se télécharge au premier regard puis reste, et **Tout télécharger** garde d'un
  coup celles d'un musée, d'une catégorie ou d'un artiste. Par musée, par catégorie, par
  siècle ; dans la recherche, les favoris et les pages d'artistes.
- **Galerie 3D** : celle du site (WebGL 2, écrite en Rust et compilée en WebAssembly),
  servie par l'application elle-même à l'adresse `arteon://musee/…` : visite guidée ou
  libre, plan, notices, ouverte devant l'œuvre choisie.
- **Artistes** : les quelque 400 artistes, ateliers et cultures de A à Z (index sur le
  côté), chacun avec sa biographie à lire ou à écouter, toutes ses œuvres et ses
  expositions.
- **Explorer** : la **chronologie** (siècle par siècle, des météorites au street art), les
  **thèmes** (la nuit, la mer, le portrait…), les **musées du monde** où voir les œuvres
  en vrai, les **techniques**, une œuvre au hasard, et le jeu **« Qui l'a fait ? »** (une
  œuvre, quatre artistes de la même époque, une série et un record).
- **Favoris** et **œuvres vues**, avec diaporama ; **recherche** dans tout le musée (œuvres,
  artistes, expositions, thèmes), sans tenir compte des accents, mot par mot
  (« monet nuit »).
- **Réglages** : la place du musée sur l'appareil, les mises à jour facultatives depuis le
  site, la vitesse de la voix, l'effacement des favoris et de l'historique, les crédits.
- Le style d'iOS 26 : onglets en barre latérale sur iPad (`TabView` `.sidebarAdaptable`),
  Liquid Glass, mode sombre, Dynamic Type, VoiceOver.

## Le musée embarqué

`ARTEON.swiftpm/Musee/` reçoit le site exporté, réduit à ce que lit l'application (environ
200 Mo) :

| Fichier | Contenu |
|---|---|
| `catalogue.json` | les expositions, les œuvres, les notices, les artistes (`src/catalogue.rs`) |
| `tableaux/` | les images des œuvres |
| `donnees/` | les mosaïques et sculptures des galeries 3D, et leurs textes |
| `pkg/` | les îlots 3D en WebAssembly |
| `<exposition>/galerie/` | la page de chaque galerie 3D |

Les collections du monde y ont leur dossier, `collections/` : `collections.json` (les
fiches) et un fichier de vignettes par musée (`vignettes-met.bin`…, les JPEG bout à bout),
écrits par [`outils/collections_monde.py`](../outils/collections_monde.py) (environ 500 Mo
pour 50 000 œuvres ; `--total 400` pour un essai).

Il n'est pas versionné (les images sont déjà dans `public/tableaux/`) :
[`preparer-musee.sh`](preparer-musee.sh) le remplit (il compile le WebAssembly, exporte le
site pour la racine et copie ce qu'il faut), et le workflow
[`ios.yml`](../.github/workflows/ios.yml) le fait avant chaque compilation. Sans lui,
l'application télécharge le catalogue et les images depuis le site, puis les garde.

Quand le site répond, l'application y cherche les nouveautés (réglable) : une exposition
publiée après l'application y apparaît, ses images se téléchargent une fois puis restent
sur l'appareil.

## Versions

- **Xcode 27.2** et le **SDK iOS 27.2**, sur l'image `xcode-27` de GitHub Actions
  (macOS 27) ; **Swift 6.4**, en mode de langage Swift 6 (concurrence stricte).
- L'application demande **iOS / iPadOS 26** ou plus récent (Liquid Glass, onglets
  adaptatifs, index des listes).
- XcodeGen 2.46.0 génère le projet Xcode ; les tests sont écrits avec Swift Testing.

## L'ouvrir sur l'iPad, sans Mac et sans compte payant : Swift Playgrounds

`ARTEON.swiftpm` est un *App Playground* : l'app **Swift Playgrounds** (gratuite, App Store)
le compile et le lance sur l'iPad.

1. Sur l'iPad, télécharger **ARTEON.swiftpm.zip** de la release
   [« ipad »](https://github.com/maxlestage/ARTEON/releases/tag/ipad) (refaite à chaque
   push sur `master`), ou l'artefact **ARTEON-swift-playgrounds** du dernier run de
   **Actions → Application iOS (Swift)**. Les deux contiennent le musée embarqué (le
   « Download ZIP » du dépôt, lui, ne l'a pas).
2. Dans l'app Fichiers, toucher le zip pour l'ouvrir, puis toucher **ARTEON.swiftpm** :
   Swift Playgrounds l'ouvre.
3. Toucher ▶ : ARTEON se lance, entièrement hors ligne.

## Sur l'iPhone

L'application est la même sur iPhone (onglets en bas ; les artistes sont dans Explorer, la
barre n'ayant que cinq places). Swift Playgrounds n'existe pas sur iPhone : il faut
**TestFlight** (ci-dessous), ou un Mac avec Xcode et un compte Apple gratuit (brancher
l'iPhone, choisir son équipe dans *Signing & Capabilities*, ▶ ; l'application doit alors
être réinstallée tous les 7 jours).

## L'installer comme une vraie app : TestFlight

Avec un compte Apple Developer (99 € par an), le workflow
[`ios.yml`](../.github/workflows/ios.yml) signe l'application (musée compris) sur un Mac de
GitHub et l'envoie sur TestFlight ; les étapes, toutes faisables depuis un téléphone, sont
en tête du fichier. L'app TestFlight l'installe ensuite sur l'iPhone comme sur l'iPad (le
même achat de l'application, les deux appareils), avec chaque nouvelle version.

## Développer (sur un Mac)

```sh
ios/preparer-musee.sh        # le musée embarqué (Rust, wasm32, wasm-bindgen : voir le README)
cd ios
brew install xcodegen        # ou la version 2.46.0 de github.com/yonaskolb/XcodeGen
xcodegen generate            # ARTEON.xcodeproj
open ARTEON.xcodeproj        # ou : open ARTEON.swiftpm (le même code)
```

Arguments de lancement : `-horsligne` (jamais de réseau), `-site http://localhost:3000/`
(un autre site pour les mises à jour), `-ouvrir van-gogh`, `-ouvrir van-gogh/tournesols`,
`-ouvrir 3d:van-gogh`, `-ouvrir chronologie`, `-ouvrir jeu`, `-ouvrir musee:Rijksmuseum`,
`-ouvrir grand-musee`, `-ouvrir collections`, `-ouvrir collection:met`, `-onglet musees`,
`-onglet artistes`.

## Tests

Le modèle (`Models.swift`, `Library.swift`, `CatalogueStore.swift`) n'utilise que
Foundation : `swift test` dans `ios/` le teste sous Linux comme sur un Mac
([`Package.swift`](Package.swift)), et vérifie le vrai musée embarqué quand il a été
préparé (chaque œuvre a son image, chaque exposition sa galerie, les dates se lisent, le
jeu tire de bonnes questions). Xcode lance les mêmes tests sur le simulateur iPad.
