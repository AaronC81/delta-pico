import subprocess, os, shutil

Import("env")

root_dir = env.GetLaunchDir()

print("Building resources...")

# "cargo build" the bridge project
res_dir = os.path.join(root_dir, "res")
for file in os.listdir(res_dir):
    if file.endswith(".vlw"):
        output = subprocess.check_output(["xxd", "-i", file], cwd=res_dir).decode()
        output = "#pragma once\n\nconst " + output

        with open(os.path.join(res_dir, file.replace(".vlw", ".h")), "w") as f:
            f.write(output)

print("Done!")
