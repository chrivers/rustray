SHELL=/bin/zsh

build:
	@cargo build

clean:
	@rm -fv **/*~(N)
