#!/bin/bash

# Needed variables
VERSION=${CI_COMMIT_TAG:1}

# clone the repo
git clone $GIT_REPO_URL pizarra-git

# enter it
cd pizarra-git

echo "# Maintainer: Abraham Toriz <categulario at gmail dot com>
pkgname=pizarra-git
pkgver=$VERSION
pkgrel=1
pkgdesc='A free-hand vector drawing application with infinite canvas (git version)'
arch=('i686' 'x86_64')
url='https://gitlab.com/categulario/pizarra-gtk'
license=('GPL3')
depends=('gtk3')
makedepends=('cargo' 'git')
provides=('pizarra')
conflicts=('pizarra')
source=('pizarra-git::git+https://gitlab.com/categulario/pizarra-gtk#branch=main')
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

    install -Dm644 README.md \"\$pkgdir\"/usr/share/doc/pizarra/README.md
    install -Dm644 LICENSE \"\$pkgdir\"/usr/share/doc/pizarra/LICENSE
    install -Dm644 CHANGELOG.md \"\$pkgdir\"/usr/share/doc/pizarra/changelog
}" | tee PKGBUILD > /dev/null

makepkg --printsrcinfo > .SRCINFO
git add .
git commit -m "Release version $VERSION"
git push
