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

