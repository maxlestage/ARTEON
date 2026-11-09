# ARTEON

**Une exposition virtuelle en 3D, écrite uniquement en Rust : « Van Gogh, la couleur en mouvement ».**

ARTEON est un musée virtuel. Sa première exposition est entièrement consacrée à Vincent van Gogh (1853–1890) : douze toiles, des *Mangeurs de pommes de terre* au *Champ de blé aux corbeaux*, accrochées dans trois salles reliées par des couloirs. Une caméra fait la visite guidée en boucle et s'arrête devant chaque œuvre. Tout est écrit en Rust avec [Paquets](https://github.com/maxlestage/Paquets) : le serveur, les pages HTML et la scène 3D, compilée en WebAssembly et rendue en WebGL 2.

## Les pages

| Route | Contenu |
|---|---|
| `/` | La présentation de l'exposition : la *Nuit étoilée* sur un chevalet tournant en 3D, le titre animé, les chiffres clés, le parcours salle par salle, des reproductions et le grand pied de page animé |
| `/galerie` | La visite guidée en 3D des salles et de leurs couloirs |
| `/oeuvres` | Le catalogue : chaque toile, son cartel (technique, collection, note) et sa place dans la galerie |
| `/ajouter` | Le guide pour ajouter, déplacer ou modifier une œuvre, avec le plan des emplacements libres |
| toute autre adresse | Une page « introuvable » (404) |

## Ajouter ou modifier une œuvre

Tout le contenu du musée est dans **un seul fichier, [`exposition.toml`](exposition.toml)**, et les images dans **[`public/tableaux/`](public/tableaux)**. Pas une ligne de code à écrire.

1. Déposer l'image de la toile (JPEG ou PNG, 900 pixels de large suffisent) dans `public/tableaux/`. Sur GitHub : *Add file → Upload files*.
2. Copier un bloc `[[oeuvres]]` dans `exposition.toml` et changer ses valeurs :

   ```toml
   [[oeuvres]]
   titre = "Le Semeur"
   annee = "1888"
   image = "semeur.jpg"
   technique = "Huile sur toile, 64 × 80,5 cm"
   collection = "Musée Kröller-Müller, Otterlo"
   note = "Un soleil immense derrière le semeur, dans un champ violet."
   salle = 3          # 1, 2, 3…
   mur = "droite"     # "gauche", "droite" (en avançant) ou "fond"
   place = 3          # 1, 2 ou 3 (1 = près de l'entrée de la salle)
   largeur = 1.3      # en mètres, de 0,6 à 2,6 ; la hauteur suit l'image
   ```

3. Enregistrer (*Commit changes*) sur `master` : le workflow republie le site en quelques minutes. La toile apparaît dans la galerie 3D, dans le catalogue et dans la visite guidée, qui se recalcule toute seule.

Pour **modifier** une œuvre, changer les valeurs de son bloc ; pour la **retirer**, supprimer le bloc. Un bloc `[[salles]]` de plus ajoute une salle et le couloir qui y mène. Les textes de l'accueil (titre, introduction, l'œuvre de la vitrine) sont dans la section `[exposition]`.

Le fichier est vérifié à la compilation (`build.rs`) : salle inexistante, image introuvable, place déjà prise, toiles qui se chevauchent, tableau trop grand… La compilation s'arrête alors avec un message en français qui dit quoi corriger, par exemple :

```
✖ exposition.toml : l'œuvre « Les Iris » : l'image « iriss.jpg » est introuvable dans public/tableaux/
```

et le site en ligne reste tel quel.

## Comment les toiles deviennent de la 3D

Le moteur 3D de Paquets n'affiche pas d'images (pas de textures). À la compilation, `build.rs` réduit donc chaque image à une grille (une case ≈ 2,8 cm dans la galerie) et la ramène à une palette de 20 couleurs. Dans la scène, chaque case devient une petite touche en relief, plus épaisse dans les tons clairs, comme les empâtements du peintre ; les toiles sombres sont légèrement éclaircies pour rester lisibles sous l'éclairage du musée. Les touches d'une même couleur forment un seul maillage : une vingtaine par tableau, ce qui reste fluide. Le catalogue, lui, montre les vraies reproductions.

## Les œuvres (domaine public)

