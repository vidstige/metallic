#!/bin/sh
RESOLUTION=${RESOLUTION:-506x253}
FPS=${FPS:-30}
if [[ $1 == *.gif ]]; then
    VF=-vf "fps=$FPS,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse"
else
    VF="-crf 15"
fi
echo ffmpeg -hide_banner -f rawvideo -pixel_format rgb32 -framerate $FPS -video_size $RESOLUTION -i - -metadata comment="vidstige" $VF $1
ffmpeg -hide_banner -f rawvideo -pixel_format rgb32 -framerate $FPS -video_size $RESOLUTION -i - -metadata comment="vidstige" $VF $1
