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
  cargoSha256 = "sha256:0m8ssn7vhlscn560wp0smhdw2910zmasvjw6a6kzq7b9ql6z2w0z";

  meta = with pkgs.lib; {
    license = licenses.mit;
    platforms = platforms.linux;
    maintainers = with lib.maintainers; [ nequissimus ];
  };
}
