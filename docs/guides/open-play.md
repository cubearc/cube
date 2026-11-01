# Open Cube Play

Install the [Electron desktop app](https://github.com/cubearc/play/blob/main/docs/installation.md)
and open it from Cube:

```sh
cube play
cube play ./puzzle
cube play clue.png
cube play challenge.json
```

A folder uses a managed private Workspace separate from the source directory.
Files enter the selected local notebook; a canonical Challenge package imports an
independent Exploration. Opening an item never executes its contents. With no
path, Play resumes its saved Workspace. Read [Play's first steps](https://github.com/cubearc/play/blob/main/docs/getting-started.md).

Place the `cube-play` desktop executable beside Cube, or set `CUBE_PLAY_APP` to
its trusted absolute executable path. Cube verifies the `cube-play 1` desktop
protocol before delegation. The desktop owns windows, private storage and native
execution. Its standalone preview does not require a container runtime.

## Use an external terminal

In Play, open **Toggle Cube terminal**, then choose **Use external terminal**.
Copy the displayed `cube play --context '/absolute/path/context.json'` prefix into
your own terminal and append a contextual command:

```sh
cube play --context /absolute/path/context.json asset show ASSET_ID --revision 1
cube play --context /absolute/path/context.json pipeline show PIPELINE_ID --revision 1
cube play --context /absolute/path/context.json run status RUN_ID
cube play --context /absolute/path/context.json run cancel RUN_ID
```

Replace the uppercase identities with the exact IDs shown in Play. `run start
--pipeline ID --revision N --target NODE_ID` requires the saved selection pinned
when the connection was issued. Each accepted invocation gets a new Run; the
engine checks its exact Stage, Trail, Asset and Pipeline revisions.

The private context file expires after ten minutes and stays bound to its original
Stage. It uses an authenticated local socket on macOS/Linux; the client receives
no database or general filesystem authority. **Disconnect external terminal** or
**File → Disconnect External Terminals** revokes it. App quit also disconnects it;
a missing, expired or unsafe file fails without ambient discovery or a shell
fallback. External attachment on Windows requires its separate native policy.

<a id="attached-native-commands"></a>

## Attached commands inside Play

**Run with Cube CLI** in a Trail canvas starts the bundled CLI through the same
coordinator as **Run cell**. Its inline evidence records **Started by: Cube CLI**.
Without an issued context file or an inherited app session, contextual commands
return `context_unavailable`.

The in-app terminal runs the real `cube play session` client in a native PTY. Type
`help` for contextual commands, or choose **Insert selected Run** and press Enter.
`selected` invokes the saved selection with a fresh Run identity. Ctrl+C clears the
command draft; use `run cancel ID` to cancel accepted work. Closing the terminal
leaves accepted Runs supervised by the engine. **Reconnect** selects the current
Trail and starts an empty transcript.

The in-app console accepts contextual Cube commands. It does not interpret shell
expressions or launch arbitrary executables. See [Play's terminal guide](https://github.com/cubearc/play/blob/main/docs/terminal.md)
and the [command reference](https://github.com/cubearc/play/blob/main/docs/reference/desktop-runtime.md#attached-cube-cli-commands)
for exact context, syntax and limits.
