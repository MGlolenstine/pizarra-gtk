#!/usr/bin/env /bin/bash

if [[ -f .env ]]; then
    source .env
fi

rm -rf build
mkdir -p build

cargo build --release
cp target/release/pizarra build/pizarra
cp res/pizarra.svg build/pizarra.svg
cp res/pizarra.desktop build/pizarra.desktop
cp CHANGELOG.md build/CHANGELOG.md

version=$(git describe --long | sed 's/\([^-]*-\)g/r\1/;s/-/./g;s/v//g')
arch=x86_64
archivename=pizarra-$version-$arch.tar.gz

tar -cvzf $archivename build/
scp $archivename $DEPLOY_USER@$DEPLOY_HOST:$RELEASES_PATH/$archivename
rm $archivename
