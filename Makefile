ifeq ($(shell uname),Darwin)
    EXT := dylib
else
    EXT := so
endif

SOURCEDIR := src
RS_SOURCES := $(shell find $(SOURCEDIR) -name '*.rs')

all: target/debug/libtwirl.$(EXT)
	g++ src/c_test.cpp -g -L ./target/debug/ -ltwirl -o run
	LD_LIBRARY_PATH=./target/debug/ ./run

target/debug/libtwirl.$(EXT): $(RS_SOURCES) Cargo.toml
	cargo build

clean:
	rm -rf target
	rm -rf run