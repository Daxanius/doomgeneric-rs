{
  description = "Rust development environment";
  
  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };

      # Libraries minifb loads at runtime
      runtimeLibs = with pkgs; [
        libX11
        libXcursor
        libXi
        libXrandr
        libxkbcommon
        wayland
        libGL
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        packages = with pkgs; [
          cargo
          clippy
          rustc
          rustfmt
          rust-analyzer
          rustPlatform.bindgenHook
          pkg-config
          lldb
          clang
          llvmPackages.libclang
          stdenv.cc
        ];

        
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;

        shellHook = ''
          export LIBCLANG_PATH=${pkgs.llvmPackages.libclang.lib}/lib
          export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=${pkgs.stdenv.cc.libc.dev}"
          export CFLAGS="-std=gnu11"
        '';
      };
    };
}
