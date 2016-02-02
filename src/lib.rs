//! The Rust LTSpice parser.
//!
//! This library reads the *.raw files generated by LTSpice to apply functions on it
//! 
extern crate byteorder;
extern crate num;
extern crate gnuplot;
extern crate statistical;


	
	
use std::fs;
use std::io::{Read, BufReader, BufRead};



use byteorder::{ByteOrder, LittleEndian};

use std::path::Path;

use statistical::*;
use results::*;
use std::str::FromStr;

pub mod results;

//


/// A set of results of one Stepped Simulation read from a File
/// 
/// # Examples
/// ```no_run
/// let mut path = PathBuf::from("./simulations/");
///	let model_name = "simulation1";
///
///	path.push(model_name);
///	
///	let results = SteppedSimulation::from_files(
/// 	path.with_extension("raw").as_path(),
///		path.with_extension("log").as_path());
///	``` 
/// This can currently only read results, which are generated with the 
/// `.ac` command. If you want to read transient files, this library has
/// to be extended 

#[derive(Debug)]
pub struct SteppedSimulation {
    steps:Vec<Step>,
    variables: Vec<SimulationVariable>,
    reals: Vec<Vec<f64>>,
    imags: Vec<Vec<f64>>,
    simulation_points: i32,
    points_per_block: usize
}


impl SteppedSimulation {
	/// Reads the Simulation result form the `.raw`-File. This also needs the 
	/// `.log`-File to get informations about the steps.
    pub fn from_files<P: AsRef<Path>>(path_raw: P, path_log: P) -> Self {
        let mut steps = Vec::new();
        let mut variables = Vec::new();
        let mut reals = Vec::new();
        let mut imags = Vec::new();

        read_log_file(path_log,&mut steps);
        let simulation_points = read_raw_file(path_raw,&mut variables,&mut reals,&mut imags);
        let ppb = (simulation_points/steps.len()as i32) as usize;
        SteppedSimulation {
            steps: steps,
            variables: variables,
            reals: reals,
            imags: imags,
            simulation_points: simulation_points,
            points_per_block: ppb
        }
    }


	/// Returns a vector with all `.param` parameters as strings.
    pub fn available_parameters(&self) -> Vec<String> {
        let mut result = Vec::new();
        if self.steps.len()>0 {
            for var in &self.steps[0] {
                result.push(var.name.to_owned());
            }
        }
        result
    }
	/// Returns a vector of all available variables which resulted of the simulation
    pub fn available_variables(&self) -> &Vec<SimulationVariable> {
        &self.variables
    }
    /// Returns a variable, which has the given name. If no variable for that name can be found, it returns None
    pub fn get_variable_for_name(&self, name: &str) -> Option<&SimulationVariable> {
    	for variable in &self.variables {
    		if variable.name.eq(name) {
    			return Some(variable);
    		}
    	}
    	None
    }
	/// Returns a vector of all steps of the simulation
    pub fn available_steps(&self) -> &Vec<Step> {
        &self.steps
    }

	/// Returns all VariableResults for one step
    pub fn get_values_at(&self, step: &Step) -> Vec<VariableResult> {
        match self.steps.iter().position(|r| r.eq(step)) {
            Some(position) =>self.get_value_block_at(position),
            None => Vec::new()
        }
    }


	/// Returns the VariableResult for one variable at a given step
    pub fn get_values_for_variable_at(&self, step: &Step, var: &SimulationVariable) -> Option<VariableResult> {

        match self.steps.iter().position(|r| r.eq(step)) {
            Some(position) =>{
                match self.variables.iter().position(|r| r.eq(var)) {
                    Some(var_pos) =>{

                        let variable_result = self.get_value_block_at(position);
                        Some(variable_result[var_pos].clone())
                    },
                    None => None
                }

            },
            None => None
        }
    }
	
	
    fn get_value_block_at(&self, pos:usize) -> Vec<VariableResult> {
        let start:usize = pos * self.points_per_block;
        let mut var_count = 0;
        let mut result = Vec::new();
        for v in &self.variables {
            let local_reals = &self.reals[var_count];
            let local_imags = &self.imags[var_count];
            result.push(VariableResult::new(v,local_reals[start .. (start+self.points_per_block)].to_vec(),local_imags[start..(start+self.points_per_block)].to_vec()));

            var_count += 1;
        }
        result
    }
    
   
	/// Calculates the fitnesses of all steps for a given variable. 
	/// The fitness is determined by the internal fitness function of the VariableResult.
	/// Currently it is not possible to define the fitness function by yourself.
    pub fn calculate_fitnesses(&self, var: &SimulationVariable) -> Vec<f64> {
        let mut fitnesses = Vec::new();
        let (averages,deviations) = self.find_averages_for_fitness(&var);
        for step in &self.steps {
            let freq = self.get_values_for_variable_at(&step,&self.variables[0]).unwrap();
            let values = self.get_values_for_variable_at(&step,&var);
            let fitness = match values {
                Some(vl) => vl.calculate_fitness(&freq,averages,deviations),
                None => panic!("wooow")
            };
            fitnesses.push(fitness.1);
         
        }
        fitnesses
    }
    
