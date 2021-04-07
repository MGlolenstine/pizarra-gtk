# Pizarra, edición GTK

Este es el frontend GTK de la [pizarra](https://gitlab.com/categulario/pizarra).

## ¿Dónde funciona?

Debería funcionar en cualquier plataforma en la que funcione Rust y Gtk. Por el
momento funciona confiablemente en Linux, le faltan íconos en Mac y no compila
en windows.

## ¿Y cómo se ve esto?

En este momento se ve así:

![Vista de la quinta pre-versión de la pizarra](https://categulario.tk/pizarra_demo_1.0.0.png)

## Para compilar

Es necesario tener [Rust instalado](https://rustup.rs), clonar el proyecto y
ejecutar en una terminal dentro del directorio del proyecto:

`cargo run`

Esto funciona de maravilla en linux (suponiendo que las cabeceras de desarrollo
de gtk están instaladas) y falla monumentalmente en windows y en mac. Sin
embargo se pueden intentar los pasos [de este otro proyecto](https://github.com/zoeyfyi/Boop-GTK)
para construir la pizarra en esos sistemas operativos.

### En mac

Podría ser que estos dos comandos funcionen:

    brew install gtk+3
    cargo run

### En windows

    git clone https://github.com/wingtk/gvsbuild.git C:\gtk-build\github\gvsbuild
    cd C:\gtk-build\github\gvsbuild; python .\build.py build -p=x64 --vs-ver=16 --msys-dir=C:\msys64 -k --enable-gi --py-wheel --py-egg gtk3 gdk-pixbuf gtksourceview3
    ${Env:GTKSOURCEVIEW_3.0_NO_PKG_CONFIG}=1; ${Env:SYSTEM_DEPS_GTKSOURCEVIEW_3.0_LIB}="gtksourceview-3.0"; cargo build

#### Instalador

Hacer primero lo de arriba, luego:

    cargo install cargo-wix
    ${Env:GTKSOURCEVIEW_3.0_NO_PKG_CONFIG}=1; ${Env:SYSTEM_DEPS_GTKSOURCEVIEW_3.0_LIB}="gtksourceview-3.0"; cargo wix -v

## Checklist antes de sacar un release

* se puede dibugar con todas las herramientas
* se puede hacer zoom para adentro y afuera, y regresar al inicio
* se puede cambiar el alpha y el grosor
* se pueden borrar
* ctrl+z, ctrl+shift+z
* se puede guardar
* se puede abrir el archivo guardado
* se puede exportar
