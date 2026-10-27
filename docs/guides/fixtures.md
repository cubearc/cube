# Create original media fixtures


These original fixtures need only Python's standard library. The PNG has eight
RGB pixels whose red-channel low bits spell the byte `A`. The WAVE file contains
four mono 16-bit PCM samples at 8000 frames/second.

```sh
python3 - <<'PY'
import base64
import wave
from pathlib import Path

png = (
    "iVBORw0KGgoAAAANSUhEUgAAAAQAAAACCAIAAADwyuo0AAAAGElEQVR4nGNh"
    "YmBgZGD4zwACLAwwFlAIABzlAguTT+b/AAAAAElFTkSuQmCC"
)
Path("sample.png").write_bytes(base64.b64decode(png))
with wave.open("sample.wav", "wb") as output:
    output.setnchannels(1)
    output.setsampwidth(2)
    output.setframerate(8000)
    output.writeframes(b"\x00\x00\x00\x40\x00\x00\x00\xc0")
PY
```



Use these inputs in the [image](images.md) and [audio](audio.md) tasks.
