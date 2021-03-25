#!/usr/bin/env /bin/bash

if [[ -f .env ]]; then
    source .env
fi

rm -rf build
mkdir -p build

cargo build --release
cp target/release/pizarra build/pizarra
cp res/icons/scalable/apps/tk.categulario.pizarra.svg build/pizarra.svg
cp res/pizarra.desktop build/pizarra.desktop
cp CHANGELOG.md build/CHANGELOG.md

version=$(git describe --long | sed 's/\([^-]*-\)g/r\1/;s/-/./g;s/v//g')
arch=x86_64
archivename=pizarra-$version-$arch.tar.gz

tar -cvzf $archivename build/
scp $archivename $DEPLOY_USER@$DEPLOY_HOST:$RELEASES_PATH/$archivename
rm $archivename

change=`git tag --format "%(refname:strip=2) %(contents:subject)" --sort version:refname | tail -n1`

cat << EOF | curl \
    -X POST \
    -H "Content-Type: application/json" \
    "https://api.telegram.org/bot$BOT_KEY/sendMessage" \
    -d @-
{
    "chat_id": $CHANNEL_ID,
    "parse_mode": "Markdown",
    "text": "✒️ Pizarra **$version** está disponible para descargar:\n\nhttps://pizarra.categulario.tk/releases/\n\nÚltimo cambio: __${change}__"
}
EOF