    pub fn find_averages_for_fitness(&self,var: &SimulationVariable) -> ([f64;5],[f64;5]) {
    	let mut result = [0.0;5];
    	let mut deviations = [0.0;5];
    	
    	let mut avgs = Vec::new();
    	let mut min_maxs = Vec::new();
    	let mut most_lefts = Vec::new();
    	let mut most_rights = Vec::new();
    	let mut at_100_hz = Vec::new();
    	
    	let freq = self.get_values_for_variable_at(&self.steps[0],&self.variables[0]).unwrap();
    	let (index_one_k,_) = self.get_values_for_variable_at(&self.steps[0],&var).unwrap().find_value_near_freq(&DataType::AbsoluteDecibel,&freq, 1000.0);
    	let (index_one_h,_) = self.get_values_for_variable_at(&self.steps[0],&var).unwrap().find_value_near_freq(&DataType::AbsoluteDecibel,&freq, 100.0);
    	for step in &self.steps {
    		
            let values = self.get_values_for_variable_at(&step,&var);
            match values {
            	Some(values) =>{
		    		avgs.push(values.avg_normalized(&DataType::AbsoluteDecibel,&freq));
		    		min_maxs.push(1.0/(values.max(&DataType::AbsoluteDecibel).1-values.min(&DataType::AbsoluteDecibel).1));
		    		most_lefts.push(values.get_data_point(&DataType::AbsoluteDecibel,0).unwrap());
		    		most_rights.push(values.get_data_point(&DataType::AbsoluteDecibel,index_one_k).unwrap());
		    		at_100_hz.push(values.get_data_point(&DataType::AbsoluteDecibel,index_one_h).unwrap());
            	},
            	None => panic!("Nicht vorhanden")
            }
    	}
    	result[0]=mean(&avgs);
    	deviations[0] = standard_deviation(&avgs,Some(result[0]));
    	result[1]=mean(&min_maxs);
    	deviations[1] = standard_deviation(&min_maxs,Some(result[1]));
    	result[2]=mean(&most_lefts);
    	deviations[2] = standard_deviation(&most_lefts,Some(result[2]));
    	result[3]=mean(&most_rights);
    	deviations[3] = standard_deviation(&most_rights,Some(result[3]));
    	result[4]=mean(&at_100_hz);
    	deviations[4] = standard_deviation(&at_100_hz,Some(result[4]));
    	(result,deviations)
    }


	/// Finds all VariableResults, that contain a resonance at the given frequency.
	/// This algorithm is not very good at the current moment, as it just takes the frequency ticks with a distance of 2
	/// to determine the borders of the allowed frame.
    pub fn find_with_resonance_at(&self,  var: &SimulationVariable, res_freq: f64) -> Vec<VariableResult> {
        let freq = self.get_values_for_variable_at(&self.steps[0],&self.variables[0]).unwrap();
        let freq_index = freq.get_data(&DataType::Real).iter().position(|b| b>&res_freq).unwrap();
        let fq_barrier_low = freq_index-2;
        let fq_barrier_high = freq_index+2;
        let mut result = Vec::new();
        for step in &self.steps {
            let values = self.get_values_for_variable_at(&step,&var).unwrap();
            let peaks =  values.find_peaks(Some(PeakType::Maximum),&DataType::AbsoluteDecibel);
            for peak in peaks {
                if peak > fq_barrier_low && peak < fq_barrier_high {
                    result.push(values.clone());
                }
            }
        }
        result

    }




}


