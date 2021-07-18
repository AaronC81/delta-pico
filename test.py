import subprocess, os, shutil

Import("env")

def test(source, target, env):
    root_dir = env.GetLaunchDir()

    # Create a test bin directory
    test_bin = os.path.join(root_dir, "test", "bin")
    shutil.rmtree(test_bin, ignore_errors=True)
    os.mkdir(test_bin)    

    # Compile a subset of the source, and the test code
    assert subprocess.run([
        "g++",
        "-g",
        os.path.join(root_dir, "src", "maths", "evaluator.cpp"),
        os.path.join(root_dir, "src", "maths", "tokens.cpp"),
        os.path.join(root_dir, "test", "evaluator.cpp"),
        f'-I{os.path.join(root_dir, "include")}',
        f'-o{os.path.join(test_bin, "evaluator")}'
    ]).returncode == 0

    # Run test
    assert subprocess.run([
        os.path.join(test_bin, "evaluator")
    ]).returncode == 0

    print("Unit tests passed")

# Run after every build, not just ones where the source changes
env.AddPostAction("checkprogsize", test)
