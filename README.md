# j2_render

j2_render is a static binary CLI tool to render [jinja 2]() templates, it can load the context from different sources.

j2_render is writen in rust and uses [tera]() as jinja 2 engine,
may be some differences between implementations, so look at tera's documentation for reference

## Installation

```bash 
export J2_RENDER_VERSION=v0.0.1 
sudo curl -L "https://github.com/lumasepa/j2_render/releases/download/${J2_RENDER_VERSION}/j2_render_$(uname | tr '[:upper:]' '[:lower:]')_amd64" -o /usr/local/bin/j2_render
sudo chmod +x /usr/local/bin/j2_render
```

## Working with the Context

The context in j2_render is a json.
The context can be loaded from different formats and ways.
The context is populated in args order so last arg overwrites keys of previous ones

### Supported Context formats

* json
* yaml
* toml
* hcl/tf/tfvars
* key=value

#### Future Supported
* http/s + json/yaml/toml/hcl/tf/tfvars `format+https?://...`

### Supported Context Inputs

* context file `--file` `-f` `file_path` or `format+file_path`
* environment variables `--env` `-e`
* arguments `--var` `-v` `key=value` or `format+key=value`
* stdin `--stdin` `-i` json/yaml/toml/hcl/tf/tfvars

## Working with templates
Templates are jinja 2 templates implemented by tera, for reference of the jinja 2 template lenguaje go to [tera doc]()

### Supported Templates Inputs

* Template file `--file` `-f` `file_path` 
* stdin `--stdin` `-i` `template`

## Usage

```
j2_render [FLAGS]

    OPTIONS:

        FORMATS = json,yaml,hcl,tfvars,tf,template,j2,tpl
        FILE_PATH = file_path.FORMAT or FORMAT+file_path   -- path to a file ctx, template or output
        VAR = key=value or FORMAT+key=value   -- if format provided value will be parsed as format

    FLAGS:

    --stdin/-i FORMAT   -- read from stdin context or template
    --out/-o file_path   -- output file for rendered template, default stdout
    --env/-e    -- load env vars in ctx
    --file/-f FILE_PATH   -- loads a file as context or template depending on extension or format
    --var/-v VAR   -- adds a pair key value to the context or a template depending on format
    --print-ctx/-p   -- print the context as json and exits
    --help/-h   -- shows this help
```

#### Usage Examples

##### Render a template using a context file

```bash
j2_render -f ctx.yaml -f template.j2 > result
```

##### Render a template using environment variables

```bash
j2_render --env -f template.j2 > result
```

##### Render a template using context from stdin

```bash
cat ctx.yaml | j2_render -i yaml -f template.j2 > result
```

#### Render a template using context from var

```bash
echo "Hello {{name}}" | j2_render --var "name=world" -i j2 > result
```

##### Render a template from stdin 

```bash
cat template.j2 | j2_render -i j2 -f ctx.yaml > result
```

#### Render a template from vars

```bash
TEMPLATE=$(cat template.j2)
j2_render --var "j2+=${TEMPLATE}" -f ctx.yaml > result
```

#### Render a template using context mixed by different inputs

```bash
cat ctx.json | j2_render --var "name=batman" -i json --env -f ctx.yaml --var "json+list=[1,2,3]" -f template.j2 > result
```

##### Abuse to convert to json

```bash
j2_render -f file.yaml --print-ctx > file.json
```

```bash
j2_render -f file.tfvars --print-ctx > file.json
```

```bash
j2_render -f file.toml --print-ctx > file.json
```

### Extensions to jinja 2

### filters

* "bash"
* "sed"
* "glob"
* "read_file"
* "file_name"
* "file_dir"
* "strip_line_breaks"
* "remove_extension"
* "b64decode"
* "b64encode"
* "str"
* "to_json"
* "from_json"

### functions

* "tab_all_lines"
* "tab_all_lines_except_first"
* "bash"
* "str"
* "to_json"
* "from_json"

### testers

* "file"
* "directory"



