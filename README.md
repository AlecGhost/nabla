# Nabla

**Warning: Work in Progress**

Nabla is a statically typed configuration language.
It aimes to be developer-friendly and human-readable,
while compiling to machine-readable formats like JSON, XML and YAML.

## Example

Let's define our configuration options for an example program.
Our program takes a mandatory input file.
Furthermore, an output folder and the used version can be supplied.
But the latter two are optional because default values are provided.
Also note that only version 1 and 2 are possible values.
Specifying version 3 (or any other number) would lead to an error.

```nabla
def Config = {
    input_file: String
    output_folder: String = "out/"
    version: 1 | 2 = 2
}
```

The config could be instantiated as follows.
The mandatory input file is supplied
and the default output folder is overwritten.

```nabla
Config {
    input_file = "input.txt"
    output_folder = "build/"
}
```

Nabla can then compile the result to the following JSON.
Since no version was specified, it will fall back on the default value for this key.

```nabla
{
    "input_file": "input.txt",
    "output_folder": "build/",
    "version": 2
}
```

## Supported Targets

- [x] JSON
- [x] YAML
- [x] XML
- [x] TOML
