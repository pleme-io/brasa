# brasa — naming

Every name in brasa is a deliberate choice. The top-level project and the
kernel-layer crates use **Brazilian-Portuguese**, following the pleme-io
convention for Tier-2 concepts (enclosed spaces, flows, growth, craft, fire).

## The umbrella: brasa

**brasa** (BRAH-za) — *ember, live coal*. The persistent fire of a running system. Once a kernel boots, brasa is always burning — a CPU never stops executing, a scheduler never fully sleeps. A kernel is the live coal that keeps everything else warm. When the machine halts, the brasa goes out.

Sibling imagery:
- **alicerce** (bedrock, passive, foundational) — installs the ground.
- **brasa** (ember, active, alive) — burns on the ground.

Other pre-authorized Tier-2 names noted in `pleme-io/CLAUDE.md`: **fagulha** (spark), **morada** (dwelling). Reserved for future projects.

## The kernel layers: a tree

The kernel-layer crate names form a coherent arboreal metaphor. A tree is a running system: rooted in `raiz` (root), drawing sap through `seiva`, wrapped in `casca` (bark) at the interface with the world, branching into `galho` (branches) that bear `folha` (leaves), with `tronco` (the trunk) holding everything upright. New systems grow from `semente` (a seed). The whole thing leaves a chain back to its roots via `raizame`.

### `semente` — the seed (bootloader)

*semente* (seh-MEN-chee) — *seed*. Every tree starts here. The bootloader is literally the seed of the running system: a tiny artifact that, when planted (loaded by firmware), grows into a full kernel. A seed has minimal genetics but encodes everything the tree will become.

### `tronco` — the trunk (kernel core)

*tronco* (TRON-koo) — *trunk*. The central structural mass of the tree. Everything else hangs off the trunk; branches, leaves, and roots all draw their authority and structure from it. The trunk is what you hold onto when the storm comes. The trunk is what remains when the leaves fall.

### `raiz` — the root (capability system)

*raiz* (hah-EEZ) — *root*. The anchor in the earth. Where authority comes from. A capability is a root-fact: "this process has the right to touch this object, and here is the chain of reasoning, traceable to the first fact the bootloader granted." Roots do not get granted; they come from the beginning. New capabilities grow out of existing ones by derivation — the way a tree's secondary roots grow out of a primary taproot.

### `seiva` — the sap (IPC)

*seiva* (SAY-va) — *sap*. The fluid that carries nutrients between parts of the tree. IPC is the same: it carries typed messages (and capabilities, which are nutrients for authority) between processes. Without seiva, the tree dies — without IPC, a microkernel cannot function. Sap is always typed in its chemistry; seiva is always typed in its protocols.

### `casca` — the bark (syscall ABI)

*casca* (KAHS-ka) — *bark*. The protective outer layer of the tree, the interface between the living tissue inside and the world outside. Userspace touches the kernel only through casca. Casca is what gets scraped when a process misbehaves; the kernel's heartwood stays safe. Bark is also the tree's most stable interface — it changes slowly, carefully, with versioning.

### `galho` — the branch (driver framework)

*galho* (GA-lyoo) — *branch*. A tree's drivers. Each galho extends from the trunk outward, reaching into the environment to interact with a specific part of the world (a device, a bus, a hardware interface). Branches bear leaves (services) that draw from the environment and return nutrients. Branches can be grafted, pruned, and grow independently.

### `folha` — the leaf (process abstraction)

*folha* (FO-lya) — *leaf*. An individual process, short-lived or long-lived, that participates in the running system. Leaves photosynthesize — they take energy from the environment and return work to the tree. When a leaf falls (a process exits), the tree continues. Leaves are cheap and plentiful; a healthy tree sheds and grows them continuously.

### `raizame` — the root-chain (attestation)

*raizame* (hah-ee-ZAH-mee) — a compound of *raiz* (root) and *ame* (*weave, chain, net* — Japanese borrowing, attested in Brazilian-Portuguese through Japanese-Brazilian vocabulary). The woven net of roots. BLAKE3 attestation traces every running process back through the boot chain to `semente` and thence to signed release artifacts. It is the woven proof of provenance — no process can claim to be running without having its root-chain computable.

(This one is the most linguistically constructed of the names. It is a neologism in service of a specific metaphor. If a native speaker finds it jarring, we can rename to `enxerto` — *graft* — which captures the composition aspect but loses the root/chain imagery. Noted as a possible rename in [ADR-0004 appendix](./adrs/0004-tatara-lisp-authoring.md#open-questions).)

## Userspace-side names (separate repos, planned)

- **floresta** (fah-LOHS-ta) — *forest*. The init system / service orchestrator. Where the forest of services lives.
- **enxerto** (en-SHEHR-too) — *graft*. Planned: runtime driver attachment.
- **cupim** (koo-PEEN) — *termite*. Planned: memory reaper / garbage collection of unreachable caps.
- **capim** (ka-PEEN) — *grass*. Planned: userspace-scheduled micro-services carpeting the space between major trees.
- **barro** (BAH-ho) — *clay*. Planned: raw memory before typing — the material from which `MemCap` is formed.

## Japanese vs Brazilian naming rule

Per the pleme-io CLAUDE.md: Japanese names stay on existing crates (garasu, egaku, madori, etc.); Brazilian names attach to *new* Tier-2+ concepts. brasa is a new OS, so everything here is Brazilian-Portuguese by design. We do not rename existing Japanese crates when they get ported to brasa — `garasu` remains `garasu` even when compiled against `folha::rt` instead of `std`.

## How to propose a name

Open an ADR. Argue:

1. What does the concept do?
2. What natural-world or cultural metaphor captures the concept?
3. What's the Portuguese word (or neologism, with justification)?
4. Does it collide with anything in the pleme-io or public-Rust namespace?

We avoid names that are already crates on crates.io (even unrelated ones) when practical — it makes `grep` and search easier. Prefer distinctive words over generic ones.
