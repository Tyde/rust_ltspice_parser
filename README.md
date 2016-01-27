# Rust LTspice parser
A litte parser programm for the raw Files of LTspice IV

This is just a simple program written in Rust, that reads a whole .raw file generated by LTSpice. 
This can be used to do some calculations on the data. I used it to implement a fitness function for several steps of a simulation.

This project is still missing documentation.

There is a file describing [the informations about the raw File, which I gained while programming this](https://github.com/Tyde/rust_ltspice_parser/blob/master/LTSpice_RAW_File_Info.md).

This program currently can only decode .AC Simulations without .fastaccess

You can retrieve the results by using the SteppedSimulation struct:
```rust
let results = SteppedSimulation::from_files("Draft2.raw","Draft2.log");
```
As you can see, this parser needs both the *.raw file and the *.log file generated by LTspice

The SteppedSimulation struct offers several methods to get informations about the simulation.

###Example
```rust
//Read results
let results = SteppedSimulation::from_files("Draft2.raw","Draft2.log");
//Get all Variables that are in the Results of the Simulation
let vars = results.available_variables();
//Get all Steps of this simulation
let steps = results.available_steps();
//Get a Variable with its results. This will get the frequency, because it is the first variable
let freq = results.get_values_for_variable_at(&steps[0],&vars[0]).unwrap();

//Find steps, which have a resonance arount a point. This is very rudimentary
let resonances = results.find_with_resonance_at(&vars[4],955.0);

//Evaluate a fitness function over all steps. The fitness function is defined in the struct implementation
// The return value is a Tuple with the fitness value and the VariableResult
let mut fit = results.calculate_fitnesses(&vars[4]);
//Get the values of the 4th Variables
let values = results.get_values_for_variable_at(&steps[0],&vars[4]).unwrap();
//Plot the values with gnuplot
values.plot(&freq,&mut fg,&format!("Fitness: {}",bundle[k].1),colors[k]);
```

##License
This Project uses the MIT Licence
