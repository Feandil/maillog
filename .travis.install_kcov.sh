#!/bin/sh

set -e

# Using cached value
if test !-d $HOME/kcov
then
  mkdir $HOME/kcov
fi
cd $HOME/kcov

if test -d .git
then
  git pull -f
else
  git clone https://github.com/Feandil/kcov.git .
  mkdir build
fi

cd build
cmake -DCMAKE_INSTALL_PREFIX=$HOME ..
make
make install

