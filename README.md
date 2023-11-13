# Moose

[reindeer](https://github.com/facebookincubator/reindeer) for python.

Moose is a tool which takes python requirements dependencies and generates Buck build rules.

This project builds on top of https://github.com/prefix-dev/rip

## Limitations

Currently only works against the current python
(it uses default `python3` executable to determine, version, os and abi).
This could be fixed in future by generating different `select` and `alias` statements inside the BUCK file,
so the generated file is cross-platform.

## Demo

```
cargo run torch
```

This will create a BUCK file with these deps in the root of the repo.

```
cat BUCK
```

This will show the file that's compatible with the default BUCK2 prelude that meta ships.

```
remote_file(
    name = "typing_extensions-download",
    url = "https://files.pythonhosted.org/packages/24/21/7d397a4b7934ff4028987914ac1044d3b7d52712f30e2ac7a2ae5bc86dd0/typing_extensions-4.8.0-py3-none-any.whl",
    sha256 = "8f92fc8806f9a6b641eaa5318da32b44d401efaac0f6678c9bc448ba3605faa0",
    out = "typing_extensions-4.8.0-py3-none-any.whl",
)

prebuilt_python_library(
    name = "typing_extensions",
    binary_src = ":typing_extensions-download",
    deps = [],
    visibility = ["PUBLIC"],
)

remote_file(
    name = "markupsafe-download",
    url = "https://files.pythonhosted.org/packages/f7/9c/86cbd8e0e1d81f0ba420f20539dd459c50537c7751e28102dbfee2b6f28c/MarkupSafe-2.1.3-cp310-cp310-macosx_10_9_x86_64.whl",
    sha256 = "e09031c87a1e51556fdcb46e5bd4f59dfb743061cf93c4d6831bf894f125eb57",
    out = "MarkupSafe-2.1.3-cp310-cp310-macosx_10_9_x86_64.whl",
)

prebuilt_python_library(
    name = "markupsafe",
    binary_src = ":markupsafe-download",
    deps = [],
    visibility = ["PUBLIC"],
)

remote_file(
    name = "fsspec-download",
    url = "https://files.pythonhosted.org/packages/e8/f6/3eccfb530aac90ad1301c582da228e4763f19e719ac8200752a4841b0b2d/fsspec-2023.10.0-py3-none-any.whl",
    sha256 = "346a8f024efeb749d2a5fca7ba8854474b1ff9af7c3faaf636a4548781136529",
    out = "fsspec-2023.10.0-py3-none-any.whl",
)

prebuilt_python_library(
    name = "fsspec",
    binary_src = ":fsspec-download",
    deps = [],
    visibility = ["PUBLIC"],
)

remote_file(
    name = "sympy-download",
    url = "https://files.pythonhosted.org/packages/d2/05/e6600db80270777c4a64238a98d442f0fd07cc8915be2a1c16da7f2b9e74/sympy-1.12-py3-none-any.whl",
    sha256 = "c3588cd4295d0c0f603d0f2ae780587e64e2efeedb3521e46b9bb1d08d184fa5",
    out = "sympy-1.12-py3-none-any.whl",
)

prebuilt_python_library(
    name = "sympy",
    binary_src = ":sympy-download",
    deps = [":mpmath"],
    visibility = ["PUBLIC"],
)

remote_file(
    name = "networkx-download",
    url = "https://files.pythonhosted.org/packages/d5/f0/8fbc882ca80cf077f1b246c0e3c3465f7f415439bdea6b899f6b19f61f70/networkx-3.2.1-py3-none-any.whl",
    sha256 = "f18c69adc97877c42332c170849c96cefa91881c99a7cb3e95b7c659ebdc1ec2",
    out = "networkx-3.2.1-py3-none-any.whl",
)

prebuilt_python_library(
    name = "networkx",
    binary_src = ":networkx-download",
    deps = [],
    visibility = ["PUBLIC"],
)

remote_file(
    name = "torch-download",
    url = "https://files.pythonhosted.org/packages/67/d8/98b2836bd83707ecf27d66587fab9dedcfe746efab775042b93cd146d027/torch-2.1.0-cp310-none-macosx_10_9_x86_64.whl",
    sha256 = "101c139152959cb20ab370fc192672c50093747906ee4ceace44d8dd703f29af",
    out = "torch-2.1.0-cp310-none-macosx_10_9_x86_64.whl",
)

prebuilt_python_library(
    name = "torch",
    binary_src = ":torch-download",
    deps = [":filelock", ":typing_extensions", ":sympy", ":networkx", ":jinja2", ":fsspec"],
    visibility = ["PUBLIC"],
)

remote_file(
    name = "filelock-download",
    url = "https://files.pythonhosted.org/packages/81/54/84d42a0bee35edba99dee7b59a8d4970eccdd44b99fe728ed912106fc781/filelock-3.13.1-py3-none-any.whl",
    sha256 = "57dbda9b35157b05fb3e58ee91448612eb674172fab98ee235ccb0b5bee19a1c",
    out = "filelock-3.13.1-py3-none-any.whl",
)

prebuilt_python_library(
    name = "filelock",
    binary_src = ":filelock-download",
    deps = [],
    visibility = ["PUBLIC"],
)

remote_file(
    name = "mpmath-download",
    url = "https://files.pythonhosted.org/packages/43/e3/7d92a15f894aa0c9c4b49b8ee9ac9850d6e63b03c9c32c0367a13ae62209/mpmath-1.3.0-py3-none-any.whl",
    sha256 = "a0b2b9fe80bbcd81a6647ff13108738cfb482d481d826cc0e02f5b35e5c88d2c",
    out = "mpmath-1.3.0-py3-none-any.whl",
)

prebuilt_python_library(
    name = "mpmath",
    binary_src = ":mpmath-download",
    deps = [],
    visibility = ["PUBLIC"],
)

remote_file(
    name = "jinja2-download",
    url = "https://files.pythonhosted.org/packages/bc/c3/f068337a370801f372f2f8f6bad74a5c140f6fda3d9de154052708dd3c65/Jinja2-3.1.2-py3-none-any.whl",
    sha256 = "6088930bfe239f0e6710546ab9c19c9ef35e29792895fed6e6e31a023a182a61",
    out = "Jinja2-3.1.2-py3-none-any.whl",
)

prebuilt_python_library(
    name = "jinja2",
    binary_src = ":jinja2-download",
    deps = [":markupsafe"],
    visibility = ["PUBLIC"],
)
```
