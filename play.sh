#!/bin/sh
RESOLUTION=506x253
ffplay -v warning -f rawvideo -pixel_format rgb32 -framerate 30 -video_size $RESOLUTION -i -
