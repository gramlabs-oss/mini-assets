with import <nixpkgs> { };

mkShell {
  name = "building-environment";
  buildInputs = [ pkg-config openssl llvmPackages_latest.libclang.lib ];

  LIBCLANG_PATH =
    pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
}
