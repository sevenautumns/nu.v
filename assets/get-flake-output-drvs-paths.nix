thatFlake:
let
  lib = (import <nixpkgs> { }).lib;
  inherit (lib.attrsets)
    collect
    isDerivation
    mapAttrsRecursiveCond
    removeAttrs
    ;
  inherit (lib.strings) concatStringsSep;
in
# TODO there is a bug lurking for a flake which has two packages, one named drvPath, one name flakePath
collect (x: x ? drvPath && x ? flakePath) (
  mapAttrsRecursiveCond (leaf: !isDerivation leaf) (path: leaf: {
    inherit (leaf) drvPath;
    flakePath = concatStringsSep "." path;
  }) (removeAttrs thatFlake [ "overlays" ])
)
