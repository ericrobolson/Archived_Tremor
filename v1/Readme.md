This is the v1 code of Tremor. It was a experiment in building an engine modeled after the Quake 3 architecture. Due to a variety of reasons, development is no longer active on V0, but feel free to use for whatever purpose you'd like.

Changes from v0:
* Better usage of subcrates for separating functionality
* rendering_ir to split out renderer agnostic functionality
* Job queue system 
* Deterministic math
* OBJ + GLTF loading
* Text rendering
* Simple Verlet integrator
* Benchmarking crate