{
  description = "github.com/mstone/wye";

  inputs.crane.url = "github:ipetkov/crane";
  inputs.crane.inputs.nixpkgs.follows = "nixpkgs";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  inputs.nixpkgs.url = "nixpkgs/nixpkgs-unstable";

  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.flake-utils.follows = "flake-utils";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  outputs = {self, nixpkgs, crane, rust-overlay, flake-utils}:
    flake-utils.lib.simpleFlake {
      inherit self nixpkgs;
      name = "wye";
      preOverlays = [ 
        rust-overlay.overlays.default
      ];
      overlay = final: prev: {
        wye = rec {

          wyeVersion = "0.1";
          wye = lib.wye { isShell = false; };
          devShell = lib.wye { isShell = true; };
          defaultPackage = wye;

          bintools = prev.bintools.overrideAttrs (old: {
            postFixup = 
              if prev.stdenv.isDarwin then 
                builtins.replaceStrings ["-no_uuid"] [""] old.postFixup
              else 
                old.postFixup;
          });

          cc = prev.stdenv.cc.overrideAttrs (old: {
            inherit bintools;
          });

          stdenv = prev.overrideCC prev.stdenv cc;

          # rust from rust-overlay adds stdenv.cc to propagatedBuildInputs 
          # and depsHostHostPropagated; therefore, to ensure that the correct
          # cc is present in downstream consumers, we need to override both these 
          # attrs.
          rust = with final; with pkgs; 
            #(rust-bin.stable.latest.minimal.override { targets = [ "wasm32-unknown-unknown" ]; })
            #(rust-bin.nightly.latest.minimal.override { extensions = [ "rustfmt" ]; targets = [ "wasm32-unknown-unknown" ]; })
            (rust-bin.selectLatestNightlyWith (toolchain: toolchain.minimal.override {
              extensions = [ "rustfmt" ];
              targets = [ "wasm32-unknown-unknown" ];
            })).overrideAttrs (old: {
              inherit stdenv;
              propagatedBuildInputs = [ stdenv.cc ];
              depsHostHostPropagated = [ stdenv.cc ];
            });

          # crane provides a buildPackage helper that calls stdenv.mkDerivation
          # which provides a default builder that sources a "setup" file defined
          # by the stdenv itself (passed as the environment variable "stdenv" that 
          # in turn defines a defaultNativeBuildInputs variable that gets added to 
          # PATH via the genericBuild initialization code. Therefore, we override
          # crane's stdenv to use our modified cc-wrapper. Then, we override
          # cargo, clippy, rustc, and rustfmt, similar to the newly introduced 
          # crane.lib.overrideToolchain helper.
          cranelib = crane.lib.${final.system}.overrideScope' (final: prev: {
            inherit stdenv;
            cargo = rust;
            clippy = rust;
            rustc = rust;
            rustfmt = rust;
          });

          tex = with final; with pkgs; texlive.combined.scheme-full;

          lib.wye = { isShell, isWasm ? false, subpkg ? "wye", subdir ? "." }: 
            let 
              buildInputs = with final; with pkgs; [
                rust
              ] ++ final.lib.optionals isShell [
                entr
                trunk
                wasm-bindgen-cli
                wabt
                cargo-expand
                cargo-outdated
                cargo-udeps
                rustfmt
              ] ++ final.lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
              ]) ++ final.lib.optionals stdenv.isLinux ([
              ]);
            in with final; with pkgs; cranelib.buildPackage {
              pname = "${subpkg}";
              version = wyeVersion;

              src = cranelib.cleanCargoSource ./.;

              cargoArtifacts = cranelib.buildDepsOnly {
                inherit buildInputs;
                src = cranelib.cleanCargoSource ./.;
                cargoCheckCommand = if isWasm then "" else "cargo check";
                cargoBuildCommand = if isWasm then "cargo build --release -p depict-web --target wasm32-unknown-unknown" else "cargo build --release";
                doCheck = false;
              };

              inherit buildInputs;

              cargoExtraArgs = if isWasm then "--target wasm32-unknown-unknown -p ${subpkg}" else "-p ${subpkg}"; 

              doCheck = false;
          };
        };
      };
    };
}
