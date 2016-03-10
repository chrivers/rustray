SHELL=/bin/zsh

build: build-release

build-debug:
	@cargo build

build-release:
	@cargo build --release

clean:
	@rm -fv **/*~(N)
