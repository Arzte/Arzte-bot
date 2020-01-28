# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    cp target/$TARGET/release/arzte $stage/

    cd $stage
    tar czf $src/arzte-$TRAVIS_TAG.tar.gz *
    cd $src

    rm -rf $stage
}

main