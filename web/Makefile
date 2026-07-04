REGISTRY              ?= registry.bigkraig.com
IMAGE_NAME            := $(REGISTRY)/libretto-web
VERSION               := $(shell cat VERSION)
GIT_HASH              := $(shell git rev-parse --short HEAD 2>/dev/null || echo dev)
CONTAINER_NAME        ?= libretto
NETWORK               ?= linuxserver
NEXT_PUBLIC_API_HOST       ?= http://libretto-api:3030
NEXT_PUBLIC_API_IMAGE_HOST ?= http://libretto-api:3030
NEXT_PUBLIC_AUTH_BYPASS    ?= false

.PHONY: help build push release bump-patch run deploy clean version

help:
	@echo "libretto-web targets:"
	@echo "  make build                  build $(IMAGE_NAME):$(VERSION) and $(IMAGE_NAME):$(GIT_HASH)"
	@echo "  make push                   push both tags to registry"
	@echo "  make release                bump-patch + build + push"
	@echo "  make bump-patch             increment patch version in VERSION file"
	@echo "  make run                    foreground dev run on :3000"
	@echo "  make version                print current version"
	@echo ""
	@echo "Override NEXT_PUBLIC_API_HOST and NEXT_PUBLIC_API_IMAGE_HOST for build-time baking."

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
		--build-arg NEXT_PUBLIC_API_HOST=$(NEXT_PUBLIC_API_HOST) \
		--build-arg NEXT_PUBLIC_API_IMAGE_HOST=$(NEXT_PUBLIC_API_IMAGE_HOST) \
		--build-arg NEXT_PUBLIC_AUTH_BYPASS=$(NEXT_PUBLIC_AUTH_BYPASS) \
		-t $(IMAGE_NAME):$(VERSION) \
		-t $(IMAGE_NAME):$(GIT_HASH) \
		-t $(IMAGE_NAME):latest \
		--push .

push:

release: bump-patch
	$(MAKE) build push \
		NEXT_PUBLIC_API_HOST=$(NEXT_PUBLIC_API_HOST) \
		NEXT_PUBLIC_API_IMAGE_HOST=$(NEXT_PUBLIC_API_IMAGE_HOST) \
		NEXT_PUBLIC_AUTH_BYPASS=$(NEXT_PUBLIC_AUTH_BYPASS)

run:
	docker run --rm -p 3000:3000 --name $(CONTAINER_NAME) $(IMAGE_NAME):latest

deploy: build
	-docker stop $(CONTAINER_NAME)
	-docker rm $(CONTAINER_NAME)
	docker run -d --restart=always --network $(NETWORK) --name $(CONTAINER_NAME) $(IMAGE_NAME):latest

clean:
	-docker rmi $(IMAGE_NAME):$(VERSION)
	-docker rmi $(IMAGE_NAME):$(GIT_HASH)
	-docker rmi $(IMAGE_NAME):latest
