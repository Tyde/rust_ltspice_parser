use std::fs;
use std::io::{Read, BufReader, BufRead};
use std::str;
use byteorder::{ByteOrder, LittleEndian};
use std::str::FromStr;
use std::path::Path;
use num::complex::Complex;
use gnuplot::{Figure, Caption, Color,AxesCommon,AutoOption};



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

    pub fn calculate_fitnesses(&self, var: &SimulationVariable) -> Vec<f64> {
        let mut fitnesses = Vec::new();
        for step in &self.steps {
            let freq = self.get_values_for_variable_at(&step,&self.variables[0]).unwrap();
            let values = self.get_values_for_variable_at(&step,&var);
            let fitness = match values {
                Some(vl) => vl.calculate_fitness(&freq),
                None => panic!("wooow")
            };
            fitnesses.push(fitness);
        }
        fitnesses
    }


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


/// A Variable Result
#[derive(Debug,Clone)]
struct VariableResult<'a> {
    variable: &'a SimulationVariable,
    reals: Vec<f64>,
    imags: Vec<f64>
}

impl<'a> VariableResult<'a> {
    fn get_abs(&self) -> Vec<f64> {
        let mut result:Vec<f64> = Vec::new();
        for ct in 0..self.reals.len() {
            let comp = Complex::new(self.reals[ct],self.imags[ct]);
            result.push(comp.norm_sqr());
        }
        result
    }

    fn get_abs_in_decibel(&self) -> Vec<f64> {
        let mut result:Vec<f64> = Vec::new();
        for ct in 0..self.reals.len() {
            let comp = Complex::new(self.reals[ct],self.imags[ct]);
            result.push(20.0*comp.norm().log(10.0));
        }
        result
    }

    fn get_bode(&self) -> Vec<(f64,f64)> {
        let mut result = Vec::new();
        for ct in 0..self.reals.len() {
            let comp = Complex::new(self.reals[ct],self.imags[ct]);
            result.push((20.0*comp.norm().log(10.0),comp.arg()));
        }
        result
    }


    pub fn diff(raw_data: &Vec<f64>) -> Vec<f64> {
        let mut result = Vec::new();
        for ct in 0..raw_data.len()-1 {
            result.push(raw_data[ct+1]-raw_data[ct]);
        }
        result
    }
    /// Finds peaks in the data
    /// If you want to find maxima, you call it with Some(PeakType::Maximum)
    /// If you want to find all peaks, you call it with None
    pub fn find_peaks(&self,peak_type: Option<PeakType>,data_type:&DataType) -> Vec<usize> {
        let modifier = match peak_type {
            Some(PeakType::Maximum) => 1.0,
            Some(PeakType::Minimum) => -1.0,
            None => 0.0
        };
        let mut df1 = VariableResult::diff(&self.get_data(&data_type));
        df1.insert(0,1.0);
        let mut df2 = VariableResult::diff(&VariableResult::diff(&self.get_data(&data_type)));
        df2.insert(0,1.0);
        df2.insert(0,1.0);
        let mut result = Vec::new();
        // Find positions, where df1 changes sign and where df2 is lower than zero
        for ct in 0..df1.len()-1 {
            if df1[ct]*df1[ct+1] < 0.0 && df2[ct] < 0.0 && (df1[ct] * modifier > 0.0 || modifier == 0.0) {
                //Peak detected
                result.push(ct);
            }
        }
        result
    }

    pub fn get_data(&self,data_type:&DataType) -> Vec<f64>{
        match data_type {
            &DataType::Real              => self.reals.clone(),
            &DataType::Imaginary         => self.imags.clone(),
            &DataType::Absolute          => self.get_abs(),
            &DataType::AbsoluteDecibel   => self.get_abs_in_decibel(),
            &DataType::Argument          => unimplemented!()
        }
    }

    pub fn min(&self,data_type:&DataType) -> (usize,f64) {
        let data = self.get_data(data_type);
        let value = data.iter().fold(1./0. /* -inf */, |a,b| f64::min(a,*b));
        (data.iter().position(|r| r.eq(&value)).unwrap(),value)
    }

    pub fn max(&self,data_type:&DataType) -> (usize,f64) {
        let data = self.get_data(data_type);
        let value = data.iter().fold(-1./0. /* -inf */, |a,b| f64::max(a,*b)) ;
        (data.iter().position(|r| r.eq(&value)).unwrap(),value)
    }

    fn normalize(&self,data_type:&DataType) -> Vec<f64> {
        let (_loc_max,max) = self.max(data_type);
        let (_log_min,min) = self.min(data_type);
        let original = self.get_data(data_type);
        let mut result = Vec::new();
        for value in original {
            result.push((value-min)/(max-min));
        }
        result
    }

    pub fn avg_normalized(&self, data_type:&DataType, frequency: &VariableResult) -> f64 {
        let original = self.get_data(data_type);
        let normalized = VariableResult::diff(&frequency.normalize(&DataType::Real));
        let mut result = 0.0;
        for ct in 0..original.len()-1 {
            result += (original[ct+1]+original[ct])/2.0*normalized[ct];
        }
        result
    }
    /// Returns the next occurence of a value higher/lower than the value at the starting
    /// point plus/minus the offset in both directions
    pub fn next_value_around(&self, data_type:&DataType, starting_point:usize, offset:f64, maximum: bool) -> (Option<usize>,Option<usize>) {
        let modifier = if maximum { -1.0 } else { 1.0 };
        let data = self.get_data(data_type);
        let value_at_starting_point =  data[starting_point];
        let mut ct_left = 0;
        let mut ct_right = 0;
        let mut found_left = false;
        let mut found_right = false;
        while starting_point-ct_left >= 0 && starting_point+ct_right < data.len() && !found_right && !found_left {
            if !found_left {
                let left = data[starting_point-ct_left];
                if (left - value_at_starting_point) * modifier > offset {
                    found_left = true;
                }
                else {
                    ct_left += 1;
                }
            }
            if !found_right {
                let right = data[starting_point+ct_right];
                if (right - value_at_starting_point) * modifier > offset {
                    found_right = true;
                }
                else {
                    ct_right += 1;
                }
            }
        }
        (if found_left {Some (ct_left)} else  {None}, if found_right {Some (ct_right)} else  {None})
    }

    pub fn calculate_fitness(&self, frequency: &VariableResult) -> f64 {
        let avg = self.avg_normalized(&DataType::AbsoluteDecibel,&frequency);
        //println!("{:?}", avg);
        let (loc_max,max) = self.max(&DataType::AbsoluteDecibel);
        let (loc_min,min) = self.min(&DataType::AbsoluteDecibel);
        (avg+100.0)/100.0+1.0/((max-min)/30.0)+ (self.get_data(&DataType::AbsoluteDecibel)[0])/30.0
    }

    pub fn plot(&self,frequency: &VariableResult,fg:&mut Figure,title:&str,color:&str) {
        let y = self.get_data(&DataType::AbsoluteDecibel);
        let x = frequency.get_data(&DataType::Real);


        fg.axes2d()
            .lines(&x,&y,&[Caption(title),Color(color)])
            .set_x_log(Some(10.0))
            .set_y_range(AutoOption::Fix(-70.0),AutoOption::Fix(0.0));

    }
 }

pub enum DataType {
    Real,
    Imaginary,
    Absolute,
    AbsoluteDecibel,
    Argument
}

pub enum PeakType {
    Minimum,
    Maximum
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
