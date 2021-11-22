import subprocess, os, shutil
from PIL import Image

Import("env")

root_dir = env.GetLaunchDir()

print("Building resources...")

# "cargo build" the bridge project
res_dir = os.path.join(root_dir, "res")

bitmaps = []

for file in os.listdir(res_dir):
    if file.endswith(".vlw"):
        output = subprocess.check_output(["xxd", "-i", file], cwd=res_dir).decode()
        output = "#pragma once\n\nconst " + output

        with open(os.path.join(res_dir, file.replace(".vlw", ".h")), "w") as f:
            f.write(output)

    if file.endswith(".png"):
        image = Image.open(os.path.join(res_dir, file))
        root_name = file.replace(".png", "")
        bitmaps.append(root_name)

        with open(os.path.join(res_dir, f"{root_name}.h"), "w") as f:
            # C boilerplate
            f.write("#pragma once\n\n")
            f.write("#include <stdint.h>\n\n")

            # Declare a constant to represent a transparency value
            f.write(f"extern const uint16_t {root_name}_transparency;\n\n")
            f.write(f"const uint16_t {root_name}[] = {{")

            # Build a set of all colours in the image, so we can find a colour we aren't using
            # An unused colour will become a transparency colour
            # (This assumes that it won't use _every_ 16-bit colour!)
            used_colours = set()
            
            # Print dimensions
            f.write(f"{image.width}, {image.height}, {root_name}_transparency")
            need_comma = False
            for x in range(image.width):
                for y in range(image.height):                    
                    (red, green, blue, alpha) = image.getpixel((x, y))

                    # Only complete transparency or opacity is allowed
                    if alpha > 200:
                        # Conversion: http://www.barth-dev.de/online/rgb565-color-picker/
                        pixel_565 = (((red & 0b11111000)<<8) + ((green & 0b11111100)<<3)+(blue>>3))
                        used_colours.add(pixel_565)

                        f.write(f", {hex(pixel_565)}")
                    else:
                        f.write(f", {root_name}_transparency")

            f.write("};\n\n")

            # Find transparency colour
            for i in range(2**16):
                if i not in used_colours:
                    f.write(f"const uint16_t {root_name}_transparency = {hex(i)};")
                    break
            else:
                print(f"WARNING: No transparency colour found for {file}, compilation will fail")

# Generate a function for looking up bitmaps by name
if len(bitmaps) > 0:
    with open(os.path.join(res_dir, "bitmap.h"), "w") as f:
        f.write("#pragma once\n\n")
        f.write("#include <string.h>\n")
        f.write("#include <stdint.h>\n\n")
        
        for bitmap in bitmaps:
            f.write(f"#include <{bitmap}.h>\n")

        f.write("\nuint16_t* getBitmapByName(char* name) {\n")
        for bitmap in bitmaps:
            f.write(f"  if (strcmp(name, \"{bitmap}\") == 0) return (uint16_t*){bitmap};\n")
        f.write("  return NULL;\n")
        f.write("}")

print("Done!")
