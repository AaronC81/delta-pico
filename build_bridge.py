import subprocess, os, shutil

Import("env")

root_dir = env.GetLaunchDir()

print("Building rbop_bridge...")

# "cargo build" the bridge project
bridge_dir = os.path.join(root_dir, "rbop_bridge")
subprocess.check_output(["cargo", "build"], cwd=bridge_dir)

print("Done!")
