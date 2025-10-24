{
  description = "An empty flake template that you can adapt to your own environment";
  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0"; # Stable Nixpkgs

  outputs =
    { self, ... }@inputs:
    let
      supportedSystems = [
        "x86_64-linux"   # 64-bit Intel/AMD Linux
        "aarch64-linux"  # 64-bit ARM Linux
        "x86_64-darwin"  # 64-bit Intel macOS
        "aarch64-darwin" # 64-bit ARM macOS
      ];

      forEachSupportedSystem =
        fn:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          fn {
            pkgs = import inputs.nixpkgs {
              inherit system;
              config.allowUnfree = true;
            };
          }
        );
    in
    {
      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShell {
            packages = with pkgs; [ openssl.dev pkg-config ];
            env = { };
            shellHook = "";
          };
        }
      );
    };
}
