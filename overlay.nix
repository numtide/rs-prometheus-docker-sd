final: prev:
rec {
  psdd = {
    pkgs = prev.callPackage ./pkgs { };
    prometheus-sd-docker = prev.naersk.buildPackage {
      src = final.builtins.filterSource (path: type: type != "directory" || final.builtins.baseNameOf path != "target") ./.;
      remapPathPrefix = true;
    };
  };
}
