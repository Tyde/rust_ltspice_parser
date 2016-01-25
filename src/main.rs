extern crate byteorder;
extern crate num;
extern crate gnuplot;
mod stepped_simulation;

use stepped_simulation::SteppedSimulation;
use gnuplot::{Figure, Caption, Color};
use gnuplot::AxesCommon;
use std::process::Command;




fn main() {

    //Rerun Simulation
    let status = Command::new("scad3.exe").arg("-Run").arg("-b").arg("Draft2.asc").status().unwrap_or_else(|e| {
        panic!("failed to execute process: {}", e)
    });
    println!("process exited with: {}", status);

    let results = SteppedSimulation::from_files("Draft2.raw","Draft2.log");

    println!("{:#?}", results.available_variables());
    //println!("{:#?}", );
    let params = results.available_parameters();
    let vars = results.available_variables();
    let steps = results.available_steps();
    let values = results.get_values_for_variable_at(&steps[0],&vars[4]);

    //println!("{:?}",  values.unwrap().get_bode());
    let freq = results.get_values_for_variable_at(&steps[0],&vars[0]);


    let x = freq.unwrap().get_abs();
    println!("{:?}", x);
    let y = values.unwrap().get_abs_in_dB();
    let mut fg = Figure::new();
    {
        let axes = fg.axes2d();
        axes.set_x_log(Some(10.0));
        axes.lines(&x, &y, &[Caption("A line"), Color("black")]);
    }
    fg.show();

}
