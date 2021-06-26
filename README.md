# Mesh Ripper

Rips Meshes

This is a a mesh series viewer for output from fluids simulations etc.

This currently only supports obj files. More coming soon.

## Installation / Run

```sh
    git clone https://github.com/rezural/mesh-ripper.git
    cargo run --release -- --features=native ./assets/data/<YOUR_DATA_DIR>
```

## TODO

- [ ] Better Camera System
- [ ] Esc to get out of mouse mode
- [ ] Ability to load files outside of assets
- [ ] More file format support (3d meshes) like stl, ply, obj, vtk, gltf et al.
- [ ] Support for points from stl, ply, vtk, obj
- [ ] Moving mouse on advance_every should increase in .01 - .0025 increments
- [ ] Look towards some point on the mesh (or center of mesh, expensive though) on startup, a little above, in front 
- [ ] Pause/Forward/Back -> Forward/Back & separate Pause
- [x] show first frame on startup
- [x] Better Loading of assets

## Asset Loading

Mesh Ripper will automatically load 100 files by default. Pass the -l, or --load-max option to change this.

### Loading Algo

Mesh Ripper automaticically sorts file alphanumerically. They should load in order, assuming a nice naming & ordering system.

Say you have 100 files like so:

```
[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100]
```

and you pass `-l 20`. Mesh Ripper will load 20 files, spread across the input files passed to the executable. Which would be:

```
[0, 5, 10, 15, 20, ...]
```

If you change `# of Frames to Load`, then it will get the midpoint from each of the originally loaded files, and load each of those files, resulting in almost the same again loaded. Again? Then it will load approx the same as have been loaded in total again. It tries to load the files so that you can get an overview of your dataset easily, from start to finish. If you want to see all the files, then you will have to wait. but you can load each #of frames successively to get results as soon as possible.
