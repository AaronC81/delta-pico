import subprocess, os, shutil

Import("env")

def test(source, target, env):
    # No tests right now, will keep this around for later
    pass

# Run after every build, not just ones where the source changes
env.AddPostAction("checkprogsize", test)
