include .env

.PHONY: 
	help 
	confirm 
	db/migrations/new 
	db/migrations/force 
	db/migrations/up 
	db/migrations/up_1 
	db/migrations/down 
	db/migrations/down_1
	run

#
# ==================================================================================== #
#   HELPERS
# ==================================================================================== #

## help: print this help message
help:
	@echo 'Usage:'
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':' | sed -e 's/^/ /'

confirm:
	@echo -n 'Are you sure? [y/N] ' && read ans && [ $${ans:-N} = y ]

## db/migrations/new name=$1: create a new database migration
db/migrations/new: 
	@echo "creating migration files for ${name}..."
	# migrate create -ext=.sql -format="2006-01-02_15-04-05" -seq -dir=./migrations ${name}
	@migrate create -seq -ext=.sql -dir=./migrations ${name}

## db/migrations/:force force fixing the migration version
db/migrations/force: confirm
	@echo "Force fixing the failed migration number: ${force}"
	@migrate -path ./migrations -database ${DB_DSN} force ${force}

## db/migrations/up: apply all up database migrations
db/migrations/up: confirm
	@echo "Running up migrations..."
	@migrate -path ./migrations -database ${DB_DSN} -verbose up

## db/migrations/up_1: apply all up before the last migration
db/migrations/up_1: confirm
	@echo "Running up before last migrations..."
	@migrate -path ./migrations -database ${DB_DSN} -verbose up 1

## db/migrations/down: apply all down database migrations
db/migrations/down: confirm
	@echo "Running down migrations..."
	@migrate -path ./migrations -database ${DB_DSN} -verbose down

## db/migrations/down_1: apply all down before the last migrations
db/migrations/down_1: confirm
	@echo "Running down before last migrations..."
	@migrate -path ./migrations -database ${DB_DSN} -verbose down 1

YAML_FILE := chaty.local.yaml
ENV_FILE := $$HOME/Downloads/personal/portfolio/chaty/chaty-web/packages/chaty-app/.env.development

run:
	docker compose down -v
	docker compose up -d redpanda
	sleep 10
	docker compose up -d
	
	@echo "Waiting for Hydra to be ready..."; \
	for i in $$(seq 1 30); do \
		if curl -f http://localhost:4445/admin/clients >/dev/null 2>&1; then \
			echo "Hydra admin API is ready!"; \
			break; \
		fi; \
		echo "Waiting for Hydra admin API... (attempt $$i/30)"; \
		sleep 2; \
		if [ $$i -eq 30 ]; then \
			echo "Hydra failed to start after 60 seconds"; \
			exit 1; \
		fi; \
	done
	
	@echo "Creating client and capturing client_id..."; \
	{ \
		client_info=$$(docker compose exec hydra hydra create client \
			--endpoint http://localhost:4445 \
			--format json \
			--name "authz-client" \
			--secret "super-secret" \
			--response-type code \
			--grant-type authorization_code \
			--grant-type refresh_token \
			--scope openid \
			--scope offline \
			--redirect-uri http://localhost:3000/api/auth/callback \
			--token-endpoint-auth-method client_secret_basic); \
		echo "$$client_info" > temp_client_info.json; \
		client_id=$$(jq -r '.client_id' temp_client_info.json); \
		rm temp_client_info.json; \
		echo "Generated client_id: $$client_id"; \
		echo "Replacing OAuth Client ID in environment files..."; \
		sed -i -E "s#(NEXT_PUBLIC_OAUTH_CLIENT_ID=).*#\1$$client_id#" $(ENV_FILE); \
		echo "Replacing OAuth Client ID in YAML config with yq..."; \
		yq eval -i ".oauth.client_id = \"$$client_id\"" $(YAML_FILE); \
		echo "Replacement complete."; \
	}
	
	./scripts/init_database.sh
	$(MAKE) db/migrations/up

