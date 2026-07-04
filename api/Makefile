REGISTRY       ?= registry.bigkraig.com
IMAGE_NAME     := $(REGISTRY)/libretto-api
VERSION        := $(shell cat VERSION)
GIT_HASH       := $(shell git rev-parse --short HEAD 2>/dev/null || echo dev)
CONTAINER_NAME ?= libretto-api
NETWORK        ?= linuxserver
DATA_DIR       ?= /mnt/appdata/libretto

.PHONY: help build push release bump-patch run deploy clean version load load-porsche load-ferrari load-audi

help:
	@echo "libretto-api targets:"
	@echo "  make build         build $(IMAGE_NAME):$(VERSION) and $(IMAGE_NAME):$(GIT_HASH)"
	@echo "  make push          push both tags to registry"
	@echo "  make release       bump-patch + build + push"
	@echo "  make bump-patch    increment patch version in VERSION file"
	@echo "  make run           foreground dev run on :3030, mounts \$$PWD at /data"
	@echo "  make deploy        build, stop+rm $(CONTAINER_NAME), run detached"
	@echo "  make version       print current version"
	@echo "  make load          run all loaders (porsche, ferrari, audi)"
	@echo "  make load-porsche  run vehicle-importer"
	@echo "  make load-ferrari  run load-ferrari"
	@echo "  make load-audi     run load-audi"
	@echo ""
	@echo "Set LIBRETTO_DATABASE_URL to target a specific database."

load: load-porsche load-ferrari load-audi

load-porsche:
	cargo run -- vehicle-importer

FERRARI_PATH  ?= source_files/ferrari/2017-F151M
FERRARI_YEAR  ?= 2017
FERRARI_MODEL ?= F151M

load-ferrari:
	cargo run -- load-ferrari --path $(FERRARI_PATH) --year $(FERRARI_YEAR) --model $(FERRARI_MODEL)

load-audi:
	pip3 install -q -r tools/requirements.txt
	cargo run -- load-audi

version:
	@echo $(VERSION)

bump-patch:
	@old=$(VERSION); \
	major=$$(echo $$old | cut -d. -f1); \
	minor=$$(echo $$old | cut -d. -f2); \
	patch=$$(echo $$old | cut -d. -f3); \
	new="$$major.$$minor.$$((patch + 1))"; \
	echo $$new > VERSION; \
	echo "Bumped $$old → $$new"

build:
	docker buildx build --platform linux/amd64 \
		-t $(IMAGE_NAME):$(VERSION) \
		-t $(IMAGE_NAME):$(GIT_HASH) \
		-t $(IMAGE_NAME):latest \
		--push .

push:

release: bump-patch
	$(MAKE) build push

run:
	docker run --rm -p 3030:3030 -v "$(CURDIR):/data" --name $(CONTAINER_NAME) $(IMAGE_NAME):latest

deploy: build
	-docker stop $(CONTAINER_NAME)
	-docker rm $(CONTAINER_NAME)
	docker run -d --restart=always --network $(NETWORK) --name $(CONTAINER_NAME) -v $(DATA_DIR):/data $(IMAGE_NAME):latest

clean:
	-docker rmi $(IMAGE_NAME):$(VERSION)
	-docker rmi $(IMAGE_NAME):$(GIT_HASH)
	-docker rmi $(IMAGE_NAME):latest
