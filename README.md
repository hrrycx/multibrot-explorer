# multibrot-explorer
rust egui project to explore multibrot fractals

## notes (maybe put these in the program as like a comment thing): 
- keep maxitr generally in range 300-1500
- higher exponents need a lower maxitr (which helps to keep frame times down)

## todo:
- pi approximation (done, add a ? how does this work)
- allow negative powers
- more colouring algorithms (and hue range)
- better styling
- make code easier to read, main is awful
- one day, when youre smart, implement compiler optimisations with simd

## running
run with `cargo run --release` for best performance
