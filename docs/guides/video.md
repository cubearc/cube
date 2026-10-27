# Inspect a saved video container

Supply a local MP4, MOV or WebM file. Metadata inspection does not establish payload validity; extraction supports only the profiles described in the reference.

## CLI

### Inspect video containers

```sh
cube video inspect sample.mp4 --json
cube video inspect sample.webm --json
```


```sh
mkdir frames
cube video frames clip.mov --track-id 1 --indexes 0,2 --output-dir frames
```


```sh
cube video compare clip.mov --track-id 1 --pairs 0:1,0:2 --max-results 100
cube image compare frames/track-1-frame-0.png frames/track-1-frame-2.png --json
```


```sh
cube video audio sound.mov --track-id 1 --output track.wav
cube audio inspect track.wav --json
```


## Python

### Inspect video metadata

```python
from pathlib import Path
import cube

video = cube.inspect_video(Path("sample.mp4").read_bytes())
assert video["verification"] == "declared_container_metadata_media_payload_unverified"
for track in video["tracks"]:
    print(track["id"], track["kind"], track["codec_id"], track["time_base"])
```


```python
from pathlib import Path
import cube

frames = cube.extract_video_frames(Path("clip.mov").read_bytes(), 1, [0, 2])
assert [f["provenance"]["frame_index"] for f in frames] == [0, 2]
for frame in frames:
    print(frame["provenance"]["decode_start_ticks"], frame["provenance"]["time_base"])
    assert not frame["image"].provenance()["source_samples_preserved"]
```


```python
from pathlib import Path
import cube

source = Path("clip.mov").read_bytes()
report = cube.compare_video_frames(source, 1, [(0, 1), (0, 2)], max_results=100)
assert report["pairs"][0]["left"]["frame_index"] == 0
frames = cube.extract_video_frames(source, 1, [0, 2])
comparison = frames[0]["image"].compare(frames[1]["image"], max_results=100)
assert comparison["equal"] == report["pairs"][1]["comparison"]["equal"]
```


```python
from pathlib import Path
import cube

track = cube.extract_video_audio(Path("sound.mov").read_bytes(), 1)
samples = cube.wave_samples(track["data"])
assert len(samples["data"]) == track["provenance"]["pcm_bytes"]
assert track["provenance"]["format"]["bits_per_sample"] == 8
```


## Reference

- [Video container metadata](../reference/topics/video-container-metadata.md)
