{
    description = "Optional development shell for template-rust";

    inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    outputs =
        { nixpkgs, ... }:
        let
            systems = [
                "x86_64-linux"
                "aarch64-linux"
                "aarch64-darwin"
            ];
        in
        {
            devShells = nixpkgs.lib.genAttrs systems (
                system:
                let
                    pkgs = import nixpkgs { inherit system; };
                in
                {
                    default = pkgs.mkShell {
                        packages = [
                            pkgs.git
                            pkgs.prek
                            pkgs.rustup
                            pkgs.cargo-llvm-cov
                            pkgs.cargo-mutants
                            pkgs.pkg-config
                        ];
                    };
                }
            );
        };
}
