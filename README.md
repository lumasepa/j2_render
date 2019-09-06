# j2_render

j2_render is CLI tool to render [jinja 2]() templates, it can load the context from different sources.

j2_render is writen in rust and uses [tera]() as jinja 2 engine,
may be some differences between implementations, so look at tera's documentation for reference

## Installation

```bash 
export J2_RENDER_VERSION=v0.0.1 
sudo curl -L "https://github.com/lumasepa/j2_render/releases/download/${J2_RENDER_VERSION}/j2_render_$(uname | tr '[:upper:]' '[:lower:]')_amd64" -o /usr/local/bin/j2_render
sudo chmod +x /usr/local/bin/j2_render
```

## Usage

#### Usage Examples
