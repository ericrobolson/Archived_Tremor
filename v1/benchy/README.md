// MIT license
// Eric R. Olson

A simple benchmarking crate. Use like so:

```
fn main() {
    let benchy = Benchy::initialize("some_output_path");
    {
        // Create a new timer. When it's dropped, it will log back to Benchy.
        benchy_timer!("main loop");
        for i in 0..100{
            println!("a computation");
        }
    }

    // benchy gets dropped, writing the results to a file.
}
