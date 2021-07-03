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
- [ ] Ability to load files outside of assets
- [ ] More file format support (3d meshes) like stl, ply, obj, vtk, gltf et al.
- [ ] Support for points from stl, ply, vtk, obj
- [ ] Moving mouse on advance_every should increase in .01 - .0025 increments
- [ ] Look towards some point on the mesh (or center of mesh, expensive though) on startup, a little above, in front 
- [ ] Load and cache from urls
- [ ] Release mouse when focus lost
- [x] When a lod is chosen that is not directly above the current lod, load the next lod, then the next higher etc, otherwise you get speedup at the start (the files that have been loaded), and jumpy framerate
- [x] Pause/Forward/Back -> Forward/Back & separate Pause
- [x] Esc to get out of mouse mode
- [x] show first frame on startup
- [x] Better Loading of assets

## Asset Loading

Mesh Ripper will automatically load 100 files by default. Pass the -l, or --load-max option to change this.

### Loading Algorithm

Mesh Ripper automaticically sorts file alphanumerically. They should load in order, assuming a nice naming & ordering system.

Say you have 100 files like so:

```
[0.obj, 1.obj, 2.obj, 3.obj, 4.obj, 5.obj, 6.obj, 7.obj, 8.obj, 9.obj, 10.obj, 11.obj, 12.obj, 13.obj, 14.obj, 15.obj, 16.obj, 17.obj, 18.obj, 19.obj, 20.obj, 21.obj, 22.obj, 23.obj, 24.obj, 25.obj, 26.obj, 27.obj, 28.obj, 29.obj, 30.obj, 31.obj, 32.obj, 33.obj, 34.obj, 35.obj, 36.obj, 37.obj, 38.obj, 39.obj, 40.obj, 41.obj, 42.obj, 43.obj, 44.obj, 45.obj, 46.obj, 47.obj, 48.obj, 49.obj, 50.obj, 51.obj, 52.obj, 53.obj, 54.obj, 55.obj, 56.obj, 57.obj, 58.obj, 59.obj, 60.obj, 61.obj, 62.obj, 63.obj, 64.obj, 65.obj, 66.obj, 67.obj, 68.obj, 69.obj, 70.obj, 71.obj, 72.obj, 73.obj, 74.obj, 75.obj, 76.obj, 77.obj, 78.obj, 79.obj, 80.obj, 81.obj, 82.obj, 83.obj, 84.obj, 85.obj, 86.obj, 87.obj, 88.obj, 89.obj, 90.obj, 91.obj, 92.obj, 93.obj, 94.obj, 95.obj, 96.obj, 97.obj, 98.obj, 99.obj, 100]
```

and you pass `-l 20`. Mesh Ripper will load 20 files.obj, spread across the input files passed to the executable. Which would be:

```
[0.obj, 5.obj, 10.obj, 15.obj, 20.obj, ...]
```

If you select the next level of `# of Frames to Load`, then it will load each file at the the midpoint from each of the originally loaded files, and load each of those files, resulting in almost the same again loaded. Select the next level again, then it will load approx the same as have been loaded in total again. It tries to load the files so that you can get an overview of your dataset easily, from start to finish.

So for 100 files, starting at 10, the levels available will be:

[10, 19, 37, 69, 100]