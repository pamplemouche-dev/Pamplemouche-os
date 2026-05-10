SHELL := /bin/sh

ARTIFACT_DIR ?= artifacts
TAG ?= dev

.PHONY: all validate build package test smoke clean

all: validate

validate:
	@scripts/validate-layout.sh

build: validate
	@ARTIFACT_DIR="$(ARTIFACT_DIR)" scripts/build-freebsd-image.sh "$(TAG)"

package:
	@ARTIFACT_DIR="$(ARTIFACT_DIR)" scripts/package-artifacts.sh "$(TAG)"

test: validate
	@tests/verify-ui-profile.sh

smoke:
	@ARTIFACT_DIR="$(ARTIFACT_DIR)" scripts/smoke-test-artifacts.sh

clean:
	rm -rf "$(ARTIFACT_DIR)/rootfs" "$(ARTIFACT_DIR)/release" \
	       "$(ARTIFACT_DIR)"/*.iso "$(ARTIFACT_DIR)"/*.img "$(ARTIFACT_DIR)"/*.sha256 \
	       "$(ARTIFACT_DIR)"/*-artifacts.tar.gz
