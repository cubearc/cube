# Run your first Cube operation

[Install Cube](installation.md), then decode a small known input:

```sh
cube decode base64 'SGVsbG8=' --output raw
```

The result is `Hello`, written as five exact bytes without an added newline.
Without `--output raw`, the decoder displays those bytes as `48656c6c6f`.
See [bytes and text](concepts/data.md) for that distinction.

Inspect the installed command and its operation metadata:

```sh
cube decode base64 --help
cube ops describe encoding.base64.decode
```

Continue with [inspect a local file](guides/files.md), choose another
[task](guides/index.md), or use the [Python interface](interfaces/python.md).

<details>
<summary>Earlier tutorial links</summary>

<a id="prepare-your-tools"></a>

[Prepare your tools](installation.md).

<a id="install-the-cli"></a>

[Install the cli](installation.md#cli).

<a id="discover-an-operation"></a>

[Discover an operation](interfaces/cli.md#discover-operations).

<a id="encode-and-decode"></a>

[Encode and decode](guides/encoding.md).

<a id="inspect-a-local-file"></a>

[Inspect a local file](guides/files.md).

<a id="keep-binary-output-intact"></a>

[Keep binary output intact](interfaces/cli.md#choose-an-output-format).

<a id="recover-an-embedded-file"></a>

[Recover an embedded file](guides/recover-zip.md).

<a id="continue"></a>

[Continue](guides/index.md).

</details>
