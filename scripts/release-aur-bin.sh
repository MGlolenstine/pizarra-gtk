#!/bin/bash

# some useful variables
VERSION=${CI_COMMIT_TAG:1}
ARCHIVENAME=pizarra-$VERSION-x86_64.tar.gz

# clone the repo
git clone $BIN_REPO_URL pizarra-bin

# enter it
cd pizarra-bin

# get the sum from the artifacts
SUM=( `cat $ARCHIVENAME.sum` )

# Generate the PKGBUILD
echo "# Maintainer: Abraham Toriz <categulario at gmail dot com>
pkgname=pizarra-bin
pkgver=$VERSION
pkgrel=1
pkgdesc='A free-hand vector drawing application with infinite canvas'
arch=('x86_64')
url='https://gitlab.com/categulario/pizarra-gtk'
license=('GPL3')
depends=('gtk3')
provides=('pizarra')
conflicts=('pizarra')
source=(\"https://pizarra.categulario.tk/releases/any-linux/pizarra-\$pkgver-\$arch.tar.gz\")
sha256sums=('$SUM')

package() {
    cd \"\$srcdir/build\"
    install -Dm755 pizarra \"\$pkgdir\"/usr/bin/pizarra
    install -Dm644 pizarra.desktop \"\$pkgdir\"/usr/share/applications/pizarra.desktop
    install -Dm644 pizarra.svg \"\$pkgdir\"/usr/share/icons/hicolor/scalable/apps/pizarra.svg

    install -Dm644 README.md \"\$pkgdir\"/usr/share/doc/pizarra/README.md
    install -Dm644 LICENSE \"\$pkgdir\"/usr/share/doc/pizarra/LICENSE
    install -Dm644 CHANGELOG.md \"\$pkgdir\"/usr/share/doc/pizarra/changelog
}
" | tee PKGBUILD > /dev/null

makepkg --printsrcinfo > .SRCINFO
git add .
git commit -m "Release version $VERSION"
git push
