# ADR-0005: The Nix store is the filesystem

**Status:** Accepted
**Deciders:** pleme-io
**Date:** 2026-04-20

## Context

Unix has `/etc`, `/usr`, `/var`, `/home`, `/tmp`. These exist because in 1970 a filesystem was the only abstraction for "named persistent data." The consequences have been severe: path-traversal bugs, configuration drift, `/etc/shadow` leaks, ambient-authority filesystem opens, and an entire industry's worth of defensive code to sanitize filenames.

NixOS already moved most of the problem: `/etc`, `/usr`, and `/nix/store` are the interesting parts, and `/nix/store` is content-addressable, immutable, and rebuildable from a spec. `/etc` is a set of symlinks into the store. `/usr` is mostly vestigial.

brasa takes this to its logical conclusion. There is no `/etc`. There is no `/usr`. There is no `/home`. There is `/nix/store/` and typed mutable blobs.

## Decision

brasa has exactly two durable namespaces:

1. **The Nix store (`/nix/store/`)** — content-addressable, read-only, served to processes as `Cap<StorePathCap, Read>` via the `jabuti-store` service.
2. **Typed mutable blobs** — served to processes as `Cap<MutableBlobCap, ReadWrite>`. Each blob is named by BLAKE3 hash of its initial content + owner identity, not by path.

There is no ambient writable directory. A service that wants to persist state receives a `MutableBlobCap` from its parent at spawn time; it does not "open a file at a path."

### The `StorePathCap`

```rust
pub struct StorePathCap {
    path_hash: StoreHash,     // BLAKE3 of the canonical store path
    content_hash: BlakeHash,  // BLAKE3 of the path's contents (for verification)
}
```

`store_open(cap) -> Cap<MemCap, Read>` maps the store path as read-only memory. The kernel verifies that the `content_hash` matches what it observes before returning the `MemCap`.

This means:

- A service reading a config file does so via an IPC call to `jabuti-store` that returns a `StorePathCap`, which it then `store_open`s.
- "Opening" a file is two typed syscalls on typed caps, never a path string.
- Two different store paths with the same content are the same cap (content-addressed).

### The `MutableBlobCap`

```rust
pub struct MutableBlobCap {
    blob_id: BlakeHash,   // BLAKE3(initial_content || owner_chain_link)
    current_hash: BlakeHash,  // updated on every write
    max_size: u64,
    owner: ChainLink,
}
```

A service creates a mutable blob by holding a `BlobFactoryCap` granted at spawn time. The blob is backed by a persistent storage service (a `galho-*` block driver + a blob allocator) — not by a filesystem with paths.

Reads return `Cap<MemCap, Read>` with a point-in-time snapshot. Writes go via typed IPC (`blob_write(blob_cap, offset, data)`) — direct writes to the mmap'd `MemCap` are not supported, because that would require maintaining an always-coherent view.

### What about traditional "config files"?

A traditional config file (`/etc/hanabi/config.yaml`) becomes a `StorePathCap` granted to the service at spawn. The `(defservice hanabi …)` form declares:

```lisp
:caps-granted [(store-read "/nix/store/def…-hanabi-config")]
```

The service receives a cap to that specific store path. It has no way to read any other store path, let alone any traditional config location. Ambient config-lookup (`getenv`, `/etc/<service>`, `$XDG_CONFIG_HOME`) does not exist.

### What about `$HOME` and user data?

Users are not a kernel concept in brasa. There is no UID, no `/home`. A "user session" is a service graph spawned by an authentication service (`kenshou-on-brasa`, planned Phase 5) that holds the user's identity caps. User data is mutable blobs owned by that session; the session grants access to apps.

This is a huge departure from Unix. It's deliberate. UIDs and `$HOME` are the other major source of ambient authority that we reject.

### What about `/tmp`?

Services that need ephemeral memory use anonymous `MemCap`s. Ephemeral blobs shared between processes are created via `blob_create_ephemeral` (consumes no persistent storage). `/tmp` as a filesystem path does not exist.

## Alternatives considered

### Keep a traditional rootfs

Rejected. The entire point of brasa is to eliminate ambient filesystem authority. Keeping `/etc` would keep the problem.

### 9P-style per-process namespaces

Plan 9 gave every process its own view of the filesystem, which is elegant. Considered and partly adopted: the cap bag is analogous to the per-process namespace. We deviate from Plan 9 in that brasa caps are typed (not path-addressed) and unforgeable (not path-derivable).

### Fuchsia-style component filesystems

Fuchsia gives each component a sandboxed filesystem namespace rendered from its manifest. Considered and partly adopted — our `(defservice …)` manifest is analogous to Fuchsia's .cm files. The difference: Fuchsia still uses paths inside the namespace; we use cap handles. A brasa service never sees a path; it sees a `StorePathCap`.

### Keep the Nix store, but add a small `/etc`

Considered. A minimal `/etc` with just `resolv.conf` and `passwd` would solve the most common complaints. Rejected because the precedent is catastrophic — once a minimal `/etc` exists, every port of every Unix service will demand additions to it. The discipline of "no paths" has to be absolute or it isn't a discipline.

## Consequences

### Good

- No path-traversal bugs. `../../etc/passwd` is a nonsensical string; there is no path parser that could be confused by it.
- No confused-deputy filesystem attacks.
- Configuration is content-addressable; a service running with a specific `StorePathCap` provably consumes that content and nothing else.
- `kensa` compliance over stored config is trivial: the set of `StorePathCap`s a service holds is enumerable and typed.
- Disaster recovery is simple: the entire system state is the set of `(defsystem …)` forms + the set of live `MutableBlobCap`s. Rebuild from those and you have the same system.

### Bad

- Every Unix program needs porting. A program that does `fopen("/etc/myservice.conf", "r")` fundamentally cannot work unchanged. The `folha::rt` runtime provides no such function.
- Debugging is different. `ls /etc` doesn't work. `cat /proc/cmdline` doesn't work. We build typed introspection tools (Phase 2) that query the cap graph and the blob registry directly.
- Existing NixOS modules are close — NixOS already builds configs into store paths — but not identical. Porting a NixOS module means regenerating its `(defservice …)` form with explicit cap grants.

### Neutral

- On-disk representation of mutable blobs is in our hands. The storage service (running on a `galho-virtio-blk` or similar) can use any scheme; we will probably use a BLAKE3-addressed content-store with a small metadata journal.

## Verification

1. Compile-time check: `casca` syscalls accept `Cap<StorePathCap>`, never strings. Grep-able invariant.
2. Integration test: boot brasa with a `(defservice echo-config …)` that reads a store-path config; verify the service receives the expected content. Add a second service without the cap; verify it cannot access that path.
3. No `/etc` or `/tmp` path parser anywhere in the kernel. CI check: `grep -r "/etc\|/tmp\|/home" crates/tronco/ crates/raiz/` must return zero hits.
4. Proptest in `raizame`: a service's attestation chain over its held caps + the blob state it has written is totally determined by the `(defsystem …)` form + blob history.

## Open questions

- **Initial "boot blobs":** how does the system initialize blobs that must exist before the first service runs (e.g., host identity key)? Current plan: `semente` produces an initial set of typed blobs from a signed artifact; `floresta` inherits the caps at spawn. ADR to be written in Phase 1.
- **Interoperation with non-brasa nodes:** a brasa node that receives a file over the network (via a network service) must typed-parse it into a blob. There is no "just save the bytes to disk" path. Tools to make this ergonomic are Phase 2 work.

## Amendments

None yet.
