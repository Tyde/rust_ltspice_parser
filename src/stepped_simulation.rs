use std::fs;
use std::io::{Read, BufReader, BufRead};
use std::str;
use byteorder::{ByteOrder, LittleEndian};
use std::str::FromStr;
use std::path::Path;
use num::complex::Complex;


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

    pub fn available_parameters(&self) -> Vec<String> {
        let mut result = Vec::new();
        if self.steps.len()>0 {
            for var in &self.steps[0] {
                result.push(var.name.to_owned());
            }
        }
        result
    }

    pub fn available_variables(&self) -> &Vec<SimulationVariable> {
        &self.variables
    }

    pub fn available_steps(&self) -> &Vec<Step> {
        &self.steps
    }

    pub fn get_values_at(&self, step: &Step) -> Vec<VariableResult> {
        match self.steps.iter().position(|r| r.eq(step)) {
            Some(position) =>self.get_value_block_at(position),
            None => Vec::new()
        }
    }



    pub fn get_values_for_variable_at(&self, step: &Step, var: &SimulationVariable) -> Option<VariableResult> {
        println!("var: {:?} Variables: {:#?}", var,self.variables);
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
            result.push(VariableResult {
                variable:v,
                reals:local_reals[start .. (start+self.points_per_block)].to_vec(),
                imags:local_imags[start..(start+self.points_per_block)].to_vec()
            });

            var_count += 1;
        }
        result
    }



}



#[derive(Debug,Clone)]
struct VariableResult<'a> {
    variable: &'a SimulationVariable,
    reals: Vec<f64>,
    imags: Vec<f64>
}

impl<'a> VariableResult<'a> {
    pub fn get_abs(&self) -> Vec<f64> {
        let mut result:Vec<f64> = Vec::new();
        for ct in 0..self.reals.len() {
            let comp = Complex::new(self.reals[ct],self.imags[ct]);
            result.push(comp.norm_sqr());
        }
        result
    }

    pub fn get_abs_in_dB(&self) -> Vec<f64> {
        let mut result:Vec<f64> = Vec::new();
        for ct in 0..self.reals.len() {
            let comp = Complex::new(self.reals[ct],self.imags[ct]);
            result.push(20.0*comp.norm().log(10.0));
        }
        result
    }

    pub fn get_bode(&self) -> Vec<(f64,f64)> {
        let mut result = Vec::new();
        for ct in 0..self.reals.len() {
            let comp = Complex::new(self.reals[ct],self.imags[ct]);
            result.push((20.0*comp.norm().log(10.0),comp.arg()));
        }
        result
    }
}




#[derive(Default,Debug,PartialEq)]
struct SteppingVariable {
    name: String,
    value: f32
}

impl SteppingVariable {
    fn new(log_excerpt:&str) -> Self {
        let split:Vec<&str> = log_excerpt.split('=').collect();
        SteppingVariable {
            name: split[0].to_owned(),
            value: f32::from_str(split[1]).unwrap()
        }
    }
}


type Step = Vec<SteppingVariable>;

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
#[derive(Debug,Hash,Eq,PartialEq)]
enum VariableType {
    Frequency,
    Voltage,
    Current,
    Unknown
}
#[derive(Debug,Hash,Eq,PartialEq)]
struct SimulationVariable {
    id:u16,
    name:String,
    var_type:VariableType,
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
    while ct<(file_buf.len()) {
        if !in_binary {
            //Wait for binary
            let k = &[file_buf[ct].clone()];
            match str::from_utf8(k) {
                Ok(_) => {},
                Err(_) => in_binary = true,
            };
            ct+=1;
        }
        else {


            let start = ct-1;
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
