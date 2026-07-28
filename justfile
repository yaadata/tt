setup:
    mise install
    mise run hooks:install

test:
    python3 scripts/test_all.py

test-dir directory:
    python3 scripts/test_dir.py {{quote(directory)}}

test-file file:
    python3 scripts/test_file.py {{quote(file)}}
