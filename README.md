# Pamplemouche-OS (FreeBSD Edition)

Pamplemouche-OS est désormais une distribution personnalisée basée sur FreeBSD avec une expérience graphique inspirée de macOS (100% composants open source).

## Périmètre

- Le noyau micro-kernel Rust historique est abandonné dans cette branche.
- La cible est une image FreeBSD bootable (`.img` + `.iso`) avec environnement desktop "macOS-like".
- Aucun composant propriétaire Apple/macOS n'est inclus.

## Arborescence principale

- `/freebsd/release`: paramètres de build FreeBSD release.
- `/distribution/packages`: manifeste des paquets installés.
- `/distribution/config`: configuration système (rc, loader, sysctl, lightdm).
- `/distribution/ui`: profil UI (Openbox + Tint2 + Plank + Rofi + centre de configuration).
- `/scripts`: build image, post-install UI, packaging, validation.

## Prérequis

### Local (Linux/macOS)

- `make`
- `sh`

> Le build bootable complet nécessite un hôte FreeBSD. Sous Linux/macOS, exécutez uniquement la validation du layout.

### Build image complet (FreeBSD)

- FreeBSD 14.1+
- Arbre des sources installé (`/usr/src/release`)
- Outils: `mkimg`, `makefs`, `pkg`

## Commandes

```sh
# Vérifie la structure et les scripts
make validate

# Construit les images bootables sur FreeBSD
TAG=dev make build

# Package ISO/IMG/checksum en tar.gz
TAG=dev make package

# Vérifie la présence des composants UI
make test

# Vérifie les checksums des artefacts générés
make smoke
```

## Lancer en VM

Exemple QEMU (depuis un artefact généré) :

```sh
qemu-system-x86_64 \
  -m 4096 \
  -drive if=virtio,file=artifacts/pamplemouche-os-dev.img,format=raw \
  -serial mon:stdio
```

## CI/CD

Le workflow `.github/workflows/build-iso.yml` exécute :

1. `validate`: validation structure/scripts,
2. `build-image`: build FreeBSD dans VM,
3. `release`: publication automatique de l’ISO sur tag `v*`.

## Conformité légale

Le look & feel est inspiré de macOS uniquement par assemblage de composants libres (Openbox, Tint2, Plank, Rofi, thèmes open source). Aucun binaire, framework, API privée ou ressource graphique propriétaire Apple n'est embarqué.

## UX desktop

- `Super+Espace`: ouvre le lanceur d'applications.
- `Super+,`: ouvre le centre de configuration.
- `Super+Entrée`: ouvre le terminal.
- Les préférences utilisateur sont stockées dans `~/.config/pamplemouche-desktop/settings.conf`.
