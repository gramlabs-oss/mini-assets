with import <nixpkgs> { };

mkShell {
  name = "features-magickwand-compile";
  buildInputs = [ pkg-config imagemagick llvmPackages_12.libclang.lib ];

  LIBCLANG_PATH = "${llvmPackages_12.libclang.lib}/lib";
}
