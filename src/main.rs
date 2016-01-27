extern crate byteorder;
extern crate num;
extern crate gnuplot;
mod stepped_simulation;


use stepped_simulation::SteppedSimulation;
use stepped_simulation::{PeakType,DataType};


use std::process::Command;
use gnuplot::{Figure, Caption, Color};




fn main() {

    //Rerun Simulation
    /*let status = Command::new("scad3.exe").arg("-Run").arg("-b").arg("Draft2.asc").status().unwrap_or_else(|e| {
        panic!("failed to execute process: {}", e)
    });
    println!("process exited with: {}", status);*/

    let results = SteppedSimulation::from_files("Draft2.raw","Draft2.log");

    //println!("{:#?}", results.available_variables());
    //println!("{:#?}", );
    let vars = results.available_variables();



    //
    let resonances = results.find_with_resonance_at(&vars[4],955.0);
    let mut fg = Figure::new();
    let vars = results.available_variables();
    let steps = results.available_steps();
    let freq = results.get_values_for_variable_at(&steps[0],&vars[0]).unwrap();
    let mut ct = 0;
    for resonance in &resonances {
        if resonances.len() - ct < 5 {
            resonance.plot(&freq,&mut fg,&format!("rp"),"red");
        }
        ct+=1;
    }

    let mut fit = results.calculate_fitnesses(&vars[4]);
    let mut bundle:Vec<(usize,f64)> = Vec::new();
    for ct in 0..fit.len() {
        bundle.push((ct,fit[ct]));
    }
    bundle.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().reverse());
    let steps = results.available_steps();
    //println!("Fitnesses {:#?}", bundle);

    let colors = vec!("black","green","yellow","blue","magenta");
    for k in 0..5 {
        println!("On place {:?} is {:#?} with {}", k,&steps[bundle[k].0],bundle[k].1);
        let freq = results.get_values_for_variable_at(&steps[bundle[k].0],&vars[0]).unwrap();
        let values = results.get_values_for_variable_at(&steps[bundle[k].0],&vars[4]).unwrap();
        values.plot(&freq,&mut fg,&format!("Fitness: {}",bundle[k].1),colors[k]);
    }

    let freq = results.get_values_for_variable_at(&steps[0],&vars[0]).unwrap();
    let values = results.get_values_for_variable_at(&steps[34],&vars[4]).unwrap();
    //values.plot(&freq,&mut fg,&format!("Fitness: {}",bundle[0].1),"black");
    fg.show();




    /*
    let params = results.available_parameters();
    let vars = results.available_variables();
    let steps = results.available_steps();
    let values = results.get_values_for_variable_at(&steps[0],&vars[4]).unwrap();

    //println!("{:?}",  values.unwrap().get_bode());
    let freq = results.get_values_for_variable_at(&steps[0],&vars[0]).unwrap();
    let peaks = values.find_peaks(Some(PeakType::Maximum),&DataType::AbsoluteDecibel);
    let (loc,max) = values.min(&DataType::AbsoluteDecibel);
    println!("{:#?} bei frequency: ; avg is : {:#?}. Max is at {} and is {}", peaks,  values.avg_normalized(&DataType::AbsoluteDecibel,&freq),loc,max);
    let freq_data = freq.get_data(&DataType::Real);
    let (l,r) = values.next_value_around(&DataType::AbsoluteDecibel,40,3.0,true);

    println!("Fitness: {:?}",values.calculate_fitness(&freq));*/

}
