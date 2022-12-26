set dotenv-load := true

help:
    @just --list --unsorted

run *ARGS:
    cargo run {{ARGS}}

test:
    cargo test

####
bootstrap:
    createdb $(extract path $DATABASE_URL)

sql:
    psql $DATABASE_URL

watch:
    cargo watch