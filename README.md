# La pizarra

Esta es una aplicación de dibujo vectorial a mano alzada con un lienzo y zoom
infinitos¹, relativamente simple y de interfaz limpia, orientada a quienes como
yo disfrutan de hacer rayones en hojas de papel, servilletas y ahora en la
computadora. La creé y utilizo para hacer explicaciones en línea, o a veces
simplemente para bosquejar cosas para mi mismo.

Para sacarle provecho sinceramente se necesita una tableta digitalizadora o
pantalla táctil.

Este repositorio es el frontend GTK de la pizarra. Lo realmente interesante está
en [este otro repositorio](https://gitlab.com/categulario/pizarra).

## ¿Qué puedo hacer en ella?

Principalmmente dibujar trazos a mano alzada usando la herramienta lápiz
(seleccionada por defecto), pero también hay algunas figuras como rectángulos,
círculos, elipses y polígonos.

Puedes guardar tus dibujos como SVG o exportarlos como PNG.

## ¿Dónde funciona?

Debería funcionar en cualquier plataforma en la que funcione Rust y Gtk. Por el
momento funciona confiablemente en Linux, [hay un instalador para
windows](https://pizarra.categulario.tk) y se de alguien que la logró compilar
en mac.

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

## Empaquetado

(esto es para mi, no para ti)

* Actualizar el changelog.
* Crear un nuevo tag de la versión.
* `git push && git push --tags`

___________________

1. El zoom y el lienzo sí son finitos, están limitados por la precisión de
   números de punto flotante de 64 bits, pero digamos para el uso cotidiano es
   un espacio de dibujo bastante grande (:
