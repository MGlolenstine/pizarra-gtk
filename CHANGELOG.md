# Cambios

## 1.7.0

- El grosor mínimo del slider es consistente con el grosor por defecto de la
  pluma.
- Añade círculos y líneas de ayuda a (casi) todas las figuras
- Si por algún azar de la naturaleza guardas un archivo sin figuras la pizarra
  no explota.
- Reemplaza el menú 'Circunferencia' por 'Círculo'.
- Dibuja el tamaño y estado de actividad de la goma al usarla.
- Elimina elipses consistentemente (antes desaparecían antes de ser tocadas).
- Al borrar las figuras considera el grosor del trazo para considerar un
  'contacto' con la goma.
- Habilita el menú salir.
- Maneja más suavemente el scroll del trackpad (solo el ojo entrenado lo nota).
- Permite cancelar las figuras cuando te acercas mucho a uno de los puntos que
  las definen.
- Añade las herramientas de rejilla y rejilla libre.
- La goma ahora borra todo lo que toca a su paso, antes dejaba algunas figuras
  sin ninguna razón aparente.
- Respeta la configuración de "export_padding" al guardar como PNG.

## 1.6.1

- Corrige un problema en windows (introducido en 1.5.5) que evitaba que la
  aplicación abriera. El problema estaba relacionado con env_logger.

## 1.6.0

- Si tu tableta digitalizadora tiene goma, la pizarra quizá la pueda usar!

## 1.5.5

- Desactiva el logging por defecto (podría mejorar el rendimiento en pantallas
  grandes)

## 1.5.4

- Scroll suave con el mousepad
- No explota con un archivo de configuración incompleto
- Corrige el manejo de la tecla shift para rotación

## 1.5.0

- Lee configuraciones de un archivo ubicado en `~/.config/pizarra/config.toml`.

## 1.4.4

- Corrige un bug que afecta la forma de cerrar polígonos a diferentes niveles de
  zoom. A mayor zoom el radio de cierre aumentaba mientras que a menor zoom el
  radio de cierre disminuia. Ahora el radio de cierre es constante respecto a
  las coordenadas de pantalla.

## 1.4.3

- Corrige un bug al hacer circunferencias por tres puntos cuando los dos últimos
  son el mismo. Ahora aparece un punto en la pantalla.

## 1.4.2

- Corrige un bug que hace que las elipses no aparezcan en su lugar, en
  particular si se hacen con el touchpad.

## 1.4.1

- Corrige bug que mueve la barra de título al poner el segundo punto de una
  elipse
- Soluciona un problema presentado cuando se ponen los dos puntos de una elipse
  en el mismo lugar

## 1.4.0

- Rotación del canvas (presionando shift al trasladar la vista).
- Herramienta de elipse ahora funciona con tres puntos: los focos y un punto
  exterior.
- Corrige inconsistencias al guardar archivos con cambios o archivos nuevos.

## 1.3.1

- Cambia el ícono de la herramienta en uso al elegir una diferente.

## 1.3.0

- Añade la posibilidad de cambiar el grosor y transparencia del trazo.

## 1.2.1

- Corrige un problema con la herramienta polígono.

## 1.2.0

- Las líneas suaves ahora son el default.

## 1.1.1

- Se resuelve un problema con que la vista no se actualizaba luego de hacer zoom
  o deshacer.

## 1.1.0

- Los trazos hechos a mano alzada ahora son suaves.

## 1.0.1

- Resuelto un bug que impedía abrir un archivo en la segunda instancia abierta
  de la pizarra.

## 1.0.0

- Primera versión estable
- Se pueden hacer líneas, círculos perfectos, elipses, polígonos y rectángulos
- Se pueden borrar figuras
- Funciona el zoom
- Se pueden abrir y guardar archivos
- Se puede exportar a PNG
- Se puede elegir color de una paleta
- Se puede deshacer/rehacer
