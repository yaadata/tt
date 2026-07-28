setup:
    mise install
    mise run hooks:install

test:
    RUSTFLAGS=-Awarnings mise exec -- cargo run --quiet --package repo-tools -- test

test-dir directory:
    RUSTFLAGS=-Awarnings mise exec -- cargo run --quiet --package repo-tools -- test-dir {{quote(directory)}}

test-file file:
    RUSTFLAGS=-Awarnings mise exec -- cargo run --quiet --package repo-tools -- test-file {{quote(file)}}
