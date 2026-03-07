.PHONY: 
	help 
	reinstall-proto

## help: print this help message
help:
	@echo 'Usage:'
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':' | sed -e 's/^/ /'
	
reinstall-proto:
	pip install --force-reinstall --no-deps -r requirements/git.txt