Van Gogh est mort en 1890 : ses œuvres sont dans le domaine public. Les reproductions viennent de [Wikimedia Commons](https://commons.wikimedia.org/wiki/Category:Paintings_by_Vincent_van_Gogh). Dans la galerie 3D, les toiles sont réinterprétées en mosaïque : ce ne sont pas des reproductions exactes.

| Salle | Œuvre | Année | Collection |
|---|---|---|---|
| I · Nuenen et Paris | Les Mangeurs de pommes de terre | 1885 | Van Gogh Museum, Amsterdam |
| | Autoportrait | 1887 | Art Institute of Chicago |
| | Autoportrait au chapeau de feutre gris | 1887 | Van Gogh Museum, Amsterdam |
| II · Arles | Les Tournesols | 1888 | National Gallery, Londres |
| | La Nuit étoilée sur le Rhône | 1888 | Musée d'Orsay, Paris |
| | La Chambre à Arles | 1888 | Van Gogh Museum, Amsterdam |
| | Terrasse du café le soir | 1888 | Musée Kröller-Müller, Otterlo |
| | Le Café de nuit | 1888 | Yale University Art Gallery |
| III · Saint-Rémy et Auvers | Les Iris | 1889 | J. Paul Getty Museum, Los Angeles |
| | La Nuit étoilée | 1889 | Museum of Modern Art, New York |
| | Amandier en fleurs | 1890 | Van Gogh Museum, Amsterdam |
| | Champ de blé aux corbeaux | 1890 | Van Gogh Museum, Amsterdam |

## Architecture

- `exposition.toml` : le contenu de l'exposition (textes, salles, œuvres).
- `public/tableaux/` : les images des œuvres, servies à `/tableaux/…`.
- `build.rs` : vérifie `exposition.toml`, transforme les images en mosaïques et écrit le catalogue en Rust.
- `src/artworks.rs` : les types du catalogue et les dimensions de la galerie.
- `src/gallery.rs` : les scènes 3D (`paquets::ecs`) et la visite guidée.
- `src/lib.rs` : les îlots (galerie et vitrine) et le point d'entrée WebAssembly.
- `src/main.rs` : le serveur et les pages.
- `src/style.rs` : le CSS, sous forme de constante Rust.
- `src/export.rs` : l'export du site en fichiers statiques, pour GitHub Pages.
- `.github/workflows/pages.yml` : compilation, tests, export et déploiement sur GitHub Pages.

La visite guidée est calculée à partir des œuvres accrochées : un arrêt face à chaque toile (d'autant plus loin qu'elle est grande), un passage dans l'axe de chaque porte, puis le retour à l'entrée. C'est une fonction pure (`tour_pose(t)`), testée sans navigateur.

Deux contraintes du moteur sont prises en compte (et testées) : la physique de `DefaultPlugins` est retirée, sinon chaque pavé (murs, sols, tableaux) serait soumis à la gravité et tomberait dans le vide ; et la scène n'a pas plus de 8 lampes ponctuelles, le maximum que le shader prend en compte (au-delà de trois salles, une lampe par salle, puis la lumière d'ambiance seule).

## Lancer le projet

- Installer la commande `paquets` : `cargo install --git https://github.com/maxlestage/Paquets paquets --locked`
- Lancer en développement (http://localhost:3000, avec rechargement) : `paquets dev`
- Lancer les tests : `cargo test`
- Compiler pour la production (serveur et bundle WebAssembly optimisés) : `paquets build`

## Publier sur GitHub Pages

Le site est publié automatiquement par le workflow `.github/workflows/pages.yml` à chaque push sur la branche par défaut (`master`), ou à la main depuis l'onglet **Actions** (« Run workflow »). Sur les autres branches et les pull requests, il compile, teste et exporte le site sans le déployer.

Réglage unique du dépôt : **Settings → Pages → Build and deployment → Source : GitHub Actions**. Le workflow essaie de l'activer tout seul ; si le premier déploiement échoue avec « Pages not enabled », il suffit de faire ce réglage puis de relancer le workflow.

Adresse du site : `https://<compte>.github.io/ARTEON/`.

Pour produire le même export en local :

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked   # la version de Cargo.lock
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release
wasm-bindgen --target web --no-typescript --remove-name-section --out-dir pkg \
  target/wasm32-unknown-unknown/wasm-release/arteon.wasm
BASE_PATH=/ARTEON cargo run --release -- export dist
```

Chaque page est rendue par le serveur lui-même (le client de test en mémoire de Paquets), puis écrite dans `dist/` avec `404.html`, les images (`public/`), le bundle WebAssembly (`pkg/`) et `.nojekyll`. `BASE_PATH` préfixe tous les liens ; sans lui, le site s'exporte pour la racine d'un domaine.

## Licence

Le code d'ARTEON est sous licence MIT. Il dépend de [Paquets](https://github.com/maxlestage/Paquets), un logiciel propriétaire qui peut être utilisé librement comme dépendance (voir sa licence). Les œuvres de Vincent van Gogh appartiennent au domaine public.
