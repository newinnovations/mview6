valgrind \
  --leak-check=full \
  --show-leak-kinds=all \
  --num-callers=20 \
  --log-file=/tmp/vg-mview6.txt \
  --suppressions=/usr/share/glib-2.0/valgrind/glib.supp \
  --suppressions=/usr/share/gtk-3.0/valgrind/gtk.supp \
  ./target/debug/MView6
