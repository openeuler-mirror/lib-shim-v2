lib:
	cargo build --release

install:
	cp shim_v2.h /usr/local/include/shim_v2.h
	cp target/release/libshim_v2.so /usr/local/lib/

.PHONY: lib install