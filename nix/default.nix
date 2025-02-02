{
  rustPlatform,
  #rust-bin,
  pkg-config,
  wrapGAppsHook4,
  gtk4,
  gtk4-layer-shell,
  libadwaita,
  dbus,
  libGL,
  libxkbcommon,
  wayland,
  libclang,
  glib,
  pango,

  cargo,
  cargo-watch,
  rustc,
  rust-analyzer,
  clippy,
  lib,
  lockFile,
  vulkan-loader,
  wayland-protocols,
  libX11,
  libXrandr,
  libXi,
  libXcursor,
  ...
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
  libPath = lib.makeLibraryPath [
    libGL
    libxkbcommon
    wayland
    pkg-config
    libclang
  ];
in
rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src = ../.;

  buildInputs = [
    pkg-config
    gtk4
    gtk4-layer-shell
    libadwaita
    dbus
    libGL
    libxkbcommon
    wayland
    libclang
    glib
    pango
  ];

  cargoLock = {
    inherit lockFile;
    outputHashes = {
      "re_set-lib-5.2.5" = "";
    };
  };

  nativeBuildInputs = [
    pkg-config
    #wrapGAppsHook4
    #(rust-bin.selectLatestNightlyWith (toolchain: toolchain.default))
    wayland
    cargo
    cargo-watch
    rustc
    rust-analyzer
    clippy
    libGL
    libxkbcommon
    libclang
    glib
    pango
  ];

  copyLibs = true;
  LD_LIBRARY_PATH = libPath;
  LIBCLANG_PATH = "${libclang.lib}/lib";

  postFixup =
    let
      libPath = lib.makeLibraryPath [
        libGL
        vulkan-loader
        wayland
        wayland-protocols
        libxkbcommon
        libX11
        libXrandr
        libXi
        libXcursor
      ];
    in
    ''
      patchelf --set-rpath "${libPath}" "$out/bin/reset"
    '';

  meta = with lib; {
    description = "A settings manager for Linux";
    homepage = "https://github.com/Xetibo/ReSet-Iced";
    changelog = "https://github.com/Xetibo/ReSet-Iced/releases/tag/${version}"; # TODO beforepr
    license = licenses.gpl3;
    maintainers = with maintainers; [ DashieTM ];
    mainProgram = "reset";
  };
}
