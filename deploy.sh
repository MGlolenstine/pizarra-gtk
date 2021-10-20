#!/usr/bin/env /bin/bash

if [[ -f .env ]]; then
    source .env
fi

# prepare the build directory, ensure it is empty
rm -rf build
mkdir -p build

# Build the app in release mode
cargo build --release

# Move everything to the build directory
cp target/release/pizarra build/pizarra
cp res/icons/tk.categulario.pizarra.svg build/pizarra.svg
cp res/pizarra.desktop build/pizarra.desktop
cp CHANGELOG.md build/CHANGELOG.md

# Create the .tar.gz archive for linux
version=$(git describe --long | sed 's/\([^-]*-\)g/r\1/;s/-/./g;s/v//g')
arch=x86_64
archivename=pizarra-$version-$arch.tar.gz

tar -cvzf $archivename build/

# Move the archive to the server
scp $archivename $DEPLOY_USER@$DEPLOY_HOST:$RELEASES_PATH/$archivename
sum=$(sha256sum $archivename)
sum=${sum:0:64}
rm $archivename

# Update the pizarra-bin AUR package
release-pizarra-bin() {
    echo $sum
    echo $version

    echo "# Maintainer: Abraham Toriz <categulario at gmail dot com>
pkgname=pizarra-bin
pkgver=$version
pkgrel=1
pkgdesc='Simple Gtk drawing application'
arch=('x86_64')
url='https://gitlab.com/categulario/pizarra-gtk'
license=('GPL3')
depends=('gtk3')
provides=('pizarra')
conflicts=('pizarra')
source=(\"https://pizarra.categulario.tk/releases/pizarra-\$pkgver-\$arch.tar.gz\")
sha256sums=('$sum')

package() {
    cd \"\$srcdir/build\"
    install -Dm755 pizarra \"\$pkgdir\"/usr/bin/pizarra
    install -Dm644 pizarra.desktop \"\$pkgdir\"/usr/share/applications/pizarra.desktop
    install -Dm644 pizarra.svg \"\$pkgdir\"/usr/share/icons/hicolor/scalable/apps/pizarra.svg
}
" | tee ../arch-bin/PKGBUILD > /dev/null

    cur=$(pwd)

    cd ../arch-bin/
    makepkg --printsrcinfo > .SRCINFO
    git add .
    git commit -m "Release version $version"
    git push aur master
    cd $cur
}

release-pizarra-git() {
    echo "# Maintainer: Abraham Toriz <categulario at gmail dot com>
pkgname=pizarra-git
pkgver=$version
pkgrel=1
pkgdesc='Simple Gtk drawing application'
arch=('i686' 'x86_64')
url='https://gitlab.com/categulario/pizarra-gtk'
license=('GPL3')
depends=('gtk3')
makedepends=('cargo' 'git')
provides=('pizarra')
conflicts=('pizarra')
source=('pizarra-git::git+https://gitlab.com/categulario/pizarra-gtk')
sha256sums=('SKIP')

pkgver() {
	cd \"\$pkgname\"
	printf \"%s\" \"\$(git describe --long | sed 's/\([^-]*-\)g/r\1/;s/-/./g;s/v//g')\"
}

build() {
	cd \"\$pkgname\"
	cargo build --release --locked
}

package() {
	cd \"\$pkgname\"
	install -Dm755 target/release/pizarra \"\$pkgdir\"/usr/bin/pizarra
	install -Dm644 res/pizarra.desktop \"\$pkgdir\"/usr/share/applications/pizarra.desktop
	install -Dm644 res/icons/tk.categulario.pizarra.svg \"\$pkgdir\"/usr/share/icons/hicolor/scalable/apps/pizarra.svg
}" | tee ../arch-git/PKGBUILD > /dev/null

    cd ../arch-git/
    makepkg --printsrcinfo > .SRCINFO
    git add .
    git commit -m "Release version $version"
    git push aur master
    cd $cur
}

release-pizarra-bin
release-pizarra-git

# Notify in the channel
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
