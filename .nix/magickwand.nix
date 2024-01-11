with import <nixpkgs> { };

mkShell {
  name = "features-magickwand-compile";
  buildInputs = [ pkg-config imagemagick llvmPackages_latest.libclang.lib ];

  LIBCLANG_PATH =
    pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
}
