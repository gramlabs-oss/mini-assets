with import <nixpkgs> { crossSystem = { config = "x86_64-w64-mingw32"; }; };

mkShell {
  name = "windows-crosscompile";
  buildInputs = with pkgs.pkgsCross.mingwW64.windows; [
    mingw_w64_pthreads
    pthreads
  ];
}
