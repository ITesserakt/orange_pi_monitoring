set shell := ["nu", "-c"]

sync_folder := 'dev_sync'
exe := sync_folder / 'orangepi_service'
orangepi_host := 'orangepi'
arch_host := 'archlinux'
debug := 'true'
release := if debug == 'true' { '' } else { '--release' }

target := 'wasm32-unknown-unknown'

default:
    just --choose

build-arm:
    cargo build --target armv7-unknown-linux-gnueabihf -p monitoring_service --release

build-linux:
    cargo build --target x86_64-unknown-linux-gnu -p monitoring_service --release

build-view:
    cargo build --target {{ target }} {{ release }} -p viewer

build: build-arm build-view

test:
    cargo test --target x86_64-pc-windows-msvc
    cargo test --target wasm32-unknown-unknown -p viewer

sync: build-arm
    -ssh server@{{ orangepi_host }} "kill -2 `(cat .service.lock | xargs)`"
    scp target/armv7-unknown-linux-gnueabihf/release/monitoring_service server@{{ orangepi_host }}:{{ exe }}
    ssh server@{{ orangepi_host }} 'chmod u+x {{ exe }}'

local-server port='50525' update_interval='100':
    cargo run --target x86_64-pc-windows-msvc {{ release }} -p monitoring_service -- -a 0.0.0.0 -p {{ port }} -u {{ update_interval }}

remote-server port='50525' update_interval='1000': sync
    ssh server@{{ orangepi_host }} 'cd /home/server/ && ./{{ exe }} -a 0.0.0.0 -p {{ port }} -u {{ update_interval }}'

view address='127.0.0.1' port='8080': build-view
    cd viewer; trunk serve --address {{ address }} --port {{ port }} {{ release }} --no-autoreload