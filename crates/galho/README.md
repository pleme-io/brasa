# galho (branch) — driver framework

Types drivers share: PCI/USB/virtio enumeration, device cap types, driver lifecycle trait. Individual drivers live in separate repos (`pleme-io/galho-virtio-net`, `galho-virtio-blk`, …) and link against this crate.

See [`../../docs/adrs/0004-tatara-lisp-authoring.md`](../../docs/adrs/0004-tatara-lisp-authoring.md) for the `(defdriver …)` authoring surface.

## Phase 0 shape

- `Driver` trait with `init` / `attach` / `detach`.
- `DeviceHandle` opaque identifier.
- `DriverError` taxonomy.
- `pci` and `virtio` module placeholders.

## Planned first drivers (Phase 2, separate repos)

- `galho-virtio-console`
- `galho-virtio-net`
- `galho-virtio-blk`
- `galho-virtio-9p` (for developer-convenience shared-directory access)
