import subprocess
import time
import csv
from datetime import datetime
import matplotlib.pyplot as plt

subprocess.run(["cargo", "build", "--execution_speed_tracking", "--manifest-path", "../gb_emu/Cargo.toml"], check = True)
binary_path = "../gb_emu/target/release/gb_emu.exe"

start_time = time.time()
subprocess.run([binary_path], check=True)
end_time = time.time()
duration =  start_time - end_time