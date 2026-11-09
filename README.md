# ARTEON

**Un musée virtuel en 3D, écrit uniquement en Rust.**

ARTEON est un musée qui n'existe pas. Chaque exposition occupe un bâtiment dessiné sur plan, souvent inspiré d'un vrai musée (la galerie d'honneur du Rijksmuseum, la nef d'Orsay, le Belvédère, la cour du Smithsonian) : les toiles sont accrochées aux murs, les sculptures — de vrais scans 3D — posées sur leurs socles. On s'y promène librement ou on suit la visite guidée, et un éditeur permet de déplacer les œuvres depuis le musée lui-même. Tout est écrit en Rust avec [Paquets](https://github.com/maxlestage/Paquets) : le serveur, les pages HTML, les scènes 3D (`paquets::ecs`) et l'interface de visite ([egui](https://github.com/emilk/egui)), compilées en WebAssembly et rendues en WebGL 2.

18 expositions sont ouvertes (177 toiles, 8 sculptures, 109 artistes), de la Renaissance à l'expressionnisme, dans l'ordre de l'histoire de l'art :

| Exposition | Style | Dates | Artistes |
|---|---|---|---|
| [La Renaissance italienne, l'invention de l'homme](expositions/renaissance.toml) | Renaissance italienne | 1460–1538 | Filippo Lippi, Piero della Francesca, Andrea Mantegna, Sandro Botticelli, Léonard de Vinci, Michel-Ange, Raphaël, Giorgione, Titien |
| [Monts, jardins et empereurs : la peinture d'Asie](expositions/arts-asie.toml) | Peinture chinoise, persane et moghole | vers 1000–1630 | Fan Kuan, Guo Xi, Zhao Mengfu, Shen Zhou, Behzad, Sultan Muhammad, Reza Abbasi, Bichitr, Abu'l Hasan, Govardhan |
| [Primitifs flamands et Renaissance du Nord, le monde au plus près](expositions/renaissance-nord.toml) | Primitifs flamands et Renaissance du Nord | 1434–1568 | Jan van Eyck, Rogier van der Weyden, Hans Memling, Jérôme Bosch, Albrecht Dürer, Lucas Cranach l'Ancien, Hans Holbein le Jeune, Pieter Bruegel l'Ancien |
| [Le Baroque, le théâtre de la lumière](expositions/baroque.toml) | Baroque | 1599–1678 | Le Caravage, Artemisia Gentileschi, Georges de La Tour, Pierre Paul Rubens, Bartolomé Esteban Murillo, Francisco de Zurbarán, Diego Velázquez, Nicolas Poussin, Claude Lorrain |
| [Vermeer et Rembrandt, le siècle d'or hollandais](expositions/vermeer-rembrandt.toml) | Siècle d'or hollandais | XVIIᵉ siècle | Rembrandt, Johannes Vermeer |
| [Le siècle des Lumières, du rococo au néoclassicisme](expositions/lumieres.toml) | Rococo et néoclassicisme | 1717–1814 | Antoine Watteau, François Boucher, Jean-Honoré Fragonard, Jean Siméon Chardin, Giovanni Antonio Canal, dit Canaletto, Thomas Gainsborough, Élisabeth Vigée Le Brun, Jacques-Louis David, Jean-Auguste-Dominique Ingres |
| [Le Romantisme, la tempête et le rêve](expositions/romantisme.toml) | Romantisme | 1781–1839 | Johann Heinrich Füssli, William Blake, Francisco de Goya, Théodore Géricault, Eugène Delacroix, Caspar David Friedrich, Joseph Mallord William Turner, John Constable |
| [Du réalisme à la vie moderne](expositions/realisme.toml) | Réalisme et impressionnisme | 1849–1882 | Édouard Manet, Gustave Courbet, Rosa Bonheur, Jean-François Millet, Gustave Caillebotte, Edgar Degas, Camille Pissarro |
| [Monet, la lumière à l'œuvre](expositions/monet.toml) | Impressionnisme | 1840–1926 | Claude Monet, Hiram Powers, Ferdinand Pettrich, Edmonia Lewis |
| [Renoir, le bonheur de peindre](expositions/renoir.toml) | Impressionnisme | 1841–1919 | Pierre-Auguste Renoir |
| [L'Amérique au XIXᵉ siècle, peinture et sculpture](expositions/smithsonian.toml) | Art américain du XIXᵉ siècle | 1836–1899 | Thomas Cole, Albert Bierstadt, Frederic Edwin Church, James McNeill Whistler, Thomas Eakins, John Singer Sargent, Mary Cassatt, Winslow Homer, Horatio Greenough, John Rogers, Clark Mills, Chine, dynastie des Qi du Nord |
| [Cézanne, la construction de la couleur](expositions/cezanne.toml) | Post-impressionnisme | 1839–1906 | Paul Cézanne |
| [Après l'impressionnisme, points, aplats et rêves](expositions/post-impressionnisme.toml) | Néo-impressionnisme, Nabis et art naïf | 1884–1910 | Georges Seurat, Paul Signac, Henri de Toulouse-Lautrec, Paul Sérusier, Émile Bernard, Pierre Bonnard, Édouard Vuillard, Henri Rousseau |
| [Van Gogh, la couleur en mouvement](expositions/van-gogh.toml) | Post-impressionnisme | 1853–1890 | Vincent van Gogh |
| [Gauguin, l'ailleurs en couleurs](expositions/gauguin.toml) | Synthétisme et primitivisme | 1848–1903 | Paul Gauguin |
| [Le Symbolisme, rêves, mythes et visions](expositions/symbolisme.toml) | Symbolisme et Art nouveau | 1851–1914 | Alfons Mucha, Pierre Puvis de Chavannes, Gustave Moreau, Odilon Redon, John Everett Millais, Dante Gabriel Rossetti, John William Waterhouse, Arnold Böcklin, Edvard Munch, Fernand Khnopff |
| [Klimt, l'or et la vie](expositions/klimt.toml) | Sécession viennoise | 1862–1918 | Gustav Klimt |
| [Fauves et expressionnistes, la couleur libérée](expositions/expressionnisme.toml) | Fauvisme et expressionnisme | 1904–1919 | Henri Matisse, André Derain, Franz Marc, August Macke, Ernst Ludwig Kirchner, Paula Modersohn-Becker, Egon Schiele, Amedeo Modigliani |

L'exposition Monet est installée dans une nef inspirée du musée d'Orsay, avec trois sculptures américaines contemporaines de ses toiles (*The Greek Slave* de Hiram Powers, *The Dying Tecumseh* de Ferdinand Pettrich, *Old Arrow Maker* d'Edmonia Lewis).

Seules des œuvres du **domaine public** sont exposées : l'artiste est mort en 1955 ou avant (70 ans après sa mort, en Europe) et l'œuvre date d'avant 1931 (aux États-Unis). C'est pourquoi il n'y a ni Picasso (protégé jusqu'en 2044), ni Dalí, Magritte, Chagall, Frida Kahlo, ni pop art ou street art : Andy Warhol est protégé jusqu'en 2057, Keith Haring jusqu'en 2060.

## Les pages

| Route | Contenu |
|---|---|
| `/` | L'accueil du musée : le concept, les chiffres clés, les expositions et comment c'est fait |
| `/<exposition>` | La présentation d'une exposition (`/van-gogh`, `/renaissance`, `/baroque`, `/estampes-japonaises`…, le nom du fichier `.toml`) : une toile sur un chevalet tournant en 3D, le parcours salle par salle, des reproductions |
| `/<exposition>/galerie` | Le musée en 3D : visite guidée ou libre, plan, éditeur |
| `/<exposition>/oeuvres` | Le catalogue : chaque toile et chaque sculpture, son cartel (technique, collection, note) et sa place dans le musée |
| `/<exposition>/oeuvres/<œuvre>` | Tout sur une œuvre : la reproduction, le cartel, ce que l'on voit, ce qu'elle évoque, son histoire, la manière, les détails à regarder, une anecdote et la biographie de l'artiste ; un lien l'ouvre dans le musée 3D, devant elle |
| `/ajouter` | Le guide pour ajouter, déplacer ou modifier une œuvre, ou créer une exposition, avec le plan des emplacements libres |
| toute autre adresse | Une page « introuvable » (404) |

## Dans le musée 3D

- **Toucher la scène** (ou le bouton **⛶**, ou la touche `F`) la met en plein écran. Sur iPhone, où le navigateur ne propose pas le plein écran aux pages web, le musée occupe tout l'écran.
- L'interface de visite est dessinée avec **egui**, par-dessus la scène : une barre d'outils, le cartel de l'œuvre regardée (avec « Voir en HD » pour les toiles), le plan et la liste des œuvres.
- **Visite guidée** : la caméra part du hall d'entrée, traverse les salles par leurs portes, contourne bancs et sculptures et s'arrête devant chaque œuvre. **‹ ❚❚ ›** (flèches, `Espace`) passent à l'œuvre précédente, mettent en pause, passent à la suivante.
- **Visite libre** (bouton « Libre » ou touche `V`) : `Z Q S D` ou `W A S D` pour marcher, les flèches pour tourner, `Maj` pour courir, la souris (glisser) pour regarder ; sur téléphone, un joystick pour avancer, l'autre pour regarder. On ne traverse ni les murs, ni les socles, ni les bancs.
- **📖 Tout savoir**, dans le cartel : la notice complète de l'œuvre regardée et la biographie de l'artiste ; la visite guidée attend devant l'œuvre pendant la lecture.
- **Plan** : cliquer une salle y emmène, cliquer une œuvre place le visiteur devant elle. **Œuvres** : la liste de l'exposition, salle par salle.
- **Éditer** : l'éditeur du musée (voir plus bas). **✕** (`Échap`) quitte le plein écran.
- La caméra recule selon la forme de l'écran : sur un téléphone en portrait, la toile reste entière.

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

### La notice complète

Chaque œuvre a sa page, `/<exposition>/oeuvres/<nom de l'image>` (ou du modèle pour une sculpture), et un panneau **📖 Tout savoir** dans le musée 3D. Ils affichent les champs facultatifs de son bloc :

```toml
description = """
Ce que l'on voit : la scène, les personnages, la composition…"""
evocation = """
Ce qu'elle évoque : le sens, les émotions, les symboles…"""
histoire = """
Son histoire : pourquoi, pour qui, l'accueil, le parcours jusqu'au musée…"""
facture = """
La manière : la touche, les couleurs, la lumière, le cadrage…"""
details = ["Un détail précis à repérer.", "Un autre."]
themes = ["nuit", "paysage", "japonisme"]
anecdote = "Un fait marquant."
```

et, à la fin du fichier, un bloc `[[artistes]]` par artiste (`nom` identique au champ `artiste` des œuvres, `vie`, `biographie`, `style`), affiché sur la présentation de l'exposition et sur la page de chacune de ses œuvres. Toutes les œuvres actuelles ont leur notice complète, et un test le vérifie.

### Depuis le musée : l'éditeur

Dans la visite 3D, le bouton **Éditer** ouvre l'éditeur : choisir une toile ou une sculpture (ou « L'œuvre regardée »), puis sa salle, son mur, sa position, sa largeur — pour une sculpture : sa position, sa rotation, sa hauteur et son socle. La scène se met à jour aussitôt et l'éditeur signale une place déjà prise, une toile sur une porte ou une sculpture contre un mur. **Télécharger** donne le fichier `expositions/<nom>.toml` modifié (seules les lignes de placement des œuvres déplacées changent), **Copier** le met dans le presse-papiers et **Publier sur GitHub** le copie puis ouvre l'éditeur du fichier dans le dépôt : il reste à coller et à valider.

### Le plan du musée

Sans coordonnées, les salles s'enchaînent en enfilade, reliées par des couloirs. Avec un plan, chaque salle a sa place (`x` vers l'est, `z` vers le sud, en mètres), sa taille et ses portes ; une porte se déclare d'un seul côté et la salle voisine reçoit la sienne :

```toml
[[salles]]
nom = "La grande galerie"
x = 0
z = 0
largeur = 9        # est-ouest
profondeur = 30    # nord-sud
hauteur = 7
portes = ["sud", "nord", { mur = "est", position = -5 }]
```

L'entrée du musée est la première salle qui a une porte vers l'extérieur. `build.rs` vérifie que les salles ne se chevauchent pas, que les portes donnent quelque part et que chaque salle est accessible.

### Les sculptures

Déposer le modèle 3D (STL, OBJ ou glTF) dans [`modeles/`](modeles), puis ajouter un bloc :

```toml
[[sculptures]]
titre = "The Greek Slave"
artiste = "Hiram Powers"
annee = "1843"
modele = "greek-slave.stl"
salle = 1
x = 0              # depuis le centre de la salle, en mètres
z = 4
hauteur = 1.66     # la sculpture, sans le socle
rotation = 0       # en degrés ; 0 : elle regarde le sud
matiere = "marbre" # marbre, platre, pierre, bronze, or ou #rrggbb
socle = 0.6
```

À la compilation, le maillage est redressé, mis à l'échelle, simplifié à 20 000 triangles et compressé ; une vignette est dessinée pour le catalogue.

**Créer une exposition** : copier un fichier de `expositions/` sous un autre nom (par exemple `renoir.toml` : l'exposition sera à l'adresse `/renoir`), créer le dossier `public/tableaux/renoir/` pour ses images, puis changer les textes (section `[exposition]` : titre, artiste, introduction, couleur d'accent, nom dans le menu, ordre sur l'accueil, œuvre de la vitrine), les salles et les œuvres. Elle apparaît d'elle-même sur l'accueil et dans le menu.

Les fichiers sont vérifiés à la compilation (`build.rs`) : salle inexistante, image introuvable, place déjà prise, toiles qui se chevauchent, tableau trop grand, couleur mal écrite… La compilation s'arrête alors avec un message en français qui dit quoi corriger, par exemple :

```
✖ expositions/monet.toml : l'œuvre « Nymphéas » : l'image « nympheaz.jpg » est introuvable dans public/tableaux/monet/
```

et le site en ligne reste tel quel.

## Comment les toiles deviennent de la 3D

Le moteur 3D de Paquets n'affiche pas d'images (pas de textures). À la compilation, `build.rs` réduit donc chaque image à une grille dont chaque case mesure **7 mm** dans la galerie (jusqu'à 340 cases de côté), et la ramène à une palette de **64 couleurs** ; les toiles sombres sont légèrement éclaircies pour rester lisibles sous l'éclairage du musée. Les cases des mosaïques et les maillages des sculptures sont écrits dans un fichier par exposition (`donnees/<nom>.bin`), que le navigateur télécharge à l'ouverture du musée : le WebAssembly reste le même quelle que soit la taille des collections.

Dans la scène, les cases voisines de même couleur sont fusionnées en rectangles, et chaque couleur de la palette forme un seul maillage (64 au plus par tableau) : environ 500 000 triangles pour toute une exposition, ce qu'un téléphone récent affiche sans peine. Les tons clairs sont très légèrement en avant, comme des empâtements. Le catalogue, lui, montre les vraies reproductions.

## Les œuvres (domaine public)

Tous les artistes exposés sont morts en 1955 ou avant, et toutes les œuvres datent d'avant 1931 : elles sont dans le domaine public en Europe comme aux États-Unis. Les reproductions viennent de [Wikimedia Commons](https://commons.wikimedia.org/). Les sculptures sont des scans du [Smithsonian 3D](https://3d.si.edu/) publiés en CC0 (domaine public), réduits pour le navigateur dans [`modeles/`](modeles). Dans la galerie 3D, les toiles sont réinterprétées en mosaïque : ce ne sont pas des reproductions exactes.

Le catalogue complet de chaque exposition est sur le site (`/<exposition>/oeuvres`), avec une page par œuvre ; les cartels, notices et biographies sont dans les fichiers [`expositions/*.toml`](expositions). Une exposition peut réunir plusieurs artistes : chaque œuvre précise alors son `artiste`.

## Architecture

- `expositions/*.toml` : le contenu de chaque exposition (textes, plan, œuvres, sculptures).
- `public/tableaux/<nom>/` : les images des œuvres, servies à `/tableaux/<nom>/…`.
- `modeles/` : les modèles 3D des sculptures.
- `build.rs` (et `build/`) : vérifie les expositions et le plan, transforme les images en mosaïques, simplifie les sculptures, écrit le catalogue en Rust et les fichiers `donnees/<nom>.bin`.
- `src/artworks.rs` : les types du catalogue ; `src/data.rs` : la lecture des fichiers de données.
- `src/museum.rs` : le bâtiment (murs, portes, lampes, obstacles, chemins entre salles).
- `src/tour.rs` : la visite guidée, une fonction pure du temps (`tour_pose`).
- `src/gallery.rs` : les scènes 3D (`paquets::ecs`), la marche libre et l'application des modifications.
- `src/editor.rs` : les modifications de l'éditeur, leur vérification et le fichier TOML réécrit.
- `src/ui.rs` : l'interface de visite et l'éditeur, en egui (`eframe`, sur un `<canvas>` au-dessus de la scène).
- `src/viewer.rs` : le chargement des données, le plein écran, le clavier, la souris et les joysticks.
- `src/lib.rs` : les îlots (musée et vitrine, avec l'exposition en paramètre) et le point d'entrée WebAssembly.
- `src/main.rs` : le serveur et les pages.
- `src/style.rs` : le CSS, sous forme de constante Rust.
- `src/export.rs` : l'export du site en fichiers statiques, pour GitHub Pages.
- `.github/workflows/pages.yml` : compilation, tests, export, vérification et déploiement sur GitHub Pages.

La visite guidée est calculée à partir du plan et des œuvres : depuis le hall d'entrée, les salles sont parcourues de proche en proche par leurs portes, avec un arrêt devant chaque œuvre (d'autant plus loin qu'elle est grande), un détour autour des bancs et des socles, puis le retour à l'entrée. C'est une fonction pure (`tour_pose`), testée sans navigateur : un test vérifie que le visiteur ne traverse jamais un mur.

Deux contraintes du moteur sont prises en compte (et testées) : la physique de `DefaultPlugins` est retirée, sinon chaque pavé (murs, sols, tableaux) tomberait dans le vide ; et le shader ne prend en compte que 8 lampes ponctuelles : seules les 8 lampes les plus proches du visiteur sont allumées, les autres salles gardent la lumière d'ambiance.

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

Le code d'ARTEON est sous licence MIT. Il dépend de [Paquets](https://github.com/maxlestage/Paquets), un logiciel propriétaire qui peut être utilisé librement comme dépendance (voir sa licence). Les œuvres exposées appartiennent au domaine public.
