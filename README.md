# ARTEON

**Un musée virtuel en 3D, écrit uniquement en Rust.**

ARTEON est un musée qui n'existe pas. Chaque exposition est consacrée à un peintre : ses toiles sont accrochées dans des salles reliées par des couloirs, et une caméra fait la visite guidée en boucle, en s'arrêtant devant chaque œuvre. Tout est écrit en Rust avec [Paquets](https://github.com/maxlestage/Paquets) : le serveur, les pages HTML et les scènes 3D, compilées en WebAssembly et rendues en WebGL 2.

Deux expositions sont ouvertes :

- **Van Gogh, la couleur en mouvement** — douze toiles de Vincent van Gogh (1885–1890), des *Mangeurs de pommes de terre* au *Champ de blé aux corbeaux* ;
- **Monet, la lumière à l'œuvre** — douze toiles de Claude Monet (1866–1922), de *Femmes au jardin* et *Impression, soleil levant* jusqu'aux *Nymphéas* et au *Pont japonais*.

## Les pages

| Route | Contenu |
|---|---|
| `/` | L'accueil du musée : le concept, les chiffres clés, les expositions et comment c'est fait |
| `/<exposition>` | La présentation d'une exposition (`/van-gogh`, `/monet`) : une toile sur un chevalet tournant en 3D, le parcours salle par salle, des reproductions |
| `/<exposition>/galerie` | La visite guidée en 3D de l'exposition |
| `/<exposition>/oeuvres` | Le catalogue : chaque toile, son cartel (technique, collection, note) et sa place dans la galerie |
| `/ajouter` | Le guide pour ajouter, déplacer ou modifier une œuvre, ou créer une exposition, avec le plan des emplacements libres |
| toute autre adresse | Une page « introuvable » (404) |

## Ajouter ou modifier une œuvre

Une exposition = **un fichier** [`expositions/<nom>.toml`](expositions) + **un dossier d'images** [`public/tableaux/<nom>/`](public/tableaux). Pas une ligne de code à écrire.

1. Déposer l'image de la toile (JPEG ou PNG, 900 à 1000 pixels de large suffisent) dans le dossier de l'exposition, par exemple `public/tableaux/monet/`. Sur GitHub : ouvrir le dossier, puis *Add file → Upload files*.
2. Copier un bloc `[[oeuvres]]` dans le fichier de l'exposition (`expositions/monet.toml`) et changer ses valeurs :

   ```toml
   [[oeuvres]]
   titre = "Le Déjeuner sur l'herbe"
   annee = "1866"
   image = "dejeuner-sur-l-herbe.jpg"
   technique = "Huile sur toile, 130 × 181 cm"
   collection = "Musée d'Orsay, Paris"
   note = "Une partie de campagne en forêt de Fontainebleau, peinte en plein air."
   salle = 1          # 1, 2, 3…
   mur = "fond"       # "gauche", "droite" (en avançant) ou "fond"
   place = 1          # 1, 2 ou 3 (1 = près de l'entrée de la salle)
   largeur = 1.6      # en mètres, de 0,6 à 2,6 ; la hauteur suit l'image
   ```

3. Enregistrer (*Commit changes*) sur `master` : le workflow republie le site en quelques minutes. La toile apparaît dans la galerie 3D, dans le catalogue et dans la visite guidée, qui se recalcule toute seule.

Pour **modifier** une œuvre, changer les valeurs de son bloc ; pour la **retirer**, supprimer le bloc. Un bloc `[[salles]]` de plus ajoute une salle et le couloir qui y mène.

**Créer une exposition** : copier un fichier de `expositions/` sous un autre nom (par exemple `renoir.toml` : l'exposition sera à l'adresse `/renoir`), créer le dossier `public/tableaux/renoir/` pour ses images, puis changer les textes (section `[exposition]` : titre, artiste, introduction, couleur d'accent, nom dans le menu, ordre sur l'accueil, œuvre de la vitrine), les salles et les œuvres. Elle apparaît d'elle-même sur l'accueil et dans le menu.

Les fichiers sont vérifiés à la compilation (`build.rs`) : salle inexistante, image introuvable, place déjà prise, toiles qui se chevauchent, tableau trop grand, couleur mal écrite… La compilation s'arrête alors avec un message en français qui dit quoi corriger, par exemple :

```
✖ expositions/monet.toml : l'œuvre « Nymphéas » : l'image « nympheaz.jpg » est introuvable dans public/tableaux/monet/
```

et le site en ligne reste tel quel.

## Comment les toiles deviennent de la 3D

Le moteur 3D de Paquets n'affiche pas d'images (pas de textures). À la compilation, `build.rs` réduit donc chaque image à une grille dont chaque case mesure **1 cm** dans la galerie (jusqu'à 240 cases de côté), et la ramène à une palette de **64 couleurs** ; les toiles sombres sont légèrement éclaircies pour rester lisibles sous l'éclairage du musée. Les cases sont stockées en binaire (`include_bytes!`).

Dans la scène, les cases voisines de même couleur sont fusionnées en rectangles, et chaque couleur de la palette forme un seul maillage (64 au plus par tableau) : environ 260 000 triangles pour toute une exposition, ce qui reste fluide. Les tons clairs sont très légèrement en avant, comme des empâtements. Le catalogue, lui, montre les vraies reproductions.

## Les œuvres (domaine public)

Vincent van Gogh (mort en 1890) et Claude Monet (mort en 1926) sont dans le domaine public. Les reproductions viennent de [Wikimedia Commons](https://commons.wikimedia.org/). Dans la galerie 3D, les toiles sont réinterprétées en mosaïque : ce ne sont pas des reproductions exactes.

**Van Gogh, la couleur en mouvement**

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

**Monet, la lumière à l'œuvre**

| Salle | Œuvre | Année | Collection |
|---|---|---|---|
| I · La naissance de l'impressionnisme | Femmes au jardin | 1866 | Musée d'Orsay, Paris |
| | La Pie | 1868–1869 | Musée d'Orsay, Paris |
| | Impression, soleil levant | 1872 | Musée Marmottan Monet, Paris |
| | Coquelicots | 1873 | Musée d'Orsay, Paris |
| | La Promenade, la femme à l'ombrelle | 1875 | National Gallery of Art, Washington |
| II · La ville et les séries | La Gare Saint-Lazare | 1877 | Musée d'Orsay, Paris |
| | La Rue Montorgueil, fête du 30 juin 1878 | 1878 | Musée d'Orsay, Paris |
| | Meules, fin de l'été | 1890–1891 | Art Institute of Chicago |
| | La Cathédrale de Rouen, harmonie bleue | 1893 | Musée d'Orsay, Paris |
| III · Londres et Giverny | Le Parlement de Londres, soleil couchant | 1903 | National Gallery of Art, Washington |
| | Nymphéas | 1906 | Art Institute of Chicago |
| | Le Pont japonais | 1920–1922 | Museum of Modern Art, New York |

## Architecture

- `expositions/*.toml` : le contenu de chaque exposition (textes, salles, œuvres).
- `public/tableaux/<nom>/` : les images des œuvres, servies à `/tableaux/<nom>/…`.
- `build.rs` : vérifie les expositions, transforme les images en mosaïques et écrit le catalogue en Rust.
- `src/artworks.rs` : les types du catalogue et les dimensions de la galerie.
- `src/gallery.rs` : les scènes 3D (`paquets::ecs`) et la visite guidée.
- `src/lib.rs` : les îlots (galerie et vitrine, avec l'exposition en paramètre) et le point d'entrée WebAssembly.
- `src/main.rs` : le serveur et les pages.
- `src/style.rs` : le CSS, sous forme de constante Rust.
- `src/export.rs` : l'export du site en fichiers statiques, pour GitHub Pages.
- `.github/workflows/pages.yml` : compilation, tests, export, vérification et déploiement sur GitHub Pages.

La visite guidée est calculée à partir des œuvres accrochées : un arrêt face à chaque toile (d'autant plus loin qu'elle est grande), un passage dans l'axe de chaque porte, puis le retour à l'entrée. Entre deux arrêts, la caméra tourne la tête par le plus court chemin et reste à l'horizontale. C'est une fonction pure (`tour_pose`), testée sans navigateur.

Deux contraintes du moteur sont prises en compte (et testées) : la physique de `DefaultPlugins` est retirée, sinon chaque pavé (murs, sols, tableaux) serait soumis à la gravité et tomberait dans le vide ; et une scène n'a pas plus de 8 lampes ponctuelles, le maximum que le shader prend en compte (au-delà de trois salles, une lampe par salle, puis la lumière d'ambiance seule).

## Lancer le projet

- Installer la commande `paquets` : `cargo install --git https://github.com/maxlestage/Paquets paquets --locked`
- Lancer en développement (http://localhost:3000, avec rechargement) : `paquets dev`
- Lancer les tests : `cargo test`
- Compiler pour la production (serveur et bundle WebAssembly optimisés) : `paquets build`

## Publier sur GitHub Pages

Le site est publié automatiquement par le workflow `.github/workflows/pages.yml` à chaque push sur la branche par défaut (`master`), ou à la main depuis l'onglet **Actions** (« Run workflow »). Sur les autres branches et les pull requests, il compile, teste, exporte et vérifie le site sans le déployer.

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

Le code d'ARTEON est sous licence MIT. Il dépend de [Paquets](https://github.com/maxlestage/Paquets), un logiciel propriétaire qui peut être utilisé librement comme dépendance (voir sa licence). Les œuvres de Vincent van Gogh et de Claude Monet appartiennent au domaine public.
