{
  description = "a Flake for node, vite, typescript.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/b2a3852bd078e68dd2b3dfa8c00c67af1f0a7d20";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      lib = pkgs.lib;
    in
    {
      devShells.${system} = {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [
            nodejs_20
            nodePackages.npm
            nodePackages.typescript-language-server
            # Needed for Tauri
            pkg-config
            dbus
            openssl_3
            glib
            gtk3
            libsoup
            webkitgtk
            librsvg
          ];

          shellHook = ''
            export LD_LIBRARY_PATH=${
              pkgs.makeLibraryPath (
                with pkgs;
                [
                  curl
                  openssl
                  stdenv.cc.cc
                ]
              )
            }
          '';
        };
      };
    };
}
