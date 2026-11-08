# ARTEON

**Une galerie d'art en 3D, écrite uniquement en Rust.**

ARTEON est un musée virtuel. Il contient trois salles d'exposition reliées par deux couloirs. Une caméra fait la visite guidée en boucle et s'arrête devant chaque œuvre. Tout est écrit en Rust avec [Paquets](https://github.com/maxlestage/Paquets) : le serveur, les pages HTML et la scène 3D, qui est compilée en WebAssembly et rendue en WebGL 2.

## Les pages

| Route | Contenu |
|---|---|
| `/` | La page de présentation : sculpture 3D animée, cube en CSS 3D, titre animé lettre par lettre, chiffres clés, parcours, aperçu des œuvres et grand pied de page animé |
| `/galerie` | La visite guidée en 3D des trois salles et de leurs couloirs |
| `/oeuvres` | Le catalogue, avec un cartel et une vignette pour chaque œuvre |

Le pied de page contient une bande De Stijl animée, un défilé des œuvres exposées, quatre colonnes de liens et les mentions sur le domaine public.

## Le parcours

1. **Salle I · De Stijl** : Mondrian et van Doesburg.
2. *Couloir* : un passage étroit, avec un liseré lumineux au pied des murs.
3. **Salle II · Suprématisme** : Malevitch, et un « architectone » blanc qui tourne au centre de la salle.
4. *Couloir*
5. **Salle III · Bauhaus, Dada et finale** : Klee, Taeuber-Arp, puis un dernier Mondrian sur le mur du fond.

## Les œuvres (domaine public)

Tous les artistes sont morts depuis plus de 70 ans : leurs œuvres sont dans le domaine public en Europe. Le moteur 3D de Paquets n'affiche que des formes et des couleurs, pas d'images. Chaque tableau est donc **réinterprété en aplats géométriques en léger relief**. Ce ne sont pas des reproductions exactes.

| Œuvre | Artiste | Année |
|---|---|---|
| Composition avec grand plan rouge, jaune, noir, gris et bleu | Piet Mondrian (1872–1944) | 1921 |
| Composition en rouge, jaune, bleu et noir | Piet Mondrian | 1921 |
| Composition XI | Theo van Doesburg (1883–1931) | 1918 |
| Composition VIII (La Vache) | Theo van Doesburg | 1918 |
| Carré noir | Kasimir Malevitch (1879–1935) | 1915 |
| Supremus n° 58 | Kasimir Malevitch | 1916 |
| Château et soleil | Paul Klee (1879–1940) | 1928 |
| Composition verticale-horizontale | Sophie Taeuber-Arp (1889–1943) | 1916 |
| Composition avec rouge, bleu et jaune | Piet Mondrian | 1930 |

## Architecture

- `src/main.rs` : le serveur, avec la mise en page, le pied de page et les trois pages.
- `src/lib.rs` : les îlots (galerie et vitrine) et le point d'entrée WebAssembly.
- `src/artworks.rs` : le catalogue, avec les œuvres, leurs aplats, les salles et les dimensions.
- `src/gallery.rs` : les scènes 3D (`paquets::ecs`) et la visite guidée.
- `src/style.rs` : le CSS, sous forme de constante Rust.

La scène 3D utilise le moteur de Paquets : des pavés (`Cuboid`), des matériaux colorés, des lampes (`PointLight`) et une caméra (`Camera3d`) déplacée par un système à chaque image. La visite guidée est une fonction pure (`tour_pose(t)`), testée sans navigateur. Les vignettes du catalogue sont générées à partir des mêmes aplats que la scène 3D. Les animations de la page respectent `prefers-reduced-motion`.

## Lancer le projet

- Installer la commande `paquets` : `cargo install --git https://github.com/maxlestage/Paquets paquets --locked`
- Lancer en développement (http://localhost:3000, avec rechargement) : `paquets dev`
- Lancer les tests de la visite guidée : `cargo test`
- Compiler pour la production (serveur et bundle WebAssembly optimisés) : `paquets build`

## Licence

Le code d'ARTEON est sous licence MIT. Il dépend de [Paquets](https://github.com/maxlestage/Paquets), un logiciel propriétaire qui peut être utilisé librement comme dépendance (voir sa licence). Les œuvres d'origine appartiennent au domaine public.
