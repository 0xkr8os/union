{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, lib, ... }:
    let
      tendermintVerifierTestSuite = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/tendermint-verifier";
        additionalTestSrcFilter = path: _:
          (lib.hasPrefix "lib/tendermint-verifier/src/test" path)
          && (lib.strings.hasSuffix ".json" path);
      };
    in
    {
      checks = tendermintVerifierTestSuite.checks;
    };
}