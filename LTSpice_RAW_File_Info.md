
# The LTSpice .raw File Format

What I learned from trying to decode it:
The .raw File format is actually rather simple. 

## ASCII Description
The first part of the File is just some ASCII-text, which includes several informations:

*   The title of the LTSpice *.asc file, that was the basis of this simulation
*   The date of the simulation
*	The name of the plot
*	Some flags. Currently I only used the following ones:
  *	complex: The saved values have both a real and a imaginary part
  *	forward: (not sure) The frequency or time counts forwards
  *	log: (not sure) The frequency has a logarithmic scale
  *	stepped: Some parameters in this simulations are stepped with the .step command. For every step, there is a complete dataset in this raw file. The stepping information can be extracted from the .log file
*	The number of variables in this raw file
*	The number of points in this raw file. No of Points / (No of variables * No of steps) = No of ticks per simulation
*	The offset. This was always 0.0 for my simulations
*	The command. This is probably just a Version identifier for LTSpice. I worked with LTspice IV
*	Variables. This is a List of all Variables with id, name and type. Known Types are:
  *	voltage
  * frequency
  * device_current

## The Binary Data
The Binary Data starts directly after "Binary:\n".
Every Datapoint consists of 8 Bytes which are Little Endian 64bit Floating Point values (IEEE754 double). There is no seperator between all values. Even between two steps the doubles just follow each other. Remember: If you have the flag complex, you will have two double values per variable in one tick of a simulation (Real and Imaginary).

If you are using the fastaccess option of ltspice, the content is different. TODO: research content on fastaccess (This is probably the absolute value and the phase)
 
