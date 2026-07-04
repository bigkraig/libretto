NEXT_PUBLIC_API_HOST       ?= http://libretto-api:3030
NEXT_PUBLIC_API_IMAGE_HOST ?= http://libretto-api:3030

.PHONY: help build push release bump-patch version

help:
	@echo "Libretto build targets (runs both api and web):"
	@echo "  make build       build both images"
	@echo "  make push        push both images"
	@echo "  make release     bump-patch + build + push for both"
	@echo "  make bump-patch  bump patch version in both"
	@echo "  make version     print versions"

version:
	@echo "api: $$(cat api/VERSION)"
	@echo "web: $$(cat web/VERSION)"

bump-patch:
	$(MAKE) -C api bump-patch
	$(MAKE) -C web bump-patch

build:
	$(MAKE) -C api build
	$(MAKE) -C web build \
		NEXT_PUBLIC_API_HOST=$(NEXT_PUBLIC_API_HOST) \
		NEXT_PUBLIC_API_IMAGE_HOST=$(NEXT_PUBLIC_API_IMAGE_HOST)

push:
	$(MAKE) -C api push
	$(MAKE) -C web push

release:
	$(MAKE) -C api release
	$(MAKE) -C web release \
		NEXT_PUBLIC_API_HOST=$(NEXT_PUBLIC_API_HOST) \
		NEXT_PUBLIC_API_IMAGE_HOST=$(NEXT_PUBLIC_API_IMAGE_HOST)
