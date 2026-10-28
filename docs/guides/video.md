# Inspect a video file

Read the tracks and timing declared by a local MP4, MOV or WebM file.
Use one of your own files in place of `sample.mp4`:

```sh
cube video inspect sample.mp4 --json
```

The report lists track IDs, kinds, codecs and time bases. It describes the
container; it does not verify that the media payload can be played.

## Extract and compare frames

Frame extraction supports the [MOV/JPEG subset](../reference/topics/video-container-metadata.md).
For a compatible file named `clip.mov`, choose the video track ID from its
inspection report. The following example uses track 1 and frames 0 and 2:

```sh
mkdir frames
cube video frames clip.mov --track-id 1 --indexes 0,2 --output-dir frames
cube image compare frames/track-1-frame-0.png frames/track-1-frame-2.png --json
```

The PNGs contain the decoded pixels. Keep the extraction report with them: it
records each frame's source range, timing and encoded hash.

## Extract an audio track

Audio extraction supports the specific uncompressed PCM8 MOV profile described in
the reference. For a compatible `sound.mov` with audio track 1:

```sh
cube video audio sound.mov --track-id 1 --output track.wav
cube audio inspect track.wav --json
```

The result is a new WAVE file with the original sample data. Existing output paths
are rejected. Other codecs require tools outside this extraction API.

See [video formats and limits](../reference/topics/video-container-metadata.md)
before choosing a source file.

<a id="cli"></a>
<a id="inspect-a-saved-video-container"></a>
<a id="inspect-video-containers"></a>
<a id="inspect-video-metadata"></a>
<a id="python"></a>
<a id="reference"></a>
