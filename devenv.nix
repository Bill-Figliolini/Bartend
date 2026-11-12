{ pkgs, lib, ... }:

let
  # cargo-generate.toml exists only in the template repo (it isn't copied into
  # generated projects). There, Cargo.toml has placeholders and can't build,
  # so the cargo-based shell check and git hook are skipped.
  isTemplate = builtins.pathExists ./cargo-generate.toml;

  # Dev tools only need to run, not pass their own test suites. When a nixpkgs
  # bump leaves one of them uncached, Nix builds it from source and its tests
  # can fail in the sandbox (e.g. cargo-llvm-cov vs. a newer LLVM), breaking
  # the whole shell. Skipping checks makes those local builds reliable.
  # Note: overriding changes the derivation hash, so these are always built
  # locally — only list packages that have actually caused trouble.
  noCheck = p: p.overrideAttrs (_: { doCheck = false; });
in
{
  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [ "rustc" "cargo" "clippy" "rust-analyzer" "rustfmt" "llvm-tools-preview" ];
  };
  packages = with pkgs; [
    bacon
    cargo-seek
    cargo-nextest
    (noCheck cargo-llvm-cov)
    cargo-generate
  ];
  scripts.watcher = {
    exec = ''
        watchexec -c -e rs \
        "cargo clippy && cargo test && cargo run"
    '';
    packages = [ pkgs.watchexec ];
  };
  enterShell = lib.optionalString (!isTemplate) ''
    if [ -f Cargo.toml ]; then
      echo "Crates ready to update:"
      cargo update -n
    fi
  '';
  git-hooks.hooks = {
      clippy.enable = !isTemplate;
      clippy.settings.denyWarnings = true;
  };
}
