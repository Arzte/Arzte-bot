# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=$(mktemp -d)

    test -f Cargo.lock || cargo generate-lockfile

    cp target/release/arzte $stage/arzte-bot
    blake2 $stage/arzte-bot > $stage/arzte-bot.blake2

    cd $stage
    tar czf $src/arzte.tar.gz *
    cd $src

    rm -rf $stage
}

main