#!/bin/sh
RESOLUTION=${RESOLUTION:-506x253}
ffmpeg -v warning -f rawvideo -pixel_format rgb32 -framerate 30 -video_size $RESOLUTION -i - out-%04d.png
