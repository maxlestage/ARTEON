# ARTEON

**Un musée virtuel en 3D, écrit uniquement en Rust.**

ARTEON est un musée qui n'existe pas. Chaque exposition occupe un bâtiment dessiné sur plan, souvent inspiré d'un vrai musée (la galerie d'honneur du Rijksmuseum, la nef d'Orsay, le Belvédère, la cour du Smithsonian) : les toiles sont accrochées aux murs, les sculptures — de vrais scans 3D — posées sur leurs socles. On s'y promène librement ou on suit la visite guidée, et un éditeur permet de déplacer les œuvres depuis le musée lui-même. Tout est écrit en Rust avec [Paquets](https://github.com/maxlestage/Paquets) : le serveur, les pages HTML, les scènes 3D (`paquets::ecs`) et l'interface de visite ([egui](https://github.com/emilk/egui)), compilées en WebAssembly et rendues en WebGL 2.

Huit expositions sont ouvertes (77 toiles et 8 sculptures) :

- **Van Gogh, la couleur en mouvement** — douze toiles de Vincent van Gogh (1885–1890), des *Mangeurs de pommes de terre* au *Champ de blé aux corbeaux* ;
- **Monet, la lumière à l'œuvre** — douze toiles de Claude Monet (1866–1922), de *Femmes au jardin* et *Impression, soleil levant* jusqu'aux *Nymphéas* et au *Pont japonais* ;
- **Renoir, le bonheur de peindre** — dix toiles de Pierre-Auguste Renoir (1874–1892), de *La Loge* au *Bal du moulin de la Galette* et aux trois grandes *Danses* ;
- **Cézanne, la construction de la couleur** — neuf toiles de Paul Cézanne (1877–1906), des *Joueurs de cartes* aux *Grandes Baigneuses* ;
- **Gauguin, l'ailleurs en couleurs** — neuf toiles de Paul Gauguin (1888–1898), de la *Vision après le sermon* à *D'où venons-nous ?* ;
- **Klimt, l'or et la vie** — sept toiles de Gustav Klimt (1900–1915), d'*Attersee* au *Baiser* ;
- **Vermeer et Rembrandt, le siècle d'or hollandais** — dix toiles (1632–1668), de la *Leçon d'anatomie* à *La Jeune Fille à la perle* et à *La Ronde de nuit*, dans une galerie d'honneur inspirée du Rijksmuseum ;
- **L'Amérique au XIXᵉ siècle, peinture et sculpture** — huit toiles (Cole, Church, Bierstadt, Whistler, Eakins, Sargent, Cassatt, Homer) et cinq sculptures (le *George Washington* de Greenough, deux groupes de John Rogers, le masque de Lincoln, un Bouddha de la dynastie des Qi du Nord), autour d'une cour couverte inspirée du Smithsonian.

L'exposition Monet est installée dans une nef inspirée du musée d'Orsay, avec trois sculptures américaines contemporaines de ses toiles (*The Greek Slave* de Hiram Powers, *The Dying Tecumseh* de Ferdinand Pettrich, *Old Arrow Maker* d'Edmonia Lewis).

Seuls des peintres du **domaine public** (morts depuis plus de 70 ans) sont exposés : c'est pourquoi il n'y a pas, par exemple, d'exposition Picasso (mort en 1973, ses œuvres sont protégées jusqu'en 2044).

## Les pages

| Route | Contenu |
|---|---|
| `/` | L'accueil du musée : le concept, les chiffres clés, les expositions et comment c'est fait |
| `/<exposition>` | La présentation d'une exposition (`/van-gogh`, `/monet`, `/renoir`, `/cezanne`, `/gauguin`, `/klimt`, `/vermeer-rembrandt`, `/smithsonian`) : une toile sur un chevalet tournant en 3D, le parcours salle par salle, des reproductions |
| `/<exposition>/galerie` | Le musée en 3D : visite guidée ou libre, plan, éditeur |
| `/<exposition>/oeuvres` | Le catalogue : chaque toile et chaque sculpture, son cartel (technique, collection, note) et sa place dans le musée |
| `/ajouter` | Le guide pour ajouter, déplacer ou modifier une œuvre, ou créer une exposition, avec le plan des emplacements libres |
| toute autre adresse | Une page « introuvable » (404) |

## Dans le musée 3D

- **Toucher la scène** (ou le bouton **⛶**, ou la touche `F`) la met en plein écran. Sur iPhone, où le navigateur ne propose pas le plein écran aux pages web, le musée occupe tout l'écran.
- L'interface de visite est dessinée avec **egui**, par-dessus la scène : une barre d'outils, le cartel de l'œuvre regardée (avec « Voir en HD » pour les toiles), le plan et la liste des œuvres.
- **Visite guidée** : la caméra part du hall d'entrée, traverse les salles par leurs portes, contourne bancs et sculptures et s'arrête devant chaque œuvre. **‹ ❚❚ ›** (flèches, `Espace`) passent à l'œuvre précédente, mettent en pause, passent à la suivante.
- **Visite libre** (bouton « Libre » ou touche `V`) : `Z Q S D` ou `W A S D` pour marcher, les flèches pour tourner, `Maj` pour courir, la souris (glisser) pour regarder ; sur téléphone, un joystick pour avancer, l'autre pour regarder. On ne traverse ni les murs, ni les socles, ni les bancs.
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

Tous les peintres exposés sont morts depuis plus de 70 ans : Rembrandt (1669), Vermeer (1675), Van Gogh (1890), Gauguin (1903), Cézanne (1906), Klimt (1918), Renoir (1919) et Monet (1926). Leurs œuvres sont dans le domaine public, comme celles des peintres américains de l'exposition Smithsonian (tous morts avant 1930). Les reproductions viennent de [Wikimedia Commons](https://commons.wikimedia.org/). Les sculptures sont des scans du [Smithsonian 3D](https://3d.si.edu/) publiés en CC0 (domaine public), réduits pour le navigateur dans [`modeles/`](modeles). Dans la galerie 3D, les toiles sont réinterprétées en mosaïque : ce ne sont pas des reproductions exactes.

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

**Renoir, le bonheur de peindre**

| Salle | Œuvre | Année | Collection |
|---|---|---|---|
| I · Les années impressionnistes | La Loge | 1874 | The Courtauld, Londres |
| | La Fillette à l'arrosoir | 1876 | National Gallery of Art, Washington |
| | Bal du moulin de la Galette | 1876 | Musée d'Orsay, Paris |
| II · La vie moderne | Le Déjeuner des canotiers | 1880–1881 | The Phillips Collection, Washington |
| | Deux Sœurs (Sur la terrasse) | 1881 | Art Institute of Chicago |
| | Les Parapluies | vers 1881–1886 | National Gallery, Londres |
| III · Danses et jeunes filles | Danse à Bougival | 1883 | Museum of Fine Arts, Boston |
| | Danse à la ville | 1883 | Musée d'Orsay, Paris |
| | Danse à la campagne | 1883 | Musée d'Orsay, Paris |
| | Jeunes filles au piano | 1892 | Musée d'Orsay, Paris |

Les cartels complets (technique, collection, note) des quatre dernières expositions sont dans leurs fichiers : [`cezanne.toml`](expositions/cezanne.toml), [`gauguin.toml`](expositions/gauguin.toml), [`klimt.toml`](expositions/klimt.toml), [`vermeer-rembrandt.toml`](expositions/vermeer-rembrandt.toml). Une exposition peut réunir plusieurs peintres : chaque œuvre précise alors son `artiste`.

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
