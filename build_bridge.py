import subprocess, os

print("Building Rust component...")

# "cargo build" the bridge project
root_dir = os.path.dirname(os.path.realpath(__file__))
bridge_dir = os.path.join(root_dir, "rust")
subprocess.check_output(["cargo", "build", "--release"], cwd=bridge_dir)

print("Done!")
