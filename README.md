# La pizarra

Esta es una aplicación de dibujo vectorial a mano alzada con un lienzo y zoom
infinitos¹, relativamente simple y de interfaz limpia, orientada a quienes como
yo disfrutan de hacer rayones en hojas de papel, servilletas y ahora en la
computadora. La creé y utilizo para hacer explicaciones en línea, o a veces
simplemente para bosquejar cosas para mi mismo.

Para sacarle el máximo provecho se necesita una tableta digitalizadora o
pantalla táctil, pero trabajo en que sea completamente usable con lo mínimo, que
podría ser un mouse o trackpad

Este repositorio es el frontend GTK de la pizarra. Lo realmente interesante está
en [este otro repositorio](https://gitlab.com/categulario/pizarra).

Para conocer más puedes visitar [el sitio web](https://pizarra.categulario.tk)

## ¿Y cómo se ve esto?

En este momento se ve así:

![Vista de la quinta pre-versión de la pizarra](https://categulario.tk/pizarra_demo_1.3.1.png)

## Para compilar

Es necesario tener [Rust instalado](https://rustup.rs), y las cabeceras de
desarrollo de Gtk3 (paquete `libgtk-3-dev` en ubuntu, probablemente `gtk` en
archlinux) clonar el proyecto y ejecutar en una terminal dentro del directorio
del proyecto:

`cargo run --release`

Esto mostraría una ventana completamente funcional de la aplicación. El binario
está en `target/release/pizarra`

### En windows

[Escribí una entrada en mi blog sobre cómo hacer esto](https://blog.categulario.tk/como-compilar-la-pizarra-en-windows.html)

### En mac

Podría ser que estos dos comandos funcionen:

    brew install gtk+3 adwaita-icon-theme
    cargo run

## Liberación de una versión

(esto es para mi, no para ti)

* Actualizar el changelog.
* Crear un nuevo tag de la versión.
* `git push && git push --tags`

___________________

1. El zoom y el lienzo sí son finitos, están limitados por la precisión de
   números de punto flotante de 64 bits, pero digamos para el uso cotidiano es
   un espacio de dibujo bastante grande (:
