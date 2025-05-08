import json 
import sys 
from pathlib import Path
from Timer import Timer
from VRPInstance import VRPInstance, IPSolver, HopefulDream

def main(filepath : str):
	filename = Path(filepath).name
	watch =  Timer()
	
	watch.start()
	inst = VRPInstance(filepath)
	new_dream = HopefulDream(inst)
	min_val = new_dream.dream()
	watch.stop()
	

		

	sol_dict ={
		"Instance" : filename,
		"Time" : round(watch.getElapsed(), 2),
		"Result" : "--",
		"Solution" : "--"
	}
	print(json.dumps(sol_dict))	

if __name__ == "__main__":
	if len(sys.argv) != 2:
		print("Usage: python main.py <input_file>")
	main(sys.argv[1])