fn read_log_file<P: AsRef<Path>>(path: P,steps: &mut Vec<Step>) -> i32 {
    let file = fs::File::open(path).unwrap();
    let mut file = BufReader::new(file);
    let mut line = String::new();
    let mut step_counter = 0;
    while file.read_line(&mut line).unwrap() > 0 {
        if line.starts_with(".step") {
            step_counter += 1;
            let mut k = line.split_whitespace();
            k.next();
            let mut step:Step = Vec::new();
            while let Some(var_text) = k.next() {
                let var = SteppingVariable::new(var_text);
                step.push(var);
            }
            steps.push(step);
        }
        line.clear();
    }
    step_counter
}


fn read_raw_file<P: AsRef<Path>>(path: P, variables: &mut Vec<SimulationVariable>, reals: &mut Vec<Vec<f64>>,imags: &mut Vec<Vec<f64>>) -> i32 {
    let nr_of_points = read_description(&path,variables);
    for _ in 0..variables.len() {
        reals.push(Vec::new());
        imags.push(Vec::new());
    }
    get_values(&path,reals,imags);

    nr_of_points
}



#[derive(PartialEq)]
enum DescriptionState {
    VarLength,
    VarList,
    Binary,
    NrOfPoints,
    Other
}


fn read_description<P: AsRef<Path>>(path: P, variables: &mut Vec<SimulationVariable>) -> i32 {
    let file = fs::File::open(path).unwrap();

    let mut file = BufReader::new(file);
    let mut state = DescriptionState::Other;

    let mut nr_of_points = 0;
    while state != DescriptionState::Binary {
        let mut line:Vec<u8> = Vec::new();
        file.read_until(b'\n',&mut line).unwrap();
        let line = String::from_utf8(line).unwrap();
        if line.contains("No. Variables") {
            state = DescriptionState::VarLength;
        }
        else if line.contains("Variables") {
            state = DescriptionState::VarList;
        }
        else if line.contains("Binary") {
            state = DescriptionState::Binary;
        }
        else if line.contains("No. Points") {
            state = DescriptionState::NrOfPoints;
            let cols:Vec<&str> = line.split(':').collect();
            nr_of_points = i32::from_str(cols[1].trim()).unwrap();
        }
        match state {
            DescriptionState::VarList => {
                let cols:Vec<&str> = line.split('\t').collect();
                if cols.len() == 4 {
                    let vt = match cols[3] {
                        "frequency" => VariableType::Frequency,
                        "voltage" => VariableType::Voltage,
                        "device_current" => VariableType::Current,
                        _ => VariableType::Unknown
                    };
                    variables.push(SimulationVariable {
                        id:u16::from_str(cols[1]).unwrap(),
                        name:String::from(cols[2]),
                        var_type:vt
                    });
                }

            }
            _ => {},
        }
    }

    nr_of_points

}




fn get_values<P: AsRef<Path>>(path: P,reals: &mut Vec<Vec<f64>>,imags: &mut Vec<Vec<f64>>)  {

    let mut file = fs::File::open(path).unwrap();
    let mut file_buf = Vec::new();
    let mut freq_step_counter = 0;
    let mut ct = 0;
    //let mut line_ct = 0;
    let mut in_binary = false;
    file.read_to_end(&mut file_buf).unwrap();

    let mut buffer = Vec::new();
    while ct<(file_buf.len()) {
        if !in_binary {
            //Wait for binary
            let k = &[file_buf[ct].clone()];

            buffer.push(k[0]);
            if String::from_utf8(buffer.clone()).unwrap().contains("Binary:\n") {
                in_binary = true;

            }


            ct+=1;
        }
        else {


            let start = ct;
            let end = start + 8;

            let real = LittleEndian::read_f64(&file_buf[start..end]);
            let imag = LittleEndian::read_f64(&file_buf[end..end+8]);


            ct+=16;
            reals[freq_step_counter].push(real);
            imags[freq_step_counter].push(imag);

            freq_step_counter = (freq_step_counter+1)%reals.len();
            //line_ct+=1;
        }


    }


}

