import subprocess, os, shutil
from PIL import Image

print("Building resources...")

# "cargo build" the bridge project
root_dir = os.path.dirname(os.path.realpath(__file__))
res_dir = os.path.join(root_dir, "res")

bitmaps = []

def pixel_to_16bit(rgba_tuple):
    # Conversion: http://www.barth-dev.de/online/rgb565-color-picker/
    (red, green, blue, alpha) = rgba_tuple
    if alpha > 200:
        return (((red & 0b11111000)<<8) + ((green & 0b11111100)<<3)+(blue>>3))
    else:
        return None

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

            # Declare a constant to represent a transparency value and run-length marker
            f.write(f"extern const uint16_t {root_name}_transparency;\n\n")
            f.write(f"extern const uint16_t {root_name}_run_length;\n\n")
            f.write(f"const uint16_t {root_name}[] = {{")

            # Build a set of all colours in the image, so we can find a colour we aren't using
            # An unused colour will become a transparency colour
            # (This assumes that it won't use _every_ 16-bit colour!)
            used_colours = set()
            
            # Print dimensions
            f.write(f"{image.width}, {image.height}, {root_name}_transparency, {root_name}_run_length\n")
            need_comma = False
            for x in range(image.width):
                y = 0
                while y < image.height:
                    # Count how many consecutive pixels on this row use this colour
                    colour_rgba = image.getpixel((x, y))
                    colour_16bit = pixel_to_16bit(image.getpixel((x, y)))
                    count = 1
                    y += 1
                    while y < image.height and count < 2**16:
                        if pixel_to_16bit(image.getpixel((x, y))) == colour_16bit:
                            count += 1
                            y += 1
                        else:
                            break

                    # Only complete transparency or opacity is allowed
                    if colour_16bit is not None:
                        used_colours.add(colour_16bit)
                        colour_array = hex(colour_16bit)
                    else:
                        colour_array = f"{root_name}_transparency"

                    f.write(f"// ({x}, {y}) {colour_array} count {count} \n")
                    # If there are more than four pixels of the same colour, use run-length
                    if count > 4:
                        f.write(f", {root_name}_run_length, {count}, {colour_array}")
                    else:
                        f.write(f", {colour_array}" * count)
                    f.write("\n\n")

                f.write("\n\n\n")

            f.write("};\n\n")

            # Find transparency colour and run-length
            for i in range(2**16):
                if i not in used_colours:
                    f.write(f"const uint16_t {root_name}_transparency = {hex(i)};")
                    used_colours.add(i)
                    break
            else:
                print(f"WARNING: No transparency colour found for {file}, compilation will fail")

            for i in range(2**16):
                if i not in used_colours:
                    f.write(f"const uint16_t {root_name}_run_length = {hex(i)};")
                    used_colours.add(i)
                    break
            else:
                print(f"WARNING: No run-length colour found for {file}, compilation will fail")

# Generate a function for looking up bitmaps by name
if len(bitmaps) > 0:
    with open(os.path.join(res_dir, "bitmap.h"), "w") as f:
        f.write("#pragma once\n\n")
        f.write("#include <string.h>\n")
        f.write("#include <stdint.h>\n\n")
        
        for bitmap in bitmaps:
            f.write(f"#include <{bitmap}.h>\n")

        f.write("\nuint16_t* get_bitmap_by_name(char* name) {\n")
        for bitmap in bitmaps:
            f.write(f"  if (strcmp(name, \"{bitmap}\") == 0) return (uint16_t*){bitmap};\n")
        f.write("  return NULL;\n")
        f.write("}")

print("Done!")
