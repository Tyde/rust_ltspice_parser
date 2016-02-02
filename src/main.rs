extern crate byteorder;
extern crate num;
extern crate gnuplot;
extern crate statistical;
extern crate ltspice_parse;
/// The Stepped Simulation Mod




use ltspice_parse::SteppedSimulation;
use ltspice_parse::results::{PeakType,DataType};

use std::path::PathBuf;

use gnuplot::{Figure};
//use gnuplot::{Figure, Caption, Color,AxesCommon,AutoOption};




fn main() {

    //Rerun Simulation
    /*let status = Command::new("scad3.exe").arg("-Run").arg("-b").arg("Draft2.asc").status().unwrap_or_else(|e| {
        panic!("failed to execute process: {}", e)
    });
    println!("process exited with: {}", status);*/

	let mut path = PathBuf::from("../../Seafile/Masterarbeit/Sonstige_Dokumente/Morphologischer_Kasten/Simulationen/");
	let model_name = "06_BewegteMasse";

	path.push(model_name);
	
    let results = SteppedSimulation::from_files(
    	path.with_extension("raw").as_path(),
		path.with_extension("log").as_path());

    
    let vars = results.available_variables();

	println!("Variables: {:#?}", vars);

    //
    
    println!("Calulating Resonances");
    //let resonances = results.find_with_resonance_at(&vars[1],955.0);
    let mut fg = Figure::new();
    let vars = results.available_variables();
    let steps = results.available_steps();
    /*
    let freq = results.get_values_for_variable_at(&steps[0],&vars[0]).unwrap();
    let mut ct = 0;

    
    for resonance in &resonances {
        if resonances.len() - ct < 50 {
            //resonance.plot(&freq,&mut fg,&format!("rp"),"red");
        }
        //println!("panalty of res: {:?}", resonance.calculate_resonance_penalty(&freq));
        ct+=1;
    }*/

	let vout = results.get_variable_for_name("V(vout)").unwrap();
	
	println!("Calculating Fitnesses");
    let fit = results.calculate_fitnesses(&vout);
    let mut bundle:Vec<(usize,f64)> = Vec::new();
    for ct in 0..fit.len() {
        bundle.push((ct,fit[ct]));
    }
    bundle.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().reverse());
    println!("Done with Fitnesses");
    let steps = results.available_steps();
    //println!("Fitnesses {:#?}", bundle);
	let (averages,deviations) =results.find_averages_for_fitness(&vout);
    let colors = vec!("black","green","yellow","blue","magenta");
    let mut ohzs = Vec::new();
    let mut khzs = Vec::new();
    let mut x = Vec::new();
    for k in 0..bundle.len() {
        //println!("On place {:?} is {:#?} with {}", k,&steps[bundle[k].0],bundle[k].1);
        if k == 0  {
        	//println!("Will auf k zugreifden: {:#?}, bundle is: {:#?} wert wird sein : {:#?}", k,bundle,bundle[k].0);
	        let freq = results.get_values_for_variable_at(&steps[bundle[k].0],&vars[0]).unwrap();
	        let values = results.get_values_for_variable_at(&steps[bundle[k].0],&vout).unwrap();
	       // println!("Got values");
	        
	        
	        let resonances = values.find_peaks(Some(PeakType::Maximum), &DataType::AbsoluteDecibel);
	        for resonance in resonances {
	        	println!("Resonance at {:?}", resonance);
	        	let (left_value,right_value) = values.next_value_around(&DataType::AbsoluteDecibel, resonance, 3.0, true);
	        	if left_value.is_some() && right_value.is_some() {
	        		println!("Has some on both: {:?} , {:?}" ,left_value,right_value);
	        	}
	        	println!("Has some on both: {:?} , {:?}" ,left_value,right_value);
	        }
	        println!("panalty: {:?}", values.calculate_resonance_penalty(&freq));
	        
	        //println!("df1 {:?}, df2: {:?}", df1,df2);
	        
	        
	        
	        println!("Winner:");
	        let (single,sum,original) = values.calculate_fitness(&freq,averages,deviations);
	        println!("Single Values are {:?}. Sum is {:?}", single, sum);
	        ohzs.push(original[2]);
	        khzs.push(original[3]);
	        x.push(k);
	        println!("Origina Values are {:?}", original);
	        values.plot(&freq,&mut fg,&format!("Fitness: {}",bundle[k].1),colors[k%5]);
        	println!("Step is {:?}", &steps[bundle[k].0]);
        	println!("Used Averages: {:?}, Used Deviations{:?}", averages,deviations);
        	
        	
        }
    }
    /*
   	fg.axes2d().lines(&x,&ohzs,&[Caption("Ein-Herz"),Color("red")])
            .set_y_range(AutoOption::Fix(-140.0),AutoOption::Fix(0.0));
   	fg.axes2d().lines(&x,&khzs,&[Caption("1k-Herz"),Color("blue")])
            .set_y_range(AutoOption::Fix(-140.0),AutoOption::Fix(0.0));*/
            //.//set_x_log(Some(10.0));
           // .//set_y_range(AutoOption::Fix(-70.0),AutoOption::Fix(0.0));

    //let freq = results.get_values_for_variable_at(&steps[0],&vars[0]).unwrap();
    //let values = results.get_values_for_variable_at(&steps[34],&vout).unwrap();
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
