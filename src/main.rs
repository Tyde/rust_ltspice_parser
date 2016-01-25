extern crate byteorder;

use std::fs;
use std::io::Read;
use std::str;
use byteorder::{ByteOrder, LittleEndian};
use std::str::FromStr;
use std::collections::HashMap;


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




fn main() {
    let mut file = fs::File::open("Draft2.raw").unwrap();
    let mut file_buf = Vec::new();

    file.read_to_end(&mut file_buf).unwrap();

    let mut description = String::new();
    let mut in_binary = false;
    for ct in 0..file_buf.len() {
        if !in_binary {
            let k = &[file_buf[ct].clone()];
            match str::from_utf8(k) {
                Ok(_) => {},
                Err(_) => in_binary = true,
            };
            description.push(file_buf[ct].clone() as char);
        }
        else {
            break;
        }
    }

    let lines:Vec<&str> = description.split('\n').collect();
    let mut state = DescriptionState::Other;
    let mut variables:Vec<SimulationVariable> = Vec::new();
    let mut nr_of_points = 0;
    for line in lines {
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


    println!("Description read finished");

    let mut reals:Vec<Vec<f64>> = Vec::new();
    let mut imags:Vec<Vec<f64>> = Vec::new();

    for v in 0..variables.len() {
        reals.push(Vec::new());
        imags.push(Vec::new());
    }

    in_binary = false;
    let mut freq_step_counter = 0;
    let mut ct = 0;
    let mut lineCT = 0;
    let mut cache:Vec<f64> = Vec::new();
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
            if lineCT%(1024*300)==0 {
                println!(" {:?}%", ((ct as f32/(file_buf.len() as f32))*100.0).round());
            }
            let v = &variables[freq_step_counter];
            let start = ct-1;
            let end = start + 8;
            let real = LittleEndian::read_f64(&file_buf[start..end]);
            let imag = LittleEndian::read_f64(&file_buf[end..end+8]);

            ct+=16;
            //cache.push(real);
            //cache.push(imag);
            reals[freq_step_counter].push(real);
            imags[freq_step_counter].push(imag);
            //imags.get_mut(v).map(|v| v.push(imag));

            freq_step_counter = (freq_step_counter+1)%variables.len();
            lineCT+=1;
        }


    }


    println!("{:?}", reals[0]);



}
