#!/usr/bin/env python3

# Adapted from: https://gist.github.com/slazav/4853bd36669bb9313ddb83f51ee1cb82
# Make sure you have:
#   - Installed FreeCAD
#   - Updated FREECADPATH to point to its `lib` directory
#   - Launched FreeCAD and selected the OpenSCAD workspace at least once before
#   - Installed the `ply` Python library with pip

# Path to FreeCAD.so
FREECADPATH = '/Applications/FreeCAD.app/Contents/Resources/lib'
import sys
sys.path.append(FREECADPATH)

if len(sys.argv)<3:
  print("Usage: sys.argv[0] <in_file> <out_file>")
  sys.exit(1)

iname=sys.argv[1]
oname=sys.argv[2]

import FreeCAD
import Part

p = FreeCAD.ParamGet("User parameter:BaseApp/Preferences/Mod/OpenSCAD")
p.SetBool('useViewProviderTree', True)
p.SetBool('useMultmatrixFeature', True)
p.SetInt('useMaxFN', 50)

FreeCAD.loadFile(iname)

# iterate through all objects
for o in App.ActiveDocument.Objects:
  # find root object and export the shape
  if len(o.InList)==0:
    o.Shape.exportStep(oname)
    sys.exit(0)

print("Error: can't find any object")
sys.exit(1)
