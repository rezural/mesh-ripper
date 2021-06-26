# Mesh Ripper

Rips Meshes

This is a a mesh series viewer for output from fluids simulations etc.

This currently only supports obj files. More coming soon.

## Installation / Run

```sh
    git clone https://github.com/rezural/mesh-ripper.git
    cargo run --release -- ./assets/data/<YOUR_DATA_DIR>
```

## TODO

- [ ] Better Camera System
- [ ] show first frame on startup
- [ ] Esc to get out of mouse mode
- [ ] Ability to load files outside of assets
- [ ] More file format support (3d meshes) like stl, ply, vtk, gltf et al.
- [x] Pause/Forward/Back -> Forward/Back & separate Pause
- [x] Better Loading of assets


PR's welcome!