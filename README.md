# La pizarra

Esta es una aplicación de dibujo a mano alzada, relativamente simple y de
interfaz limpia, orientada a quienes como yo disfrutan de hacer rayones en hojas
de papel, servilletas y ahora en la computadora.

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

Es necesario tener [Rust instalado](https://rustup.rs), clonar el proyecto y
ejecutar en una terminal dentro del directorio del proyecto:

`cargo run`

Esto funciona de maravilla en linux (suponiendo que las cabeceras de desarrollo
de gtk están instaladas).

### En windows

[Escribí una entrada en mi blog entera sobre cómo hacer esto](https://blog.categulario.tk/como-compilar-la-pizarra-en-windows.html)

### En mac

Podría ser que estos dos comandos funcionen:

    brew install gtk+3
    cargo run

## Empaquetado

Primero hay que crear un nuevo tag de la nueva versión.

Luego utilizo el script `deploy.sh` para crear el archivador que se usa para
los paquetes `pizarra-git` y `pizarra-bin` de AUR.

## Checklist antes de sacar un release

* se puede dibugar con todas las herramientas
* se puede hacer zoom para adentro y afuera, y regresar al inicio
* se puede rotar
* se puede cambiar el alpha y el grosor
* se pueden borrar
* ctrl+z, ctrl+shift+z
* se puede guardar
* se puede abrir el archivo guardado
* se puede exportar
