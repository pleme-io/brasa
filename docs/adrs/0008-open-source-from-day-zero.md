# ADR-0008: Open source from day zero

**Status:** Accepted
**Deciders:** pleme-io
**Date:** 2026-04-20

## Context

There is a pattern where a small team builds a kernel (or any ambitious systems project) privately, iterates to "something good," then open-sources it. The argument is: avoid noise during the hard early design phase, avoid external pressure, avoid premature abandonment.

We explicitly reject this pattern for brasa.

## Decision

**brasa is open-source from the very first commit.** All ADRs, all design docs, all code, all discussion lives in public. The repository is public on GitHub under `pleme-io/brasa` from day zero. No "stealth phase." No private design documents.

License: MIT. Consistent with the pleme-io ecosystem.

## Why

### Kernel projects that succeed do so in public

- Linux was public from 1991.
- seL4 was academic (public).
- Redox has been public since 2015.
- Fuchsia was public from early on.

Kernel projects that went private-first and later tried to open-source (several commercial efforts) mostly failed. Public development accretes contributors, accountability, and reality-testing. Private development accretes assumptions.

### Attestation culture requires public artifacts

brasa's attestation model (ADR-0002) chains back to `tameshi`-signed release artifacts. For that chain to be meaningful, the artifacts must be publicly verifiable — which means the source that built them must be public. Private source + public attestation is theater.

### ADRs work best in public

An ADR is a document arguing for a decision. The argument improves when outsiders can stress-test it. The argument atrophies when only the deciders read it. Our ADRs are where the design lives; putting them in public is how we make sure the design is real.

### Our ecosystem is already public

Every other pleme-io project is public: nix, substrate, alicerce, kasou, sui, tatara, all of it. A closed brasa would be anomalous. It would also be unwelcoming to anyone who finds pleme-io via another project and wants to see the full story.

### We have nothing to hide

The design is the product. We do not have trade secrets. The value is in the types, the discipline, the execution — not in the secrecy.

## Alternatives considered

### Private until Phase 1 (proof-of-concept works)

Considered. The appeal is "we don't want people watching a broken kernel." Rejected because:

1. The broken state *is* the interesting state. Watching us solve problems is educational, not embarrassing.
2. Private development loses the compounding benefit of early external feedback.
3. "Just a few more months" extends. There is no natural opening moment if we start private.

### Private ADR discussion, public code

Rejected. ADRs are the reasoning; code is the consequence. Separating them leaves readers unable to understand why code is shaped how it is. Every ADR is in `docs/adrs/`, in the same repo as the code.

### Private "inner circle" channel + public repo

Considered. An "inner circle" Slack/Discord where maintainers coordinate, with public repo for artifacts. Acceptable for routine coordination (travel, meeting times). **Not** acceptable for design discussions — those happen in PRs, issues, or ADRs on the public repo. If a design discussion starts in DMs, someone says "let's move this to a GitHub discussion."

### Public but CC-BY-NC (non-commercial only)

Rejected. Incompatible with pleme-io's culture (everything else is MIT) and with kernel culture generally. A kernel should be unambiguously usable downstream.

## Consequences

### Good

- Contributors can find us from day one.
- Reality-testing happens continuously. Bad assumptions get challenged early.
- The narrative of "building a capability kernel in the open, with every decision documented" is itself an attractor for the kind of person we want contributing.
- We build a public track record. Future hires, partners, and investors see the actual work.
- Attestation claims are verifiable.

### Bad

- Premature critique. People will show up and say the design is wrong when the design is fine. We respond in the ADRs.
- Typo/WIP embarrassment. Every half-finished document is visible. We accept this.
- Competitive concern: someone else sees our design and ships it first. Unlikely (capability kernels are not a crowded space) and uninteresting (the execution is the product, not the idea).

### Neutral

- Some documents (like naming.md or the README) change often in early weeks. Git history handles this; observers see the evolution.

## What this means in practice

1. The repo is public on GitHub from the first commit.
2. All discussion happens in GitHub issues, PRs, or ADR markdown. No private design channels.
3. The CI badge on README shows current build state honestly. Red CI is a fact, not a shame.
4. Release notes and status updates go on the repo. No "announcements" ahead of the code.
5. Communication about brasa in talks, blog posts, or social media is OK — but must link to the public repo, not hint at private work.

## Verification

1. `gh repo view pleme-io/brasa` shows visibility: PUBLIC. First thing we verify after creating the remote.
2. Every ADR merged by PR, PR is merge-commit visible in history.
3. CI badge in README.md is live.
4. Contributor onboarding doc (to be written Phase 1) tells new contributors to open a GitHub issue, not to email anyone.

## Amendments

None anticipated. If we ever decide to split components of brasa into a private track (e.g., a specific compliance-focused variant for an enterprise customer), that would be a separate project with its own repo and its own ADRs — not an amendment to this one.
