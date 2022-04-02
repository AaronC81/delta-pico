import os, subprocess, tempfile
from font_tools import generate_font_source

root_dir = os.path.dirname(os.path.realpath(__file__))

try:
    ffpython = os.environ["DELTA_PICO_FFPYTHON"]
except KeyError:
    print("Error: Could not locate FFPython.")
    print("FFPython is a special Python distribution provided by FontForge.")
    print("To build fonts, you must:")
    print("  1. Install FontForge")
    print("  2. Set the DELTA_PICO_FFPYTHON environment variable to the path to the FFPython executable")
    exit(1)

fonts_to_build = []
with open(os.path.join(root_dir, "font", "fontspec")) as f:
    fonts_to_build = f.readlines()

source = "pub mod font_data {\n\n"

for font in fonts_to_build:
    name, path, size = font.split()
    glyphs_dir = tempfile.mkdtemp()

    # Invoke FFPython to generate glyphs
    subprocess.check_output([
        ffpython, f"{root_dir}/font_tools.py", "glyphs",
        os.path.join(root_dir, "font", path), str(size), glyphs_dir
    ])

    # Generate source
    source += generate_font_source(name, glyphs_dir)

source += "\n}"

# Print source, Rust side will save it
print(source)
