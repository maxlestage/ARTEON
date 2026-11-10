# ARTEON sur iPad (Flutter)

L'application native d'ARTEON, pour iPad d'abord (et iPhone, Android) :

- **Expositions** : les cinquante et quelques expositions du musée, filtrées par style ou
  « en ce moment » (les expositions éphémères), avec leur bâtiment, leurs salles et leurs
  artistes.
- **Œuvres** : sur iPad à l'horizontale, l'image à gauche et la notice complète à droite
  (ce que l'on voit, ce qu'elle évoque, son histoire, la manière, les détails, les thèmes,
  l'anecdote, l'artiste, la reproduction et sa licence) ; œuvre précédente et suivante.
- **Image en grand** : pincer pour agrandir, glisser, toucher deux fois pour revenir.
- **Galerie 3D** : la visite du site (WebGL 2, écrite en Rust), en plein écran, ouverte
  devant l'œuvre choisie.
- **Recherche** dans tout le musée (titre, artiste, technique, collection, année, sans
  tenir compte des accents) et **favoris** gardés sur l'appareil.
- **Hors ligne** : le catalogue est gardé sur l'appareil et les images déjà vues aussi ;
  seule la galerie 3D a besoin d'Internet.

Le contenu vient du site : `catalogue.json`, publié avec lui (route `/catalogue.json` du
serveur, écrite par l'export statique, voir `src/catalogue.rs`). Une exposition ajoutée au
site apparaît donc dans l'application sans la republier.

## Et iOS ?

L'application iOS officielle d'ARTEON est désormais en Swift (`ios/`), c'est elle que
TestFlight distribue. Cette version Flutter reste pour Android ; le workflow
[`ipad.yml`](../.github/workflows/ipad.yml) vérifie quand même qu'elle compile pour iOS.

## Développer

```sh
cd flutter
flutter pub get
flutter test                 # modèles, recherche, écrans iPad et iPhone
flutter run                  # sur un simulateur ou un appareil branché (Mac)
# Un autre site que GitHub Pages (Heroku, local) :
flutter run --dart-define=ARTEON_URL=http://localhost:3000/
```

`test/fixtures/catalogue_extrait.json` est un extrait du vrai catalogue (Van Gogh et
Monet) : les tests vérifient que l'application lit ce que le site publie.
