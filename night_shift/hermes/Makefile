.PHONY: up down logs smoke
up:
	docker compose up -d --build

down:
	docker compose down

logs:
	docker compose logs -f --tail=200

smoke:
	bash infra/scripts/smoke-test.sh
