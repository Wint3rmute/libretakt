#!/usr/bin/env bash

# pandoc -s -i -t revealjs presentation.md --css presentation.css -o presentation.html  --metadata title="Libretakt"
pandoc -t beamer presentation.md -o presentation.pdf -V theme:Singapore

