set positional-arguments
set dotenv-load

help:
    @just --list --unsorted

run *ARGS:
    cargo run {{ARGS}}

test *ARGS='':
    cargo test "$@"

test-all:
    just sqlmo_openapi/test
    just sqlmo_sqlx/test
    just test

####
bootstrap:
    createdb $(extract path $DATABASE_URL)

sql:
    psql $DATABASE_URL

watch:
    cargo watch

# Bump version. level=major,minor,patch
version level:
    git diff-index --exit-code HEAD > /dev/null || ! echo You have untracked changes. Commit your changes before bumping the version.
    cargo set-version --bump {{level}}
    cargo update # This bumps Cargo.lock
    VERSION=$(rg  "version = \"([0-9.]+)\"" -or '$1' Cargo.toml | head -n1) && \
        git commit -am "Bump version {{level}} to $VERSION" && \
        git tag v$VERSION && \
        git push origin v$VERSION
    git push

publish:
    cargo publish

patch: test
    just version patch
    just publish

publish-all:
    just sqlmo_sqlx/publish
    just sqlmo_openapi/publish
