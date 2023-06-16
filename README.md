# MCVideo
## A video player for Minecraft

MCVideo 3 is the successor to the unpublished MCVideo 2, which was iself a successor to the original datapack-based [MCVideo](https://github.com/MediumDifficulty/mcvideo).

Unlike it's predecessors with MCVideo 3, you don't need to run and install a generated resource pack for sound when you load a new video. The pack is generated at startup then hosted by the server itself.

## Requirements
- [FFMpeg](https://ffmpeg.org/download.html) installed and visible to the mcvideo executable (added to the path or in the same directory)

## Usage
run the mcvideo executable to show the help menu and it's straight forward from there. When you join the server, type !play in the chat to start playing the video.

## How it works
MCVideo 3 works by hosting a custom server built with [Valence](https://github.com/valence-rs/valence). The playback of the video works by having a grid of maps in item frames and sending map data packets to the client every frame. Every [Clip duration] seconds the server tells the client to play the next audio clip (this isn't entirely necessary now as it will skip frames and not play frames to be in realtime, set to a higher number of seconds than the video length to disable audio segmentation). The resource pack is generated at startup then hosted asynchronously by [Hyper](https://github.com/hyperium/hyper).