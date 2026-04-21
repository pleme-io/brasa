# casca (bark) — syscall ABI

The outer typed surface userspace touches. The `Casca` trait is the single source of truth for the syscall list; both kernel and userspace implement it from the same declaration.

See [`../../docs/adrs/0003-syscall-surface.md`](../../docs/adrs/0003-syscall-surface.md).

## Phase 0 shape

- `Casca` trait (empty; Phase 1 populates).
- `SyscallId` enum enumerating the 25 Phase 1 syscall integer IDs.
- `userspace` feature reserved for the `svc`-issuing impl (Phase 1).

## Hard constraint

The count of methods on `Casca` **may not exceed 40** without a new ADR superseding ADR-0003. CI enforces this.
