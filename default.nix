with import <nixpkgs> { };

let
  version = "0.1.0";
in rustPlatform.buildRustPackage rec {
  inherit version;
  pname = "kafka-lag-slack";

  src = builtins.filterSource
    (path: type: baseNameOf path != "default.nix"
              && baseNameOf path != "result"
              && baseNameOf path != "target"
              )
    ./.;

  nativeBuildInputs = [ openssl pkgconfig ];

  doCheck = false;
  cargoSha256 = "sha256:0jacm96l1gw9nxwavqi1x4669cg6lzy9hr18zjpwlcyb3qkw9z7f";

  meta = with pkgs.lib; {
    license = licenses.mit;
    platforms = platforms.linux;
    maintainers = with lib.maintainers; [ nequissimus ];
  };
}
