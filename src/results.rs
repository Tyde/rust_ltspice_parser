//! This module contains all Functions to work with a single Step
//!
//! The most important struct is the VariableResult. This struct contains all data and various functions to work with that data.
use num::complex::Complex;
use gnuplot::{Figure, Caption, Color,AxesCommon,AutoOption};
use std::str::FromStr;


/// This struct contains all data for one result of a simulation step. The data is accessible in different formats
///
/// # Examples
///
/// ```no_run
/// //Gets the avsolute values in decibel
/// let decibel_data = result.get_data(&DataType::AbsoluteDecibel);
/// let starting_freq = decibel_data[0];
/// ```
#[derive(Debug,Clone)]
pub struct VariableResult<'a> {
    variable: &'a SimulationVariable,
    reals: Vec<f64>,
    imags: Vec<f64>
}


impl<'a> VariableResult<'a> {
	/// Creates a new instance of a VariableResult
	///
	/// To get a VariableResult from a `.raw`-File, the use of `SteppedSimulation::from_files` is recommended 
	pub fn new(variable: &'a SimulationVariable, reals:Vec<f64>, imags:Vec<f64>) -> Self {
		VariableResult {
			variable: variable,
			reals: reals,
			imags: imags
		}
	}
	
	/// Returns the data as a vector.
	/// 
	/// The contents are determined by the `data_type`
	/// ## Example
	/// ```no_run
	/// let imaginary_values = result.get_data(&DataType::Imaginary);
	/// ```
    pub fn get_data(&self,data_type:&DataType) -> Vec<f64>{
        match data_type {
            &DataType::Real              => self.reals.clone(),
            &DataType::Imaginary         => self.imags.clone(),
            &DataType::Absolute          => self.get_abs(),
            &DataType::AbsoluteDecibel   => self.get_abs_in_decibel(),
            &DataType::Argument          => unimplemented!()
        }
    }
    
	
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



	/// Calculates a difference vector of your raw data: `d(n) = x(n+1)-x(n)`
    pub fn diff(raw_data: &Vec<f64>) -> Vec<f64> {
        let mut result = Vec::new();
        for ct in 0..raw_data.len()-1 {
            result.push(raw_data[ct+1]-raw_data[ct]);
        }
        result
    }
    /// Finds peaks in the data
    ///
    /// To find maxima, call it with `Some(PeakType::Maximum)`
    /// To find all peaks, call it with None
    ///
    /// ## Example 
    /// ```no_run
    ///	let peaks = result.find_peaks(Some(PeakType::Maximum),&DataType::AbsoluteDecibel);
    /// ```
    pub fn find_peaks(&self,peak_type: Option<PeakType>,data_type:&DataType) -> Vec<usize> {
        let modifier = match peak_type {
            Some(PeakType::Maximum) => 1.0,
            Some(PeakType::Minimum) => -1.0,
            None => 0.0
        };
        /*
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
        }*/
        let data = &self.get_data(&data_type);
        let mut result = Vec::new();
        for ct in 1..data.len()-1 {
        	if (data[ct] - data[ct-1])*modifier > 0.0 && (data[ct] - data[ct+1])*modifier > 0.0 {
        		result.push(ct);
        	}
        }
        result
    }


	/// Returns a single data point converted to the given data type
    pub fn get_data_point(&self, data_type:&DataType, index:usize) -> Option<f64> {
    	if index < self.reals.len() {
    		 match data_type {
            &DataType::Real              => Some(self.reals[index]),
            &DataType::Imaginary         => Some(self.imags[index]),
            &DataType::Absolute          => Some(Complex::new(self.reals[index],self.imags[index]).norm()),
            &DataType::AbsoluteDecibel   => Some(20.0*Complex::new(self.reals[index],self.imags[index]).norm().log(10.0)),
            &DataType::Argument          => unimplemented!()
        }
    	} else {
    		None
    	}
    }
    /// Returns the length of the data set
    pub fn len(&self) -> usize {
    	self.reals.len()
    }

	/// Returns the minimal value in the data set of the given data type
    pub fn min(&self,data_type:&DataType) -> (usize,f64) {
        let data = self.get_data(data_type);
        let value = data.iter().fold(1./0. /* -inf */, |a,b| f64::min(a,*b));
        (data.iter().position(|r| r.eq(&value)).unwrap(),value)
    }
	/// Returns the maximal value in the data set of the given data type
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
	/// Returns the average of the values in the given data types. This average is normalized so that
	/// the logarithmic scaling of the frequency is not present anymore. This method tries to compute the average
	/// in a way, that it would seem, that the frequency is scaled linearly. 
    pub fn avg_normalized(&self, data_type:&DataType, frequency: &VariableResult) -> f64 {
        let original = self.get_data(data_type);
        let normalized = VariableResult::diff(&frequency.normalize(&DataType::Real));
        let mut result = 0.0;
        for ct in 0..original.len()-1 {
            result += (original[ct+1]+original[ct])/2.0*normalized[ct];
        }
        result
    }
    
