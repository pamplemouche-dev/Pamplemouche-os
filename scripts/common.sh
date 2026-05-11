#!/bin/sh

ARTIFACT_PREFIX="${ARTIFACT_PREFIX:-pamplemouche-os}"

artifact_tag() {
  printf '%s' "${1:-dev}" | tr -c 'A-Za-z0-9._-' '-'
}
