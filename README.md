# Pamplemouche-OS

Un système d'exploitation souverain "from scratch" — noyau micro-kernel x86-64 en Rust avec couche de compatibilité Darwin/macOS.

---

## Architecture

```
Pamplemouche-OS/
├── kernel/          # Micro-noyau (no_std, x86-64)
│   └── src/
│       ├── arch/x86_64/
│       │   ├── gdt.rs          GDT + TSS
│       │   └── interrupts.rs   IDT + PIC 8259 (timer, clavier)
│       ├── memory/
│       │   ├── frame_allocator.rs  Allocateur de frames physiques
│       │   ├── paging.rs           Gestion des tables de pages
│       │   └── allocator.rs        Tas kernel (linked-list)
│       ├── scheduler/
│       │   ├── mod.rs          Ordonnanceur préemptif round-robin
│       │   └── task.rs         Descripteur de tâche + contexte CPU
│       ├── ipc/
│       │   └── mod.rs          IPC par passage de messages (ports Mach)
│       ├── vga.rs              Driver console VGA 80×25
│       └── serial.rs           UART 16550 (debug)
└── compat/          # Couche de compatibilité Darwin (no_std)
    └── src/
        ├── macho/
        │   └── loader.rs       Parseur Mach-O 64 bits (zero-copy)
        └── darwin/
            └── syscalls.rs     Table + dispatch des appels système Darwin/XNU
```

## Composants clés

### Micro-noyau (`kernel`)

| Module | Rôle |
|--------|------|
| `arch/x86_64/gdt` | Table de descripteurs globale, segments noyau/utilisateur, TSS |
| `arch/x86_64/interrupts` | IDT — exceptions CPU + IRQ matérielles (PIC 8259) |
| `memory/frame_allocator` | Allocateur de frames 4 Kio depuis la memory-map du bootloader |
| `memory/paging` | Mapping/unmapping de pages virtuelles via `OffsetPageTable` |
| `memory/allocator` | Tas noyau de 1 Mio (extensible) avec `linked_list_allocator` |
| `scheduler` | Ordonnanceur préemptif round-robin déclenché par IRQ0 (timer) |
| `ipc` | Ports de type Mach — envoi/réception de messages entre tâches |

### Couche de compatibilité (`compat`)

| Module | Rôle |
|--------|------|
| `macho/loader` | Lecture zero-copy des en-têtes, load commands et segments Mach-O 64 |
| `darwin/syscalls` | Table complète des syscalls BSD Darwin + dispatcher `darwin_syscall()` |

---

## Prérequis

| Outil | Version minimale |
|-------|-----------------|
| Rust nightly | ≥ 1.97-nightly |
| QEMU | ≥ 8.x |
| `bootimage` | `cargo install bootimage` |

```sh
rustup toolchain install nightly
rustup component add rust-src llvm-tools-preview
cargo install bootimage
```

## Compilation

```sh
# Compiler le noyau (cible x86_64-unknown-none)
make build

# Lancer dans QEMU (nécessite bootimage + qemu-system-x86_64)
make run

# Tests unitaires de la couche compat (cible hôte)
make test
```

---

## Feuille de route

- [x] Boot + GDT + IDT + PIC
- [x] Allocateur de frames physiques
- [x] Gestion des tables de pages virtuelles
- [x] Tas noyau
- [x] Ordonnanceur préemptif (round-robin, timer IRQ)
- [x] IPC par messages (ports Mach)
- [x] Driver VGA et UART
- [x] Parseur Mach-O 64 bits
- [x] Table des syscalls Darwin/XNU
- [ ] Changement de contexte assembleur complet (registres flottants + FPU)
- [ ] Support multi-cœur (APIC / SMP)
- [ ] VFS minimal (ramfs)
- [ ] Espace utilisateur + appels système natifs
- [ ] Chargeur d'exécutables Mach-O (mapping segments, dyld)
- [ ] Traduction complète des syscalls Darwin (read/write/mmap/…)
- [ ] Pilotes open-source : virtio-net, virtio-blk, e1000

---

## Licence

Ce projet est développé entièrement à partir de zéro en utilisant exclusivement
des frameworks et pilotes open-source. Aucun code propriétaire Apple/macOS n'est
inclus ou dérivé. La couche de compatibilité Darwin se base sur la documentation
publique des ABI et sur les sources XNU publiées par Apple sous licence APSL-2.0.
