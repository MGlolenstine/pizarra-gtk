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