    /// Searches the dataset for the first value after the given search frequency. To search for a value near 500 Hz call
    ///
    /// ```no_run
    /// let (position_in_dataset, value) = find_value_near_freq(&DataType::AbsoluteDecibel,&frequency_dataset,500.0)
    /// ```
    pub fn find_value_near_freq(&self, data_type:&DataType, frequency: &VariableResult, search_freq:f64) -> (usize,f64) {
     	let fqs = frequency.get_data(&DataType::Real);
     	let mut index = 0;
     	let mut found = false;
     	for ct in 0..frequency.len(){
     		if fqs[ct] >= search_freq {
     			index = ct;
     			found = true;
     			break;
     		}
     	}
     	if !found {
     		println!("Frequenzen sind: {:#?} Gesucht ist {:?}", fqs,search_freq);
     		panic!("Frequenz nicht vorhanden!");
     	}
     	
     	(index,self.get_data_point(data_type,index).unwrap())
     	
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
        let mut position_left = 0;
        let mut position_right = 0;
        while  (!found_left && !found_right) 
        	|| (found_right && !found_left && starting_point>=ct_left) 
        	|| (found_left && !found_right && starting_point+ct_right < data.len()){
    		if starting_point<ct_left && starting_point+ct_right > data.len() {
    			break;
    		}
        	//println!("Starting point: {:?} Looping: ct_left {:?} ct_right: {:?} found_right {} found_left{}",starting_point,ct_left,ct_right,found_right,found_left);
            if !found_left {
            	if starting_point > ct_left {
	                let left = data[starting_point-ct_left];
	                //println!("Left is {:?} ct is {:?} val@starting_point is {:?}", left,ct_left,value_at_starting_point);
	                if (left - value_at_starting_point) * modifier > offset {
	                    found_left = true;
	                    position_left = starting_point-ct_left;
	                    //println!("Found left");
	                }
            	}
            	 ct_left += 1;
            }
            if !found_right {
            	if starting_point+ct_right < data.len()-1 {
	                let right = data[starting_point+ct_right];
	                if (right - value_at_starting_point) * modifier > offset {
	                    found_right = true;
	                    position_right = starting_point+ct_right;
	                }
            	}
                ct_right += 1;
                
            }
        }
        
        (if found_left {Some (position_left)} else  {None}, if found_right {Some (position_right)} else  {None})
    }
    
	/// Used for myself. Not really documented
	pub fn calculate_resonance_penalty(&self,frequency: &VariableResult) -> f64{
		let resonances = self.find_peaks(Some(PeakType::Maximum), &DataType::AbsoluteDecibel);

		let mut penalty = 0.0;
        for resonance in resonances {
        	
        	let (left_value,right_value) = self.next_value_around(&DataType::AbsoluteDecibel, resonance, 3.0, true);
        	
        	if left_value.is_some() && right_value.is_some() {
        		let left_freq = frequency.get_data_point(&DataType::Real, left_value.unwrap()).unwrap();
        		let right_freq = frequency.get_data_point(&DataType::Real, right_value.unwrap()).unwrap();
        		
        		penalty += 10.0/(right_freq-left_freq);
        	}
        	if right_value.is_some() && left_value.is_none() {
        		let right_freq = frequency.get_data_point(&DataType::Real, right_value.unwrap()).unwrap();
        		let res_freq = frequency.get_data_point(&DataType::Real, resonance).unwrap();
        		if right_freq-res_freq < 100.0 {
        			penalty += 10.0/(right_freq-res_freq);
        		}
        		
        	}
        	
        }
        
        let anti_resonances = self.find_peaks(Some(PeakType::Minimum), &DataType::AbsoluteDecibel);
	    for resonance in anti_resonances {
	      	let (left_value,right_value) = self.next_value_around(&DataType::AbsoluteDecibel, resonance, 3.0, false);
	      	let res_freq = frequency.get_data_point(&DataType::Real, resonance).unwrap();
	      	if left_value.is_some() && right_value.is_none() {
	      		let left_freq = frequency.get_data_point(&DataType::Real, left_value.unwrap()).unwrap();
        		
        		if res_freq-left_freq< 100.0 {
        			penalty += 20.0/(res_freq-left_freq);
        		}
        		
	      	}
	      	if left_value.is_some() && right_value.is_some() {
	      		let left_freq = frequency.get_data_point(&DataType::Real, left_value.unwrap()).unwrap();
        		let right_freq = frequency.get_data_point(&DataType::Real, right_value.unwrap()).unwrap();
        		penalty += 10.0/(right_freq-left_freq);
	      	}
	    }
        penalty
	}


