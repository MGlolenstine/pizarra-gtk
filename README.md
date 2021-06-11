# Pizarra, edición GTK

Este es el frontend GTK de la [pizarra](https://gitlab.com/categulario/pizarra).

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

## Checklist antes de sacar un release

* se puede dibugar con todas las herramientas
* se puede hacer zoom para adentro y afuera, y regresar al inicio
* se puede cambiar el alpha y el grosor
* se pueden borrar
* ctrl+z, ctrl+shift+z
* se puede guardar
* se puede abrir el archivo guardado
* se puede exportar
