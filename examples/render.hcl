in "stdin" {
  format = "json"
}

in "value" {
  as = "numbers[7]"
  value = 7
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

  pick {
    path = "obj.field[2].user"
    as = "user.username"
  }

  pick {
    path = "obj.field[2].roles"
    as = "user.roles"
  }
}

in "http" {
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
  file = "./template.j2"
}

out "file" {
  file = "./rendered"
}

out "std" {}

out "http" {
  method = "POST"
  url = "{{env_ctx.url_conf}}/{{env_ctx.id}}"
  headers {
    X-API-KEY = "{{env_ctx.api_key}}"
  }
  basic = "{{my.user}}:{{secret.password}}"
}