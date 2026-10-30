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

## Attached native commands

The Electron preview can start the bundled Cube CLI for a selected saved Pipeline
using **Run with Cube CLI** in its Trail canvas. The result appears inline and the
Run's evidence records **Started by: Cube CLI**. This uses the same local
coordinator as **Run cell**; Cube does not read Play's private database.

The attached client supports `cube play asset show ID --revision N`,
`cube play pipeline show ID --revision N`, and `cube play run start`, `status`
and `cancel`. These commands require an inherited session supplied by Play; typing
them in an unrelated terminal returns `context_unavailable`. There is no automatic
launcher, shell or network fallback for contextual commands. The existing
`cube play [path]` launcher delegation is a separate interface.

See the [attached command reference](https://github.com/cubearc/play/blob/main/docs/reference/desktop-runtime.md#attached-cube-cli-commands)
for exact syntax, context and transport limits. External terminal attachment and
native PTYs are not implemented by this client.
