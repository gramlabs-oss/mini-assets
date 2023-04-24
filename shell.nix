with import <nixpkgs> { };

mkShell {
  name = "building-environment";
  buildInputs = [ pkg-config openssl llvmPackages_12.libclang.lib ];

  LIBCLANG_PATH = "${llvmPackages_12.libclang.lib}/lib";
}
