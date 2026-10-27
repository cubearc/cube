# Open Cube Play

Install the [Play host launcher](https://github.com/cubearc/play/blob/main/docs/installation.md)
beside Cube. The CLI delegates to that separately installed application:

```sh
cube play
cube play ./investigation
cube play sample.mp4
cube play rabbit.cube
```

The optional path defaults to the current directory. A file opens its parent
workspace and selects that file. A challenge intent does not execute its contents.
Read [Play's first steps](https://github.com/cubearc/play/blob/main/docs/getting-started.md).

If the launcher is installed elsewhere, set `CUBE_PLAY_LAUNCHER` to its trusted
absolute executable path. Cube checks the `cube-play-launcher 1` protocol before
delegation. The launcher owns Docker setup, readiness, browser opening and shutdown.
