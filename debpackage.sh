COPYRIGHT_YEARS="2018 - "$(date "+%Y")
DPKG_STAGING="debian-package"
DPKG_DIR="${DPKG_STAGING}/dpkg"

PROJECT_MANTAINER="Abraham Toriz Cruz"
PROJECT_HOMEPAGE="https://pizarra.categulario.tk"
PROJECT_NAME=pizarra
PROJECT_VERSION=${VERSION:1}

mkdir -p "${DPKG_DIR}"

DPKG_BASENAME=${PROJECT_NAME}
DPKG_CONFLICTS=
DPKG_VERSION=${PROJECT_VERSION}

DPKG_ARCH=amd64
DPKG_NAME="${DPKG_BASENAME}_${DPKG_VERSION}_${DPKG_ARCH}.deb"

# Binary
install -Dm755 "target/release/pizarra" "${DPKG_DIR}/usr/bin/pizarra"
# README and LICENSE
install -Dm644 "README.md" "${DPKG_DIR}/usr/share/doc/${DPKG_BASENAME}/README.md"
install -Dm644 "LICENSE" "${DPKG_DIR}/usr/share/doc/${DPKG_BASENAME}/LICENSE"
install -Dm644 "CHANGELOG.md" "${DPKG_DIR}/usr/share/doc/${DPKG_BASENAME}/changelog"
gzip -n --best "${DPKG_DIR}/usr/share/doc/${DPKG_BASENAME}/changelog"
install -Dm644 res/pizarra.desktop "${DPKG_DIR}/usr/share/applications/pizarra.desktop"
install -Dm644 res/icons/tk.categulario.pizarra.svg "${DPKG_DIR}/usr/share/icons/hicolor/scalable/apps/pizarra.svg"

cat > "${DPKG_DIR}/usr/share/doc/${DPKG_BASENAME}/copyright" <<EOF
Format: http://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: $PROJECT_NAME
Source: $PROJECT_HOMEPAGE
Files: *
Copyright: $PROJECT_MANTAINER
Copyright: $COPYRIGHT_YEARS $PROJECT_MANTAINER
License: GPL
EOF

chmod 644 "${DPKG_DIR}/usr/share/doc/${DPKG_BASENAME}/copyright"

# control file
mkdir -p "${DPKG_DIR}/DEBIAN"
cat > "${DPKG_DIR}/DEBIAN/control" <<EOF
Package: ${DPKG_BASENAME}
Version: ${DPKG_VERSION}
Section: graphics
Priority: optional
Maintainer: ${PROJECT_MANTAINER}
Homepage: ${PROJECT_HOMEPAGE}
Architecture: ${DPKG_ARCH}
Provides: ${PROJECT_NAME}
Depends: libgtk-3-0
Conflicts: ${DPKG_CONFLICTS}
Description: A simple infinite-canvas free-hand vector drawing application
  A simple infinite-canvas free-hand vector drawing application
EOF
DPKG_PATH="${DPKG_STAGING}/${DPKG_NAME}"
# build dpkg
fakeroot dpkg-deb --build "${DPKG_DIR}" "${DPKG_PATH}"
