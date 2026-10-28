# Send a command result to the Lab

Save a command's result as a capture file, then import it into Cube Play:

```sh
cube --lab-capture=reverse.json text reverse --text CUBE
# EBUC
```

The command prints `EBUC` and creates `reverse.json` with the typed result.
Choose a new filename; existing destinations are rejected.

In Play, select the capture file in Files, then choose **Import selected CLI capture**
in Explorations. The file must be inside an authorized source or import location.

## Capture from the Play terminal

Inside Play, omit the path to stage the capture for the current terminal session:

```sh
cube --lab-capture text reverse --text CUBE
```

Select **Refresh staged captures**, then **Send to Lab**. Save the imported result
as a named artifact if you want to reuse it. Ended terminal sessions lose unpublished
captures; published artifacts persist.

Captures are supported for `text reverse`, `decode hex`, `image invert` and
`image histogram`. Other commands reject the option.

A capture records a result, not the original input or proof that the command ran.
See the [capture format and limits](../reference/lab-captures.md) and
[Play terminal guide](https://github.com/cubearc/play/blob/main/docs/terminal.md#send-a-computed-result-to-the-lab).

<a id="send-a-typed-cli-result-to-the-lab"></a>
