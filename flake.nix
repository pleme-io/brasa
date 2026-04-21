{
  description = "brasa (ember) — capability-native microkernel";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    substrate = {
      url = "github:pleme-io/substrate";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  # Phase 0 note: this flake provides the dev shell only. Kernel image builds
  # (brasa-image, brasa-qemu, brasa-kasou) arrive in Phase 1 once tronco has a
  # bootable entry point. See docs/roadmap.md.
  outputs = { self, nixpkgs, flake-utils, fenix, substrate }:
    flake-utils.lib.eachSystem [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Nightly toolchain pinned via rust-toolchain.toml at the workspace root.
        # fenix reads that file and provides the matching toolchain with rust-src,
        # clippy, rustfmt, and the cross targets we need.
        rustToolchain = (fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = pkgs.lib.fakeSha256;  # replace on first `nix develop`
        });

        # Cross-toolchain for aarch64 bare-metal builds. On Darwin we use the
        # LLVM toolchain; on Linux we can use gcc-cross if needed.
        crossTools = with pkgs; [
          lld
          llvm
          qemu
        ] ++ lib.optionals pkgs.stdenv.isLinux [
          # Linux-only cross tooling goes here as the bringup requires it.
        ];
      in
      {
        devShells.default = pkgs.mkShellNoCC {
          name = "brasa-dev";
          packages = [ rustToolchain ] ++ crossTools ++ (with pkgs; [
            cargo-nextest
            cargo-watch
            cargo-expand
            just
          ]);

          shellHook = ''
            echo "brasa dev shell — Phase 0 (design)"
            echo "Target triples: aarch64-unknown-none, aarch64-apple-darwin"
            echo "Run 'cargo check --workspace' to typecheck."
          '';
        };

        # Placeholder check that walks the workspace. Real checks (kernel image
        # build, QEMU boot smoke test) land in Phase 1.
        checks.workspace-check = pkgs.runCommand "brasa-workspace-check" { } ''
          echo "brasa Phase 0: check placeholder"
          echo "Real workspace check attaches once crates compile."
          touch $out
        '';

        # Reserved outputs for Phase 1+ — stubs so downstream flakes can
        # `follows` brasa and have stable attribute names to expect.
        packages = {
          default = pkgs.writeTextFile {
            name = "brasa-phase-0-marker";
            text = "brasa Phase 0 — see crates/brasa-bin for the QEMU PoC.\n";
            destination = "/STATUS";
          };
        };

        # `nix run .#brasa-qemu` — Phase 0 boot. 3-line inline glue
        # (build + exec-qemu) is the acceptable-shell boundary per
        # pleme-io CLAUDE.md; a proper Rust runner lands once the image
        # build pipeline matures. Run from the repo root.
        apps.brasa-qemu = {
          type = "app";
          program = toString (pkgs.writeShellScript "brasa-qemu" ''
            ${rustToolchain}/bin/cargo build -p brasa-bin --target aarch64-unknown-none --release && \
            exec ${pkgs.qemu}/bin/qemu-system-aarch64 -machine virt -cpu cortex-a72 -m 128 -nographic -kernel target/aarch64-unknown-none/release/brasa-kernel
          '');
        };
      }
    );
}
