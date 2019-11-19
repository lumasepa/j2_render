stdin {
  format = "json"
}

var {
  as = "numbers[7]"
  value = "{{ctx.a.b.c}}"
}

var {
  format = "j2"
  value = <<J2
  {% for x in y %}
    . {{y}}
  {% endfor %}
  J2

}

env {}

env {
  key = "SECRET_PASSWORD"
  as = "secrets.password"
}

env {
  key = "ENVIRONMENT_NAME"
  as = "env_name"
}

file {
  file = "./{{env_name}}-ctx.json"
  as = "env_ctx"
}

in "file" {
  file = "./ctx.toml"
}

in "file" {
  if = "{{ctx.use_template}}"

  file = "./template.j2"
}
