variable "version" {
}

variable "local" {
  default = ""
}

variable "tag-prefix" {
  default = ""
}

variable "registry" {
  default = ""
}

variable "suffix" {
  default = ""
}

target "default" {
  args = {
    alpine_version = "3.20.3"
  }
  tags = [
    notequal(local, "") ? "web-index": "",
    "${registry}/web-index${suffix}:${tag-prefix}latest",
    "${registry}/web-index${suffix}:${tag-prefix}${version}"
  ]
}

target "local" {
  args = {
    alpine_version = "3.20.3"
  }
  tags = [
    "web-index"
  ]
}
