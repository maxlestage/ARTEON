# ARTEON

**Un musée virtuel en 3D, écrit uniquement en Rust.**

[![Déployer sur Heroku](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy?template=https://github.com/maxlestage/ARTEON)

ARTEON est un musée qui n'existe pas. Chaque exposition occupe un bâtiment dessiné sur plan, souvent inspiré d'un vrai musée (la galerie d'honneur du Rijksmuseum, la nef d'Orsay, le Belvédère, la cour du Smithsonian) : les toiles sont accrochées aux murs, les sculptures — de vrais scans 3D — posées sur leurs socles. On s'y promène librement ou on suit la visite guidée, et un éditeur permet de déplacer les œuvres depuis le musée lui-même. Tout est écrit en Rust avec [Paquets](https://github.com/maxlestage/Paquets) : le serveur, les pages HTML, les scènes 3D (`paquets::ecs`) et l'interface de visite ([egui](https://github.com/emilk/egui)), compilées en WebAssembly et rendues en WebGL 2.

51 expositions sont ouvertes (514 œuvres et objets, 8 sculptures en 3D, 408 artistes, ateliers ou cultures), de l'histoire naturelle au street art, plus des expositions éphémères renouvelées chaque semaine (voir plus bas) :

| Exposition | Style | Dates | Artistes ou cultures |
|---|---|---|---|
| [L'Antiquité, des bords du Nil à la baie de Naples](expositions/antiquite.toml) | Antiquité : Égypte, Orient, Grèce, Rome | vers 2090 av. J.-C. – 120 apr. J.-C. | Mésopotamie, époque néo-sumérienne, Assyrie, règne d'Assurnasirpal II, Égypte, Moyen Empire, Égypte, Nouvel Empire, Égypte romaine, Peintre d'Amasis, Peintre de Berlin, Grèce hellénistique, Rome, fin de la République, Ennion |
| [Byzance et l'Orient chrétien, le ciel en or](expositions/byzance.toml) | Byzance et l'Orient chrétien | VIᵉ–XVIIᵉ siècle | Mosaïstes de Ravenne, Byzance, Angelos Akotantos, Égypte copte, Arménie, Éthiopie chrétienne |
| [Le Moyen Âge, l'or et la lumière](expositions/moyen-age.toml) | Art médiéval et gothique | vers 1280–1500 | Tapissiers anonymes, Cimabue, Giotto, Jean Fouquet, Frères de Limbourg, Enguerrand Quarton, Simone Martini, Fra Angelico, Duccio di Buoninsegna, Andreï Roublev |
| [Manuscrits et tapisseries, le Moyen Âge en pages et en fils](expositions/manuscrits-textiles.toml) | Manuscrits enluminés et tapisseries | VIIIᵉ – XVIᵉ siècle | Moines d'Iona et de Kells, Eadfrith de Lindisfarne, Scriptorium de Saint-Sever, Enlumineurs de la Bible de Souvigny, Peintres-scribes aztèques, Atelier anglo-saxon, Jean de Bruges et Nicolas Bataille, Ateliers des Pays-Bas du Sud, Jean Bourdichon, Abraham Cresques |
| [La Renaissance italienne, l'invention de l'homme](expositions/renaissance.toml) | Renaissance italienne | 1460–1538 | Filippo Lippi, Piero della Francesca, Andrea Mantegna, Sandro Botticelli, Léonard de Vinci, Michel-Ange, Raphaël, Giorgione, Titien |
| [Les arts de l'Islam, de Cordoue à Ispahan](expositions/arts-islam.toml) | Arts de l'Islam | IXᵉ–XVIᵉ siècle | Iran safavide, Empire ottoman, Sultanat mamelouk, Céramistes de Kashan et d'Ispahan, Ifriqiya, Califat de Cordoue |
| [Céramiques et porcelaines du monde, la terre et le feu](expositions/ceramiques.toml) | Céramiques et porcelaines du monde | XIIIᵉ–XIXᵉ siècle | Chine, fours de Longquan, Chine, fours impériaux de Jingdezhen, Japon, céramique raku, Pueblo de Zuni, Maestro Giorgio Andreoli, Faïencerie De Metaale Pot, Johann Joachim Kändler, Manufacture de Sèvres, Josiah Wedgwood |
| [Monts, jardins et empereurs : la peinture d'Asie](expositions/arts-asie.toml) | Peinture chinoise, persane et moghole | vers 1000–1630 | Fan Kuan, Guo Xi, Zhao Mengfu, Shen Zhou, Behzad, Sultan Muhammad, Reza Abbasi, Bichitr, Abu'l Hasan, Govardhan |
| [Dieux, bouddhas et princes : l'art de l'Inde et de l'Asie du Sud-Est](expositions/inde-asie-sud-est.toml) | Inde et Asie du Sud-Est | IIIᵉ siècle – 1899 | Gandhara, époque kouchane, Inde du Nord, époque gupta, Inde du Sud, époque chola, Java, dynastie Shailendra, Empire khmer, style du Bayon, Birmanie, royaume de Pagan, Thaïlande, héritage de Sukhothai, Sahibdin, Devidasa de Nurpur, Raja Ravi Varma |
| [Corée et Japon, le vert du céladon et l'or des paravents](expositions/coree-japon.toml) | Corée et Japon, paravents, céladons et rouleaux | XIIe–XVIIIe siècle | Corée, dynastie Goryeo, Corée, dynastie Joseon, Ogata Kenzan, Jeong Seon, Kim Hong-do, Ogata Kōrin, Hasegawa Tōhaku, Tawaraya Sōtatsu, Japon, époque de Heian |
| [Visages des ancêtres : arts d'Afrique, d'Océanie et des Amériques](expositions/afrique-oceanie-ameriques.toml) | Arts d'Afrique, d'Océanie et des Amériques | 900 av. J.-C. – vers 1900 | Ifé, royaume yoruba, Royaume du Bénin, Edo, Dan, Fang, Māori, Elema, golfe de Papouasie, Olmèques, Tairona, Lakota |
| [Primitifs flamands et Renaissance du Nord, le monde au plus près](expositions/renaissance-nord.toml) | Primitifs flamands et Renaissance du Nord | 1434–1568 | Jan van Eyck, Rogier van der Weyden, Hans Memling, Jérôme Bosch, Albrecht Dürer, Lucas Cranach l'Ancien, Hans Holbein le Jeune, Pieter Bruegel l'Ancien |
| [Venise, la couleur et la lumière, de Bellini à Tiepolo](expositions/venise.toml) | Venise, de Bellini à Tiepolo | 1475–1780 | Antonello de Messine, Giovanni Bellini, Vittore Carpaccio, Lorenzo Lotto, Le Tintoret, Paolo Véronèse, Giambattista Tiepolo, Rosalba Carriera, Bernardo Bellotto, Pietro Longhi, Francesco Guardi |
| [Le Baroque, le théâtre de la lumière](expositions/baroque.toml) | Baroque | 1599–1678 | Le Caravage, Artemisia Gentileschi, Georges de La Tour, Pierre Paul Rubens, Bartolomé Esteban Murillo, Francisco de Zurbarán, Diego Velázquez, Nicolas Poussin, Claude Lorrain |
| [Armes et armures, l'art de la guerre](expositions/armes-armures.toml) | Armes et armures | VIIᵉ siècle av. J.-C. – 1819 | Armuriers de la cour de France, Ateliers royaux de Greenwich, Grèce archaïque, Royaume d'Est-Anglie, Japon, époque de Kamakura, Japon, époque d'Edo, Pays-Bas bourguignons, Russie, armuriers de la cour impériale, Empire ottoman, Inde du Nord, Lucknow |
| [Vermeer et Rembrandt, le siècle d'or hollandais](expositions/vermeer-rembrandt.toml) | Siècle d'or hollandais | XVIIᵉ siècle | Rembrandt, Johannes Vermeer |
| [Flandres et Hollande, le siècle d'or du quotidien](expositions/flandres-hollande.toml) | Paysages, natures mortes et scènes de genre des Pays-Bas | 1615–1705 | Carel Fabritius, Jacob Jordaens, Antoine van Dyck, Clara Peeters, Pieter Claesz, Willem Claesz Heda, Jan Davidsz de Heem, Adriaen Coorte, Jacob van Ruisdael, Meindert Hobbema, Frans Hals, Jan Steen, Pieter de Hooch |
| [Joyaux et trésors, quatre mille ans d'orfèvrerie](expositions/joyaux.toml) | Joyaux, orfèvrerie et trésors | vers 1887 av. J.-C. – 1907 | Égypte, Moyen Empire, Rome impériale, Basse-Saxe, orfèvrerie du XIᵉ siècle, Limoges, ateliers d'émaillerie, Allemagne, orfèvres de la Renaissance, Popayán, orfèvres de la Nouvelle-Grenade, Paris, orfèvres-bijoutiers du XVIIIᵉ siècle, Maison Fabergé, René Lalique, Louis Comfort Tiffany |
| [Le siècle des Lumières, du rococo au néoclassicisme](expositions/lumieres.toml) | Rococo et néoclassicisme | 1717–1814 | Antoine Watteau, François Boucher, Jean-Honoré Fragonard, Jean Siméon Chardin, Giovanni Antonio Canal, dit Canaletto, Thomas Gainsborough, Élisabeth Vigée Le Brun, Jacques-Louis David, Jean-Auguste-Dominique Ingres |
| [Art et science, la nature dessinée](expositions/art-et-science.toml) | Art et science : la nature dessinée | 1291–1904 | Albrecht Dürer, John James Audubon, Ernst Haeckel, Maria Sibylla Merian, Pierre-Joseph Redouté, André Vésale, Léonard de Vinci, Yémen, dynastie rasoulide, Abraham Ortelius, Robert Hooke |
| [Instruments de musique du monde, l'art de faire chanter le bois](expositions/instruments-musique.toml) | Instruments de musique du monde | vers 1560–1928 | Michele Todini, Bartolomeo Cristofori, Andrea Amati, Antonio Stradivari, Joseph Benedid, Manol (Emmanuel Venios), Inde du Sud, Atelier des Gotō, Tradition mandingue, Style mangbetu |
| [Mode et costume, deux siècles d'élégance](expositions/mode-costume.toml) | Mode et costume, du XVIIIᵉ siècle aux années folles | 1765–1924 | Couturières anonymes, Charles Frederick Worth, Jacques Doucet, Paul Poiret, Mariano Fortuny, Jeanne Lanvin, Japon, XIXᵉ siècle, Tisserands asante (Ghana) |
| [Le Romantisme, la tempête et le rêve](expositions/romantisme.toml) | Romantisme | 1781–1839 | Johann Heinrich Füssli, William Blake, Francisco de Goya, Théodore Géricault, Eugène Delacroix, Caspar David Friedrich, Joseph Mallord William Turner, John Constable |
| [Ukiyo-e, les images du monde flottant](expositions/estampes-japonaises.toml) | Ukiyo-e, l'estampe japonaise | 1759–1857 | Utagawa Hiroshige, Katsushika Hokusai, Suzuki Harunobu, Kitagawa Utamaro, Tōshūsai Sharaku, Utagawa Kuniyoshi, Itō Jakuchū |
| [Du réalisme à la vie moderne](expositions/realisme.toml) | Réalisme et impressionnisme | 1849–1882 | Édouard Manet, Gustave Courbet, Rosa Bonheur, Jean-François Millet, Gustave Caillebotte, Edgar Degas, Camille Pissarro |
| [Monet, la lumière à l'œuvre](expositions/monet.toml) | Impressionnisme | 1840–1926 | Claude Monet, Hiram Powers, Ferdinand Pettrich, Edmonia Lewis |
| [Renoir, le bonheur de peindre](expositions/renoir.toml) | Impressionnisme | 1841–1919 | Pierre-Auguste Renoir |
| [Femmes peintres, quatre siècles de regards](expositions/femmes-peintres.toml) | Femmes peintres, XVIᵉ–XXᵉ siècle | 1555–1923 | Adélaïde Labille-Guiard, Sofonisba Anguissola, Judith Leyster, Rachel Ruysch, Angelica Kauffman, Berthe Morisot, Eva Gonzalès, Mary Cassatt, Anna Ancher, Suzanne Valadon |
| [L'Amérique au XIXᵉ siècle, peinture et sculpture](expositions/smithsonian.toml) | Art américain du XIXᵉ siècle | 1836–1899 | Thomas Cole, Albert Bierstadt, Frederic Edwin Church, James McNeill Whistler, Thomas Eakins, John Singer Sargent, Mary Cassatt, Winslow Homer, Horatio Greenough, John Rogers, Clark Mills, Chine, dynastie des Qi du Nord |
| [Le Salon, triomphe de l'art académique](expositions/salon-academique.toml) | Le Salon et l'art académique | 1833–1895 | Thomas Couture, Paul Delaroche, Ernest Meissonier, Lawrence Alma-Tadema, Jean-Léon Gérôme, Alexandre Cabanel, William Bouguereau, Frederic Leighton, Elizabeth Jane Gardner, Jules Bastien-Lepage |
| [Les débuts de la photographie, l'empreinte de la lumière](expositions/photographie.toml) | Les débuts de la photographie | 1827–1911 | Nicéphore Niépce, Louis Daguerre, William Henry Fox Talbot, Nadar, Julia Margaret Cameron, Gustave Le Gray, Eadweard Muybridge, Eugène Atget, Alfred Stieglitz, Sergueï Prokoudine-Gorski |
| [Cézanne, la construction de la couleur](expositions/cezanne.toml) | Post-impressionnisme | 1839–1906 | Paul Cézanne |
| [La peinture russe, des Ambulants à l'âge d'argent](expositions/peinture-russe.toml) | Peinture russe, des Ambulants à l'âge d'argent | 1850–1898 | Ilia Répine, Vassili Sourikov, Ivan Aïvazovski, Arkhip Kouïndji, Ivan Chichkine, Isaac Levitan, Viktor Vasnetsov, Mikhaïl Vroubel, Valentin Serov |
| [Europe centrale et orientale, les peintres des nations](expositions/europe-centrale.toml) | Europe centrale et orientale, XIXᵉ siècle | 1862–1912 | Jan Matejko, Mikoláš Aleš, Alfons Mucha, Mihály Munkácsy, Nicolae Grigorescu, Józef Chełmoński, Pál Szinyei Merse, Tivadar Csontváry Kosztka, Jacek Malczewski, Olga Boznańska, Osman Hamdi Bey |
| [Après l'impressionnisme, points, aplats et rêves](expositions/post-impressionnisme.toml) | Néo-impressionnisme, Nabis et art naïf | 1884–1910 | Georges Seurat, Paul Signac, Henri de Toulouse-Lautrec, Paul Sérusier, Émile Bernard, Pierre Bonnard, Édouard Vuillard, Henri Rousseau |
| [Espagne et Amérique latine, les deux rives d'une peinture](expositions/espagne-ameriques.toml) | Espagne et Amérique latine au XIXᵉ siècle | 1851–1913 | Marià Fortuny, Joaquín Sorolla, Ramon Casas, Ignacio Zuloaga, José María Velasco, Saturnino Herrán, Pedro Américo, Almeida Júnior, Prilidiano Pueyrredón, Francisco Oller |
| [Van Gogh, la couleur en mouvement](expositions/van-gogh.toml) | Post-impressionnisme | 1853–1890 | Vincent van Gogh |
| [La sculpture, du marbre de Michel-Ange au bronze de Rodin](expositions/sculpture.toml) | La sculpture, de la Renaissance à Rodin | 1513–1909 | Auguste Rodin, Michel-Ange, Le Bernin, Antonio Canova, Jean-Baptiste Carpeaux, Camille Claudel, Edgar Degas, Antoine Bourdelle, Aristide Maillol |
| [Gauguin, l'ailleurs en couleurs](expositions/gauguin.toml) | Synthétisme et primitivisme | 1848–1903 | Paul Gauguin |
| [La lumière du Nord, peinture scandinave et finlandaise](expositions/peinture-nordique.toml) | La lumière du Nord, peinture scandinave | 1883–1926 | Peder Severin Krøyer, Carl Larsson, Vilhelm Hammershøi, Anna Ancher, Harriet Backer, Anders Zorn, Nikolai Astrup, Eugène Jansson, Akseli Gallen-Kallela, Helene Schjerfbeck, Hugo Simberg |
| [L'image imprimée, de la gravure à l'affiche et à la bande dessinée](expositions/affiches-illustration.toml) | Affiches, gravure et bande dessinée | 1751–1916 | William Hogarth, Francisco de Goya, Katsushika Hokusai, Rodolphe Töpffer, John Tenniel, Gustave Doré, Jules Chéret, Aubrey Beardsley, Théophile-Alexandre Steinlen, Winsor McCay, Leonetto Cappiello, Abel Faivre |
| [Le Symbolisme, rêves, mythes et visions](expositions/symbolisme.toml) | Symbolisme et Art nouveau | 1851–1914 | Alfons Mucha, Pierre Puvis de Chavannes, Gustave Moreau, Odilon Redon, John Everett Millais, Dante Gabriel Rossetti, John William Waterhouse, Arnold Böcklin, Edvard Munch, Fernand Khnopff |
| [De l'Art nouveau à l'Art déco, l'art dans la maison](expositions/arts-decoratifs.toml) | Art nouveau et Art déco, les arts décoratifs | 1897–1926 | Alphonse Mucha, Hector Guimard, Émile Gallé, Daum frères, Tiffany Studios, Charles Rennie Mackintosh, Jacques-Émile Ruhlmann, René Lalique |
| [Klimt, l'or et la vie](expositions/klimt.toml) | Sécession viennoise | 1862–1918 | Gustav Klimt |
| [Fauves et expressionnistes, la couleur libérée](expositions/expressionnisme.toml) | Fauvisme et expressionnisme | 1904–1919 | Henri Matisse, André Derain, Franz Marc, August Macke, Ernst Ludwig Kirchner, Paula Modersohn-Becker, Egon Schiele, Amedeo Modigliani |
| [Les avant-gardes, la naissance de l'abstraction](expositions/abstraction.toml) | Avant-gardes et abstraction | 1907–1930 | Hilma af Klint, Vassily Kandinsky, Robert Delaunay, Juan Gris, Umberto Boccioni, Fernand Léger, Kasimir Malevitch, Piet Mondrian, Paul Klee, Kurt Schwitters |
| [Le pop art dans la ville, des objets géants aux icônes](expositions/pop-art.toml) | Pop art et néo-pop | 1966–2006 | Roy Lichtenstein, Robert Indiana, Claes Oldenburg et Coosje van Bruggen, Keith Haring, Jeff Koons, Niki de Saint Phalle |
| [L'art contemporain, des œuvres pour la ville](expositions/art-contemporain.toml) | Art contemporain | 1976–2010 | Louise Bourgeois, Antony Gormley, Jaume Plensa, Anish Kapoor, Rachel Whiteread, Yinka Shonibare, Fernando Botero, Yue Minjun, Eduardo Chillida, Sokari Douglas Camp |
| [Street art, l'art à ciel ouvert](expositions/street-art.toml) | Street art et graffiti | 1990–2018 | Dmitri Vrubel, Birgit Kinder, Thierry Noir, Victor Ash, Os Gêmeos, Invader, Vhils, Etam Cru, Okuda San Miguel |
| [Bâtir le monde : l'architecture en gravures, dessins et photographies](expositions/architecture.toml) | L'architecture du monde, en images | 1750–1912 | Giovanni Battista Piranesi, Étienne-Louis Boullée, Francis Frith, Sébah & Joaillier, Samuel Bourne, Émile Gsell, Louis-Émile Durandelle, Antoni Gaudí |
| [Trésors de la Terre, minéraux, fossiles et météorites](expositions/tresors-de-la-terre.toml) | Histoire naturelle : minéraux, fossiles et météorites | Il y a 4,5 milliards d'années – aujourd'hui | Gibeon, Namibie, Mines de Golconde, Inde, Massif du Mont-Blanc, France, Coulées de lave de la Serra Geral, Brésil, Schistes de Wheeler, Utah, Falaises de Whitby, Angleterre, Récifs de l'Indo-Pacifique, Gisement de Solnhofen, Allemagne, Forêts d'ambre de la Baltique, Formation de Morrison, Wyoming |

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

### Le bâtiment et les cadres

Une section `[musee]` présente le vrai musée dont le bâtiment s'inspire ; elle s'affiche sur la présentation de l'exposition (« Le bâtiment », avec le plan des salles dessiné en SVG) et dans le musée 3D (bouton **🏛 Le musée**) :

```toml
[musee]
nom = "Musée du Prado"
ville = "Madrid, Espagne"
architecte = "Juan de Villanueva"
construction = "1785–1819"
presentation = """Le vrai musée…"""
ici = """Ce que le bâtiment d'ARTEON lui emprunte, salle par salle…"""
```

Les tableaux ont un cadre en 3D, moulure par moulure : `cadre = "dore"` dans `[exposition]` pour toute l'exposition, ou dans une œuvre pour elle seule. Styles : `dore`, `baroque` (large et sculpté), `ebene` (noir, filet doré, comme au Rijksmuseum), `bois`, `blanc`, `noir` (baguette fine), `estampe` (passe-partout et baguette, pour les estampes et les miniatures) et `aucun`.

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

## Les expositions éphémères

`outils/ephemeres.py` monte chaque semaine des expositions qui changent avec les musées : il interroge les collections ouvertes de l'**Art Institute of Chicago**, du **Cleveland Museum of Art** et du **Metropolitan Museum de New York**, tire au sort douze œuvres parmi celles qu'ils exposent **en ce moment** dans leurs salles (domaine public, photographies publiées en CC0 par le musée), et écrit pour chacun `expositions/ephemere-<ville>.toml` et ses images. Quand un musée change son accrochage, l'exposition change avec lui ; chaque cartel dit dans quelle salle voir l'œuvre en vrai.

- Le workflow GitHub Pages lance le script avant chaque compilation et **chaque lundi** (tâche planifiée) : le site est refait et redéployé avec le tirage de la semaine.
- Un musée qui ne répond pas est sauté ; si le résultat ne se compile pas, les expositions éphémères sont retirées et le reste du site est publié.
- Les fichiers produits ne sont pas versionnés (`.gitignore`). En local : `python3 outils/ephemeres.py` (bibliothèque standard de Python seulement), ou `--graine 42` pour un autre tirage.
- Pour ajouter un musée : une fonction qui renvoie ses œuvres exposées (titre, artiste, date, technique, mesures, salle, image, lien) et une entrée dans `MUSEES`.

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

## Déployer sur Heroku depuis un téléphone

Tout se fait depuis le navigateur du téléphone, avec le [buildpack emk/rust](https://elements.heroku.com/buildpacks/emk/heroku-buildpack-rust) :

1. Appuyer sur le bouton **Deploy to Heroku** en haut de ce README (ou ouvrir `https://heroku.com/deploy?template=https://github.com/maxlestage/ARTEON`), choisir un nom d'application, puis **Deploy app**. Heroku lit [`app.json`](app.json) (pile heroku-26, buildpack `emk/rust`) ; le buildpack installe Rust et lance [`heroku-build.sh`](heroku-build.sh) par [`RustConfig`](RustConfig) : les expositions éphémères de la semaine, le serveur, les îlots 3D en WebAssembly et les données des expositions. Le [`Procfile`](Procfile) démarre `target/release/arteon`, qui écoute sur `$PORT`.
2. Déploiements automatiques : dans GitHub (l'application mobile ou le site), *Settings → Secrets and variables → Actions*, ajouter le secret `HEROKU_API_KEY` (Heroku → *Account settings* → *API Key* → *Reveal*) et la variable `HEROKU_APP_NAME` (le nom choisi). Le workflow [`heroku.yml`](.github/workflows/heroku.yml) déploie alors chaque push sur `master`, et **chaque lundi** pour renouveler les expositions éphémères, par l'API de Heroku (sans ligne de commande). Il crée l'application si elle n'existe pas et lui met le buildpack `emk/rust`. On peut aussi le lancer à la main : *Actions → Déployer sur Heroku → Run workflow*.
3. Variable de configuration facultative (Heroku → *Settings → Config Vars*) : `EPHEMERES=0` pour ne pas monter les expositions éphémères. `REPOS_TOKEN` (un jeton GitHub) ne sert que si Paquets devenait privé.

`build.rs` garde un cache des expositions déjà calculées (`arteon-cache`, dans le dossier de compilation que le buildpack conserve d'un déploiement à l'autre) : seules les expositions modifiées sont recalculées. Le premier déploiement prend plusieurs minutes ; les suivants sont bien plus rapides. Heroku n'a pas d'offre gratuite (un dyno Eco coûte environ 5 $ par mois).

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
