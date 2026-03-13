doc *args:
    #!/usr/bin/env bash
    set -- {{args}}
    if [ $# -eq 0 ]; then
        python3 scripts/doc_runner.py
    elif [ $# -eq 1 ]; then
        python3 scripts/doc_runner.py -c "$1"
    else
        python3 scripts/doc_runner.py -c "$1" -f "$2"
    fi



release version:
    cargo xtask release {{version}}

test *args:
    cargo xtask test {{args}}
