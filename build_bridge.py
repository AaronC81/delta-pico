import subprocess, os, shutil

Import("env")

root_dir = env.GetLaunchDir()

print("Building Rust component...")

# "cargo build" the bridge project
bridge_dir = os.path.join(root_dir, "rust")
subprocess.check_output(["cargo", "build", "--release"], cwd=bridge_dir)

print("Done!")
