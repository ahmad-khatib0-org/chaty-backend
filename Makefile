include .env

.PHONY: help confirm \
	db/migrations/sql/new db/migrations/sql/up db/migrations/sql/down db/migrations/sql/force \
	db/migrations/nosql/new db/migrations/nosql/up db/migrations/nosql/down db/migrations/nosql/force

# ==================================================================================== #
# HELPERS
# ==================================================================================== #

## help: print this help message
help:
	@echo 'Usage:'
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':' | sed -e 's/^/ /'

confirm:
	@echo -n 'Are you sure? [y/N] ' && read ans && [ $${ans:-N} = y ]

# ==================================================================================== #
# SQL MIGRATIONS (PostgreSQL)
# ==================================================================================== #

## db/migrations/sql/new name=$1: create a new SQL migration
db/migrations/sql/new:
	@echo "Creating SQL migration files for ${name}..."
	@migrate create -seq -ext=.sql -dir=./migrations/sql ${name}

## db/migrations/sql/up: apply all up SQL migrations
db/migrations/sql/up: confirm
	@echo "Running UP SQL migrations..."
	@migrate -path ./migrations/sql -database ${DB_DSN_SQL} -verbose up

## db/migrations/sql/down: apply all down SQL migrations
db/migrations/sql/down: confirm
	@echo "Running DOWN SQL migrations..."
	@migrate -path ./migrations/sql -database ${DB_DSN_SQL} -verbose down

## db/migrations/sql/force force=$1: force fixing the SQL migration version
db/migrations/sql/force: confirm
	@echo "Force fixing SQL migration to version ${force}"
	@migrate -path ./migrations/sql -database ${DB_DSN_SQL} force ${force}


# ==================================================================================== #
# NOSQL MIGRATIONS (ScyllaDB)
# ==================================================================================== #

## db/migrations/nosql/new name=$1: create a new NoSQL (CQL) migration
db/migrations/nosql/new:
	@echo "Creating NoSQL migration files for ${name}..."
	@migrate create -seq -ext=.cql -dir=./migrations/nosql ${name}

## db/migrations/nosql/up: apply all up NoSQL migrations
db/migrations/nosql/up: confirm
	@echo "Running UP NoSQL migrations..."
	@migrate -path ./migrations/nosql -database ${DB_DSN_NOSQL} -verbose up

## db/migrations/nosql/down: apply all down NoSQL migrations
db/migrations/nosql/down: confirm
	@echo "Running DOWN NoSQL migrations..."
	@migrate -path ./migrations/nosql -database ${DB_DSN_NOSQL} -verbose down

## db/migrations/nosql/force force=$1: force fixing the NoSQL migration version
db/migrations/nosql/force: confirm
	@echo "Force fixing NoSQL migration to version ${force}"
	@migrate -path ./migrations/nosql -database ${DB_DSN_NOSQL} force ${force}

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
	$(MAKE) db/migrations/sql/up

