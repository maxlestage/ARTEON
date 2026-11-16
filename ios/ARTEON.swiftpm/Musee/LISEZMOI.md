# Le musée embarqué

Ce dossier reçoit le site d'ARTEON (catalogue, images, galeries 3D en WebAssembly) que
l'application lit sans Internet. Il n'est pas versionné : `ios/preparer-musee.sh` le
remplit, et le workflow `ios.yml` le fait avant chaque compilation (l'artefact
**ARTEON-swift-playgrounds** le contient déjà).

Vide, l'application marche quand même : elle télécharge alors le catalogue et les images
depuis le site, puis les garde sur l'appareil.
