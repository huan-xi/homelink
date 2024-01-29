apt-get install protobuf-compiler libavahi-client-dev

cross compile

apt-get install musl-tools 

sudo apt-get install pkg-config libglib2.0-dev libssl-dev llvm

export GN_ARGS='use_custom_libcxx=false v8_enable_backtrace=false v8_enable_debugging_features=false use_lld=false symbol_level=0 v8_builtins_profiling_log_file=""'
export CLANG_BASE_PATH=/usr
export V8_FROM_SOURCE=1
export CROSS_COMPILE=x86_64-linux-musl-
cargo build --release --target x86_64-unknown-linux-musl --bin bin

~/.cargo/config

[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"

//export TARGET=x86_64-alpine-linux-musl


hap
https://github.com/ewilken/hap-rs/issues/77

https://github.com/al-one/hass-xiaomi-miot?tab=readme-ov-file
https://github.com/AlexxIT/XiaomiGateway3
https://github.com/AlexxIT/openmiio_agent