{
  description = "Index of accesses to the web";
  # Largely from https://github.com/cpu/rust-flake

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs @ {
    self,
    flake-parts,
    nixpkgs,
    rust-overlay,
    flake-utils,
  }:
    flake-parts.lib.mkFlake {inherit inputs;} (
      top @ {
        config,
        lib,
        getSystem,
        ...
      }: {
        systems = flake-utils.lib.defaultSystems;
        perSystem = {
          config,
          self',
          pkgs,
          lib,
          system,
          ...
        }: let
          makeRuntimeDeps = pkgs: [pkgs.openssl];
          makeBuildDeps = pkgs: [pkgs.pkg-config];
          makedevDeps = pkgs: [pkgs.gdb pkgs.copier pkgs.pre-commit];

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          msrv = cargoToml.package.rust-version;

          mkDevShell = rustc:
            pkgs.mkShell {
              env = {
                RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
              };
              buildInputs = makeRuntimeDeps pkgs;
              nativeBuildInputs = (makeBuildDeps pkgs) ++ (makedevDeps pkgs) ++ [rustc];
              shellHook = ''
                pre-commit install
                
              '';
            };
          overlays = [(import rust-overlay)];

        in {
          _module.args.pkgs = import nixpkgs {inherit system overlays;};

          devShells.nightly =
            mkDevShell (pkgs.rust-bin.selectLatestNightlyWith
              (toolchain: toolchain.default));
          devShells.stable = mkDevShell pkgs.rust-bin.stable.latest.default;
          devShells.msrv = mkDevShell pkgs.rust-bin.stable.${msrv}.default;
          devShells.default = self'.devShells.nightly;
        };
      }
    );
}