	/// Used for myself. Not really documented
    pub fn calculate_fitness(&self, frequency: &VariableResult, averages: [f64;5], deviations: [f64;5]) -> ([f64;6],f64,[f64;6]) {
    	
        let avg = self.avg_normalized(&DataType::AbsoluteDecibel,&frequency);
        //println!("{:?}", avg);
        let (_,one_k_value) = self.find_value_near_freq(&DataType::AbsoluteDecibel,&frequency, 1000.0);
        let (_,one_h_value) = self.find_value_near_freq(&DataType::AbsoluteDecibel,&frequency, 100.0);
        let (_,max) = self.max(&DataType::AbsoluteDecibel);
        let (_,min) = self.min(&DataType::AbsoluteDecibel);
      
        //println!("Deviations {:?} Averages: {:?}", deviations,averages);
        let mut arguments:[f64;6]= [0.0;6];
        let mut original_values_of_arguments:[f64;6]= [0.0;6];
        //println!("Average is: {:?}", avg);
        arguments[0] =  1.5 * logistic_function(avg, deviations[0],averages[0]);
        arguments[1] = logistic_function(1.0/(max-min),deviations[1], averages[1]);
        arguments[2] =  1.5 * logistic_function(self.get_data_point(&DataType::AbsoluteDecibel,0).unwrap(),deviations[2], averages[2]);
        arguments[3] =  1.5 * logistic_function(one_k_value,deviations[3], averages[3]);
        arguments[4] =  logistic_function(one_h_value,deviations[4], averages[4]);
        arguments[5] = -1.0 * self.calculate_resonance_penalty(&frequency);


		original_values_of_arguments[0] = avg;
		original_values_of_arguments[1] = max-min;
		original_values_of_arguments[2] = self.get_data_point(&DataType::AbsoluteDecibel,0).unwrap();
		original_values_of_arguments[3] = one_k_value;
		original_values_of_arguments[4] = one_h_value;
		original_values_of_arguments[5] = arguments[5];
		
        
        //println!("Arguments: {:#?}", arguments);
        
        
        
        
        (arguments, arguments.into_iter().fold(0.0, |acc, &x| acc + x),original_values_of_arguments)
    }
    
   
	/// Uses GNUplot to plot the Bode plot (just absolute, no phase)
    pub fn plot(&self,frequency: &VariableResult,fg:&mut Figure,title:&str,color:&str) {
        let y = self.get_data(&DataType::AbsoluteDecibel);
        let x = frequency.get_data(&DataType::Real);


        fg.axes2d()
            .lines(&x,&y,&[Caption(title),Color(color)])
            .set_x_log(Some(10.0))
            .set_y_range(AutoOption::Fix(-70.0),AutoOption::Fix(0.0));

    }
 }

/// This enum is used to tell the SteppingVariable, which kind of Data it should use in different functions
pub enum DataType {
	/// Take the real part of the complex value
    Real,
    /// Take the imaginary part of the complex value
    Imaginary,
    /// Take the absolute part of the complex value
    Absolute,
    /// Take the absolute part of the complex value as decibel (follows: `20*log(abs(x))`)
    AbsoluteDecibel,
    /// Take the argument of the complex value. This is not yet implemented.
    Argument
}


pub enum PeakType {
    Minimum,
    Maximum
}

/// This struct contains the name and the value of one param in a step
#[derive(Default,Debug,PartialEq)]
pub struct SteppingVariable {
	/// The name of the `.param` parameter
    pub name: String,
    /// The value of the parameter in this step
    pub value: f32
}

impl SteppingVariable {
	/// Creates a new SteppingVariable using the string format, which can be usually found in the `.log` file
    pub fn new(log_excerpt:&str) -> Self {
        let split:Vec<&str> = log_excerpt.split('=').collect();
        SteppingVariable {
            name: split[0].to_owned(),
            value: f32::from_str(split[1]).unwrap()
        }
    }
}
   
///This structure defines a variable on the netlist of the simulation
#[derive(Debug,Hash,Eq,PartialEq)]
pub struct SimulationVariable {
	/// The internal id
    pub id:u16,
    /// The string in the netlist
    pub name:String,
    /// The type of the variable
    pub var_type:VariableType,
}

/// This enum is used to determine the physical type of the variable
#[derive(Debug,Hash,Eq,PartialEq)]
pub enum VariableType {
    Frequency,
    Voltage,
    Current,
    Unknown
}

/// A step is a list of SteppingVariables
pub type Step = Vec<SteppingVariable>;

/// Calculates the logistic function which is scaled by a scaling factor and an offset
fn logistic_function(input:f64,scale:f64,offset:f64) -> f64 {
	let x = (input-offset)/scale ;
	let e:f64 = 2.71828182846;
	(1.0/(1.0+e.powf(-x)))
}