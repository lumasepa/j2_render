in "stdin" {
  format = "json"
}

in "var" {
  as = "numbers[7]"
  value = "{{ctx.a.b.c}}"
}

in "var" {
  format = "j2"
  value = <<J2
  {% for x in y %}
    . {{y}}
  {% endfor %}
  J2

}

in "env" {}

in "env" {
  key = "SECRET_PASSWORD"
  as = "secrets.password"
}

in "env" {
  key = "ENVIRONMENT_NAME"
  as = "env_name"
}

in "file" {
  file = "./{{env_name}}-ctx.json"
  as = "env_ctx"
}

in "file" {
  file = "./ctx.toml"
}

in "http" {
  if = "{{obj.bolean_field}}"
  format = "json"
  method = "GET"
  url = "{{env_ctx.url}}"
  headers {
    X-API-KEY = "{{env_ctx.api_key}}"
  }
  basic  = "{{my.user}}:{{secret.password}}"
  token = "{{secrets.token}}"
  digest = "{{my.user}}:{{secret.password}}"
}

in "file" {
  if = "{{ctx.use_template}}"

  file = "./template.j2"
}

out "file" {
  file = "./rendered"
}

out "std" {}

out "http" {
    for_each = "{{jmespath_expr}}"
    if = "{{tera expr}}"
    method = "POST"
    url = "{{env_ctx.url_conf}}/{{for.element.id}}"
    headers {
      X-API-KEY = "{{env_ctx.api_key}}"
      Host = "{{rel_host}}"
    }
    basic = "{{for.element.user}}:{{for.element.password}}"
}