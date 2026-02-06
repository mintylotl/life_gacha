{
  description = "a Flake for node, vite, typescript.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-25.11";
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
            libsoup_3
            webkitgtk_4_1
            librsvg
          ];

          shellHook = ''
            export LD_LIBRARY_PATH=${
              lib.makeLibraryPath (
                with pkgs;
                [
                  curl
                  openssl
                  stdenv.cc.cc
                ]
              )
            }
            export LD_LIBRARY_PATH="${pkgs.libglvnd}/lib:$LD_LIBRARY_PATH"
          '';
        };
      };
    };
}
